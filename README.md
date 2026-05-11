# honey

SSH/Telnet honeypot that captures attacker sessions and serves them in a live web dashboard with downloadable credential wordlists.

**Stack:** Cowrie → PostgreSQL → Flask → Nginx (SSL + reCAPTCHA)

Cowrie listens on 2222/2223 (redirected from 22/23 via iptables). Sessions, logins, commands, and downloads go to PostgreSQL. Stats and wordlists are precomputed by background workers and served instantly by the web server.

---

## Prerequisites

- Linux VPS with a public IP and a domain pointed at it
- Docker and Docker Compose v2
- Root / sudo access

---

## Setup

### 1. Clone

```bash
git clone <repo-url> ~/honey
cd ~/honey
```

### 2. Move your real SSH off port 22

Do this **before** touching iptables or you will lock yourself out.

```bash
echo "Port 22222" | sudo tee -a /etc/ssh/sshd_config
sudo systemctl restart sshd
```

Open a new terminal and confirm login on the new port before continuing.

### 3. Configure .env

```bash
cp .env.example .env
nano .env
```

```env
POSTGRES_HOST=127.0.0.1
POSTGRES_PORT=5432
POSTGRES_DB=cowrie
POSTGRES_USER=cowrie
POSTGRES_PASSWORD=strong-password-here

TARGET_HOST=honey.example.com

RECAPTCHA_SITE_KEY=
RECAPTCHA_SECRET_KEY=
```

Get reCAPTCHA keys at [google.com/recaptcha/admin](https://www.google.com/recaptcha/admin) — add your domain, choose v2 Checkbox, paste both keys. Leave blank to disable the gate.

### 4. Start the stack

```bash
docker compose up -d
```

| Service | Role |
|---|---|
| `postgres` | Stores all honeypot data |
| `postgres-init` | Applies DB schema (runs once, exits) |
| `cowrie` | SSH/Telnet honeypot on 2222/2223 |
| `stats-gen` | Precomputes dashboard stats every 5 minutes |
| `wordlist-gen` | Generates credential wordlists every 6 hours |
| `web` | Dashboard on localhost:8373 |
| `nginx` | Routes 80/443 by domain |
| `certbot` | Renews SSL every 12 hours |

### 5. Obtain SSL certificate

```bash
TARGET_HOST=honey.example.com CERTBOT_EMAIL=you@example.com ./scripts/init-letsencrypt.sh
docker compose restart nginx
```

### 6. Redirect ports 22 and 23 to Cowrie

```bash
sudo bash scripts/setup-port-redirect.sh
sudo apt-get install -y iptables-persistent
sudo netfilter-persistent save
```

### 7. Verify

```bash
ssh root@your-server-ip    # should land in a fake shell
```

Open `https://honey.example.com`.

---

## Importing existing Cowrie logs

If you have existing Cowrie JSON logs, import them into the database:

```bash
pip install psycopg2-binary python-dotenv
python3 scripts/import-cowrie-json.py /path/to/cowrie.json*
```

- Reads DB credentials from `.env` automatically
- Safe to re-run on overlapping files — duplicates are skipped
- After import, `stats-gen` and `wordlist-gen` will pick up the new data on their next run

---

## Notes

- All containers use `network_mode: host` and communicate via `127.0.0.1`.
- `.env` is gitignored — never commit it.
- Dashboard stats refresh every 5 minutes; wordlists every 6 hours.
- `setup-port-redirect.sh` is idempotent — safe to re-run.
