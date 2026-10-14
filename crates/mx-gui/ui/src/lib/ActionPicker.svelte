<script>
  import { selectedCid, statusData, isPendingAction, errorMessage, BUTTON_DEFINITIONS, uiState } from './stores.js';
  import { setAction, getStatus } from './api.js';
  import { isActionEqual, getActionForCid } from './actions.js';

  const CATEGORIES = [
    { id: 'all', label: 'Todas' },
    { id: 'nav', label: 'Navegación' },
    { id: 'prod', label: 'Productividad' },
    { id: 'hw', label: 'Hardware' },
  ];

  const PRESET_ACTIONS = [
    {
      id: 'super',
      category: 'nav',
      label: 'Super / Meta (Actividades)',
      desc: 'Abre la vista general del escritorio en GNOME / KDE.',
      payload: { type: 'SingleKey', key: 'KEY_LEFTMETA' },
      icon: '❖',
    },
    {
      id: 'workspace_right',
      category: 'nav',
      label: 'Espacio de Trabajo: Derecha (Ctrl + Alt + →)',
      desc: 'Cambia al espacio de trabajo virtual a la derecha.',
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_RIGHT' },
      icon: '→',
    },
    {
      id: 'workspace_left',
      category: 'nav',
      label: 'Espacio de Trabajo: Izquierda (Ctrl + Alt + ←)',
      desc: 'Cambia al espacio de trabajo virtual a la izquierda.',
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_LEFT' },
      icon: '←',
    },
    {
      id: 'alttab',
      category: 'nav',
      label: 'Cambio de Ventana (Alt + Tab)',
      desc: 'Conmuta entre las aplicaciones activas.',
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTALT'], key: 'KEY_TAB' },
      icon: '⇥',
    },
    {
      id: 'copy',
      category: 'prod',
      label: 'Copiar (Ctrl + C)',
      desc: 'Copia el texto o archivo seleccionado al portapapeles.',
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL'], key: 'KEY_C' },
      icon: '📋',
    },
    {
      id: 'paste',
      category: 'prod',
      label: 'Pegar (Ctrl + V)',
      desc: 'Pega el contenido del portapapeles.',
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL'], key: 'KEY_V' },
      icon: '📄',
    },
    {
      id: 'mute',
      category: 'prod',
      label: 'Silenciar / Desmutear Audio',
      desc: 'Alterna el estado de silencio del sistema.',
      payload: { type: 'SingleKey', key: 'KEY_MUTE' },
      icon: '🔇',
    },
    {
      id: 'playpause',
      category: 'prod',
      label: 'Reproducir / Pausar Multimedia',
      desc: 'Control de reproducción global (Spotify, navegadores).',
      payload: { type: 'SingleKey', key: 'KEY_PLAYPAUSE' },
      icon: '⏯',
    },
    {
      id: 'none',
      category: 'hw',
      label: 'Deshabilitar Botón',
      desc: 'No emite ninguna acción uinput ante clics.',
      payload: { type: 'None' },
      icon: '⊘',
    },
    {
      id: 'default',
      category: 'hw',
      label: 'Función de Fábrica (Navegación / Clic Nativo)',
      desc: 'Restaura el control del hardware nativo (ej. Atrás/Adelante en Chrome/Firefox, clic central).',
      payload: { type: 'SystemDefault' },
      icon: '↺',
    },
  ];

  let selectedButtonDef;
  $: selectedButtonDef = BUTTON_DEFINITIONS.find((b) => b.cid === $selectedCid) || BUTTON_DEFINITIONS[0];

  // Acción reactiva actualmente configurada para el botón seleccionado
  let currentAction = null;
  $: currentAction = getActionForCid($statusData.mapping, $selectedCid);
  $: isDefaultSystem = !currentAction;

  function isPresetActive(preset, current) {
    if (preset.payload.type === 'SystemDefault') {
      return !current;
    }
    return isActionEqual(current, preset.payload);
  }

  let selectedCategory = 'all';

  // Lista reactiva ordenada y filtrada: la acción activa se despliega primero
  let displayPresets = [];
  $: {
    const list = selectedCategory === 'all'
      ? PRESET_ACTIONS
      : PRESET_ACTIONS.filter((p) => p.category === selectedCategory);

    displayPresets = [...list].sort((a, b) => {
      const aActive = isPresetActive(a, currentAction);
      const bActive = isPresetActive(b, currentAction);
      if (aActive && !bActive) return -1;
      if (!aActive && bActive) return 1;
      return 0;
    });
  }

  async function applyAction(preset) {
    if ($isPendingAction || $uiState !== 'Connected') return;
    isPendingAction.set(true);
    errorMessage.set(null);

    try {
      await setAction($selectedCid, preset.payload);
      // Reconciliación con el daemon tras éxito
      const freshStatus = await getStatus();
      statusData.set(freshStatus);
    } catch (err) {
      console.error('Error aplicando acción:', err);
      errorMessage.set(typeof err === 'string' ? err : 'Error al comunicar con mx-daemon');
      // Reconciliación para asegurar integridad
      try {
        const freshStatus = await getStatus();
        statusData.set(freshStatus);
      } catch (_) {}
    } finally {
      isPendingAction.set(false);
    }
  }
