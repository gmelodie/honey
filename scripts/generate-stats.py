#!/usr/bin/env python3
"""
Precompute /api/stats for all time windows and write to STATS_DIR/{window}.json.
Run on a schedule (e.g. every 5 minutes); the web server serves the cached files.
"""

import json
import os
import statistics
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

import psycopg2
import psycopg2.extras

try:
    import maxminddb
    _MAXMINDDB_AVAILABLE = True
except ImportError:
    _MAXMINDDB_AVAILABLE = False

STATS_DIR    = os.environ.get("STATS_DIR",    "/stats")
WORDLIST_DIR = os.environ.get("WORDLIST_DIR", "/wordlists")

WINDOW_TO_WL_PERIOD = {
    "1h": "daily", "6h": "daily", "24h": "daily",
    "7d": "weekly", "30d": "monthly", "all": "alltime",
}

WINDOWS = {
    "1h":  timedelta(hours=1),
    "6h":  timedelta(hours=6),
    "24h": timedelta(hours=24),
    "7d":  timedelta(days=7),
    "30d": timedelta(days=30),
    "all": None,
}

BUCKET = {
    "1h":  "date_trunc('hour', {col}) + (EXTRACT(MINUTE FROM {col})::int / 5)  * INTERVAL '5 minutes'",
    "6h":  "date_trunc('hour', {col}) + (EXTRACT(MINUTE FROM {col})::int / 30) * INTERVAL '30 minutes'",
    "24h": "date_trunc('hour', {col})",
    "7d":  "date_trunc('day',  {col}) + (EXTRACT(HOUR   FROM {col})::int / 6)  * INTERVAL '6 hours'",
    "30d": "date_trunc('day',  {col})",
    "all": "date_trunc('day',  {col})",
}


def connect():
    conn = psycopg2.connect(
        host=os.environ.get("POSTGRES_HOST", "127.0.0.1"),
        port=os.environ.get("POSTGRES_PORT", "5432"),
        dbname=os.environ["POSTGRES_DB"],
        user=os.environ["POSTGRES_USER"],
        password=os.environ["POSTGRES_PASSWORD"],
        sslmode="disable",
    )
    conn.autocommit = True
    return conn


def cond(col, since):
    if since is None:
        return "TRUE", ()
    return f"{col} >= %s", (since,)


def rows(cur):
    return [dict(r) for r in cur.fetchall()]


