<script>
  import { selectedCid, statusData, isPendingAction, errorMessage, uiState } from './stores.js';
  import { t, localizedButtons } from './i18n.js';
  import { setAction, getStatus } from './api.js';
  import { isActionEqual, getActionForCid } from './actions.js';

  $: CATEGORIES = [
    { id: 'all', label: $t('cat_all') },
    { id: 'nav', label: $t('cat_nav') },
    { id: 'prod', label: $t('cat_prod') },
    { id: 'hw', label: $t('cat_hw') },
  ];

  $: PRESET_ACTIONS = [
    {
      id: 'super',
      category: 'nav',
      label: $t('act_super_label'),
      desc: $t('act_super_desc'),
      payload: { type: 'SingleKey', key: 'KEY_LEFTMETA' },
      icon: '❖',
    },
    {
      id: 'workspace_right',
      category: 'nav',
      label: $t('act_ws_right_label'),
      desc: $t('act_ws_right_desc'),
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_RIGHT' },
      icon: '→',
    },
    {
      id: 'workspace_left',
      category: 'nav',
      label: $t('act_ws_left_label'),
      desc: $t('act_ws_left_desc'),
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_LEFT' },
      icon: '←',
    },
    {
      id: 'alttab',
      category: 'nav',
      label: $t('act_alttab_label'),
      desc: $t('act_alttab_desc'),
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTALT'], key: 'KEY_TAB' },
      icon: '⇥',
    },
    {
      id: 'copy',
      category: 'prod',
      label: $t('act_copy_label'),
      desc: $t('act_copy_desc'),
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL'], key: 'KEY_C' },
      icon: '📋',
    },
    {
      id: 'paste',
      category: 'prod',
      label: $t('act_paste_label'),
      desc: $t('act_paste_desc'),
      payload: { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL'], key: 'KEY_V' },
      icon: '📄',
    },
    {
      id: 'mute',
      category: 'prod',
      label: $t('act_mute_label'),
      desc: $t('act_mute_desc'),
      payload: { type: 'SingleKey', key: 'KEY_MUTE' },
      icon: '🔇',
    },
    {
      id: 'playpause',
      category: 'prod',
      label: $t('act_playpause_label'),
      desc: $t('act_playpause_desc'),
      payload: { type: 'SingleKey', key: 'KEY_PLAYPAUSE' },
      icon: '⏯',
    },
    {
      id: 'none',
      category: 'hw',
      label: $t('act_none_label'),
      desc: $t('act_none_desc'),
      payload: { type: 'None' },
      icon: '⊘',
    },
    {
      id: 'default',
      category: 'hw',
      label: $t('act_default_label'),
      desc: $t('act_default_desc'),
      payload: { type: 'SystemDefault' },
      icon: '↺',
    },
  ];

  let selectedButtonDef;
  $: selectedButtonDef = $localizedButtons.find((b) => b.cid === $selectedCid) || $localizedButtons[0];

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
      errorMessage.set(typeof err === 'string' ? err : $t('err_apply_action_failed'));
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
          {$t('picker_factory_badge')}
        </span>
      {:else}
        <span class="inline-flex items-center whitespace-nowrap px-2.5 py-1 rounded-full text-[11px] font-medium bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 space-x-1.5 shadow-sm shadow-emerald-500/10">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>{$t('picker_custom_badge')}</span>
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
              {$t('picker_active_badge')}
            </span>
          {:else}
            <span class="text-xs text-slate-500 opacity-0 group-hover:opacity-100 transition-opacity">
              {$t('picker_assign_hint')}
            </span>
          {/if}
        </div>
      </button>
    {/each}
  </div>

  {#if $isPendingAction}
    <div class="mt-2 text-center text-xs text-indigo-400 animate-pulse flex items-center justify-center space-x-2">
      <span class="inline-block w-2 h-2 rounded-full bg-indigo-400 animate-ping"></span>
      <span>{$t('picker_applying')}</span>
    </div>
  {/if}
</div>
