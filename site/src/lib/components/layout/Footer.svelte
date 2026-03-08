<footer class="footer">
  <div class="footer-inner">
    <span class="footer-brand">FRESH<span>BLU</span></span>
    <div class="footer-badges">
      <span class="f-badge">HTTP</span>
      <span class="f-badge">WebSocket</span>
      <span class="f-badge">MQTT</span>
      <span class="f-badge accent">Rust</span>
    </div>
    <span class="footer-meta">Meshblu-compatible IoT Messaging</span>
  </div>
</footer>

<style>
  .footer {
    background: var(--void-lift);
    border-top: 1px solid var(--border);
    padding: 32px 40px;
    margin-top: 80px;
  }
  .footer-inner {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 16px;
  }
  .footer-brand {
    font-family: var(--font-display);
    font-size: var(--text-base);
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .footer-brand span { color: var(--pulse); }
  .footer-badges {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .f-badge {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 4px 10px;
    border: 1px solid var(--border-strong);
    color: var(--ink-muted);
  }
  .f-badge.accent { border-color: var(--pulse); color: var(--pulse); }
  .footer-meta {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    color: var(--ink-muted);
  }

  @media (max-width: 600px) {
    .footer { padding: 24px 16px; }
  }
</style>
