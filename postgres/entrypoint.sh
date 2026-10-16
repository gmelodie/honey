#!/bin/bash
set -e

# Host networking: without listen_addresses the server answers on every public interface.
docker-entrypoint.sh postgres -c listen_addresses=127.0.0.1 &
PG_PID=$!

trap 'kill -TERM "$PG_PID"; wait "$PG_PID"; exit' TERM INT

# Wait for the real TCP server — the temp init server only binds the Unix socket,
# so this only succeeds once initialization is complete and the permanent server is up.
until pg_isready -U "$POSTGRES_USER" -h 127.0.0.1 -q 2>/dev/null; do
    sleep 1
done

# Sync the password from the environment on every start.
PGPASSWORD="$POSTGRES_PASSWORD" psql -h 127.0.0.1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -q \
    -v user="$POSTGRES_USER" -v pw="$POSTGRES_PASSWORD" <<'SQL'
ALTER ROLE :"user" WITH PASSWORD :'pw';
SQL

wait "$PG_PID"
