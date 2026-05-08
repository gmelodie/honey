#!/usr/bin/env bash
# Obtain Let's Encrypt certificates for TARGET_HOST.
# Run once before (or just after) starting the stack for the first time.
#
# Usage:  TARGET_HOST=honey.gmelodie.com CERTBOT_EMAIL=you@example.com ./scripts/init-letsencrypt.sh
set -euo pipefail

: "${TARGET_HOST:?Set TARGET_HOST to your public hostname}"
: "${CERTBOT_EMAIL:?Set CERTBOT_EMAIL to your email address}"

CERT_DIR="./letsencrypt"

echo "[init] Starting nginx in the background for ACME challenge…"
docker compose up -d nginx

echo "[init] Requesting certificate for ${TARGET_HOST}…"
docker run --rm \
  --network host \
  -v "$(pwd)/letsencrypt:/etc/letsencrypt" \
  -v "$(pwd)/certbot/www:/var/www/certbot" \
  certbot/certbot certonly \
    --webroot \
    --webroot-path /var/www/certbot \
    --email "${CERTBOT_EMAIL}" \
    --agree-tos \
    --no-eff-email \
    -d "${TARGET_HOST}"

echo "[init] Certificate obtained. Reloading nginx…"
docker compose exec nginx nginx -s reload

echo "[init] Done! Certificates are at ${CERT_DIR}/live/${TARGET_HOST}/"
echo "       Set up a cron job to run: docker compose run --rm certbot renew"
