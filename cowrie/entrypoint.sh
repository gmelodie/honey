#!/bin/sh
set -e

# Fix ownership of mounted volumes so the cowrie user can write to them
chown -R cowrie:cowrie \
    /cowrie/cowrie-git/var/log/cowrie \
    /cowrie/cowrie-git/var/lib/cowrie

# Install psycopg2 into the venv (as root so the files are world-readable)
/cowrie/cowrie-env/bin/pip install psycopg2-binary -q

# Substitute ${VAR} placeholders in the config template
python3 -c "
import os, re
t = open('/tmp/cowrie.cfg.template').read()
c = re.sub(r'\\\${(\w+)}', lambda m: os.environ.get(m.group(1), m.group(0)), t)
open('/cowrie/cowrie-git/etc/cowrie.cfg', 'w').write(c)
"

# Drop to the cowrie user for the actual process
exec su cowrie -s /bin/bash -c 'exec /cowrie/cowrie-env/bin/twistd -n cowrie'
