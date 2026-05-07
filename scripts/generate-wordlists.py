#!/usr/bin/env python3
"""
Generate wordlists from Cowrie honeypot auth data.
Produces daily/weekly/monthly/all-time lists for passwords, usernames,
and combined password:username pairs — sorted by frequency descending.

Usage: generate-wordlists.py
Reads DB connection from env: POSTGRES_USER, POSTGRES_PASSWORD,
                               POSTGRES_HOST, POSTGRES_PORT, POSTGRES_DB
Output directory: WORDLIST_DIR (default: /wordlists)

Add to cron to run daily, e.g.:
  0 3 * * * /usr/local/bin/generate-wordlists.py >> /var/log/wordlists.log 2>&1
"""

import os
import sys
import psycopg2
from datetime import datetime, timezone

WORDLIST_DIR = os.environ.get("WORDLIST_DIR", "/wordlists")

WINDOWS = {
    "daily":   "NOW() - INTERVAL '1 day'",
    "weekly":  "NOW() - INTERVAL '7 days'",
    "monthly": "NOW() - INTERVAL '30 days'",
    "alltime": None,
}


def connect():
    return psycopg2.connect(
        host=os.environ["POSTGRES_HOST"],
        port=os.environ.get("POSTGRES_PORT", "5432"),
        dbname=os.environ["POSTGRES_DB"],
        user=os.environ["POSTGRES_USER"],
        password=os.environ["POSTGRES_PASSWORD"],
    )


def fetch_ranked(cur, column, since_expr):
    where = f"AND \"timestamp\" >= {since_expr}" if since_expr else ""
    cur.execute(f"""
        SELECT {column}, COUNT(*) AS cnt
        FROM auth
        WHERE {column} IS NOT NULL
          AND {column} <> ''
          {where}
        GROUP BY {column}
        ORDER BY cnt DESC, {column}
    """)
    return [row[0] for row in cur.fetchall()]


def fetch_pairs(cur, since_expr):
    where = f"AND \"timestamp\" >= {since_expr}" if since_expr else ""
    cur.execute(f"""
        SELECT password || ':' || username AS pair, COUNT(*) AS cnt
        FROM auth
        WHERE username IS NOT NULL AND username <> ''
          AND password IS NOT NULL AND password <> ''
          {where}
        GROUP BY pair
        ORDER BY cnt DESC, pair
    """)
    return [row[0] for row in cur.fetchall()]


def write_wordlist(path, entries):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        for entry in entries:
            f.write(entry + "\n")
    os.replace(tmp, path)


def main():
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"[{ts}] Starting wordlist generation...")

    try:
        conn = connect()
    except Exception as e:
        print(f"[{ts}] ERROR: could not connect to database: {e}", file=sys.stderr)
        sys.exit(1)

    with conn:
        cur = conn.cursor()
        for window, since_expr in WINDOWS.items():
            subdir = os.path.join(WORDLIST_DIR, window)

            passwords = fetch_ranked(cur, "password", since_expr)
            usernames = fetch_ranked(cur, "username", since_expr)
            pairs     = fetch_pairs(cur, since_expr)

            write_wordlist(os.path.join(subdir, "passwords.txt"), passwords)
            write_wordlist(os.path.join(subdir, "usernames.txt"), usernames)
            write_wordlist(os.path.join(subdir, "passwords_usernames.txt"), pairs)

            print(
                f"[{ts}] {window}: "
                f"{len(passwords)} passwords, "
                f"{len(usernames)} usernames, "
                f"{len(pairs)} pairs"
            )

    conn.close()
    print(f"[{ts}] Done.")


if __name__ == "__main__":
    main()
