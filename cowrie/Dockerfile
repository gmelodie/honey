FROM cowrie/cowrie:3.1.2@sha256:019726c8deef20729162cda492d9385917b579c9ace6368e47e7566cd2491ba1
USER root
RUN ["/cowrie/cowrie-env/bin/python3", "-c", "import psycopg2"]
COPY postgresql_patch.py /cowrie/cowrie-git/src/cowrie/output/postgresql.py
COPY userdb.txt /cowrie/cowrie-git/etc/userdb.txt
COPY entrypoint.py persona.py /
ENTRYPOINT ["/cowrie/cowrie-env/bin/python3", "/entrypoint.py"]
