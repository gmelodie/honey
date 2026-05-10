#!/bin/sh
set -e
pip install psycopg2-binary pybloom-live -q
python /generate-wordlists.py
trap exit TERM
while :; do
    sleep 6h &
    wait $!
    python /generate-wordlists.py
done
