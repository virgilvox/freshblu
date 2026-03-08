<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Deploy to Production - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Deploy to Production</h1>
  <p class="doc-intro">Run FreshBlu with PostgreSQL, Redis, and NATS using Docker Compose. Add Caddy for TLS termination.</p>

  <h2>Prerequisites</h2>
  <ul>
    <li>Docker and Docker Compose v2</li>
    <li>A domain name with DNS pointing to your server</li>
    <li>Ports 80, 443, and 1883 available</li>
  </ul>

  <h2>The Stack</h2>
  <p>A production FreshBlu deployment has five services:</p>
  <ul>
    <li><strong>NATS</strong> - message bus for cross-pod event routing (JetStream enabled)</li>
    <li><strong>PostgreSQL</strong> - persistent device and subscription storage</li>
    <li><strong>Redis</strong> - device presence tracking and store cache layer</li>
    <li><strong>Gateway</strong> - HTTP/WS/MQTT server pods (scalable replicas)</li>
    <li><strong>Router</strong> - NATS consumer that resolves subscriptions and fans out deliveries</li>
  </ul>

  <h2>Docker Compose File</h2>
  <p>The project includes <code>docker/docker-compose.prod.yml</code>. Here is the full configuration:</p>
  <CodeBlock lang="yaml" code={`version: "3.8"

services:
  nats:
    image: nats:2.10-alpine
    command: --js --cluster_name freshblu
    ports:
      - "4222:4222"
      - "8222:8222"
    healthcheck:
      test: ["CMD", "nats-server", "--signal", "ldm"]
      interval: 10s
      timeout: 5s
      retries: 3

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: freshblu
      POSTGRES_USER: freshblu
      POSTGRES_PASSWORD: "\${POSTGRES_PASSWORD}"
    volumes:
      - pg_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U freshblu"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  gateway:
    build:
      context: ..
      dockerfile: docker/Dockerfile.gateway
    environment:
      NATS_URL: nats://nats:4222
      DATABASE_URL: postgresql://freshblu:\${POSTGRES_PASSWORD}@postgres/freshblu
      REDIS_URL: redis://redis:6379
      FRESHBLU_HTTP_PORT: "3000"
      FRESHBLU_MQTT_PORT: "1883"
      RUST_LOG: "freshblu=info"
    ports:
      - "3000:3000"
      - "1883:1883"
    depends_on:
      nats: { condition: service_healthy }
      postgres: { condition: service_healthy }
      redis: { condition: service_healthy }
    deploy:
      replicas: 2
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/status"]
      interval: 10s
      timeout: 5s
      retries: 3

  router:
    build:
      context: ..
      dockerfile: docker/Dockerfile.router
    environment:
      NATS_URL: nats://nats:4222
      DATABASE_URL: postgresql://freshblu:\${POSTGRES_PASSWORD}@postgres/freshblu
      REDIS_URL: redis://redis:6379
      RUST_LOG: "freshblu=info"
    depends_on:
      nats: { condition: service_healthy }
      postgres: { condition: service_healthy }
      redis: { condition: service_healthy }
    deploy:
      replicas: 2

volumes:
  pg_data:`} />

  <h2>Set the Postgres Password</h2>
  <p>Create an <code>.env</code> file next to the compose file:</p>
  <CodeBlock lang="bash" code={`echo "POSTGRES_PASSWORD=$(openssl rand -hex 16)" > .env`} />

  <h2>Start the Stack</h2>
  <CodeBlock lang="bash" code={`cd docker
docker compose -f docker-compose.prod.yml up -d`} />
  <p>Docker builds the gateway and router images, then starts all services. Health checks ensure dependencies are ready before the application pods start.</p>

  <h2>Verify</h2>
  <CodeBlock lang="bash" code={`# Check all services are healthy
docker compose -f docker-compose.prod.yml ps

# Hit the status endpoint
curl http://localhost:3000/status

# Check metrics
curl http://localhost:3000/metrics`} />

  <h2>Add TLS with Caddy</h2>
  <p>Add a Caddy service to the compose file for automatic HTTPS:</p>
  <CodeBlock lang="yaml" code={`  caddy:
    image: caddy:2-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
    depends_on:
      - gateway`} />
  <p>Create a <code>Caddyfile</code>:</p>
  <CodeBlock lang="text" code={`freshblu.example.com {
    reverse_proxy gateway:3000
}`} />
  <p>Caddy obtains and renews Let's Encrypt certificates automatically.</p>

  <h2>Scale Gateway Pods</h2>
  <p>Add more gateway replicas to handle higher connection counts:</p>
  <CodeBlock lang="bash" code={`docker compose -f docker-compose.prod.yml up -d --scale gateway=4`} />
  <p>Each gateway registers its own pod ID. NATS and the router handle cross-pod message delivery automatically.</p>

  <h2>Scale Router Pods</h2>
  <p>Router pods can also scale independently. Each consumes from the same NATS subjects using queue groups, so work is distributed evenly.</p>
  <CodeBlock lang="bash" code={`docker compose -f docker-compose.prod.yml up -d --scale router=3`} />

  <h2>Environment Variables</h2>
  <p>Key environment variables for the gateway and router:</p>
  <ul>
    <li><code>NATS_URL</code> - NATS connection string. Triggers NatsBus mode (required for multi-pod).</li>
    <li><code>DATABASE_URL</code> - PostgreSQL connection string.</li>
    <li><code>REDIS_URL</code> - Redis connection string. Enables CachedStore and presence tracking.</li>
    <li><code>FRESHBLU_HTTP_PORT</code> - HTTP/WS listen port (default 3000).</li>
    <li><code>FRESHBLU_MQTT_PORT</code> - MQTT listen port (default 1883).</li>
    <li><code>RUST_LOG</code> - Log level filter.</li>
  </ul>

  <h2>Single-Pod Dev Mode</h2>
  <p>For local development, omit <code>NATS_URL</code> and <code>REDIS_URL</code>. FreshBlu falls back to LocalBus (in-memory messaging) and SQLite. No external services required.</p>
  <CodeBlock lang="bash" code={`cargo run -p freshblu-server`} />
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  ul, ol { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; padding-left: 20px; }
  li { margin-bottom: 4px; }
</style>
