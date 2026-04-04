#!/usr/bin/env bash
# Updates the Brazil ipset with fresh ranges from LACNIC.
# Add to cron: 0 2 * * * /path/to/scripts/update-brazil-ipset.sh >> /var/log/brazil-ipset.log 2>&1

set -euo pipefail

RANGES_FILE="/etc/ipset/brazil-ipv4.txt"
IPSET_NAME="br-grafana"
TEMP_FILE=$(mktemp)
trap 'rm -f "$TEMP_FILE"' EXIT

echo "[$(date -u)] Fetching latest Brazil IP ranges..."
curl -s https://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-extended-latest \
  | grep "^lacnic|BR|ipv4" \
  | awk -F'|' '{
      n = $5
      bits = 0
      while (n > 1) { n /= 2; bits++ }
      printf "%s/%d\n", $4, 32 - bits
    }' \
  | sort > "$TEMP_FILE"

if diff -q "$RANGES_FILE" "$TEMP_FILE" > /dev/null 2>&1; then
  echo "[$(date -u)] No changes, skipping update."
  exit 0
fi

mv "$TEMP_FILE" "$RANGES_FILE"
COUNT=$(wc -l < "$RANGES_FILE")
echo "[$(date -u)] Updated: $COUNT ranges. Reloading ipset..."

ipset flush "$IPSET_NAME"
while read -r cidr; do
  ipset add "$IPSET_NAME" "$cidr"
done < "$RANGES_FILE"

ipset save > /etc/ipset/ipset.rules
echo "[$(date -u)] Done."
