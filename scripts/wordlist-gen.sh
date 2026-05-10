#!/bin/sh
set -e
python /generate-wordlists.py
trap exit TERM
while :; do
    sleep 6h &
    wait $!
    python /generate-wordlists.py
done
