#!/usr/bin/env bash
# Restrict Grafana (port 3000) to Brazilian IPs only using ipset.
# Run once to set up, then cron keeps it updated.
#
# Usage: sudo ./scripts/setup-brazil-ipset.sh

set -euo pipefail

RANGES_FILE="/etc/ipset/brazil-ipv4.txt"
IPSET_NAME="br-grafana"
PORT=47321

mkdir -p /etc/ipset

echo "[*] Downloading Brazil IP ranges from LACNIC..."
curl -s https://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-extended-latest \
  | grep "^lacnic|BR|ipv4" \
  | awk -F'|' '{
      n = $5
      bits = 0
      while (n > 1) { n /= 2; bits++ }
      printf "%s/%d\n", $4, 32 - bits
    }' \
  | sort > "$RANGES_FILE"

COUNT=$(wc -l < "$RANGES_FILE")
echo "[*] Got $COUNT Brazil CIDR ranges"

echo "[*] Building ipset..."
ipset destroy "$IPSET_NAME" 2>/dev/null || true
ipset create "$IPSET_NAME" hash:net family inet maxelem 65536

while read -r cidr; do
  ipset add "$IPSET_NAME" "$cidr"
done < "$RANGES_FILE"

echo "[*] Applying iptables rules for port $PORT..."
# Remove any existing rules for this port first
iptables -D INPUT -p tcp --dport "$PORT" -m set --match-set "$IPSET_NAME" src -j ACCEPT 2>/dev/null || true
iptables -D INPUT -p tcp --dport "$PORT" -j DROP 2>/dev/null || true

# Allow Brazilian IPs, drop everyone else
iptables -I INPUT -p tcp --dport "$PORT" -j DROP
iptables -I INPUT -p tcp --dport "$PORT" -m set --match-set "$IPSET_NAME" src -j ACCEPT

echo "[*] Saving ipset and iptables rules..."
ipset save > /etc/ipset/ipset.rules
iptables-save > /etc/iptables/rules.v4 2>/dev/null || iptables-save > /etc/ipset/iptables.rules

echo "[*] Done. Port $PORT is now restricted to Brazilian IPs only."
echo "    To make iptables persist across reboots: sudo apt install iptables-persistent"
