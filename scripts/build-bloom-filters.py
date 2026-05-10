#!/usr/bin/env python3
"""
Build a Bloom filter from reference wordlist files.

Usage:
    build-bloom-filters.py [--output PATH] [--error-rate RATE] FILE [FILE ...]

Defaults:
    --output      /bloom/reference.bloom
    --error-rate  0.001  (0.1% false positive rate)
"""

import argparse
import os
from datetime import datetime, timezone
from pybloom_live import ScalableBloomFilter

BLOOM_DIR = os.environ.get("BLOOM_DIR", "/bloom")
DEFAULT_OUTPUT = os.path.join(BLOOM_DIR, "reference.bloom")


def build_filter(paths, error_rate):
    bf = ScalableBloomFilter(
        mode=ScalableBloomFilter.LARGE_SET_GROWTH,
        error_rate=error_rate,
    )
    total = 0
    for path in paths:
        ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        print(f"[{ts}] Processing {path} ...")
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                word = line.rstrip("\n")
                if word:
                    bf.add(word)
                    total += 1
        ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        print(f"[{ts}]   -> running total: {total:,} items")
    return bf, total


def save_filter(bf, output):
    os.makedirs(os.path.dirname(output) or ".", exist_ok=True)
    tmp = output + ".tmp"
    with open(tmp, "wb") as f:
        bf.tofile(f)
    os.replace(tmp, output)
    return os.path.getsize(output)


def main():
    parser = argparse.ArgumentParser(description="Build a Bloom filter from reference wordlist files.")
    parser.add_argument("files", nargs="+", metavar="FILE", help="Wordlist files to process")
    parser.add_argument("--output", default=DEFAULT_OUTPUT, help="Output path for the filter binary")
    parser.add_argument("--error-rate", type=float, default=0.001, help="False positive rate (default: 0.001)")
    args = parser.parse_args()

    bf, total = build_filter(args.files, args.error_rate)
    size = save_filter(bf, args.output)

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"[{ts}] Done.")
    print(f"      Items processed:     {total:,}")
    print(f"      False positive rate: {args.error_rate:.3%}")
    print(f"      Filter file size:    {size / 1024**2:.1f} MB  ({args.output})")


if __name__ == "__main__":
    main()
