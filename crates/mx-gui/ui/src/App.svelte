<script>
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import {
    uiState,
    statusData,
    errorMessage,
    showHealthModal,
    healthData,
    guiMemoryKb,
    lastUpdated
  } from './lib/stores.js';
  import { getStatus, getHealth, getGuiMemory, startDaemonService, setGuiLocale } from './lib/api.js';
  import { locale, setLocale, t } from './lib/i18n.js';
  import MouseViewer from './lib/MouseViewer.svelte';
  import ActionPicker from './lib/ActionPicker.svelte';
  import BatteryBadge from './lib/BatteryBadge.svelte';
  import HealthModal from './lib/HealthModal.svelte';

  let isStartingService = false;
  let unlistenStatusChanged = null;
  let unlistenStatusError = null;

  function switchLanguage(lang) {
    setLocale(lang);
    setGuiLocale(lang);
  }

  async function syncStatus() {
    try {
      const data = await getStatus();
      statusData.set(data);
      lastUpdated.set(Date.now());
      if (data.connected) {
        uiState.set('Connected');
      } else {
        uiState.set('DeviceDisconnected');
      }
      errorMessage.set(null);
    } catch (err) {
      console.warn('Fallo al obtener estado inicial:', err);
      uiState.set('DaemonDown');
      errorMessage.set(typeof err === 'string' ? err : $t('err_daemon_unreachable'));
    }
  }

  async function handleStartService() {
    isStartingService = true;
    errorMessage.set(null);
    try {
      await startDaemonService();
      await syncStatus();
    } catch (err) {
      console.error('Error al iniciar servicio:', err);
      errorMessage.set(typeof err === 'string' ? err : $t('err_start_service_failed'));
    } finally {
      isStartingService = false;
    }
  }

  async function openTelemetry() {
    try {
      const [h, mem] = await Promise.all([getHealth(), getGuiMemory()]);
      healthData.set(h);
      guiMemoryKb.set(mem);
    } catch (err) {
      console.warn('Error al cargar telemetría:', err);
    }
    showHealthModal.set(true);
  }

  onMount(async () => {
    setGuiLocale($locale);
    await syncStatus();

    // Escuchar eventos en tiempo real emitidos por el poller de Rust
    try {
      unlistenStatusChanged = await listen('status-changed', (event) => {
        const data = event.payload;
        statusData.set(data);
        lastUpdated.set(Date.now());
        if (data.connected) {
          uiState.set('Connected');
        } else {
          uiState.set('DeviceDisconnected');
        }
      });

      unlistenStatusError = await listen('status-error', (event) => {
        uiState.set('DaemonDown');
        errorMessage.set(event.payload);
      });
    } catch (err) {
      console.warn('No se pudieron registrar listeners de eventos Tauri:', err);
    }
  });

  onDestroy(() => {
    if (unlistenStatusChanged) unlistenStatusChanged();
    if (unlistenStatusError) unlistenStatusError();
  });
</script>

