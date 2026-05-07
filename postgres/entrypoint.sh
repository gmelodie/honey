#!/bin/bash
set -e

# Start postgres using the official entrypoint in the background
docker-entrypoint.sh postgres &
PG_PID=$!

# Forward shutdown signals so postgres exits cleanly
trap "kill -TERM $PG_PID" SIGTERM SIGINT

# Wait for the unix socket to accept connections (no auth required)
until pg_isready -q 2>/dev/null; do
    sleep 1
done

# Sync the password from the environment on every start.
# Uses the local unix socket (trust auth) so it works even when the
# TCP password is stale (e.g. after the user changes POSTGRES_PASSWORD in .env).
psql -U postgres \
    -c "ALTER ROLE \"${POSTGRES_USER}\" WITH PASSWORD '${POSTGRES_PASSWORD}';" \
    >/dev/null

wait $PG_PID
