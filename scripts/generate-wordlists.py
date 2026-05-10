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
import re
import sys
import psycopg2
from collections import Counter
from datetime import datetime, timezone

WORDLIST_DIR = os.environ.get("WORDLIST_DIR", "/wordlists")
BLOOM_DIR    = os.environ.get("BLOOM_DIR", "/bloom")
BLOOM_PATH   = os.path.join(BLOOM_DIR, "reference.bloom")

WINDOWS = {
    "daily":   "NOW() - INTERVAL '1 day'",
    "weekly":  "NOW() - INTERVAL '7 days'",
    "monthly": "NOW() - INTERVAL '30 days'",
    "alltime": None,
}

# Rule generation helpers
_YEARS = [str(y) for y in range(2018, 2030)]
_SPECIALS = sorted(
    ['!', '!@#', '@', '#', '$', '1', '123', '12', '2', '01', '1!', '2!', '3!'],
    key=len, reverse=True,
)


def _detect_suffix(pw):
    for year in _YEARS:
        for sfx in _SPECIALS:
            combo = year + sfx
            if pw.endswith(combo) and len(pw) > len(combo):
                return combo
        if pw.endswith(year) and len(pw) > len(year):
            return year
    for sfx in _SPECIALS:
        if pw.endswith(sfx) and len(pw) > len(sfx):
            return sfx
    return None


def _hc(suffix):
    return ' '.join(f'${c}' for c in suffix)


def _jt(suffix):
    return f'Az"{suffix}"'


def generate_rules(passwords, top_n=50):
    counts = Counter(_detect_suffix(pw) for pw in passwords if _detect_suffix(pw))

    hc = [
        '# autopot — derived mutation rules (hashcat)',
        '# usage: hashcat -a 0 -r hashcat.rule hashes.txt wordlist.txt',
        '',
        'c',
        'u',
        'l',
    ]
    jt = [
        '[List.Rules:autopot]',
        '# autopot — derived mutation rules',
        '# append section to john.conf, then:',
        '# john --wordlist=words.txt --rules=autopot hashes',
        '',
        'c',
        'u',
        'l',
    ]

    if counts:
        hc.append('')
        jt.append('')

    for suffix, _ in counts.most_common(top_n):
        h, j = _hc(suffix), _jt(suffix)
        hc += [h, f'c {h}']
        jt += [j, f'c{j}']

    return '\n'.join(hc) + '\n', '\n'.join(jt) + '\n'


def load_bloom_filter():
    try:
        from pybloom_live import ScalableBloomFilter
    except ImportError:
        return None
    if not os.path.exists(BLOOM_PATH):
        return None
    with open(BLOOM_PATH, "rb") as f:
        return ScalableBloomFilter.fromfile(f)


def filter_novel(passwords, bloom):
    return [p for p in passwords if p not in bloom]


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

    bloom = load_bloom_filter()
    if bloom is None:
        print(f"[{ts}] No Bloom filter at {BLOOM_PATH}; novel_passwords.txt will be skipped.")

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

            hc_rules, jt_rules = generate_rules(passwords)
            write_wordlist(os.path.join(subdir, "hashcat.rule"), hc_rules.splitlines())
            write_wordlist(os.path.join(subdir, "john.rule"),    jt_rules.splitlines())

            novel_count = 0
            if bloom is not None:
                novel = filter_novel(passwords, bloom)
                write_wordlist(os.path.join(subdir, "novel_passwords.txt"), novel)
                novel_count = len(novel)

            print(
                f"[{ts}] {window}: "
                f"{len(passwords)} passwords, "
                f"{len(usernames)} usernames, "
                f"{len(pairs)} pairs"
                + (f", {novel_count} novel" if bloom is not None else "")
            )

    conn.close()
    print(f"[{ts}] Done.")


if __name__ == "__main__":
    main()
