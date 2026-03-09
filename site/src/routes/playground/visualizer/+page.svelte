<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { vaultDevices } from '$lib/stores/vault';
  import { FreshBluClient, getServerUrl } from '$lib/api/client';
  import type { VaultDevice } from '$lib/stores/vault';

  interface MeshNode {
    uuid: string;
    label: string;
    x: number;
    y: number;
    connected: boolean;
  }

  interface MeshEdge {
    id: string;
    from: string;
    to: string;
    timestamp: number;
  }

  let devices: VaultDevice[] = $state([]);
  let nodes: MeshNode[] = $state([]);
  let edges: MeshEdge[] = $state([]);
  let connections: Map<string, FreshBluClient> = new Map();
  let messageCount = $state(0);
  let edgeIdCounter = 0;
  let animFrame: number;

  const MAX_NODES = 20;
  const EDGE_TTL = 4000;

  const unsubVault = vaultDevices.subscribe((v) => (devices = v));

  function layoutNodes(count: number): { x: number; y: number }[] {
    const cx = 400, cy = 225, r = 160;
    return Array.from({ length: count }, (_, i) => {
      const angle = (2 * Math.PI * i) / count - Math.PI / 2;
      return { x: cx + r * Math.cos(angle), y: cy + r * Math.sin(angle) };
    });
  }

  function updateLayout() {
    const visible = devices.slice(0, MAX_NODES);
    const positions = layoutNodes(visible.length);
    nodes = visible.map((d, i) => ({
      uuid: d.uuid,
      label: d.uuid.substring(0, 8),
      x: positions[i].x,
      y: positions[i].y,
      connected: connections.has(d.uuid),
    }));
  }

  function addEdge(from: string, to: string) {
    edges = [...edges, { id: `e${edgeIdCounter++}`, from, to, timestamp: Date.now() }];
    messageCount++;
  }

  function connectDevice(device: VaultDevice) {
    if (connections.has(device.uuid)) return;
    const serverUrl = getServerUrl();
    const ws = new FreshBluClient(serverUrl);
    ws.setCredentials(device.uuid, device.token);
    ws.on('message', (event: Record<string, unknown>) => {
      const fromUuid = event.fromUuid as string | undefined;
      if (fromUuid) addEdge(fromUuid, device.uuid);
    });
    ws.on('broadcast', (event: Record<string, unknown>) => {
      const fromUuid = event.fromUuid as string | undefined;
      if (fromUuid) addEdge(fromUuid, device.uuid);
    });
    ws.on('ready', () => {
      connections.set(device.uuid, ws);
      updateLayout();
    });
    ws.on('close', () => {
      connections.delete(device.uuid);
      updateLayout();
    });
    ws.connect().catch(() => {});
    connections.set(device.uuid, ws);
  }

  function connectAll() {
    for (const d of devices.slice(0, MAX_NODES)) {
      connectDevice(d);
    }
  }

  function disconnectAll() {
    for (const ws of connections.values()) {
      ws.close();
    }
    connections.clear();
    updateLayout();
  }

  function tick() {
    const now = Date.now();
    edges = edges.filter((e) => now - e.timestamp < EDGE_TTL);
    animFrame = requestAnimationFrame(tick);
  }

  onMount(() => {
    updateLayout();
    animFrame = requestAnimationFrame(tick);
  });

  onDestroy(() => {
    cancelAnimationFrame(animFrame);
    unsubVault();
    disconnectAll();
  });

  $effect(() => {
    devices;
    updateLayout();
  });

  function nodeColor(connected: boolean): string {
    return connected ? 'var(--pulse)' : 'var(--ink-muted)';
  }

  function edgeOpacity(timestamp: number): number {
    const age = Date.now() - timestamp;
    return Math.max(0, 1 - age / EDGE_TTL);
  }

  function findNode(uuid: string): MeshNode | undefined {
    return nodes.find((n) => n.uuid === uuid);
  }
</script>

<svelte:head>
  <title>Visualizer - Playground - FreshBlu</title>
</svelte:head>

