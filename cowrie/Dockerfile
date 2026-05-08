FROM cowrie/cowrie:latest
USER root
RUN ["/cowrie/cowrie-env/bin/pip", "install", "psycopg2-binary", "-q"]
COPY entrypoint.py /entrypoint.py
ENTRYPOINT ["/cowrie/cowrie-env/bin/python3", "/entrypoint.py"]
