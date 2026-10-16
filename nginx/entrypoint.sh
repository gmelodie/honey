#!/bin/sh
set -e

CERT_DIR="/etc/letsencrypt/live/${TARGET_HOST}"
SELFSIGNED_DIR="/etc/nginx/ssl"

# Self-signed fallback lets nginx start so certbot can pass the HTTP-01 challenge
render_conf() {
    if [ -f "${CERT_DIR}/fullchain.pem" ]; then
        export CERT_PATH="${CERT_DIR}"
    else
        echo "[nginx] No LE cert found for ${TARGET_HOST} — using self-signed bootstrap cert"
        mkdir -p "${SELFSIGNED_DIR}"
        openssl req -x509 -nodes -newkey rsa:2048 -days 1 \
            -keyout "${SELFSIGNED_DIR}/privkey.pem" \
            -out    "${SELFSIGNED_DIR}/fullchain.pem" \
            -subj   "/CN=${TARGET_HOST}" 2>/dev/null
        export CERT_PATH="${SELFSIGNED_DIR}"
    fi

    # Substitute only our env vars; nginx variables like $host stay untouched
    # shellcheck disable=SC2016
    envsubst '${TARGET_HOST} ${CERT_PATH}' \
        < /etc/nginx/templates/default.conf.template \
        > /etc/nginx/conf.d/default.conf
}

# Catch-all 443 cert, so TLS handshakes from unknown clients never show TARGET_HOST
if [ ! -f "/etc/nginx/ssl/catchall.crt" ]; then
    mkdir -p /etc/nginx/ssl
    openssl req -x509 -nodes -newkey rsa:2048 -days 3650 \
        -keyout /etc/nginx/ssl/catchall.key \
        -out    /etc/nginx/ssl/catchall.crt \
        -subj   "/CN=localhost" 2>/dev/null
fi

render_conf

# Re-render so a first LE cert or a renewed one from the certbot container is picked up
(while :; do sleep 12h; render_conf; nginx -s reload || true; done) &

exec nginx -g 'daemon off;'
