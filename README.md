# Cowrie Honeypot Dashboard

An SSH/Telnet honeypot that captures attacker sessions and visualises them in a live web dashboard, protected by reCAPTCHA.

**Stack:** [Cowrie](https://github.com/cowrie/cowrie) → PostgreSQL → Flask web app → Nginx (SSL)

---

## How it works

Cowrie listens on ports 2222/2223 (redirected from 22/23 via iptables). Every session, login attempt, command, and file download is stored in PostgreSQL. The web dashboard queries PostgreSQL and serves live stats at your public domain behind HTTPS and a reCAPTCHA v3 gate. Nginx also serves a fake router admin panel to any unknown HTTP/HTTPS clients (the web honeypot).

---

## Prerequisites

- A Linux VPS with a public IP and a domain name pointed at it
- Docker and Docker Compose v2
- Root / sudo access

---

## Setup

### 1. Clone the repo

```bash
git clone <repo-url> ~/cowrie-dashboard
cd ~/cowrie-dashboard
```

### 2. Move your real SSH to a different port

Cowrie will intercept port 22. Do this **before** touching iptables or you will lock yourself out.

```bash
# Add a high port to sshd (e.g. 22222)
echo "Port 22222" | sudo tee -a /etc/ssh/sshd_config
sudo systemctl restart sshd
```

Open a **new terminal** and confirm you can still connect:

```bash
ssh -p 22222 user@your-server
```

Only continue once the new port works.

### 3. Configure environment variables

```bash
cp .env.example .env
nano .env
```

Required variables:

```env
# PostgreSQL
POSTGRES_HOST=127.0.0.1
POSTGRES_PORT=5432
POSTGRES_DB=cowrie
POSTGRES_USER=cowrie
POSTGRES_PASSWORD=strong-password-here

# Nginx / SSL
TARGET_HOST=honey.example.com

# reCAPTCHA v3 — see step 4
RECAPTCHA_SITE_KEY=
RECAPTCHA_SECRET_KEY=
```

### 4. Set up reCAPTCHA v3

1. Go to [google.com/recaptcha/admin](https://www.google.com/recaptcha/admin) and create a new key
2. Choose **reCAPTCHA v3**
3. Add your domain (e.g. `honey.example.com`)
4. Copy the **Site Key** and **Secret Key** into `.env`

To disable the gate entirely (no CAPTCHA), leave both keys empty — the dashboard will be accessible directly.

### 5. Start the stack

```bash
docker compose up -d
```

Services started:

| Service | Role |
|---|---|
| `postgres` | Stores all honeypot data |
| `postgres-init` | Applies the DB schema (runs once, exits) |
| `cowrie` | SSH/Telnet honeypot on ports 2222/2223 |
| `web` | Stats dashboard on localhost:8373 |
| `honeypot-web` | Fake router admin panel on localhost:8374 |
| `nginx` | Routes port 80/443 by domain name |
| `certbot` | Renews SSL certificates every 12 hours |
| `grafana` | Grafana dashboard on port 47321 (optional) |
| `wordlists` | Generates credential wordlists from captured data |

### 6. Obtain an SSL certificate

Run once after the stack is up:

```bash
TARGET_HOST=honey.example.com CERTBOT_EMAIL=you@example.com ./scripts/init-letsencrypt.sh
```

Then restart nginx to load the certificate:

```bash
docker compose restart nginx
```

### 7. Redirect ports 22 and 23 to Cowrie

Cowrie listens on 2222/2223 (non-privileged). This iptables rule redirects real internet traffic:

```bash
sudo bash scripts/setup-port-redirect.sh
```

Make the rules persist across reboots:

```bash
sudo apt-get install -y iptables-persistent
sudo netfilter-persistent save
```

### 8. Verify everything is working

Check Cowrie is listening and logging:

```bash
docker compose logs cowrie --tail=20
```

Try connecting from another machine — you should hit the honeypot:

```bash
ssh root@your-server-ip    # should get a fake shell
```

Check for data in the database:

```bash
docker compose exec postgres psql -U $POSTGRES_USER -d $POSTGRES_DB \
  -c "SELECT COUNT(*) FROM sessions; SELECT COUNT(*) FROM auth;"
```

Open the dashboard at `https://honey.example.com`.

---

## Useful commands

```bash
# Follow all logs
docker compose logs -f

# Follow a single service
docker compose logs -f cowrie

# Rebuild and restart after code changes
docker compose up -d --build

# Restart a single service
docker compose restart web

# Stop everything (keeps data)
docker compose down

# Wipe all data and start fresh (destructive)
docker compose down -v
```

---

## Optional: restrict Grafana to specific IPs

Grafana runs on port 47321. The `scripts/` directory includes helpers to restrict it to a country's IP ranges using `ipset`:

```bash
sudo apt install ipset iptables-persistent

# Build the ipset and apply firewall rules (example: Brazil)
sudo ./scripts/setup-brazil-ipset.sh

# Keep IP ranges fresh — add to cron (daily at 02:00)
echo "0 2 * * * root $(pwd)/scripts/update-brazil-ipset.sh >> /var/log/brazil-ipset.log 2>&1" \
  | sudo tee /etc/cron.d/brazil-ipset
```

---

## Notes

- `network_mode: host` is used throughout — all containers share the host network and reach each other via `127.0.0.1`.
- `.env` is gitignored. Never commit it.
- `cowrie/var/` is gitignored (runtime logs and captured files). Back it up if you want to preserve captures.
- The port redirect (`setup-port-redirect.sh`) is idempotent — safe to run multiple times.
