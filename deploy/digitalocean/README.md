# Deploy FreshBlu to DigitalOcean

Deploy the full FreshBlu stack (gateway, router, PostgreSQL, Redis, NATS) on a single DigitalOcean droplet with automatic HTTPS via Caddy + Let's Encrypt.

## Architecture

```
Internet
  |
  |-- :80/:443 --> Caddy --> gateway:3000  (HTTP/WS + REST API)
  |-- :1883 ----> gateway:1883            (MQTT)
  |
  gateway <--> NATS <--> router
  gateway <--> PostgreSQL
  gateway <--> Redis (cache + presence)
```

All persistent data lives on a DO Block Storage volume mounted at `/mnt/freshblu-data/`.

## Prerequisites

- DigitalOcean account
- A domain name with DNS managed (you'll add an A record)
- A DO Block Storage volume named `freshblu-data` attached to your droplet

## Recommended Specs

| Workload   | Droplet     | Volume |
|------------|-------------|--------|
| Dev/test   | s-1vcpu-2gb | 10 GB  |
| Production | s-2vcpu-4gb | 25 GB  |

## Quick Start

### Option A: DigitalOcean Console (cloud-init)

1. Create a Block Storage volume named `freshblu-data` in your region.
2. Create a Droplet (Ubuntu 22.04+), attach the volume, and paste `cloud-init.yml` into **User Data**.
3. Point your domain's A record to the droplet IP.
4. SSH in and configure:
   ```bash
   cd /opt/freshblu/deploy/digitalocean
   cp .env.example .env
   nano .env        # set DOMAIN, POSTGRES_PASSWORD, FRESHBLU_PEPPER
   docker compose up -d
   ```

### Option B: doctl

```bash
# Create volume
doctl compute volume create freshblu-data --region nyc1 --size 25GiB

# Create droplet with cloud-init
doctl compute droplet create freshblu \
  --region nyc1 \
  --size s-2vcpu-4gb \
  --image ubuntu-22-04-x64 \
  --volumes $(doctl compute volume list --format ID --no-header) \
  --user-data-file deploy/digitalocean/cloud-init.yml \
  --ssh-keys $(doctl compute ssh-key list --format ID --no-header | head -1)

# Then SSH in, configure .env, and docker compose up -d
```

### Option C: Manual Setup

```bash
ssh root@your-droplet-ip
git clone https://github.com/YOUR_ORG/freshblu.git /opt/freshblu
cd /opt/freshblu/deploy/digitalocean
bash scripts/setup.sh
cp .env.example .env
nano .env
docker compose up -d
```

## DNS Setup

Create an A record pointing your domain to the droplet's public IP:

```
Type  Name              Value
A     freshblu          203.0.113.50
```

Caddy will automatically obtain a Let's Encrypt certificate once DNS propagates.

## Services & Ports

| Service    | Internal Port | External Port | Protocol    |
|------------|---------------|---------------|-------------|
| Caddy      | 80, 443       | 80, 443       | HTTP/HTTPS  |
| Gateway    | 3000          | (via Caddy)   | HTTP/WS     |
| Gateway    | 1883          | 1883          | MQTT        |
| NATS       | 4222          | -             | Internal    |
| PostgreSQL | 5432          | -             | Internal    |
| Redis      | 6379          | -             | Internal    |

## Configuration

All configuration is in `.env`. See `.env.example` for the full list of variables.

Key variables:
- `DOMAIN` - Your domain name (required for HTTPS)
- `POSTGRES_PASSWORD` - Database password
- `FRESHBLU_PEPPER` - Bcrypt pepper for token security
- `FRESHBLU_OPEN_REGISTRATION` - Set `false` to require auth for device registration

## Updating

Pull latest code and rebuild app containers without touching infrastructure:

```bash
cd /opt/freshblu/deploy/digitalocean
bash scripts/update.sh
```

## Backups

A daily cron job runs at 2 AM, dumping PostgreSQL and snapshotting Redis to `/mnt/freshblu-data/backups/`. Backups older than 7 days are pruned automatically.

Run a backup manually:

```bash
bash /opt/freshblu/deploy/digitalocean/scripts/backup.sh
```

## Monitoring

FreshBlu exposes Prometheus metrics at `/metrics`. By default, Caddy blocks public access to this endpoint (returns 403). To scrape metrics, either:

- SSH tunnel: `ssh -L 3000:localhost:3000 root@your-droplet` then `curl http://localhost:3000/metrics`
- Or remove the `@metrics` block from `Caddyfile` to expose it publicly

View container logs:

```bash
cd /opt/freshblu/deploy/digitalocean
docker compose logs -f gateway
docker compose logs -f router
```

## Security Notes

- **Firewall**: UFW is configured to allow only ports 22, 80, 443, and 1883.
- **Internal services**: PostgreSQL, Redis, and NATS are not exposed to the internet.
- **HTTPS**: Caddy handles TLS automatically via Let's Encrypt.
- **MQTT**: Port 1883 is unencrypted. For production use with sensitive data, consider adding a TLS termination proxy or using WebSocket-based MQTT over HTTPS.
- **Secrets**: Never commit your `.env` file. Generate strong random values for `POSTGRES_PASSWORD` and `FRESHBLU_PEPPER`.

## Troubleshooting

**Caddy won't get a certificate**
- Verify DNS A record points to the droplet IP: `dig +short your-domain.com`
- Check Caddy logs: `docker compose logs caddy`
- Ensure ports 80 and 443 are open: `ufw status`

**Gateway won't start**
- Check that PostgreSQL is healthy: `docker compose ps postgres`
- View gateway logs: `docker compose logs gateway`

**Volume not mounted**
- Verify the volume is attached: `lsblk`
- Check fstab: `cat /etc/fstab | grep freshblu`
- Mount manually: `mount /mnt/freshblu-data`

**Build fails on small droplet**
- Rust builds are memory-intensive. Use at least 2 GB RAM, or build images on a larger machine and push to a container registry, then change `docker-compose.yml` to use `image:` instead of `build:`.

**MQTT not connecting**
- Verify port 1883 is open: `ufw status`
- Check gateway logs for MQTT broker errors: `docker compose logs gateway | grep mqtt`
