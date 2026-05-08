#!/usr/bin/env python3
"""
Import Cowrie JSON log files into the PostgreSQL database.

Usage:
    python3 import-cowrie-json.py cowrie.json [cowrie.json.2024-01-01 ...]

Reads .env from the project root automatically, or set env vars directly.
"""

import json
import os
import sys
import glob
from datetime import timezone
from pathlib import Path
from dotenv import dotenv_values

import psycopg2
from psycopg2.extras import execute_values

# ── Config ────────────────────────────────────────────────────────────────────

def load_env():
    env_file = Path(__file__).parent.parent / ".env"
    if env_file.exists():
        env = dotenv_values(env_file)
    else:
        env = {}
    return {
        "host":     os.environ.get("POSTGRES_HOST",     env.get("POSTGRES_HOST",     "127.0.0.1")),
        "port":     int(os.environ.get("POSTGRES_PORT", env.get("POSTGRES_PORT",     5432))),
        "dbname":   os.environ.get("POSTGRES_DB",       env.get("POSTGRES_DB",       "cowrie")),
        "user":     os.environ.get("POSTGRES_USER",     env.get("POSTGRES_USER",     "cowrie")),
        "password": os.environ.get("POSTGRES_PASSWORD", env.get("POSTGRES_PASSWORD", "")),
    }

# ── Parsing ───────────────────────────────────────────────────────────────────

def parse_logs(paths):
    """Yield parsed JSON objects from one or more log files."""
    for path in paths:
        with open(path) as f:
            for lineno, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    yield json.loads(line)
                except json.JSONDecodeError as e:
                    print(f"  skip {path}:{lineno} — {e}", file=sys.stderr)

def clean(s):
    """Strip NUL bytes that PostgreSQL rejects in text columns."""
    if isinstance(s, str):
        return s.replace("\x00", "")
    return s

def collect(events):
    """
    Group raw events by session ID, building up the data structures
    needed for each DB table.
    """
    sensors    = {}   # sensor_name -> None (dedup)
    clients    = {}   # version -> None (dedup)
    sessions   = {}   # session_id -> dict
    auth_rows  = []
    input_rows = []
    ttylog_rows = []
    download_rows = []
    fingerprint_rows = []

    for ev in events:
        # Strip NUL bytes from every string value in the event
        ev = {k: clean(v) if isinstance(v, str) else v for k, v in ev.items()}
        eid     = ev.get("eventid", "")
        sid     = ev.get("session")
        sensor  = ev.get("sensor", "cowrie")
        ts      = ev.get("timestamp")
        src_ip  = ev.get("src_ip")

        sensors[sensor] = None

        if sid and sid not in sessions:
            sessions[sid] = {
                "id":        sid,
                "sensor":    sensor,
                "ip":        src_ip,
                "starttime": None,
                "endtime":   None,
                "termsize":  "",
                "client":    None,
            }

        s = sessions.get(sid) if sid else None

        if eid == "cowrie.session.connect":
            if s:
                s["starttime"] = ts
                s["ip"] = src_ip

        elif eid == "cowrie.session.closed":
            if s:
                s["endtime"] = ts

        elif eid == "cowrie.client.version":
            version = ev.get("version", "")[:50]
            clients[version] = None
            if s:
                s["client"] = version

        elif eid == "cowrie.client.size":
            if s:
                w = ev.get("width", "")
                h = ev.get("height", "")
                s["termsize"] = f"{w}x{h}"

        elif eid in ("cowrie.login.success", "cowrie.login.failed"):
            auth_rows.append({
                "session":   sid,
                "success":   eid == "cowrie.login.success",
                "username":  clean(ev.get("username")),
                "password":  clean(ev.get("password")),
                "timestamp": ts,
            })

        elif eid == "cowrie.command.input":
            input_rows.append({
                "session":   sid,
                "timestamp": ts,
                "realm":     clean(ev.get("realm", "")),
                "input":     clean(ev.get("input", "")),
                "success":   ev.get("success", False),
            })

        elif eid in ("cowrie.log.open", "cowrie.log.closed"):
            ttylog = ev.get("ttylog") or ev.get("filename")
            if sid and ttylog:
                ttylog_rows.append({"session": sid, "ttylog": ttylog})

        elif eid == "cowrie.session.file_download":
            download_rows.append({
                "session":   sid,
                "timestamp": ts,
                "url":       ev.get("url", ""),
                "outfile":   ev.get("outfile", ""),
                "shasum":    ev.get("shasum", ""),
            })

        elif eid == "cowrie.client.fingerprint":
            fingerprint_rows.append({
                "session":     sid,
                "username":    ev.get("username"),
                "fingerprint": ev.get("fingerprint"),
            })

    return sensors, clients, sessions, auth_rows, input_rows, ttylog_rows, download_rows, fingerprint_rows

