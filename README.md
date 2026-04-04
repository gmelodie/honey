# Cowrie Grafana Dashboard

Grafana dashboard for visualizing [Cowrie](https://github.com/cowrie/cowrie) honeypot data from an existing PostgreSQL database.

## Requirements

- Docker + Docker Compose
- Cowrie already running with PostgreSQL output enabled

## Setup

1. Clone this repo onto the machine running Cowrie/Postgres

2. Edit `.env` with your database credentials:
   ```
   POSTGRES_HOST=localhost
   POSTGRES_PORT=5432
   POSTGRES_DB=cowrie
   POSTGRES_USER=cowrie
   POSTGRES_PASSWORD=cowrie
   ```

3. Start Grafana:
   ```bash
   docker compose up -d
   ```

4. Open [http://localhost:3000](http://localhost:3000) — login `admin` / `admin`

5. Navigate to **Dashboards → Cowrie → Cowrie Dashboard**

## Dashboard Panels

| Panel | Description |
|---|---|
| Total Connections | Session count over selected time range |
| Unique Attacking IPs | Distinct source IPs |
| Auth Attempts | Total login attempts |
| Commands Executed | Total commands run by attackers |
| Top Attacking IPs | Ranked by connection count |
| Top Usernames / Passwords | Most attempted credentials |
| Top Credential Pairs | Username + password combos |
| Failed / Successful Logins | Time series |
| "ubuntu" Failed Logins | Time series for ubuntu account |
| Commands | Full command log with session and source IP |
| Sessions | Raw session log |
| Downloads | Files downloaded by attackers |

## Notes

- `network_mode: host` is used so Grafana can reach Postgres on `localhost`. If Grafana is on a separate machine, set `POSTGRES_HOST` to the Postgres machine's IP.
- Grafana credentials can be changed in `.env` via `GF_SECURITY_ADMIN_USER` / `GF_SECURITY_ADMIN_PASSWORD`.
- `.env` is gitignored — don't commit credentials.
