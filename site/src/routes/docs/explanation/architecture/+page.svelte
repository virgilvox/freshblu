<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Architecture - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Architecture</h1>
  <p class="doc-intro">How FreshBlu is structured internally. Crate boundaries, the Hub/Bus abstraction, the Gateway/Router pod model, and shared application state.</p>

  <h2>Crate Structure</h2>
  <p>FreshBlu is a Cargo workspace with six crates. Each has a single responsibility.</p>
  <ul>
    <li><strong>freshblu-core</strong> &mdash; Domain types. Devices, permissions, subscriptions, forwarders, messages, tokens, auth. No I/O. Feature-gated: the <code>auth</code> feature enables bcrypt hashing.</li>
    <li><strong>freshblu-store</strong> &mdash; The <code>DeviceStore</code> trait and its implementations. SQLite (default), PostgreSQL (<code>postgres</code> feature), and Redis-backed <code>CachedStore</code> (<code>cache</code> feature). All storage is async.</li>
    <li><strong>freshblu-proto</strong> &mdash; Wire types shared between the gateway and router. <code>NatsEvent</code>, <code>DeliveryEnvelope</code>, and NATS subject helpers.</li>
    <li><strong>freshblu-server</strong> &mdash; The HTTP/WS/MQTT gateway binary. Built on axum and rumqttd. Contains the <code>MessageBus</code> trait, request handlers, webhook executor, presence tracker, rate limiter, and Prometheus metrics.</li>
    <li><strong>freshblu-router</strong> &mdash; The NATS consumer binary. Reads device events from NATS, resolves subscriptions via the store, and fans out <code>DeliveryEnvelope</code>s to the correct gateway pod.</li>
    <li><strong>freshblu-cli</strong> &mdash; Command-line tool compatible with meshblu-util. Built with clap v4.</li>
  </ul>

  <h2>AppState</h2>
  <p>The gateway server holds shared state in <code>AppState</code>, passed to every handler via axum's <code>State</code> extractor.</p>
  <CodeBlock lang="rust" code={`pub struct AppState {
    pub store: DynStore,          // Arc<dyn DeviceStore>
    pub bus: DynBus,              // Arc<dyn MessageBus>
    pub config: ServerConfig,
    pub rate_limiter: Arc<RateLimiter>,
    pub webhook_executor: Arc<WebhookExecutor>,
}`} />
  <p><code>DynStore</code> and <code>DynBus</code> are trait objects. The concrete implementations are chosen at startup based on environment variables.</p>

  <h2>The MessageBus Trait</h2>
  <p>All event delivery goes through the <code>MessageBus</code> trait. This abstraction lets FreshBlu run as a single process or across many pods without changing handler code.</p>
  <CodeBlock lang="rust" code={`#[async_trait]
pub trait MessageBus: Send + Sync + 'static {
    async fn publish(&self, target: &Uuid, event: DeviceEvent) -> anyhow::Result<()>;
    async fn publish_many(&self, targets: &[Uuid], event: DeviceEvent) -> anyhow::Result<()>;
    fn connect(&self, uuid: Uuid) -> broadcast::Receiver<DeviceEvent>;
    fn disconnect(&self, uuid: &Uuid);
    fn is_online(&self, uuid: &Uuid) -> bool;
    fn online_count(&self) -> usize;
}`} />
  <p>Two implementations exist:</p>

  <h3>LocalBus</h3>
  <p>Wraps <code>MessageHub</code>, an in-memory broadcast map. All delivery happens within the same process. Used when no <code>NATS_URL</code> is set. Suitable for development and single-pod deployments.</p>

  <h3>NatsBus</h3>
  <p>Combines a local <code>MessageHub</code> with a NATS client. If the target device is connected to this pod, delivery is local. Otherwise the event is published to the device's NATS inbox subject (<code>freshblu.device.{'{uuid}'}</code>). The router picks it up, resolves subscriptions, and sends a <code>DeliveryEnvelope</code> back to the correct gateway pod's delivery subject (<code>freshblu.delivery.{'{pod_id}'}</code>).</p>
  <p><code>Ready</code> and <code>NotReady</code> events are local-only. They are never published to NATS.</p>

  <h2>Gateway/Router Pod Model</h2>
  <p>In production, the system runs two types of pods:</p>

  <h3>Gateway Pods</h3>
  <p>Handle client connections over HTTP, WebSocket, and MQTT. Each gateway pod has a unique pod ID. When a device connects, the gateway records its presence in Redis (<code>freshblu:presence:{'{uuid}'}</code> maps to the pod ID). The gateway listens on its NATS delivery subject for inbound events from the router.</p>

  <h3>Router Pods</h3>
  <p>Consume events from NATS device inbox subjects. For each event, the router:</p>
  <ol>
    <li>Queries the store for subscribers matching the event type.</li>
    <li>Looks up each subscriber's pod ID in Redis.</li>
    <li>Wraps the event in a <code>DeliveryEnvelope</code> and publishes it to the target pod's delivery subject.</li>
  </ol>
  <p>Router pods use NATS queue groups, so multiple routers distribute the work without duplication.</p>

  <h2>Event Flow: Direct Message</h2>
  <p>Device A sends a message to Device B. Both are on different gateway pods.</p>
  <ol>
    <li>Gateway 1 receives the HTTP POST from Device A.</li>
    <li>Handler checks <code>message.from</code> permission on Device B.</li>
    <li>Gateway 1 publishes a <code>NatsEvent::Message</code> to <code>freshblu.device.{'{B}'}</code>.</li>
    <li>The router consumes the event from NATS.</li>
    <li>Router looks up <code>message.received</code> subscribers of B and <code>message.sent</code> subscribers of A.</li>
    <li>Router queries Redis for the pod IDs of B and all subscribers.</li>
    <li>Router publishes <code>DeliveryEnvelope</code>s to each target pod's delivery subject.</li>
    <li>Gateway 2 receives the envelope, deserializes it, and delivers the event to Device B's WebSocket/MQTT connection.</li>
  </ol>

  <h2>Event Flow: Broadcast</h2>
  <p>Same pattern, but the router looks up <code>broadcast.sent</code> subscribers of the emitter instead of a single target. Each subscriber gets a <code>DeliveryEnvelope</code> routed to their gateway pod. The router also resolves <code>broadcast.received</code> subscribers of each recipient for secondary fan-out.</p>

  <h2>Store Layer</h2>
  <p>The <code>DeviceStore</code> trait defines all persistence operations: register, get, update, unregister, search, authenticate, subscriptions, tokens. Three backends:</p>
  <ul>
    <li><strong>SQLite</strong> &mdash; default, zero-config, uses PRAGMA foreign_keys with CASCADE deletes on subscriptions.</li>
    <li><strong>PostgreSQL</strong> &mdash; production backend, activated by <code>DATABASE_URL</code>.</li>
    <li><strong>CachedStore</strong> &mdash; wraps any store with a Redis read-through cache. Activated by <code>REDIS_URL</code>.</li>
  </ul>

  <h2>Feature Flags</h2>
  <p>Compile-time feature flags control which backends are included:</p>
  <ul>
    <li><code>freshblu-core</code>: <code>auth</code> (default) gates bcrypt/rand/sha2 dependencies.</li>
    <li><code>freshblu-store</code>: <code>sqlite</code> (default), <code>postgres</code>, <code>cache</code>.</li>
    <li><code>freshblu-server</code>: mirrors the store flags.</li>
  </ul>
  <p>For WASM builds, disable the <code>auth</code> feature to drop native crypto dependencies. The <code>freshblu-wasm</code> crate demonstrates this.</p>
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
