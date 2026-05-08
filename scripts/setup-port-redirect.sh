#!/bin/bash
# Redirect external SSH/Telnet traffic to cowrie's unprivileged ports.
# Run once on the host after docker compose up.
set -e

iptables -t nat -C PREROUTING -p tcp --dport 22 -j REDIRECT --to-port 2222 2>/dev/null \
  || iptables -t nat -A PREROUTING -p tcp --dport 22 -j REDIRECT --to-port 2222

iptables -t nat -C PREROUTING -p tcp --dport 23 -j REDIRECT --to-port 2223 2>/dev/null \
  || iptables -t nat -A PREROUTING -p tcp --dport 23 -j REDIRECT --to-port 2223

echo "Port redirect active: 22->2222, 23->2223"
