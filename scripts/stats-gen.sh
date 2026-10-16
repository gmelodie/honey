#!/bin/sh
trap exit TERM
while :; do
    honey-stats
    sleep 5m &
    wait $!
done
