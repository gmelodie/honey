#!/usr/bin/env python3
"""
Precompute /api/stats for all time windows and write to STATS_DIR/{window}.json.
Run on a schedule (e.g. every 5 minutes); the web server serves the cached files.
"""

import hashlib
import json
import os
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

import psycopg2
import psycopg2.extras

STATS_DIR = os.environ.get("STATS_DIR", "/stats")

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
            GROUP BY v.user_agent ORDER BY visits DESC LIMIT 20
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
            "visits":          web_visits,
            "unique_ips":      web_unique_ips,
            "submissions":     web_submissions,
            "unique_paths":    web_unique_paths,
            "unique_uas":      web_unique_uas,
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


def compute(conn, window):
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
            SELECT a.password, count(*) AS attempts
            FROM auth a WHERE {ac}
            GROUP BY a.password ORDER BY attempts DESC LIMIT 100
        """, ap)
        password_hashes = [
            {
                "password": r["password"],
                "sha256": hashlib.sha256((r["password"] or "").encode()).hexdigest(),
                "attempts": int(r["attempts"]),
            }
            for r in cur.fetchall()
        ]

        cur.execute(f"""
            SELECT a.username, a.password, count(*) AS attempts
            FROM auth a WHERE {ac}
            GROUP BY a.username, a.password ORDER BY attempts DESC LIMIT 25
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
            GROUP BY c.version ORDER BY connections DESC LIMIT 20
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
            "unique_passwords":     unique_passwords,
            "unique_usernames":     unique_usernames,
            "unique_malware_hashes": unique_malware_hashes,
        },
        "top_usernames":   top_usernames,
        "top_passwords":   top_passwords,
        "password_hashes": password_hashes,
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
        "web": _safe_compute_web(conn, window),
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

    for window in WINDOWS:
        print(f"[{ts}] Computing {window}...", end=" ", flush=True)
        try:
            data = compute(conn, window)
            write(window, data)
            print(f"done ({data['overview']['auth_attempts']} auth rows)")
        except Exception as e:
            print(f"ERROR: {e}", file=sys.stderr)

    conn.close()
    print(f"[{ts}] Done.")


if __name__ == "__main__":
    main()
