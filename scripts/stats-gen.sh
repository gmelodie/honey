#!/bin/sh
set -e
python /generate-stats.py
trap exit TERM
while :; do
    sleep 5m &
    wait $!
    python /generate-stats.py
done
