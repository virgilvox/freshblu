<svelte:head>
  <title>Documentation - FreshBlu</title>
</svelte:head>

<div class="docs-index">
  <h1 class="docs-title">Documentation</h1>
  <p class="docs-intro">
    FreshBlu is a Meshblu-compatible IoT messaging platform written in Rust.
    It provides HTTP, WebSocket, and MQTT interfaces for device-to-device communication
    with fine-grained permission control.
  </p>

  <div class="docs-grid">
    <a href="/docs/tutorials/getting-started" class="doc-card">
      <span class="doc-card-label">Tutorials</span>
      <span class="doc-card-title">Getting Started</span>
      <span class="doc-card-desc">Register your first device and send a message in under five minutes.</span>
    </a>
    <a href="/docs/reference/http-api" class="doc-card">
      <span class="doc-card-label">Reference</span>
      <span class="doc-card-title">HTTP API</span>
      <span class="doc-card-desc">Complete reference for all 24 REST endpoints.</span>
    </a>
    <a href="/docs/how-to/register-device" class="doc-card">
      <span class="doc-card-label">How-To</span>
      <span class="doc-card-title">Register a Device</span>
      <span class="doc-card-desc">Create devices with properties, permissions, and tokens.</span>
    </a>
    <a href="/docs/explanation/architecture" class="doc-card">
      <span class="doc-card-label">Explanation</span>
      <span class="doc-card-title">Architecture</span>
      <span class="doc-card-desc">Hub/Bus abstraction, Gateway/Router pods, and horizontal scaling.</span>
    </a>
  </div>
</div>

<style>
  .docs-title {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    margin-bottom: 16px;
  }
  .docs-intro {
    font-size: var(--text-sm);
    color: var(--ink-soft);
    line-height: var(--leading-relaxed);
    max-width: 600px;
    margin-bottom: 48px;
  }
  .docs-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .doc-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 20px;
    background: var(--void-lift);
    border: 1px solid var(--border);
    text-decoration: none;
    transition: transform var(--dur-med) var(--ease-snap), border-color var(--dur-med);
  }
  .doc-card:hover {
    transform: translateY(-2px);
    border-color: var(--border-pulse);
  }
  .doc-card-label {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--pulse);
  }
  .doc-card-title {
    font-family: var(--font-display);
    font-size: var(--text-md);
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--ink);
  }
  .doc-card-desc {
    font-size: var(--text-sm);
    color: var(--ink-muted);
    line-height: var(--leading-normal);
  }

  @media (max-width: 600px) {
    .docs-grid { grid-template-columns: 1fr; }
  }
</style>