</script>

<div class="flex flex-col h-full bg-slate-900/40 rounded-2xl border border-slate-800/80 p-4 shadow-inner backdrop-blur-sm">
  <!-- Encabezado del configurador de acción -->
  <div class="flex items-center justify-between pb-3 border-b border-slate-800/60 mb-3 gap-3">
    <div class="min-w-0 flex-1">
      <h2 class="text-sm font-semibold text-white tracking-wide truncate">
        {selectedButtonDef.name}
      </h2>
      <p class="text-xs text-slate-400 mt-0.5 line-clamp-1">
        {selectedButtonDef.description}
      </p>
    </div>

    <!-- Indicador de estado de fábrica o personalizado (sin códigos técnicos) -->
    <div class="flex-shrink-0">
      {#if isDefaultSystem}
        <span class="inline-flex items-center whitespace-nowrap px-2.5 py-1 rounded-full text-[11px] font-medium bg-slate-800/90 border border-slate-700/80 text-slate-300">
          Función de Fábrica
        </span>
      {:else}
        <span class="inline-flex items-center whitespace-nowrap px-2.5 py-1 rounded-full text-[11px] font-medium bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 space-x-1.5 shadow-sm shadow-emerald-500/10">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>Personalizado</span>
        </span>
      {/if}
    </div>
  </div>

  <!-- Selector de Categorías (Pills) estilo Libadwaita / macOS -->
  <div class="flex items-center space-x-1 p-1 bg-slate-950/60 rounded-xl border border-slate-800/80 mb-2.5 flex-shrink-0">
    {#each CATEGORIES as cat}
      <button
        type="button"
        on:click={() => selectedCategory = cat.id}
        class="flex-1 py-1 px-1.5 rounded-lg text-[11px] font-medium transition-all duration-150 text-center truncate
          {selectedCategory === cat.id
            ? 'bg-slate-800 text-white shadow-sm border border-slate-700/80'
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'}"
      >
        {cat.label}
      </button>
    {/each}
  </div>

  <!-- Lista de acciones permitidas estilo Libadwaita Cards -->
  <div class="flex-1 overflow-y-auto space-y-1.5 pr-1">
    {#each displayPresets as preset (preset.id)}
      {@const isActive = isPresetActive(preset, currentAction)}
      <button
        type="button"
        disabled={$isPendingAction || $uiState !== 'Connected'}
        on:click={() => applyAction(preset)}
        class="group w-full flex items-center justify-between py-2 px-3 rounded-xl border text-left transition-all duration-150
          {isActive 
            ? 'bg-emerald-950/30 border-emerald-500/70 shadow-sm shadow-emerald-500/20 ring-1 ring-emerald-500/40' 
            : 'bg-slate-800/40 border-slate-700/40 hover:bg-slate-800/70 hover:border-slate-600/60'}
          {$isPendingAction || $uiState !== 'Connected' ? 'opacity-60 cursor-not-allowed' : 'cursor-pointer'}"
      >
        <div class="flex items-center space-x-3 min-w-0">
          <div class="flex items-center justify-center w-7 h-7 rounded-lg {isActive ? 'bg-emerald-900/60 border border-emerald-600/60 text-emerald-300' : 'bg-slate-900/60 border border-slate-700/60 text-slate-300'} text-xs flex-shrink-0">
            {preset.icon}
          </div>
          <div class="min-w-0">
            <div class="text-xs font-medium {isActive ? 'text-emerald-200' : 'text-slate-200'} truncate">
              {preset.label}
            </div>
            <div class="text-[11px] {isActive ? 'text-emerald-400/80' : 'text-slate-400'} truncate mt-0.5">
              {preset.desc}
            </div>
          </div>
        </div>

        <div class="ml-2 flex-shrink-0">
          {#if isActive}
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 shadow-sm shadow-emerald-500/20">
              ✓ Activo
            </span>
          {:else}
            <span class="text-xs text-slate-500 opacity-0 group-hover:opacity-100 transition-opacity">
              Asignar
            </span>
          {/if}
        </div>
      </button>
    {/each}
  </div>

  {#if $isPendingAction}
    <div class="mt-2 text-center text-xs text-indigo-400 animate-pulse flex items-center justify-center space-x-2">
      <span class="inline-block w-2 h-2 rounded-full bg-indigo-400 animate-ping"></span>
      <span>Aplicando configuración en mx-daemon...</span>
    </div>
  {/if}
</div>
