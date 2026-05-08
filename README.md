# Cowrie Honeypot Dashboard

SSH/Telnet honeypot that captures attacker sessions and displays them in a live web dashboard.

**Stack:** Cowrie → PostgreSQL → Flask → Nginx (SSL + reCAPTCHA v3)

Cowrie listens on 2222/2223 (redirected from 22/23 via iptables). Sessions, logins, commands, and downloads go to PostgreSQL. Nginx serves the dashboard at your domain over HTTPS and fronts a fake router admin panel for unknown HTTP/HTTPS clients.

---

## Prerequisites

- Linux VPS with a public IP and a domain pointed at it
- Docker and Docker Compose v2
- Root / sudo access

---

## Setup

### 1. Clone

```bash
git clone <repo-url> ~/cowrie-dashboard
cd ~/cowrie-dashboard
```

### 2. Move your real SSH off port 22

Do this **before** touching iptables or you will lock yourself out.

```bash
echo "Port 22222" | sudo tee -a /etc/ssh/sshd_config
sudo systemctl restart sshd
```

Open a new terminal and confirm login works on the new port before continuing:

```bash
ssh -p 22222 user@your-server
```

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

Get reCAPTCHA v3 keys at [google.com/recaptcha/admin](https://www.google.com/recaptcha/admin) — add your domain, choose v3, paste both keys. Leave both empty to disable the gate.

### 4. Start the stack

```bash
docker compose up -d
```

| Service | Role |
|---|---|
| `postgres` | Stores all honeypot data |
| `postgres-init` | Applies DB schema (runs once, exits) |
| `cowrie` | SSH/Telnet honeypot on 2222/2223 |
| `web` | Stats dashboard on localhost:8373 |
| `honeypot-web` | Fake router admin panel on localhost:8374 |
| `nginx` | Routes 80/443 by domain |
| `certbot` | Renews SSL every 12 hours |
| `wordlists` | Generates credential wordlists from captures |

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
docker compose logs cowrie --tail=20
```

```bash
ssh root@your-server-ip    # should land in a fake shell
```

```bash
docker compose exec postgres psql -U $POSTGRES_USER -d $POSTGRES_DB \
  -c "SELECT COUNT(*) FROM sessions; SELECT COUNT(*) FROM auth;"
```

Open `https://honey.example.com`.

---

## Useful commands

```bash
docker compose logs -f                  # all logs
docker compose logs -f cowrie           # single service
docker compose up -d --build            # rebuild after code changes
docker compose restart web              # restart one service
docker compose down                     # stop (keeps data)
docker compose down -v                  # stop and wipe all data
```

---

## Notes

- All containers use `network_mode: host` and communicate via `127.0.0.1`.
- `.env` is gitignored — never commit it.
- `cowrie/var/` is gitignored. Back it up if you want to preserve captured files.
- `setup-port-redirect.sh` is idempotent — safe to re-run.
