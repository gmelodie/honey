#!/bin/sh
trap exit TERM
while :; do
    honey-wordlists
    sleep 6h &
    wait $!
done
