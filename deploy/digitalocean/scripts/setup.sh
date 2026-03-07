#!/usr/bin/env bash
# Manual setup script for deploying FreshBlu on an existing Ubuntu droplet.
# Run as root or with sudo.
set -euo pipefail

DATA_PATH="${DATA_PATH:-/mnt/freshblu-data}"
VOLUME_DEVICE="/dev/disk/by-id/scsi-0DO_Volume_freshblu-data"

echo "==> Updating system packages..."
apt-get update && apt-get upgrade -y

echo "==> Installing Docker..."
if ! command -v docker &>/dev/null; then
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
    chmod a+r /etc/apt/keyrings/docker.asc
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" > /etc/apt/sources.list.d/docker.list
    apt-get update
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
fi

echo "==> Setting up block storage volume..."
if [ -e "$VOLUME_DEVICE" ]; then
    blkid "$VOLUME_DEVICE" || mkfs.ext4 "$VOLUME_DEVICE"
    mkdir -p "$DATA_PATH"
    mount -o defaults,nofail,discard "$VOLUME_DEVICE" "$DATA_PATH" 2>/dev/null || true
    grep -q freshblu-data /etc/fstab || echo "$VOLUME_DEVICE $DATA_PATH ext4 defaults,nofail,discard 0 2" >> /etc/fstab
else
    echo "WARNING: Block storage volume not found at $VOLUME_DEVICE"
    echo "         Data will be stored at $DATA_PATH on the root filesystem."
    mkdir -p "$DATA_PATH"
fi

echo "==> Creating data directories..."
mkdir -p "$DATA_PATH"/{postgres,redis,nats,caddy/data,caddy/config,backups}

echo "==> Configuring firewall..."
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 1883/tcp
ufw --force enable

echo "==> Setting up daily backup cron..."
cat > /etc/cron.d/freshblu-backup << 'CRON'
0 2 * * * root /opt/freshblu/deploy/digitalocean/scripts/backup.sh >> /var/log/freshblu-backup.log 2>&1
CRON
chmod 644 /etc/cron.d/freshblu-backup

echo ""
echo "Setup complete. Next steps:"
echo "  1. Clone the repo:  git clone <your-repo-url> /opt/freshblu"
echo "  2. Configure env:   cp /opt/freshblu/deploy/digitalocean/.env.example /opt/freshblu/deploy/digitalocean/.env"
echo "  3. Edit .env:       nano /opt/freshblu/deploy/digitalocean/.env"
echo "  4. Start services:  cd /opt/freshblu/deploy/digitalocean && docker compose up -d"