def compute_web(conn, window):
    delta = WINDOWS[window]
    now = datetime.now(timezone.utc)
    since = None if delta is None else now - delta

    vc, vp = cond("v.timestamp", since)
    fc, fp = cond("f.timestamp", since)
    bucket = BUCKET[window].format(col="v.timestamp")

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(f"SELECT count(*) AS v FROM web_visits v WHERE {vc}", vp)
        web_visits = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT v.ip) AS v FROM web_visits v WHERE {vc}", vp)
        web_unique_ips = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(*) AS v FROM web_form_submissions f WHERE {fc}", fp)
        web_submissions = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT v.path) AS v FROM web_visits v WHERE {vc}", vp)
        web_unique_paths = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT v.user_agent) AS v FROM web_visits v WHERE {vc} AND v.user_agent != ''", vp)
        web_unique_uas = int(cur.fetchone()["v"])

        cur.execute(f"""
            SELECT count(DISTINCT f.form_data->>'password') AS v
            FROM web_form_submissions f WHERE {fc}
              AND f.form_data->>'password' IS NOT NULL
              AND f.form_data->>'password' != ''
        """, fp)
        web_unique_passwords = int(cur.fetchone()["v"])

        cur.execute(f"""
            SELECT count(DISTINCT f.form_data->>'username') AS v
            FROM web_form_submissions f WHERE {fc}
              AND f.form_data->>'username' IS NOT NULL
              AND f.form_data->>'username' != ''
        """, fp)
        web_unique_usernames = int(cur.fetchone()["v"])

        cur.execute(f"""
            SELECT v.path, count(*) AS visits
            FROM web_visits v WHERE {vc}
            GROUP BY v.path ORDER BY visits DESC LIMIT 20
        """, vp)
        web_top_paths = rows(cur)

        cur.execute(f"""
            SELECT v.ip, count(*) AS visits
            FROM web_visits v WHERE {vc}
            GROUP BY v.ip ORDER BY visits DESC LIMIT 20
        """, vp)
        web_top_ips = rows(cur)

        cur.execute(f"""
            SELECT v.user_agent, count(*) AS visits
            FROM web_visits v WHERE {vc} AND v.user_agent != ''
            GROUP BY v.user_agent ORDER BY visits DESC LIMIT 10
        """, vp)
        web_top_uas = rows(cur)

        cur.execute(f"""
            SELECT v.method, count(*) AS visits
            FROM web_visits v WHERE {vc}
            GROUP BY v.method ORDER BY visits DESC
        """, vp)
        web_methods = rows(cur)

        cur.execute(f"""
            SELECT v.referrer, count(*) AS visits
            FROM web_visits v WHERE {vc} AND v.referrer != ''
            GROUP BY v.referrer ORDER BY visits DESC LIMIT 15
        """, vp)
        web_top_referrers = rows(cur)

        cur.execute(f"""
            SELECT {bucket} AS t, count(*) AS visits,
                   count(f.id) AS submissions
            FROM web_visits v
            LEFT JOIN web_form_submissions f ON f.visit_id = v.id
            WHERE {vc}
            GROUP BY 1 ORDER BY 1
        """, vp)
        web_timeseries = [
            {"t": r["t"].isoformat(), "visits": int(r["visits"]), "submissions": int(r["submissions"])}
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT f.form_data->>'username' AS username, count(*) AS attempts
            FROM web_form_submissions f WHERE {fc}
              AND f.form_data->>'username' IS NOT NULL
              AND f.form_data->>'username' != ''
            GROUP BY username ORDER BY attempts DESC LIMIT 20
        """, fp)
        web_top_usernames = rows(cur)

        cur.execute(f"""
            SELECT f.form_data->>'password' AS password, count(*) AS attempts
            FROM web_form_submissions f WHERE {fc}
              AND f.form_data->>'password' IS NOT NULL
              AND f.form_data->>'password' != ''
            GROUP BY password ORDER BY attempts DESC LIMIT 20
        """, fp)
        web_top_passwords = rows(cur)

        cur.execute(f"""
            SELECT v.timestamp AS time, v.ip, v.method, v.path, v.user_agent
            FROM web_visits v WHERE {vc}
            ORDER BY v.timestamp DESC LIMIT 100
        """, vp)
        web_visit_log = [
            {"time": r["time"].isoformat(), "ip": r["ip"],
             "method": r["method"], "path": r["path"], "user_agent": r["user_agent"]}
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT f.timestamp AS time, v.ip, f.form_data
            FROM web_form_submissions f JOIN web_visits v ON f.visit_id = v.id
            WHERE {fc}
            ORDER BY f.timestamp DESC LIMIT 50
        """, fp)
        web_submission_log = [
            {"time": r["time"].isoformat(), "ip": r["ip"], "form_data": r["form_data"]}
            for r in cur.fetchall()
        ]

    return {
        "overview": {
            "visits":            web_visits,
            "unique_ips":        web_unique_ips,
            "submissions":       web_submissions,
            "unique_paths":      web_unique_paths,
            "unique_uas":        web_unique_uas,
            "unique_passwords":  web_unique_passwords,
            "unique_usernames":  web_unique_usernames,
        },
        "top_paths":       web_top_paths,
        "top_ips":         web_top_ips,
        "top_uas":         web_top_uas,
        "methods":         web_methods,
        "top_referrers":   web_top_referrers,
        "timeseries":      web_timeseries,
        "top_usernames":   web_top_usernames,
        "top_passwords":   web_top_passwords,
        "visit_log":       web_visit_log,
        "submission_log":  web_submission_log,
    }


