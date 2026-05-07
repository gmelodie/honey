FROM nginx:1.27-alpine
RUN apk add --no-cache openssl gettext
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
COPY templates/ /etc/nginx/templates/
ENTRYPOINT ["/entrypoint.sh"]