<main class="flex flex-col h-screen w-screen bg-slate-950 text-slate-100 select-none overflow-hidden font-sans">
  <!-- Barra de Título / Encabezado Superior -->
  <header class="border-b border-slate-800/80 bg-slate-900/60 backdrop-blur px-5 py-3">
    <div class="max-w-[1100px] mx-auto w-full flex items-center justify-between">
      <!-- Logotipo y Nombre del Dispositivo -->
      <div class="flex items-center space-x-3">
        <div class="flex items-center justify-center w-8 h-8 rounded-xl bg-gradient-to-tr from-emerald-900 to-emerald-800 shadow-md shadow-emerald-950/40 border border-emerald-500/40 text-white font-bold text-sm">
          MX
        </div>
        <div>
          <h1 class="text-sm font-semibold tracking-tight text-white flex items-center space-x-2">
            <span>{$statusData.device_name || 'Logitech MX Vertical'}</span>
            {#if $uiState === 'Connected'}
              <span class="inline-block w-2 h-2 rounded-full bg-emerald-500 shadow-sm shadow-emerald-500/50"></span>
            {:else if $uiState === 'DeviceDisconnected'}
              <span class="inline-block w-2 h-2 rounded-full bg-amber-500"></span>
            {:else}
              <span class="inline-block w-2 h-2 rounded-full bg-rose-500"></span>
            {/if}
          </h1>
          <p class="text-[10px] text-slate-400 font-mono tracking-wide">
            {$statusData.device_path || $t('header_driver_fallback')}
          </p>
        </div>
      </div>

      <!-- Controles de la Barra Superior -->
      <div class="flex items-center space-x-2.5">
        <BatteryBadge />

        <!-- Selector de Idioma Bilingüe (ES | EN) -->
        <div
          class="inline-flex items-center rounded-full bg-slate-900/90 border border-slate-700/80 p-0.5 text-[11px] font-semibold"
          role="group"
          aria-label={$t('header_lang_title')}
          title={$t('header_lang_title')}
        >
          <button
            type="button"
            on:click={() => switchLanguage('es')}
            class="px-2 py-0.5 rounded-full transition-colors duration-150 {$locale === 'es' ? 'bg-emerald-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
            aria-pressed={$locale === 'es'}
          >
            ES
          </button>
          <button
            type="button"
            on:click={() => switchLanguage('en')}
            class="px-2 py-0.5 rounded-full transition-colors duration-150 {$locale === 'en' ? 'bg-emerald-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
            aria-pressed={$locale === 'en'}
          >
            EN
          </button>
        </div>

        <button
          type="button"
          on:click={openTelemetry}
          title={$t('header_diagnostics_title')}
          class="flex items-center space-x-1.5 px-3 py-1 rounded-full text-xs font-medium bg-slate-800 hover:bg-slate-700/80 text-slate-300 border border-slate-700 transition-colors"
        >
          <span class="text-[12px]">🛠</span>
          <span>{$t('header_diagnostics')}</span>
        </button>
      </div>
    </div>
  </header>

  <!-- Banner de Estado de la Máquina Honesta de 4 Vías -->
  {#if $uiState === 'DaemonDown'}
    <div class="px-5 py-2.5 bg-rose-950/40 border-b border-rose-800/60 text-xs text-rose-300">
      <div class="max-w-[1100px] mx-auto w-full flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <span class="text-rose-400 font-bold">⚠</span>
          <span>{$t('banner_daemon_down')}</span>
        </div>
        <button
          type="button"
          disabled={isStartingService}
          on:click={handleStartService}
          class="px-3 py-1 rounded-lg text-xs font-semibold bg-rose-600 hover:bg-rose-500 text-white transition-colors disabled:opacity-50"
        >
          {isStartingService ? $t('banner_starting_service') : $t('banner_start_service')}
        </button>
      </div>
    </div>
  {:else if $uiState === 'DeviceDisconnected'}
    <div class="px-5 py-2 bg-amber-950/40 border-b border-amber-800/60 text-xs text-amber-300">
      <div class="max-w-[1100px] mx-auto w-full flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <span class="text-amber-400">ℹ</span>
          <span>{$t('banner_disconnected')}</span>
        </div>
        <span class="text-[10px] font-mono text-amber-400/80">{$t('banner_autoreconnect')}</span>
      </div>
    </div>
  {/if}

  {#if $errorMessage && $uiState !== 'DaemonDown'}
    <div class="px-5 py-1.5 bg-rose-950/30 border-b border-rose-800/40 text-[11px] text-rose-300">
      <div class="max-w-[1100px] mx-auto w-full flex items-center justify-between">
        <span>{$errorMessage}</span>
        <button type="button" on:click={() => errorMessage.set(null)} class="text-rose-400 hover:text-rose-200">✕</button>
      </div>
    </div>
  {/if}

  <!-- Contenido Principal: Anclado Superior (Top-Pinning) con Flujo Vertical Flexible -->
  <div class="flex-1 min-h-0 w-full flex items-start justify-center px-5 pt-6 pb-5 overflow-hidden">
    <div class="grid grid-cols-2 gap-5 w-full max-w-[1100px] h-full min-h-0 max-h-[720px]">
      <!-- Columna Izquierda: Visualizador Blueprint SVG -->
      <div class="flex flex-col h-full min-h-0">
        <MouseViewer />
      </div>

      <!-- Columna Derecha: Selector de Acciones -->
      <div class="flex flex-col h-full min-h-0">
        <ActionPicker />
      </div>
    </div>
  </div>

  <!-- Modal de Telemetría -->
  <HealthModal />
</main>