<div class="viz-page">
  <div class="viz-header">
    <h1 class="page-title">Mesh Visualizer</h1>
    <div class="viz-stats">
      <span class="stat"><i class="fa-solid fa-microchip"></i> {nodes.length} nodes</span>
      <span class="stat"><i class="fa-solid fa-link"></i> {connections.size} connected</span>
      <span class="stat"><i class="fa-solid fa-paper-plane"></i> {messageCount} messages</span>
    </div>
    <div class="viz-actions">
      <button class="viz-btn" onclick={connectAll}>Connect All</button>
      <button class="viz-btn viz-btn-muted" onclick={disconnectAll}>Disconnect All</button>
    </div>
  </div>

  <div class="viz-container">
    <svg viewBox="0 0 800 450" class="viz-svg">
      <!-- Edges -->
      {#each edges as edge (edge.id)}
        {@const fromNode = findNode(edge.from)}
        {@const toNode = findNode(edge.to)}
        {#if fromNode && toNode}
          <line
            x1={fromNode.x} y1={fromNode.y}
            x2={toNode.x} y2={toNode.y}
            stroke="var(--signal)"
            stroke-width="2"
            opacity={edgeOpacity(edge.timestamp)}
          />
          <!-- Arrow -->
          {@const dx = toNode.x - fromNode.x}
          {@const dy = toNode.y - fromNode.y}
          {@const len = Math.sqrt(dx * dx + dy * dy) || 1}
          {@const ux = dx / len}
          {@const uy = dy / len}
          {@const ax = toNode.x - ux * 24}
          {@const ay = toNode.y - uy * 24}
          <polygon
            points="{ax},{ay} {ax - uy * 4 - ux * 8},{ay + ux * 4 - uy * 8} {ax + uy * 4 - ux * 8},{ay - ux * 4 - uy * 8}"
            fill="var(--signal)"
            opacity={edgeOpacity(edge.timestamp)}
          />
        {/if}
      {/each}

      <!-- Nodes -->
      {#each nodes as node (node.uuid)}
        <g>
          <circle
            cx={node.x} cy={node.y} r="18"
            fill="var(--void-lift)"
            stroke={nodeColor(node.connected)}
            stroke-width="2"
          />
          <text
            x={node.x} y={node.y + 32}
            text-anchor="middle"
            fill="var(--ink-soft)"
            font-size="10"
            font-family="var(--font-body)"
          >{node.label}</text>
          {#if node.connected}
            <circle cx={node.x + 14} cy={node.y - 14} r="4" fill="var(--online)" />
          {/if}
        </g>
      {/each}

      {#if nodes.length === 0}
        <text x="400" y="225" text-anchor="middle" fill="var(--ink-muted)" font-size="14" font-family="var(--font-ui)">
          No devices in vault. Register devices to visualize the mesh.
        </text>
      {/if}
    </svg>
  </div>
</div>

<style>
  .viz-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .viz-header {
    display: flex;
    align-items: center;
    gap: 24px;
    flex-wrap: wrap;
  }
  .page-title {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .viz-stats {
    display: flex;
    gap: 16px;
    margin-left: auto;
  }
  .stat {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    color: var(--ink-muted);
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .viz-actions {
    display: flex;
    gap: 8px;
  }
  .viz-btn {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    background: none;
    border: 1px solid var(--signal);
    color: var(--signal);
    padding: 4px 12px;
    cursor: pointer;
    transition: background var(--dur-fast);
  }
  .viz-btn:hover { background: var(--signal-dim); }
  .viz-btn-muted {
    border-color: var(--border);
    color: var(--ink-muted);
  }
  .viz-btn-muted:hover { border-color: var(--ink-soft); color: var(--ink-soft); background: none; }
  .viz-container {
    border: 1px solid var(--border);
    background: var(--void);
    aspect-ratio: 16 / 9;
  }
  .viz-svg {
    width: 100%;
    height: 100%;
  }

  @media (max-width: 768px) {
    .viz-stats { margin-left: 0; }
  }
</style>
