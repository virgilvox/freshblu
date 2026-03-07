#!/usr/bin/env bash
# Backup PostgreSQL and Redis data.
# Intended to run via cron. Keeps 7 days of backups.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_DIR="$(dirname "$SCRIPT_DIR")"
DATA_PATH="${DATA_PATH:-/mnt/freshblu-data}"
BACKUP_DIR="$DATA_PATH/backups"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"

mkdir -p "$BACKUP_DIR"

echo "[$(date)] Starting backup..."

# PostgreSQL dump
docker compose -f "$COMPOSE_DIR/docker-compose.yml" exec -T postgres \
    pg_dump -U freshblu freshblu | gzip > "$BACKUP_DIR/postgres-$TIMESTAMP.sql.gz"
echo "  PostgreSQL dump: postgres-$TIMESTAMP.sql.gz"

# Redis RDB snapshot
docker compose -f "$COMPOSE_DIR/docker-compose.yml" exec -T redis \
    redis-cli BGSAVE >/dev/null 2>&1
sleep 2
if [ -f "$DATA_PATH/redis/dump.rdb" ]; then
    cp "$DATA_PATH/redis/dump.rdb" "$BACKUP_DIR/redis-$TIMESTAMP.rdb"
    echo "  Redis snapshot:  redis-$TIMESTAMP.rdb"
fi

# Prune backups older than 7 days
find "$BACKUP_DIR" -name "postgres-*.sql.gz" -mtime +7 -delete
find "$BACKUP_DIR" -name "redis-*.rdb" -mtime +7 -delete

echo "[$(date)] Backup complete."
