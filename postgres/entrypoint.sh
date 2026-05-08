#!/bin/bash
set -e

# Start postgres using the official entrypoint in the background
docker-entrypoint.sh postgres &
PG_PID=$!

# Forward shutdown signals so postgres exits cleanly
trap "kill -TERM $PG_PID" SIGTERM SIGINT

# Wait for the real TCP server — the temp init server only binds the Unix socket,
# so this only succeeds once initialization is complete and the permanent server is up.
until pg_isready -U "$POSTGRES_USER" -h 127.0.0.1 -q 2>/dev/null; do
    sleep 1
done

# Sync the password from the environment on every start.
psql "postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1/${POSTGRES_DB}" \
    -c "ALTER ROLE \"${POSTGRES_USER}\" WITH PASSWORD '${POSTGRES_PASSWORD}';" \
    >/dev/null

wait $PG_PID
