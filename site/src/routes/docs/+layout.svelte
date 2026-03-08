<script lang="ts">
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  const sections = [
    {
      title: 'Tutorials',
      items: [
        { href: '/docs/tutorials/getting-started', label: 'Getting Started' },
        { href: '/docs/tutorials/arduino-sensor', label: 'Arduino Sensor' },
        { href: '/docs/tutorials/websocket-dashboard', label: 'WebSocket Dashboard' },
      ]
    },
    {
      title: 'How-To Guides',
      items: [
        { href: '/docs/how-to/register-device', label: 'Register a Device' },
        { href: '/docs/how-to/manage-permissions', label: 'Manage Permissions' },
        { href: '/docs/how-to/set-up-webhooks', label: 'Set Up Webhooks' },
        { href: '/docs/how-to/send-messages', label: 'Send Messages' },
        { href: '/docs/how-to/use-subscriptions', label: 'Use Subscriptions' },
        { href: '/docs/how-to/deploy-production', label: 'Deploy to Production' },
      ]
    },
    {
      title: 'Reference',
      items: [
        { href: '/docs/reference/http-api', label: 'HTTP API' },
        { href: '/docs/reference/websocket-protocol', label: 'WebSocket Protocol' },
        { href: '/docs/reference/mqtt-protocol', label: 'MQTT Protocol' },
        { href: '/docs/reference/permissions', label: 'Permissions' },
        { href: '/docs/reference/arduino-library', label: 'Arduino Library' },
        { href: '/docs/reference/error-codes', label: 'Error Codes' },
        { href: '/docs/reference/configuration', label: 'Configuration' },
      ]
    },
    {
      title: 'Explanation',
      items: [
        { href: '/docs/explanation/architecture', label: 'Architecture' },
        { href: '/docs/explanation/permission-model', label: 'Permission Model' },
        { href: '/docs/explanation/subscriptions', label: 'Subscriptions' },
        { href: '/docs/explanation/forwarders', label: 'Forwarders' },
      ]
    }
  ];
</script>

<div class="docs-layout">
  <Sidebar {sections} />
  <div class="docs-content">
    {@render children()}
  </div>
</div>

<style>
  .docs-layout {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
  }
  .docs-content {
    flex: 1;
    padding: 40px;
    min-width: 0;
  }

  @media (max-width: 900px) {
    .docs-layout { flex-direction: column; }
    .docs-content { padding: 24px 16px; }
  }
</style>
