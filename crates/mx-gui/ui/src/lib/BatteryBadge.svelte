<script>
  import { statusData, uiState, lastUpdated } from './stores.js';

  let batteryLevel;
  let batteryStatus;
  let isConnected;

  $: isConnected = $uiState === 'Connected';
  $: batteryLevel = $statusData.battery_level !== null && $statusData.battery_level !== undefined ? $statusData.battery_level : null;
  $: batteryStatus = $statusData.battery_status || 'Desconocido';

  function getBatteryColor(level, connected) {
    if (!connected) return 'text-slate-500 border-slate-700 bg-slate-800/40';
    if (level === null) return 'text-slate-400 border-slate-700 bg-slate-800/40';
    if (level > 40) return 'text-emerald-400 border-emerald-500/30 bg-emerald-500/10';
    if (level > 15) return 'text-amber-400 border-amber-500/30 bg-amber-500/10';
    return 'text-rose-400 border-rose-500/30 bg-rose-500/10';
  }
</script>

<div class="inline-flex items-center space-x-2 px-2.5 py-1 rounded-full text-xs font-medium border transition-colors duration-200 {getBatteryColor(batteryLevel, isConnected)}">
  <!-- Ícono SVG de Batería con relleno dinámico -->
  <svg class="w-4 h-4 fill-current" viewBox="0 0 24 24">
    <rect x="2" y="6" width="17" height="12" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
    <path d="M21 10v4" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    {#if batteryLevel !== null}
      <rect 
        x="4" 
        y="8" 
        width={Math.max(2, Math.min(13, (batteryLevel / 100) * 13))} 
        height="8" 
        rx="1" 
        fill="currentColor" 
      />
    {/if}
  </svg>

  <span>
    {#if batteryLevel !== null}
      {batteryLevel}%
      {#if !isConnected}
        <span class="text-[10px] text-slate-400 font-normal ml-0.5">(stale)</span>
      {/if}
    {:else}
      --
    {/if}
  </span>
</div>