# ── DB import ─────────────────────────────────────────────────────────────────

def import_all(conn, sensors, clients, sessions, auth_rows, input_rows, ttylog_rows, download_rows, fingerprint_rows):
    cur = conn.cursor()

    # sensors
    if sensors:
        execute_values(cur,
            "INSERT INTO sensors (ip) VALUES %s ON CONFLICT (ip) DO NOTHING",
            [(name,) for name in sensors])
    cur.execute("SELECT ip, id FROM sensors")
    sensor_id = {row[0]: row[1] for row in cur.fetchall()}

    # clients
    if clients:
        execute_values(cur,
            "INSERT INTO clients (version) VALUES %s ON CONFLICT DO NOTHING",
            [(v,) for v in clients if v])
    cur.execute("SELECT version, id FROM clients")
    client_id = {row[0]: row[1] for row in cur.fetchall()}

    # sessions
    if sessions:
        execute_values(cur, """
            INSERT INTO sessions (id, starttime, endtime, sensor, ip, termsize, client)
            VALUES %s
            ON CONFLICT (id) DO NOTHING
        """, [
            (
                s["id"],
                s["starttime"],
                s["endtime"],
                sensor_id.get(s["sensor"]),
                s["ip"],
                s["termsize"],
                client_id.get(s["client"]) if s["client"] else None,
            )
            for s in sessions.values()
        ])

    # auth
    if auth_rows:
        execute_values(cur, """
            INSERT INTO auth (session, success, username, password, timestamp)
            VALUES %s
            ON CONFLICT DO NOTHING
        """, [(r["session"], r["success"], r["username"], r["password"], r["timestamp"]) for r in auth_rows])

    # input
    if input_rows:
        execute_values(cur, """
            INSERT INTO input (session, timestamp, realm, input, success)
            VALUES %s
            ON CONFLICT DO NOTHING
        """, [(r["session"], r["timestamp"], r["realm"][:20] if r["realm"] else "", r["input"], r["success"]) for r in input_rows])

    # ttylog — deduplicate (same session+file can appear in open+closed)
    seen_ttylogs = set()
    deduped_ttylogs = []
    for r in ttylog_rows:
        key = (r["session"], r["ttylog"])
        if key not in seen_ttylogs:
            seen_ttylogs.add(key)
            deduped_ttylogs.append(r)
    if deduped_ttylogs:
        execute_values(cur, """
            INSERT INTO ttylog (session, ttylog)
            VALUES %s
            ON CONFLICT DO NOTHING
        """, [(r["session"], r["ttylog"]) for r in deduped_ttylogs])

    # downloads
    if download_rows:
        execute_values(cur, """
            INSERT INTO downloads (session, timestamp, url, outfile, shasum)
            VALUES %s
            ON CONFLICT DO NOTHING
        """, [(r["session"], r["timestamp"], r["url"], r["outfile"], r["shasum"]) for r in download_rows])

    # keyfingerprints
    if fingerprint_rows:
        execute_values(cur, """
            INSERT INTO keyfingerprints (session, username, fingerprint)
            VALUES %s
            ON CONFLICT DO NOTHING
        """, [(r["session"], r["username"], r["fingerprint"]) for r in fingerprint_rows])

    conn.commit()
    cur.close()
    return {
        "sensors":      len(sensors),
        "clients":      len(clients),
        "sessions":     len(sessions),
        "auth":         len(auth_rows),
        "input":        len(input_rows),
        "ttylog":       len(deduped_ttylogs),
        "downloads":    len(download_rows),
        "fingerprints": len(fingerprint_rows),
    }

# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <cowrie.json> [more files or globs...]")
        sys.exit(1)

    # Expand any shell globs that weren't expanded by the shell
    paths = []
    for arg in sys.argv[1:]:
        expanded = glob.glob(arg)
        paths.extend(expanded if expanded else [arg])

    missing = [p for p in paths if not os.path.exists(p)]
    if missing:
        print(f"File(s) not found: {', '.join(missing)}", file=sys.stderr)
        sys.exit(1)

    cfg = load_env()
    print(f"Connecting to {cfg['host']}:{cfg['port']}/{cfg['dbname']}...")
    try:
        conn = psycopg2.connect(**cfg)
    except psycopg2.OperationalError as e:
        print(f"Connection failed: {e}", file=sys.stderr)
        sys.exit(1)

    totals = {}
    for path in paths:
        print(f"Processing {path}...")
        events = parse_logs([path])
        data = collect(events)
        counts = import_all(conn, *data)
        for k, v in counts.items():
            totals[k] = totals.get(k, 0) + v
        print(f"  sessions={counts['sessions']} auth={counts['auth']} input={counts['input']}")

    conn.close()
    print("\nTotal:")
    for table, n in totals.items():
        print(f"  {table:<14} {n}")

if __name__ == "__main__":
    main()
