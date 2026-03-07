#!/usr/bin/env bash
# Pull latest code and rebuild/restart app containers (gateway + router).
# Infrastructure services (postgres, redis, nats, caddy) are not restarted.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
COMPOSE_DIR="$(dirname "$SCRIPT_DIR")"

echo "==> Pulling latest code..."
cd "$REPO_DIR"
git pull --ff-only

echo "==> Building app containers..."
cd "$COMPOSE_DIR"
docker compose build gateway router

echo "==> Restarting app containers..."
docker compose up -d --no-deps gateway router

echo "==> Done. Current status:"
docker compose ps
