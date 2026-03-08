<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Subscriptions - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Subscriptions</h1>
  <p class="doc-intro">Subscriptions connect devices in the event system. They define who receives which events from whom. This page explains what subscriptions do, the eight types, fan-out mechanics, and how the router resolves subscribers.</p>

  <h2>What Subscriptions Do</h2>
  <p>A subscription is a record with three fields:</p>
  <CodeBlock lang="rust" code={`pub struct Subscription {
    pub emitter_uuid: Uuid,         // the device producing events
    pub subscriber_uuid: Uuid,      // the device receiving events
    pub subscription_type: SubscriptionType,  // which event to listen for
}`} />
  <p>When the emitter produces an event matching the subscription type, FreshBlu delivers a copy to the subscriber. Without a subscription, events are only delivered to their direct target (for messages) or not at all (for broadcasts).</p>

  <h2>The 8 Subscription Types</h2>
  <p>Subscription types are organized in four categories, each with a sent/received pair.</p>

  <h3>broadcast.sent / broadcast.received</h3>
  <p><code>broadcast.sent</code> is the primary broadcast mechanism. When a device sends a broadcast (<code>devices: ["*"]</code>), the system looks up all <code>broadcast.sent</code> subscribers of the sender and delivers the event to each. Without subscribers, a broadcast goes nowhere.</p>
  <p><code>broadcast.received</code> provides secondary fan-out. If Device C subscribes to <code>broadcast.received</code> on Device B, and Device B receives a broadcast from Device A, Device C also gets a copy.</p>

  <h3>message.sent / message.received</h3>
  <p><code>message.sent</code> lets a third party monitor all direct messages sent by a device. If Device C subscribes to <code>message.sent</code> on Device A, every message A sends is copied to C.</p>
  <p><code>message.received</code> does the same for inbound messages. If Device C subscribes to <code>message.received</code> on Device B, every message B receives is copied to C.</p>

  <h3>configure.sent / configure.received</h3>
  <p><code>configure.sent</code> fires when a device is updated via <code>PUT /devices/:uuid</code>. Subscribers get the new device document. Useful for dashboards or audit logs.</p>
  <p><code>configure.received</code> fires when a config change is directed at a device. Subscribers see what changes were applied.</p>

  <h3>unregister.sent / unregister.received</h3>
  <p><code>unregister.sent</code> fires when a device is deleted via <code>DELETE /devices/:uuid</code>. Subscribers are notified that the device is gone. Both types require only <code>discover.view</code> permission on the emitter, since unregister events are informational.</p>

  <h2>Implicit vs Explicit</h2>
  <p>Direct message delivery to the target device is implicit. You do not need a subscription for Device B to receive messages sent to Device B. The handler delivers directly.</p>
  <p>Everything else requires an explicit subscription. Broadcasts need <code>broadcast.sent</code> subscribers. Third-party monitoring needs <code>message.sent</code> or <code>message.received</code> subscribers. Config notifications need <code>configure.sent</code> subscribers.</p>

  <h2>Fan-out Mechanics</h2>
  <p>When an event fires, FreshBlu resolves subscribers in two stages:</p>

  <h3>Stage 1: Primary Fan-out</h3>
  <p>The handler (or router) queries the store for all subscribers of the relevant type on the emitter. For a direct message from A to B:</p>
  <ul>
    <li>Query <code>message.sent</code> subscribers of A.</li>
    <li>Query <code>message.received</code> subscribers of B.</li>
    <li>Deliver the event to B, plus all resolved subscribers.</li>
  </ul>

  <h3>Stage 2: Secondary Fan-out (broadcasts)</h3>
  <p>For broadcasts, after delivering to <code>broadcast.sent</code> subscribers, the system also checks <code>broadcast.received</code> subscribers of each recipient. This allows chained observation.</p>

  <h2>How the Router Resolves Subscribers</h2>
  <p>In multi-pod deployments, the router performs fan-out. The <code>Fanout</code> struct encapsulates the logic:</p>
  <CodeBlock lang="rust" code={`pub struct Fanout {
    pub store: DynStore,
    pub redis: redis::aio::ConnectionManager,
    pub nats: async_nats::Client,
}`} />
  <p>For each event type, the router calls a specialized method:</p>
  <ul>
    <li><code>route_direct</code> &mdash; looks up the target's pod in Redis, sends a <code>DeliveryEnvelope</code>.</li>
    <li><code>route_broadcast</code> &mdash; queries <code>broadcast.sent</code> subscribers, looks up each subscriber's pod, delivers envelopes.</li>
    <li><code>route_config</code> &mdash; delivers to the device itself, then fans out to <code>configure.sent</code> subscribers.</li>
    <li><code>route_unregister</code> &mdash; fans out to <code>unregister.sent</code> subscribers.</li>
  </ul>
  <p>If a subscriber is not online (no Redis presence entry), the delivery is dropped. FreshBlu does not queue messages for offline devices.</p>

  <h2>Permission Checks on Subscription Creation</h2>
  <p>Creating a subscription requires permission on the emitter's whitelist. The mapping:</p>
  <ul>
    <li><code>broadcast-sent</code> requires <code>broadcast.sent</code> whitelist.</li>
    <li><code>broadcast-received</code> requires <code>broadcast.received</code> whitelist.</li>
    <li><code>message-sent</code> requires <code>message.sent</code> whitelist.</li>
    <li><code>message-received</code> requires <code>message.received</code> whitelist.</li>
    <li><code>configure-sent</code> requires <code>configure.sent</code> whitelist.</li>
    <li><code>configure-received</code> requires <code>configure.received</code> whitelist.</li>
    <li><code>unregister-sent</code> and <code>unregister-received</code> require <code>discover.view</code> whitelist.</li>
  </ul>

  <h2>Subscription Lifecycle</h2>
  <p>Subscriptions are stored in the database with foreign key constraints. When a device is unregistered, its subscriptions (both as emitter and subscriber) are deleted via CASCADE. There is no expiration or TTL. Subscriptions persist until explicitly deleted or the device is removed.</p>

  <h2>Route Hops</h2>
  <p>The <code>RouteHop</code> struct tracks the path of an event through subscription chains. Each hop records the source, destination, and subscription type. This is used internally for debugging and is not exposed to clients.</p>
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
