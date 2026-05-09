#!/bin/sh
set -e
pip install psycopg2-binary -q
python /generate-stats.py
trap exit TERM
while :; do
    sleep 5m &
    wait $!
    python /generate-stats.py
done
