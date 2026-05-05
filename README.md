# Cowrie Honeypot Stack

A self-contained SSH/Telnet honeypot that captures attacker sessions and visualises them in Grafana.

**Stack:** [Cowrie](https://github.com/cowrie/cowrie) → PostgreSQL → Grafana

---

## How it works

Cowrie listens on ports 22 (SSH) and 23 (Telnet), presenting a fake shell to attackers. Every session, login attempt, command, and file download is stored in PostgreSQL and surfaced through the Grafana dashboard.

---

## Prerequisites

- A dedicated Linux server or VPS with a public IP
- Docker and Docker Compose v2
- Root or sudo access
- Ports 22 and 23 free for Cowrie (see step 2 below)

---

## Setup

### 1. Clone the repo

```bash
git clone <repo-url> /opt/honeypot
cd /opt/honeypot
```

### 2. Move your real SSH to a different port

Cowrie needs to own port 22. Before deploying, move your SSH daemon so you don't lock yourself out:

```bash
# Edit /etc/ssh/sshd_config and change:
#   Port 22
# to:
#   Port 2222   (or any high port you prefer)

sudo sed -i 's/^#\?Port 22$/Port 2222/' /etc/ssh/sshd_config
sudo systemctl restart sshd

# Reconnect on the new port, then verify the old one is free:
ss -tlnp | grep ':22'
```

> **Do this before starting the stack.** Once Cowrie binds port 22, your existing SSH session stays alive but new connections on 22 go to the honeypot.

### 3. Configure credentials

```bash
cp .env .env.local  # optional backup
```

Edit `.env`:

```env
POSTGRES_HOST=127.0.0.1
POSTGRES_PORT=5432
POSTGRES_DB=cowrie
POSTGRES_USER=cowrie
POSTGRES_PASSWORD=change-me-strong-password

GF_SECURITY_ADMIN_USER=admin
GF_SECURITY_ADMIN_PASSWORD=change-me-grafana-password
```

### 4. Review Cowrie config (optional)

`config/cowrie.cfg` has sane defaults. Notable options:

```ini
[ssh]
enabled = true
listen_endpoints = tcp:22:interface=0.0.0.0

[telnet]
enabled = true
listen_endpoints = tcp:23:interface=0.0.0.0

[output_postgresql]
enabled = true
host = 127.0.0.1
port = 5432
database = cowrie
username = cowrie
password = strongpassword   # must match POSTGRES_PASSWORD in .env
```

Update the `password` in `cowrie.cfg` to match `POSTGRES_PASSWORD` in `.env` if you changed it.

### 5. Start the stack

```bash
docker compose up -d
```

This starts four services in order:

| Service | Role |
|---|---|
| `postgres` | Stores all honeypot data |
| `postgres-init` | Applies the schema (runs once, exits) |
| `cowrie` | SSH/Telnet honeypot on ports 22 and 23 |
| `grafana` | Dashboard on port 47321 |

### 6. Verify Cowrie is capturing

```bash
# Watch live logs
docker logs -f cowrie

# Try connecting from another machine (you should get a fake shell)
ssh root@<your-server-ip>

# Check the database
docker exec -it postgres psql -U cowrie -d cowrie -c "SELECT COUNT(*) FROM sessions;"
```

### 7. Open the dashboard

Navigate to `http://<your-server-ip>:47321` and log in with the credentials from `.env`.

Go to **Dashboards → Cowrie → Cowrie Dashboard**.

---

## Optional: Restrict Grafana access by IP

The `scripts/` directory includes helpers to lock Grafana (port 47321) to a specific country's IPs using `ipset` and `iptables`. The example restricts to Brazilian IPs — adapt the scripts for your country/CIDR if needed.

```bash
# Install ipset
sudo apt install ipset iptables-persistent

# Run once to build the ipset and apply firewall rules
sudo ./scripts/setup-brazil-ipset.sh

# Add to cron to keep IP ranges fresh (runs daily at 02:00)
echo "0 2 * * * root /opt/honeypot/scripts/update-brazil-ipset.sh >> /var/log/brazil-ipset.log 2>&1" \
  | sudo tee /etc/cron.d/brazil-ipset
```

---

## Log rotation (80 GB cap)

Cowrie stores JSON logs and downloaded malware samples on disk. A rotation script deletes the oldest files when combined usage exceeds 80 GB:

```bash
# Test manually
sudo ./scripts/rotate-cowrie-logs.sh

# Install as an hourly cron job
echo "0 * * * * root /opt/honeypot/scripts/rotate-cowrie-logs.sh 2>&1 | logger -t cowrie-rotate" \
  | sudo tee /etc/cron.d/cowrie-rotate
```

Logs go to syslog (`journalctl -t cowrie-rotate`) and show each deleted file with its size.

---

## Dashboard panels

| Panel | Description |
|---|---|
| Total Connections | Session count over the selected time range |
| Unique Attacking IPs | Distinct source IPs |
| Auth Attempts | Total login attempts |
| Commands Executed | Total commands run by attackers |
| Top Attacking IPs | Ranked by connection count |
| Top Usernames / Passwords | Most attempted credentials |
| Top Credential Pairs | Username + password combos |
| Failed / Successful Logins | Time series |
| Success Rate Over Time | Login success % as a time series |
| Unique Malware Hashes | Count of distinct SHA256s seen |
| Top Downloaded URLs | Most-fetched URLs by download count |
| Malware Hashes | SHA256 hash table with download counts |
| Attacks by Hour of Day | Bar chart of attempts grouped by UTC hour |
| Attacks by Day of Week | Bar chart of attempts grouped by day |
| SSH Client Versions | Ranked table of attacker SSH client strings |
| Commands | Full command log with session and source IP |
| Sessions | Raw session log |
| Downloads | Files downloaded by attackers |

---

## Useful commands

```bash
# View all service logs
docker compose logs -f

# Restart a single service
docker compose restart cowrie

# Stop everything
docker compose down

# Wipe all data and start fresh (destructive)
docker compose down -v
sudo rm -rf cowrie/
```

## Notes

- `network_mode: host` is used throughout so containers can reach each other on `localhost`. Grafana, Postgres, and Cowrie all bind directly to the host network.
- `.env` is gitignored — never commit credentials.
- Cowrie data persists in `./cowrie/` on the host. Back this up if you want to preserve captures.
