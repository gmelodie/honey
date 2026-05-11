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


def _detect_case_rules(passwords, sample=500, threshold=0.05):
    """Return case rules ordered by prevalence, only those above threshold."""
    pw_sample = passwords[:sample]
    has_alpha = [pw for pw in pw_sample if any(ch.isalpha() for ch in pw)]
    if not has_alpha:
        return []
    counts = {'l': 0, 'c': 0, 'u': 0}
    for pw in has_alpha:
        alpha = [ch for ch in pw if ch.isalpha()]
        if all(ch.islower() for ch in alpha):
            counts['l'] += 1
        elif all(ch.isupper() for ch in alpha):
            counts['u'] += 1
        elif pw[0].isupper() and all(ch.islower() for ch in pw[1:] if ch.isalpha()):
            counts['c'] += 1
    total = len(has_alpha)
    return [rule for rule, cnt in sorted(counts.items(), key=lambda x: -x[1])
            if cnt / total >= threshold]


def generate_rules(passwords, top_n=50):
    suffix_counts = Counter(_detect_suffix(pw) for pw in passwords if _detect_suffix(pw))
    case_rules = _detect_case_rules(passwords)

    hc = [
        '# autopot — derived mutation rules (hashcat)',
        '# usage: hashcat -a 0 -r hashcat.rule hashes.txt wordlist.txt',
    ]
    jt = [
        '[List.Rules:autopot]',
        '# autopot — derived mutation rules',
        '# append section to john.conf, then:',
        '# john --wordlist=words.txt --rules=autopot hashes',
    ]

    if case_rules:
        hc.append('')
        jt.append('')
        hc += case_rules
        jt += case_rules

    if suffix_counts:
        hc.append('')
        jt.append('')

    for suffix, _ in suffix_counts.most_common(top_n):
        h, j = _hc(suffix), _jt(suffix)
        hc += [h, f'c {h}']
        jt += [j, f'c{j}']

    return '\n'.join(hc) + '\n', '\n'.join(jt) + '\n'

# (recent_since_expr, baseline_since_expr) — alltime has no meaningful baseline
TREND_WINDOWS = {
    "daily":   ("NOW() - INTERVAL '1 day'",   "NOW() - INTERVAL '2 days'"),
    "weekly":  ("NOW() - INTERVAL '7 days'",  "NOW() - INTERVAL '14 days'"),
    "monthly": ("NOW() - INTERVAL '30 days'", "NOW() - INTERVAL '60 days'"),
}



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


def fetch_trending(cur, recent_since, baseline_since):
    cur.execute(f"""
        SELECT
            password,
            COUNT(*) FILTER (WHERE "timestamp" >= {recent_since})                                    AS recent_cnt,
            COUNT(*) FILTER (WHERE "timestamp" >= {baseline_since} AND "timestamp" < {recent_since}) AS baseline_cnt
        FROM auth
        WHERE password IS NOT NULL
          AND password <> ''
          AND "timestamp" >= {baseline_since}
        GROUP BY password
    """)
    rows = cur.fetchall()

    if not rows:
        return [], [], 2

    # Poisson-based adaptive floor: for random arrivals, std ≈ sqrt(mean),
    # so sqrt(mean combined count) is the expected noise magnitude per password.
    mean_count = sum(r + b for _, r, b in rows) / len(rows)
    min_count = max(2, round(mean_count ** 0.5))

    trending, dying = [], []
    for pw, recent, baseline in rows:
        if recent >= min_count and recent > baseline:
            trending.append((pw, recent, baseline))
        elif baseline >= min_count and baseline > recent:
            dying.append((pw, recent, baseline))

    # Sort trending by relative growth (new passwords — baseline=0 — float to top)
    trending.sort(key=lambda x: x[1] / max(x[2], 0.1), reverse=True)
    # Sort dying by absolute decline
    dying.sort(key=lambda x: x[2] - x[1], reverse=True)

    return [pw for pw, _, _ in trending], [pw for pw, _, _ in dying], min_count


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

            trend_info = ""
            if window in TREND_WINDOWS:
                recent_expr, baseline_expr = TREND_WINDOWS[window]
                trending, dying, min_count = fetch_trending(cur, recent_expr, baseline_expr)
                write_wordlist(os.path.join(subdir, "trending_passwords.txt"), trending)
                write_wordlist(os.path.join(subdir, "dying_passwords.txt"), dying)
                trend_info = f", {len(trending)} trending, {len(dying)} dying (floor={min_count})"
            else:
                # alltime has no baseline — write empty files so the API reports ready
                write_wordlist(os.path.join(subdir, "trending_passwords.txt"), [])
                write_wordlist(os.path.join(subdir, "dying_passwords.txt"), [])

            print(
                f"[{ts}] {window}: "
                f"{len(passwords)} passwords, "
                f"{len(usernames)} usernames, "
                f"{len(pairs)} pairs"
                + (f", {novel_count} novel" if bloom is not None else "")
                + trend_info
            )

    conn.close()
    print(f"[{ts}] Done.")


if __name__ == "__main__":
    main()
