import os
import requests
import psycopg2
import psycopg2.extras
from flask import Flask, render_template, request, jsonify
from datetime import datetime, timedelta, timezone

app = Flask(__name__)

DB_DSN = (
    f"host={os.environ.get('POSTGRES_HOST', '127.0.0.1')} "
    f"port={os.environ.get('POSTGRES_PORT', '5432')} "
    f"dbname={os.environ.get('POSTGRES_DB', 'cowrie')} "
    f"user={os.environ.get('POSTGRES_USER', 'cowrie')} "
    f"password={os.environ.get('POSTGRES_PASSWORD', '')} "
    f"sslmode=disable"
)

RECAPTCHA_SEC  = os.environ.get("RECAPTCHA_SECRET_KEY", "")
RECAPTCHA_SITE = os.environ.get("RECAPTCHA_SITE_KEY", "")

WINDOWS = {
    "1h":  timedelta(hours=1),
    "6h":  timedelta(hours=6),
    "24h": timedelta(hours=24),
    "7d":  timedelta(days=7),
    "30d": timedelta(days=30),
    "all": None,
}

# SQL expression to bucket a timestamp column into time slots
BUCKET = {
    "1h":  "date_trunc('hour', {col}) + (EXTRACT(MINUTE FROM {col})::int / 5)  * INTERVAL '5 minutes'",
    "6h":  "date_trunc('hour', {col}) + (EXTRACT(MINUTE FROM {col})::int / 30) * INTERVAL '30 minutes'",
    "24h": "date_trunc('hour', {col})",
    "7d":  "date_trunc('day',  {col}) + (EXTRACT(HOUR   FROM {col})::int / 6)  * INTERVAL '6 hours'",
    "30d": "date_trunc('day',  {col})",
    "all": "date_trunc('day',  {col})",
}


def _conn():
    return psycopg2.connect(DB_DSN)


def _since(window):
    delta = WINDOWS.get(window)
    return None if delta is None else datetime.now(timezone.utc) - delta


def _cond(col, since_dt):
    if since_dt is None:
        return "TRUE", ()
    return f"{col} >= %s", (since_dt,)


def _rows(cur):
    return [dict(r) for r in cur.fetchall()]


@app.route("/")
def index():
    return render_template("index.html", recaptcha_site_key=RECAPTCHA_SITE)


@app.route("/api/verify-captcha", methods=["POST"])
def verify_captcha():
    data = request.get_json(silent=True) or {}
    token = data.get("token", "")
    if not token:
        return jsonify({"success": False}), 400
    resp = requests.post(
        "https://www.google.com/recaptcha/api/siteverify",
        data={"secret": RECAPTCHA_SEC, "response": token},
        timeout=10,
    )
    return jsonify({"success": bool(resp.json().get("success"))})


