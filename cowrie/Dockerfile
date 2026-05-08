FROM cowrie/cowrie:latest
USER root
RUN ["/cowrie/cowrie-env/bin/pip", "install", "psycopg2-binary", "-q"]
COPY postgresql_patch.py /cowrie/cowrie-git/src/cowrie/output/postgresql.py
COPY userdb.txt /cowrie/cowrie-git/etc/userdb.txt
COPY entrypoint.py /entrypoint.py
ENTRYPOINT ["/cowrie/cowrie-env/bin/python3", "/entrypoint.py"]