def _safe_compute_web(conn, window):
    try:
        return compute_web(conn, window)
    except Exception as e:
        print(f"WARNING: web stats skipped: {e}", file=sys.stderr)
        return {"overview": {}, "top_paths": [], "top_ips": [], "top_uas": [],
                "methods": [], "top_referrers": [], "timeseries": [],
                "top_usernames": [], "top_passwords": [],
                "visit_log": [], "submission_log": []}

def load_geo_readers():
    if not _MAXMINDDB_AVAILABLE:
        return None, None
    country_path = os.environ.get("GEOIP_COUNTRY_DB")
    asn_path     = os.environ.get("GEOIP_ASN_DB")
    if not country_path or not asn_path:
        return None, None
    try:
        cr = maxminddb.open_database(country_path)
        ar = maxminddb.open_database(asn_path)
        return cr, ar
    except Exception as e:
        print(f"WARNING: GeoIP readers failed to load: {e}", file=sys.stderr)
        return None, None


def _lookup_ip(ip, country_reader, asn_reader):
    country_iso = country_name = asn = asn_org = None
    if country_reader:
        try:
            r = country_reader.get(ip)
            if r:
                c = r.get("country")
                if isinstance(c, dict):
                    # MaxMind GeoLite2 format: {"country": {"iso_code": "US", "names": {...}}}
                    country_iso  = c.get("iso_code")
                    country_name = (c.get("names") or {}).get("en")
                elif isinstance(c, str) and c:
                    # db-ip.com flat format: {"country": "US"}
                    country_iso = c
        except Exception as e:
            print(f"  WARNING: country lookup failed for {ip}: {e}", file=sys.stderr)
    if asn_reader:
        try:
            r = asn_reader.get(ip)
            if r:
                asn     = r.get("autonomous_system_number")
                asn_org = r.get("autonomous_system_organization")
        except Exception as e:
            print(f"  WARNING: ASN lookup failed for {ip}: {e}", file=sys.stderr)
    return country_iso, country_name, asn, asn_org


def enrich_new_ips(conn, country_reader, asn_reader):
    if country_reader is None and asn_reader is None:
        print("  GeoIP readers unavailable, skipping enrichment", file=sys.stderr)
        return
    with conn.cursor() as cur:
        # Pick up IPs not yet cached AND IPs previously cached with no country data
        # (e.g. from a prior run where the db-ip format was misread)
        cur.execute("""
            SELECT DISTINCT s.ip FROM sessions s
            WHERE s.ip IS NOT NULL
              AND NOT EXISTS (
                SELECT 1 FROM ip_geo_cache c
                WHERE c.ip = s.ip AND c.country_iso IS NOT NULL
              )
            LIMIT 5000
        """)
        ips = [r[0] for r in cur.fetchall()]
    if not ips:
        return
    batch = []
    for ip in ips:
        country_iso, country_name, asn, asn_org = _lookup_ip(ip, country_reader, asn_reader)
        batch.append((ip, country_iso, country_name, asn, asn_org))
    with conn.cursor() as cur:
        psycopg2.extras.execute_values(cur, """
            INSERT INTO ip_geo_cache (ip, country_iso, country_name, asn, asn_org)
            VALUES %s
            ON CONFLICT (ip) DO UPDATE SET
                country_iso  = EXCLUDED.country_iso,
                country_name = EXCLUDED.country_name,
                asn          = COALESCE(EXCLUDED.asn,     ip_geo_cache.asn),
                asn_org      = COALESCE(EXCLUDED.asn_org, ip_geo_cache.asn_org),
                looked_up_at = NOW()
            WHERE ip_geo_cache.country_iso IS NULL
        """, batch)
    print(f"  Geo-enriched {len(batch)} new IPs")


