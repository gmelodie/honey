#!/bin/sh
set -e

BLOOM_FILE="${BLOOM_DIR:-/bloom}/reference.bloom"

if [ -f "$BLOOM_FILE" ]; then
    echo "[bloom-init] Filter already exists at $BLOOM_FILE, skipping."
    exit 0
fi

pip install pybloom-live -q

# Download each URL in BLOOM_WORDLIST_URLS to a temp dir, then build the filter.
# Downloaded files are cleaned up automatically via trap — never persisted to disk.
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

FILES=""
i=1
for url in $BLOOM_WORDLIST_URLS; do
    dest="$TMPDIR/wordlist_$i.txt"
    echo "[bloom-init] Downloading $url ..."
    python -c "
import urllib.request, sys
url, dest = sys.argv[1], sys.argv[2]
urllib.request.urlretrieve(url, dest)
" "$url" "$dest"
    FILES="$FILES $dest"
    i=$((i + 1))
done

echo "[bloom-init] Building Bloom filter..."
# shellcheck disable=SC2086
python /build-bloom-filters.py --output "$BLOOM_FILE" $FILES
