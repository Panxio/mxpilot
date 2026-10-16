<script>
  import { showHealthModal, healthData, guiMemoryKb } from './stores.js';
  import { t } from './i18n.js';
  import { getHealth, getGuiMemory } from './api.js';

  let isLoading = false;
  let copyFeedback = false;

  async function refreshHealth() {
    isLoading = true;
    try {
      const [h, mem] = await Promise.all([getHealth(), getGuiMemory()]);
      healthData.set(h);
      guiMemoryKb.set(mem);
    } catch (err) {
      console.error('Error actualizando métricas de salud:', err);
    } finally {
      isLoading = false;
    }
  }

  async function copyDiagnostics() {
    try {
      const payload = {
        timestamp: new Date().toISOString(),
        health: $healthData,
        gui_memory_kb: $guiMemoryKb,
      };
      await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
      copyFeedback = true;
      setTimeout(() => { copyFeedback = false; }, 2000);
    } catch (err) {
      console.error('Error al copiar diagnóstico:', err);
    }
  }

  function closeModal() {
    showHealthModal.set(false);
  }
</script>

{#if $showHealthModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm transition-opacity">
    <div class="w-full max-w-lg bg-slate-900 border border-slate-700/80 rounded-2xl p-5 shadow-2xl flex flex-col max-h-[85vh]">
      <!-- Encabezado del modal -->
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <div class="flex items-center space-x-2">
          <span class="w-2.5 h-2.5 rounded-full bg-indigo-500 animate-pulse"></span>
          <h3 class="text-sm font-semibold text-white tracking-wide">{$t('modal_title')}</h3>
        </div>
        <button
          type="button"
          on:click={closeModal}
          class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-colors"
        >
          ✕
        </button>
      </div>

      <!-- Contenido de telemetría -->
      <div class="flex-1 overflow-y-auto py-3 space-y-3 text-xs">
        {#if $healthData}
          <div class="grid grid-cols-2 gap-2">
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_daemon_state')}</span>
              <div class="font-medium text-slate-200 mt-0.5">{$healthData.state}</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_uptime')}</span>
              <div class="font-medium text-slate-200 mt-0.5">{$healthData.uptime_seconds}s</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_hidpp_device')}</span>
              <div class="font-medium text-slate-200 mt-0.5 truncate">{$healthData.device_name || $t('modal_device_none')}</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_hidraw_node')}</span>
              <div class="font-medium text-slate-200 mt-0.5 truncate font-mono">{$healthData.device_path || 'N/A'}</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_divert_epoch')}</span>
              <div class="font-medium text-slate-200 mt-0.5 font-mono">#{$healthData.divert_epoch}</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_reconnections')}</span>
              <div class="font-medium text-slate-200 mt-0.5">{$healthData.reconnect_count}</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_queue_dropped')}</span>
              <div class="font-medium text-slate-200 mt-0.5 font-mono">{$healthData.queue_depth} / {$healthData.dropped_events}</div>
            </div>
            <div class="p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80">
              <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_consecutive_failures')}</span>
              <div class="font-medium text-slate-200 mt-0.5">{$healthData.consecutive_failures}</div>
            </div>
          </div>

          <!-- Métricas de Memoria VmRSS -->
          <div class="p-3 rounded-xl bg-indigo-950/20 border border-indigo-500/30">
            <div class="flex items-center space-x-2 text-indigo-300 font-semibold mb-2">
              <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
              <span>{$t('modal_ram_title')}</span>
              <span class="text-[10px] font-mono text-slate-400 uppercase font-normal">{$t('modal_ram_rss_tag')}</span>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <div class="p-2.5 rounded-lg bg-slate-950/70 border border-slate-800/80">
                <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_ram_daemon')}</span>
                <div class="font-medium text-slate-200 mt-0.5 font-mono">{($healthData.rss_kb / 1024).toFixed(2)} MB</div>
              </div>
              <div class="p-2.5 rounded-lg bg-slate-950/70 border border-slate-800/80">
                <span class="text-[10px] text-slate-400 uppercase font-mono">{$t('modal_ram_gui')}</span>
                <div class="font-medium text-slate-200 mt-0.5 font-mono">{($guiMemoryKb / 1024).toFixed(2)} MB</div>
              </div>
            </div>
          </div>

          {#if $healthData.last_error}
            <div class="p-2.5 rounded-xl bg-rose-950/30 border border-rose-500/30 text-rose-300">
              <span class="text-[10px] font-mono uppercase block text-rose-400 font-semibold">{$t('modal_last_error')}</span>
              <span class="font-mono text-[11px] break-all">{$healthData.last_error}</span>
            </div>
          {/if}
        {:else}
          <div class="text-center py-6 text-slate-400">
            {$t('modal_empty')}
          </div>
        {/if}
      </div>

      <!-- Pie del modal -->
      <div class="flex items-center justify-between pt-3 border-t border-slate-800 mt-auto">
        <div class="flex items-center space-x-2">
          <button
            type="button"
            disabled={isLoading}
            on:click={refreshHealth}
            class="px-3 py-1.5 rounded-xl text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition-colors flex items-center space-x-1.5"
          >
            <span>{isLoading ? $t('modal_querying') : $t('modal_refresh')}</span>
          </button>
          
          <button
            type="button"
            disabled={!$healthData}
            on:click={copyDiagnostics}
            class="px-3 py-1.5 rounded-xl text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition-colors flex items-center space-x-1.5 disabled:opacity-50"
          >
            <span>{copyFeedback ? $t('modal_copied') : $t('modal_copy_btn')}</span>
          </button>
        </div>

        <button
          type="button"
          on:click={closeModal}
          class="px-4 py-1.5 rounded-xl text-xs font-medium bg-indigo-600 hover:bg-indigo-500 text-white transition-colors"
        >
          {$t('modal_close')}
        </button>
      </div>
    </div>
  </div>
{/if}
