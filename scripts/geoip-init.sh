#!/bin/sh
GEOIP_DIR="${GEOIP_DIR:-/geoip}"
mkdir -p "$GEOIP_DIR"

COUNTRY="$GEOIP_DIR/country.mmdb"
ASN="$GEOIP_DIR/asn.mmdb"

if [ -f "$COUNTRY" ] && [ -f "$ASN" ]; then
    echo "GeoIP databases already exist, skipping download."
    exit 0
fi

BASE="https://download.db-ip.com/free"

# Try current month, then fall back to previous month
MONTH=$(date +%Y-%m)
PREV_MONTH=$(date -d "1 month ago" +%Y-%m 2>/dev/null || date -v-1m +%Y-%m 2>/dev/null)

download_db() {
    local name="$1"   # country or asn
    local dest="$2"   # destination path without .gz

    for month in "$MONTH" "$PREV_MONTH"; do
        [ -z "$month" ] && continue
        url="$BASE/dbip-${name}-lite-${month}.mmdb.gz"
        echo "Trying $url ..."
        if wget -q --show-progress -O "${dest}.gz" "$url"; then
            gunzip "${dest}.gz"
            echo "${name} database ready: ${dest}"
            return 0
        fi
        rm -f "${dest}.gz"
    done

    echo "WARNING: could not download ${name} GeoIP database — geo features will be unavailable" >&2
    return 1
}

echo "Downloading db-ip.com free GeoIP databases..."
country_ok=0
asn_ok=0

download_db "country" "$COUNTRY" && country_ok=1
download_db "asn"     "$ASN"     && asn_ok=1

if [ "$country_ok" -eq 0 ] && [ "$asn_ok" -eq 0 ]; then
    echo "WARNING: both GeoIP databases unavailable; stats-gen will run without geo enrichment." >&2
fi

# Always exit 0 so stats-gen is not blocked
exit 0
