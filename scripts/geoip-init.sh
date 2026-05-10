#!/bin/sh
set -e
GEOIP_DIR="${GEOIP_DIR:-/geoip}"
mkdir -p "$GEOIP_DIR"

COUNTRY="$GEOIP_DIR/country.mmdb"
ASN="$GEOIP_DIR/asn.mmdb"

if [ -f "$COUNTRY" ] && [ -f "$ASN" ]; then
    echo "GeoIP databases already exist, skipping download."
    exit 0
fi

MONTH=$(date +%Y-%m)
BASE="https://download.db-ip.com/free"

echo "Downloading db-ip.com free GeoIP databases for $MONTH..."

wget -q --show-progress -O "$COUNTRY.gz" "$BASE/dbip-country-lite-${MONTH}.mmdb.gz"
gunzip "$COUNTRY.gz"
echo "Country database ready: $COUNTRY"

wget -q --show-progress -O "$ASN.gz" "$BASE/dbip-asn-lite-${MONTH}.mmdb.gz"
gunzip "$ASN.gz"
echo "ASN database ready: $ASN"
