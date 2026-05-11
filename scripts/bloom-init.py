#!/usr/bin/env python3
"""
Build the reference Bloom filter by downloading wordlists one at a time.
Each file is downloaded, added to the filter, then deleted before the next
download begins — keeping peak disk usage to a single wordlist file at a time.
"""

import os
import sys
import tempfile
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

from pybloom_live import ScalableBloomFilter

BLOOM_FILE = Path(os.environ.get("BLOOM_DIR", "/bloom")) / "reference.bloom"
BLOOM_WORDLIST_URLS = os.environ.get("BLOOM_WORDLIST_URLS", "").split()


def ts():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def main():
    if BLOOM_FILE.exists():
        print(f"[{ts()}] Filter already exists at {BLOOM_FILE}, skipping.")
        sys.exit(0)

    if not BLOOM_WORDLIST_URLS:
        print(f"[{ts()}] BLOOM_WORDLIST_URLS is empty — nothing to build.", file=sys.stderr)
        sys.exit(1)

    bf = ScalableBloomFilter(
        mode=ScalableBloomFilter.LARGE_SET_GROWTH,
        error_rate=0.001,
    )
    total = 0
    n = len(BLOOM_WORDLIST_URLS)

    for i, url in enumerate(BLOOM_WORDLIST_URLS, 1):
        print(f"[{ts()}] ({i}/{n}) Downloading {url} ...")
        tmp_fd, tmp_path = tempfile.mkstemp(suffix=".txt")
        os.close(tmp_fd)
        try:
            urllib.request.urlretrieve(url, tmp_path)
            size_mb = os.path.getsize(tmp_path) / 1024 ** 2
            print(f"[{ts()}]   Downloaded {size_mb:.1f} MB — processing ...")
            with open(tmp_path, "r", encoding="utf-8", errors="replace") as f:
                for line in f:
                    word = line.rstrip("\n")
                    if word:
                        bf.add(word)
                        total += 1
                        if total % 100_000 == 0:
                            print(f"[{ts()}]   {total:,} items so far ...")
        finally:
            os.unlink(tmp_path)
            print(f"[{ts()}]   Temp file deleted — running total: {total:,} items")

    BLOOM_FILE.parent.mkdir(parents=True, exist_ok=True)
    tmp_out = str(BLOOM_FILE) + ".tmp"
    with open(tmp_out, "wb") as f:
        bf.tofile(f)
    os.replace(tmp_out, BLOOM_FILE)

    size_mb = BLOOM_FILE.stat().st_size / 1024 ** 2
    print(f"[{ts()}] Done.")
    print(f"      Items processed:     {total:,}")
    print(f"      False positive rate: 0.100%")
    print(f"      Filter file size:    {size_mb:.1f} MB  ({BLOOM_FILE})")


if __name__ == "__main__":
    main()