def compute_geo(conn, window):
    delta = WINDOWS[window]
    now   = datetime.now(timezone.utc)
    since = None if delta is None else now - delta
    sc, sp = cond("s.starttime", since)

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(f"SELECT count(*) AS v FROM sessions s WHERE {sc}", sp)
        total = int(cur.fetchone()["v"])

        cur.execute(f"""
            SELECT count(*) AS v FROM sessions s
            JOIN ip_geo_cache g ON g.ip = s.ip
            WHERE {sc} AND g.country_iso IS NOT NULL
        """, sp)
        covered = int(cur.fetchone()["v"])

        cur.execute(f"""
            SELECT g.country_iso, g.country_name, count(*) AS sessions
            FROM sessions s
            JOIN ip_geo_cache g ON g.ip = s.ip
            WHERE {sc} AND g.country_iso IS NOT NULL
            GROUP BY g.country_iso, g.country_name
            ORDER BY sessions DESC LIMIT 15
        """, sp)
        top_countries = [
            {
                "country_iso":  r["country_iso"],
                "country_name": r["country_name"],
                "sessions":     int(r["sessions"]),
                "pct":          round(100 * r["sessions"] / max(total, 1), 1),
            }
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT g.asn, g.asn_org, count(*) AS sessions
            FROM sessions s
            JOIN ip_geo_cache g ON g.ip = s.ip
            WHERE {sc} AND g.asn IS NOT NULL
            GROUP BY g.asn, g.asn_org
            ORDER BY sessions DESC LIMIT 15
        """, sp)
        top_asns = [
            {
                "asn":      r["asn"],
                "asn_org":  r["asn_org"],
                "sessions": int(r["sessions"]),
                "pct":      round(100 * r["sessions"] / max(total, 1), 1),
            }
            for r in cur.fetchall()
        ]

        new_asns = []
        if since is not None:
            cur.execute("""
                SELECT g.asn, g.asn_org,
                       min(s.starttime) AS first_seen,
                       count(DISTINCT s.id) AS sessions
                FROM sessions s
                JOIN ip_geo_cache g ON g.ip = s.ip
                WHERE g.asn IS NOT NULL
                GROUP BY g.asn, g.asn_org
                HAVING min(s.starttime) >= %s
                ORDER BY sessions DESC LIMIT 20
            """, (since,))
            new_asn_rows = cur.fetchall()
            if new_asn_rows:
                asn_ids = [r["asn"] for r in new_asn_rows]
                cur.execute("""
                    SELECT g.asn, count(*) AS attempts
                    FROM auth a
                    JOIN sessions s ON a.session = s.id
                    JOIN ip_geo_cache g ON g.ip = s.ip
                    WHERE a.timestamp >= %s AND g.asn = ANY(%s)
                    GROUP BY g.asn
                """, (since, asn_ids))
                attempt_map = {r["asn"]: int(r["attempts"]) for r in cur.fetchall()}
                new_asns = [
                    {
                        "asn":        r["asn"],
                        "asn_org":    r["asn_org"],
                        "first_seen": r["first_seen"].isoformat(),
                        "sessions":   int(r["sessions"]),
                        "attempts":   attempt_map.get(r["asn"], 0),
                    }
                    for r in new_asn_rows
                ]

    coverage_pct = round(100 * covered / max(total, 1), 1) if total > 0 else 0.0
    return {
        "top_countries": top_countries,
        "top_asns":      top_asns,
        "new_asns":      new_asns,
        "coverage_pct":  coverage_pct,
    }


def _safe_compute_geo(conn, window):
    try:
        return compute_geo(conn, window)
    except Exception as e:
        print(f"WARNING: geo stats skipped: {e}", file=sys.stderr)
        return {"top_countries": [], "top_asns": [], "new_asns": [], "coverage_pct": 0.0}


def _get_active_campaigns(conn):
    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute("""
            SELECT id, detected_at, onset_time, z_score, spike_ratio,
                   new_asn_count, peak_rate_per_hour, baseline_rate_per_hour,
                   new_asns, top_pairs, credential_pattern
            FROM campaign_events
            WHERE active = TRUE
            ORDER BY onset_time DESC
        """)
        return [
            {
                "id":                     r["id"],
                "detected_at":            r["detected_at"].isoformat(),
                "onset_time":             r["onset_time"].isoformat(),
                "z_score":                float(r["z_score"]),
                "spike_ratio":            float(r["spike_ratio"]),
                "new_asn_count":          r["new_asn_count"],
                "peak_rate_per_hour":     float(r["peak_rate_per_hour"]),
                "baseline_rate_per_hour": float(r["baseline_rate_per_hour"]),
                "new_asns":               r["new_asns"],
                "top_pairs":              r["top_pairs"],
                "credential_pattern":     r["credential_pattern"],
            }
            for r in cur.fetchall()
        ]


def compute_campaigns(conn):
    now                    = datetime.now(timezone.utc)
    detection_window_start = now - timedelta(hours=2)
    baseline_start         = now - timedelta(days=7)
    baseline_end           = detection_window_start

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute("""
            SELECT date_trunc('hour', a.timestamp) AS bucket, count(*) AS attempts
            FROM auth a
            WHERE a.timestamp >= %s AND a.timestamp < %s
            GROUP BY bucket ORDER BY bucket
        """, (baseline_start, baseline_end))
        baseline_buckets = [int(r["attempts"]) for r in cur.fetchall()]

        if len(baseline_buckets) < 12:
            return _get_active_campaigns(conn)

        baseline_mean   = statistics.mean(baseline_buckets)
        baseline_stddev = statistics.stdev(baseline_buckets) if len(baseline_buckets) > 1 else 0.0

        cur.execute("""
            SELECT count(*) AS attempts FROM auth a WHERE a.timestamp >= %s
        """, (detection_window_start,))
        current_attempts = int(cur.fetchone()["attempts"])
        current_rate_per_hour = current_attempts / 2.0

        z_score = (current_rate_per_hour - baseline_mean) / max(baseline_stddev, 1.0)

        if z_score < 3.0:
            return _get_active_campaigns(conn)

        cur.execute("""
            SELECT DISTINCT g.asn FROM sessions s
            JOIN ip_geo_cache g ON g.ip = s.ip
            WHERE s.starttime >= %s AND s.starttime < %s AND g.asn IS NOT NULL
        """, (baseline_start, baseline_end))
        baseline_asns = {r["asn"] for r in cur.fetchall()}

        cur.execute("""
            SELECT g.asn, g.asn_org, count(DISTINCT s.id) AS sessions,
                   count(a.id) AS attempts
            FROM sessions s
            JOIN ip_geo_cache g ON g.ip = s.ip
            LEFT JOIN auth a ON a.session = s.id AND a.timestamp >= %s
            WHERE s.starttime >= %s AND g.asn IS NOT NULL
            GROUP BY g.asn, g.asn_org
        """, (detection_window_start, detection_window_start))
        current_asn_rows = cur.fetchall()

        new_asn_rows = [r for r in current_asn_rows if r["asn"] not in baseline_asns]

        if len(new_asn_rows) < 3:
            return _get_active_campaigns(conn)

        new_asn_attempts = sum(int(r["attempts"]) for r in new_asn_rows)
        new_asn_ratio    = new_asn_attempts / max(current_attempts, 1)

        if new_asn_ratio < 0.30:
            return _get_active_campaigns(conn)

        # Backtrack to find onset time
        onset_threshold = baseline_mean + 2.0 * baseline_stddev
        cur.execute("""
            SELECT date_trunc('minute', a.timestamp) -
                   (EXTRACT(MINUTE FROM a.timestamp)::int %% 5) * INTERVAL '1 minute' AS bucket,
                   count(*) AS attempts
            FROM auth a
            WHERE a.timestamp >= %s
            GROUP BY bucket ORDER BY bucket
        """, (now - timedelta(hours=6),))
        onset_time = detection_window_start
        for row in cur.fetchall():
            if int(row["attempts"]) * 12 > onset_threshold:
                onset_time = row["bucket"]
                break

        # Credential pattern
        cur.execute("""
            SELECT a.username, a.password, count(*) AS attempts
            FROM auth a
            WHERE a.timestamp >= %s AND a.timestamp < %s
            GROUP BY a.username, a.password ORDER BY attempts DESC LIMIT 10
        """, (baseline_start, baseline_end))
        baseline_pairs = {(r["username"], r["password"]) for r in cur.fetchall()}

        new_asn_ids = [r["asn"] for r in new_asn_rows]
        cur.execute("""
            SELECT a.username, a.password, count(*) AS attempts
            FROM auth a
            JOIN sessions s ON a.session = s.id
            JOIN ip_geo_cache g ON g.ip = s.ip
            WHERE a.timestamp >= %s AND g.asn = ANY(%s)
            GROUP BY a.username, a.password ORDER BY attempts DESC LIMIT 10
        """, (detection_window_start, new_asn_ids))
        new_asn_pairs_raw = cur.fetchall()

        top_pairs = [
            {
                "username": r["username"],
                "password": r["password"],
                "attempts": int(r["attempts"]),
                "novel":    (r["username"], r["password"]) not in baseline_pairs,
            }
            for r in new_asn_pairs_raw
        ]
        novel_count = sum(1 for p in top_pairs if p["novel"])
        credential_pattern = "novel" if novel_count >= 2 else "established"

        new_asns_json = [
            {"asn": r["asn"], "asn_org": r["asn_org"], "attempts": int(r["attempts"])}
            for r in sorted(new_asn_rows, key=lambda x: -int(x["attempts"]))[:10]
        ]

        with conn.cursor() as cur2:
            cur2.execute("""
                INSERT INTO campaign_events
                    (onset_time, z_score, spike_ratio, new_asn_count,
                     peak_rate_per_hour, baseline_rate_per_hour,
                     new_asns, top_pairs, credential_pattern)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (onset_time) DO UPDATE SET
                    z_score              = EXCLUDED.z_score,
                    spike_ratio          = EXCLUDED.spike_ratio,
                    new_asn_count        = EXCLUDED.new_asn_count,
                    peak_rate_per_hour   = EXCLUDED.peak_rate_per_hour,
                    new_asns             = EXCLUDED.new_asns,
                    top_pairs            = EXCLUDED.top_pairs,
                    credential_pattern   = EXCLUDED.credential_pattern
            """, (
                onset_time,
                round(z_score, 2),
                round(new_asn_ratio, 3),
                len(new_asn_rows),
                round(current_rate_per_hour, 2),
                round(baseline_mean, 2),
                json.dumps(new_asns_json),
                json.dumps(top_pairs),
                credential_pattern,
            ))

        with conn.cursor() as cur3:
            cur3.execute("""
                UPDATE campaign_events SET active = FALSE
                WHERE onset_time < %s AND active = TRUE
            """, (now - timedelta(hours=48),))

    return _get_active_campaigns(conn)


def count_novel_passwords(window):
    period = WINDOW_TO_WL_PERIOD[window]
    path = Path(WORDLIST_DIR) / period / "novel_passwords.txt"
    try:
        with open(path, "rb") as f:
            return sum(1 for _ in f)
    except FileNotFoundError:
        return None


def compute(conn, window, campaigns):
    delta = WINDOWS[window]
    now = datetime.now(timezone.utc)
    since = None if delta is None else now - delta

    ac, ap   = cond("a.timestamp", since)
    ic, ip_  = cond("i.timestamp", since)
    sc, sp   = cond("s.starttime", since)
    dc, dp   = cond("d.timestamp", since)
    bucket   = BUCKET[window].format(col="a.timestamp")

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(f"SELECT count(DISTINCT a.session) AS v FROM auth a WHERE {ac}", ap)
        connections = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT i.session) AS v FROM input i WHERE {ic}", ip_)
        cmd_sessions = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(*) AS v FROM auth a WHERE {ac}", ap)
        auth_attempts = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(*) AS v FROM input i WHERE {ic}", ip_)
        commands = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT s.ip) AS v FROM sessions s WHERE {sc}", sp)
        unique_ips = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(*) AS v FROM downloads d WHERE {dc}", dp)
        dl_count = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT a.password) AS v FROM auth a WHERE {ac}", ap)
        unique_passwords = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT a.username) AS v FROM auth a WHERE {ac}", ap)
        unique_usernames = int(cur.fetchone()["v"])

        cur.execute(f"SELECT count(DISTINCT d.shasum) AS v FROM downloads d WHERE {dc} AND d.shasum IS NOT NULL", dp)
        unique_malware_hashes = int(cur.fetchone()["v"])

        cur.execute(
            f"SELECT round(sum(CASE WHEN a.success THEN 1 ELSE 0 END)::numeric "
            f"/ NULLIF(count(*), 0) * 100, 2) AS v FROM auth a WHERE {ac}", ap
        )
        success_pct = float(cur.fetchone()["v"] or 0)

        cur.execute(f"""
            SELECT a.username,
                   count(*) AS attempts,
                   sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful
            FROM auth a WHERE {ac}
            GROUP BY a.username ORDER BY attempts DESC LIMIT 15
        """, ap)
        top_usernames = rows(cur)

        cur.execute(f"""
            SELECT a.password, count(*) AS attempts
            FROM auth a WHERE {ac}
            GROUP BY a.password ORDER BY attempts DESC LIMIT 15
        """, ap)
        top_passwords = rows(cur)

        cur.execute(f"""
            SELECT a.username, a.password, count(*) AS attempts
            FROM auth a WHERE {ac}
            GROUP BY a.username, a.password ORDER BY attempts DESC LIMIT 10
        """, ap)
        top_pairs = rows(cur)

        cur.execute(f"""
            SELECT {bucket} AS t,
                   sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful,
                   sum(CASE WHEN NOT a.success THEN 1 ELSE 0 END) AS failed
            FROM auth a WHERE {ac}
            GROUP BY 1 ORDER BY 1
        """, ap)
        timeseries = [
            {"t": r["t"].isoformat(), "successful": int(r["successful"]), "failed": int(r["failed"])}
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT LPAD(EXTRACT(HOUR FROM a.timestamp)::int::text, 2, '0') || ':00' AS hour,
                   EXTRACT(HOUR FROM a.timestamp)::int AS h,
                   count(*) AS attempts
            FROM auth a WHERE {ac}
            GROUP BY hour, h ORDER BY h
        """, ap)
        by_hour = rows(cur)

        cur.execute(f"""
            SELECT TO_CHAR(a.timestamp, 'Dy') AS day,
                   EXTRACT(DOW FROM a.timestamp)::int AS dow,
                   count(*) AS attempts
            FROM auth a WHERE {ac}
            GROUP BY day, dow ORDER BY dow
        """, ap)
        by_dow = rows(cur)

        cur.execute(f"""
            SELECT c.version AS client_version, count(*) AS connections
            FROM sessions s JOIN clients c ON s.client = c.id
            WHERE {sc}
            GROUP BY c.version ORDER BY connections DESC LIMIT 10
        """, sp)
        ssh_clients = rows(cur)

        cur.execute(f"""
            SELECT d.url, count(*) AS downloads
            FROM downloads d WHERE {dc} AND d.url IS NOT NULL
            GROUP BY d.url ORDER BY downloads DESC LIMIT 10
        """, dp)
        top_urls = rows(cur)

        cur.execute(f"""
            SELECT i.timestamp AS time, s.ip, i.session, i.input
            FROM input i JOIN sessions s ON i.session = s.id
            WHERE {ic}
            ORDER BY i.timestamp DESC LIMIT 100
        """, ip_)
        cmd_log = [
            {"time": r["time"].isoformat(), "ip": r["ip"],
             "session": r["session"][:8], "input": r["input"]}
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT a.timestamp AS time, s.ip,
                   a.username, a.password, a.success, a.session
            FROM auth a JOIN sessions s ON a.session = s.id
            WHERE {ac}
            ORDER BY a.timestamp DESC LIMIT 100
        """, ap)
        auth_log = [
            {"time": r["time"].isoformat(), "ip": r["ip"],
             "username": r["username"], "password": r["password"],
             "success": r["success"], "session": r["session"][:8]}
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT d.timestamp AS time, s.ip, d.session, d.url, d.outfile, d.shasum
            FROM downloads d JOIN sessions s ON d.session = s.id
            WHERE {dc}
            ORDER BY d.timestamp DESC LIMIT 50
        """, dp)
        dl_log = [
            {"time": r["time"].isoformat(), "ip": r["ip"],
             "session": r["session"][:8], "url": r["url"],
             "outfile": r["outfile"], "shasum": r["shasum"]}
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT d.shasum, d.url, count(*) AS downloads, min(d.timestamp) AS first_seen
            FROM downloads d WHERE {dc} AND d.shasum IS NOT NULL
            GROUP BY d.shasum, d.url ORDER BY downloads DESC LIMIT 20
        """, dp)
        malware_hashes_detail = [
            {"shasum": r["shasum"], "url": r["url"],
             "downloads": int(r["downloads"]),
             "first_seen": r["first_seen"].isoformat()}
            for r in cur.fetchall()
        ]

    novel_passwords = count_novel_passwords(window)

    return {
        "window":        window,
        "generated_at":  now.isoformat(),
        "overview": {
            "connections":     connections,
            "cmd_sessions":    cmd_sessions,
            "auth_attempts":   auth_attempts,
            "commands":        commands,
            "unique_ips":      unique_ips,
            "downloads":       dl_count,
            "success_pct":     success_pct,
            "unique_passwords":      unique_passwords,
            "unique_usernames":      unique_usernames,
            "unique_malware_hashes": unique_malware_hashes,
            "novel_passwords":       novel_passwords,
        },
        "top_usernames":   top_usernames,
        "top_passwords":   top_passwords,
        "top_pairs":       top_pairs,
        "timeseries":    timeseries,
        "by_hour":       by_hour,
        "by_dow":        by_dow,
        "ssh_clients":   ssh_clients,
        "top_urls":      top_urls,
        "cmd_log":              cmd_log,
        "auth_log":             auth_log,
        "dl_log":               dl_log,
        "malware_hashes_detail": malware_hashes_detail,
        "web":       _safe_compute_web(conn, window),
        "geo":       _safe_compute_geo(conn, window),
        "campaigns": campaigns if window in ("6h", "24h", "7d") else [],
    }


def write(window, data):
    os.makedirs(STATS_DIR, exist_ok=True)
    tmp = os.path.join(STATS_DIR, f"{window}.json.tmp")
    dst = os.path.join(STATS_DIR, f"{window}.json")
    with open(tmp, "w") as f:
        json.dump(data, f, default=str)
    os.replace(tmp, dst)


def main():
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"[{ts}] Connecting...")
    try:
        conn = connect()
    except Exception as e:
        print(f"[{ts}] ERROR: {e}", file=sys.stderr)
        sys.exit(1)

    country_reader, asn_reader = load_geo_readers()
    enrich_new_ips(conn, country_reader, asn_reader)

    print(f"[{ts}] Running campaign detection...", end=" ", flush=True)
    try:
        campaigns = compute_campaigns(conn)
        print(f"done ({len(campaigns)} active)")
    except Exception as e:
        print(f"WARNING: campaign detection failed: {e}", file=sys.stderr)
        campaigns = []

    for window in WINDOWS:
        print(f"[{ts}] Computing {window}...", end=" ", flush=True)
        try:
            data = compute(conn, window, campaigns)
            write(window, data)
            print(f"done ({data['overview']['auth_attempts']} auth rows)")
        except Exception as e:
            print(f"ERROR: {e}", file=sys.stderr)

    conn.close()
    print(f"[{ts}] Done.")


if __name__ == "__main__":
    main()