@app.route("/api/stats")
def stats():
    window = request.args.get("window", "6h")
    if window not in WINDOWS:
        window = "6h"

    t = _since(window)
    ac,  ap  = _cond("a.timestamp",  t)
    ic,  ip_ = _cond("i.timestamp",  t)
    sc,  sp  = _cond("s.starttime",  t)
    dc,  dp  = _cond("d.timestamp",  t)

    bucket = BUCKET[window].format(col="a.timestamp")

    conn = _conn()
    try:
        with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:

            # ── Overview stat cards ───────────────────────────────────────
            cur.execute(f"SELECT count(DISTINCT a.session) AS v FROM auth a WHERE {ac}", ap)
            connections = cur.fetchone()["v"]

            cur.execute(f"SELECT count(DISTINCT i.session) AS v FROM input i WHERE {ic}", ip_)
            cmd_sessions = cur.fetchone()["v"]

            cur.execute(f"SELECT count(*) AS v FROM auth a WHERE {ac}", ap)
            auth_attempts = cur.fetchone()["v"]

            cur.execute(f"SELECT count(*) AS v FROM input i WHERE {ic}", ip_)
            commands = cur.fetchone()["v"]

            cur.execute(f"SELECT count(DISTINCT s.ip) AS v FROM sessions s WHERE {sc}", sp)
            unique_ips = cur.fetchone()["v"]

            cur.execute(f"SELECT count(*) AS v FROM downloads d WHERE {dc}", dp)
            dl_count = cur.fetchone()["v"]

            cur.execute(
                f"SELECT round(sum(CASE WHEN a.success THEN 1 ELSE 0 END)::numeric "
                f"/ NULLIF(count(*), 0) * 100, 2) AS v FROM auth a WHERE {ac}", ap
            )
            success_pct = float(cur.fetchone()["v"] or 0)

            # ── Top credentials ───────────────────────────────────────────
            cur.execute(f"""
                SELECT a.username,
                       count(*) AS attempts,
                       sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful
                FROM auth a WHERE {ac}
                GROUP BY a.username ORDER BY attempts DESC LIMIT 15
            """, ap)
            top_usernames = _rows(cur)

            cur.execute(f"""
                SELECT a.password, count(*) AS attempts
                FROM auth a WHERE {ac}
                GROUP BY a.password ORDER BY attempts DESC LIMIT 15
            """, ap)
            top_passwords = _rows(cur)

            cur.execute(f"""
                SELECT a.username, a.password, count(*) AS attempts
                FROM auth a WHERE {ac}
                GROUP BY a.username, a.password ORDER BY attempts DESC LIMIT 25
            """, ap)
            top_pairs = _rows(cur)

            # ── Timeseries ────────────────────────────────────────────────
            cur.execute(f"""
                SELECT {bucket} AS t,
                       sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful,
                       sum(CASE WHEN NOT a.success THEN 1 ELSE 0 END) AS failed
                FROM auth a WHERE {ac}
                GROUP BY 1 ORDER BY 1
            """, ap)
            timeseries = [
                {
                    "t": r["t"].isoformat(),
                    "successful": int(r["successful"]),
                    "failed": int(r["failed"]),
                }
                for r in cur.fetchall()
            ]

            # ── Timing patterns ───────────────────────────────────────────
            cur.execute(f"""
                SELECT LPAD(EXTRACT(HOUR FROM a.timestamp)::int::text, 2, '0') || ':00' AS hour,
                       EXTRACT(HOUR FROM a.timestamp)::int AS h,
                       count(*) AS attempts
                FROM auth a WHERE {ac}
                GROUP BY hour, h ORDER BY h
            """, ap)
            by_hour = _rows(cur)

            cur.execute(f"""
                SELECT TO_CHAR(a.timestamp, 'Dy') AS day,
                       EXTRACT(DOW FROM a.timestamp)::int AS dow,
                       count(*) AS attempts
                FROM auth a WHERE {ac}
                GROUP BY day, dow ORDER BY dow
            """, ap)
            by_dow = _rows(cur)

            # ── SSH client versions ───────────────────────────────────────
            cur.execute(f"""
                SELECT c.version AS client_version, count(*) AS connections
                FROM sessions s JOIN clients c ON s.client = c.id
                WHERE {sc}
                GROUP BY c.version ORDER BY connections DESC LIMIT 20
            """, sp)
            ssh_clients = _rows(cur)

            # ── Downloads ─────────────────────────────────────────────────
            cur.execute(f"""
                SELECT d.url, count(*) AS downloads
                FROM downloads d WHERE {dc} AND d.url IS NOT NULL
                GROUP BY d.url ORDER BY downloads DESC LIMIT 10
            """, dp)
            top_urls = _rows(cur)

            # ── Activity logs ─────────────────────────────────────────────
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

        return jsonify({
            "window": window,
            "overview": {
                "connections":    int(connections),
                "cmd_sessions":   int(cmd_sessions),
                "auth_attempts":  int(auth_attempts),
                "commands":       int(commands),
                "unique_ips":     int(unique_ips),
                "downloads":      int(dl_count),
                "success_pct":    success_pct,
            },
            "top_usernames": top_usernames,
            "top_passwords":  top_passwords,
            "top_pairs":      top_pairs,
            "timeseries":     timeseries,
            "by_hour":        by_hour,
            "by_dow":         by_dow,
            "ssh_clients":    ssh_clients,
            "top_urls":       top_urls,
            "cmd_log":        cmd_log,
            "auth_log":       auth_log,
            "dl_log":         dl_log,
        })
    except psycopg2.Error as e:
        return jsonify({"error": str(e)}), 500
    finally:
        conn.close()


