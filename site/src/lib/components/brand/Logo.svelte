<script lang="ts">
  interface Props {
    size?: number;
  }

  let { size = 40 }: Props = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width={size} height={size} aria-label="FreshBlu mark">
  <!-- bracket corners -->
  <line x1="30" y1="55" x2="30" y2="30" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="30" y1="30" x2="55" y2="30" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="145" y1="30" x2="170" y2="30" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="170" y1="30" x2="170" y2="55" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="30" y1="145" x2="30" y2="170" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="30" y1="170" x2="55" y2="170" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="145" y1="170" x2="170" y2="170" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <line x1="170" y1="170" x2="170" y2="145" stroke="currentColor" stroke-width="4" stroke-linecap="square"/>
  <!-- row 1 -->
  <rect x="50" y="70" width="14" height="14" fill="#0077ff"/>
  <rect x="72" y="70" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="94" y="70" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="116" y="70" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="138" y="70" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <!-- separator -->
  <line x1="50" y1="95" x2="152" y2="95" stroke="currentColor" stroke-width="1" stroke-dasharray="3 3" opacity="0.3"/>
  <!-- row 2 -->
  <rect x="50" y="108" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="72" y="108" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="94" y="108" width="14" height="14" fill="#0077ff" opacity="0.7"/>
  <rect x="116" y="108" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="138" y="108" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <!-- row 3 -->
  <rect x="50" y="130" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="72" y="130" width="14" height="14" fill="#0077ff" opacity="0.45"/>
  <rect x="94" y="130" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="116" y="130" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <rect x="138" y="130" width="14" height="14" fill="currentColor" opacity="0.35"/>
  <!-- corner pulses -->
  <rect x="26" y="26" width="6" height="6" fill="#0077ff"/>
  <rect x="168" y="26" width="6" height="6" fill="#0077ff" opacity="0.5"/>
  <rect x="26" y="168" width="6" height="6" fill="#0077ff" opacity="0.5"/>
  <rect x="168" y="168" width="6" height="6" fill="#0077ff"/>
</svg>
