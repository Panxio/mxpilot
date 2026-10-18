import { writable, derived } from 'svelte/store';

export const STORAGE_KEY = 'mxpilot_locale';
export const SUPPORTED_LOCALES = ['es', 'en'];

/**
 * Detecta el idioma preferido:
 * 1. Valor persistido en localStorage ('mxpilot_locale')
 * 2. Idioma del sistema/navegador (navigator.language)
 * 3. Fallback determinista a 'en' (si no empieza con 'es')
 * @param {object} [opts] - Opciones de inyección para tests unitarios
 * @returns {'es' | 'en'}
 */
export function detectInitialLocale(opts = {}) {
  const storage = opts.storage !== undefined
    ? opts.storage
    : (typeof window !== 'undefined' && window.localStorage ? window.localStorage : null);

  if (storage && typeof storage.getItem === 'function') {
    try {
      const saved = storage.getItem(STORAGE_KEY);
      if (saved === 'es' || saved === 'en') {
        return saved;
      }
    } catch (_) {
      // Ignorar errores de acceso a storage en modo restringido
    }
  }

  const navLang = opts.navigatorLanguage !== undefined
    ? opts.navigatorLanguage
    : (typeof navigator !== 'undefined' ? (navigator.language || navigator.userLanguage || '') : 'es');

  if (typeof navLang === 'string' && navLang.toLowerCase().startsWith('es')) {
    return 'es';
  }
  return 'en';
}

export const DICTIONARY = {
  es: {
    // Header & Global
    header_driver_fallback: 'Controlador Nativo Linux',
    header_diagnostics: 'Diagnóstico',
    header_diagnostics_title: 'Ver diagnóstico y telemetría',
    header_lang_title: 'Cambiar idioma (ES / EN)',

    // Banners de estado
    banner_daemon_down: 'mx-daemon no está en ejecución. Los atajos personalizados están inactivos.',
    banner_start_service: 'Iniciar Servicio',
    banner_starting_service: 'Iniciando...',
    banner_disconnected: 'Ratón desconectado o en reposo Bluetooth. Configuración guardada en perfil.',
    banner_autoreconnect: 'Reconexión automática activa',
    err_daemon_unreachable: 'mx-daemon no responde en el socket IPC',
    err_start_service_failed: 'No se pudo iniciar el servicio mx-options',
    err_apply_action_failed: 'Error al comunicar con mx-daemon',

    // BatteryBadge
    battery_unknown: 'Desconocido',
    battery_stale: '(en caché)',
    battery_tooltip_connected: 'Batería del ratón',
    battery_tooltip_disconnected: 'Última lectura conocida (ratón desconectado)',

    // MouseViewer
    viewer_service_inactive: 'Servicio Inactivo',
    viewer_disconnected: 'Desconectado',
    viewer_interactive_blueprint: 'Blueprint Interactivo',
    viewer_system_default: 'Por defecto del sistema',
    viewer_super_activities: 'Super (Actividades)',
    viewer_mute: 'Silenciar',
    viewer_play_pause: 'Play / Pausa',
    viewer_workspace_right: 'Espacio Der. (Ctrl+Alt+→)',
    viewer_workspace_left: 'Espacio Izq. (Ctrl+Alt+←)',
    viewer_alttab: 'Cambio Ventana (Alt+Tab)',
    viewer_copy: 'Copiar (Ctrl+C)',
    viewer_paste: 'Pegar (Ctrl+V)',
    viewer_mouse_btn_prefix: 'Botón ',
    viewer_disabled: 'Deshabilitado',
    viewer_custom: 'Personalizado',

    // Botones físicos (CIDs)
    btn_top_name: 'Botón Superior (DPI)',
    btn_top_desc: 'Ubicado en la cresta metálica de 57°. Controla acciones globales o atajos rápidos.',
    btn_top_cat: 'Cúspide',
    btn_forward_name: 'Botón Adelante',
    btn_forward_desc: 'Botón superior del pulgar para avance de navegación o atajos.',
    btn_forward_cat: 'Pulgar',
    btn_back_name: 'Botón Atrás',
    btn_back_desc: 'Botón inferior del pulgar para retroceso de navegación o atajos.',
    btn_back_cat: 'Pulgar',
    btn_wheel_name: 'Rueda de Scroll (Clic Central)',
    btn_wheel_desc: 'Rueda de desplazamiento texturizada con clic mecánico central.',
    btn_wheel_cat: 'Frontal',

    // ActionPicker
    picker_factory_badge: 'Función de Fábrica',
    picker_custom_badge: 'Personalizado',
    picker_active_badge: '✓ Activo',
    picker_assign_hint: 'Asignar',
    picker_applying: 'Aplicando configuración en mx-daemon...',

    // Categorías ActionPicker
    cat_all: 'Todas',
    cat_nav: 'Navegación',
    cat_prod: 'Productividad',
    cat_hw: 'Hardware',

    // Acciones predefinidas
    act_super_label: 'Super / Meta (Actividades)',
    act_super_desc: 'Abre la vista general del escritorio en GNOME / KDE.',
    act_ws_right_label: 'Espacio de Trabajo: Derecha (Ctrl + Alt + →)',
    act_ws_right_desc: 'Cambia al espacio de trabajo virtual a la derecha.',
    act_ws_left_label: 'Espacio de Trabajo: Izquierda (Ctrl + Alt + ←)',
    act_ws_left_desc: 'Cambia al espacio de trabajo virtual a la izquierda.',
    act_alttab_label: 'Cambio de Ventana (Alt + Tab)',
    act_alttab_desc: 'Conmuta entre las aplicaciones activas.',
    act_copy_label: 'Copiar (Ctrl + C)',
    act_copy_desc: 'Copia el texto o archivo seleccionado al portapapeles.',
    act_paste_label: 'Pegar (Ctrl + V)',
    act_paste_desc: 'Pega el contenido del portapapeles.',
    act_mute_label: 'Silenciar / Desmutear Audio',
    act_mute_desc: 'Alterna el estado de silencio del sistema.',
    act_playpause_label: 'Reproducir / Pausar Multimedia',
    act_playpause_desc: 'Control de reproducción global (Spotify, navegadores).',
    act_none_label: 'Deshabilitar Botón',
    act_none_desc: 'No emite ninguna acción uinput ante clics.',
    act_default_label: 'Función de Fábrica (Navegación / Clic Nativo)',
    act_default_desc: 'Restaura el control nativo del hardware (Atrás/Adelante en navegador, clic central).',

    // HealthModal
    modal_title: 'Telemetría y Diagnóstico Interno',
    modal_daemon_state: 'Estado Daemon',
    modal_uptime: 'Tiempo Activo (Uptime)',
    modal_hidpp_device: 'Dispositivo HID++',
    modal_device_none: 'Ninguno',
    modal_hidraw_node: 'Nodo hidraw',
    modal_divert_epoch: 'Divert Epoch',
    modal_reconnections: 'Reconexiones',
    modal_queue_dropped: 'Cola Eventos / Perdidos',
    modal_consecutive_failures: 'Fallos Consecutivos',
    modal_ram_title: 'Consumo de Memoria RAM',
    modal_ram_rss_tag: '(Residente / RSS)',
    modal_ram_daemon: 'Daemon (Servicio)',
    modal_ram_gui: 'Interfaz (GUI)',
    modal_last_error: 'Último Error Registrado:',
    modal_empty: 'No se han obtenido datos de telemetría todavía.',
    modal_querying: 'Consultando...',
    modal_refresh: 'Actualizar',
    modal_copied: '✓ Copiado',
    modal_copy_btn: '📋 Copiar Diagnóstico',
    modal_autostart_title: 'Autoarranque con el Sistema',
    modal_autostart_desc: 'Inicia mx-daemon en segundo plano al encender el equipo o iniciar sesión.',
    modal_autostart_enabled: 'Habilitado (Activo en cada reinicio)',
    modal_autostart_disabled: 'Deshabilitado (Manual)',
    modal_autostart_btn_enable: 'Habilitar',
    modal_autostart_btn_disable: 'Deshabilitar',
    modal_close: 'Cerrar',
  },
  en: {
    // Header & Global
    header_driver_fallback: 'Linux Native Driver',
    header_diagnostics: 'Diagnostics',
    header_diagnostics_title: 'View diagnostics and telemetry',
    header_lang_title: 'Switch language (ES / EN)',

    // Status Banners
    banner_daemon_down: 'mx-daemon is not running. Custom shortcuts are inactive.',
    banner_start_service: 'Start Service',
    banner_starting_service: 'Starting...',
    banner_disconnected: 'Mouse disconnected or in Bluetooth sleep. Configuration saved in profile.',
    banner_autoreconnect: 'Auto-reconnect active',
    err_daemon_unreachable: 'mx-daemon is not responding on the IPC socket',
    err_start_service_failed: 'Could not start mx-options service',
    err_apply_action_failed: 'Error communicating with mx-daemon',

    // BatteryBadge
    battery_unknown: 'Unknown',
    battery_stale: '(stale)',
    battery_tooltip_connected: 'Mouse battery',
    battery_tooltip_disconnected: 'Last known reading (mouse disconnected)',

    // MouseViewer
    viewer_service_inactive: 'Service Inactive',
    viewer_disconnected: 'Disconnected',
    viewer_interactive_blueprint: 'Interactive Blueprint',
    viewer_system_default: 'System Default',
    viewer_super_activities: 'Super (Activities)',
    viewer_mute: 'Mute',
    viewer_play_pause: 'Play / Pause',
    viewer_workspace_right: 'Workspace Right (Ctrl+Alt+→)',
    viewer_workspace_left: 'Workspace Left (Ctrl+Alt+←)',
    viewer_alttab: 'Window Switcher (Alt+Tab)',
    viewer_copy: 'Copy (Ctrl+C)',
    viewer_paste: 'Paste (Ctrl+V)',
    viewer_mouse_btn_prefix: 'Button ',
    viewer_disabled: 'Disabled',
    viewer_custom: 'Custom',

    // Physical Buttons (CIDs)
    btn_top_name: 'Top Button (DPI)',
    btn_top_desc: 'Located on the 57° aluminum ridge. Triggers global actions or quick shortcuts.',
    btn_top_cat: 'Crown',
    btn_forward_name: 'Forward Button',
    btn_forward_desc: 'Upper thumb button for forward navigation or custom shortcuts.',
    btn_forward_cat: 'Thumb',
    btn_back_name: 'Back Button',
    btn_back_desc: 'Lower thumb button for back navigation or custom shortcuts.',
    btn_back_cat: 'Thumb',
    btn_wheel_name: 'Scroll Wheel (Middle Click)',
    btn_wheel_desc: 'Precision textured scroll wheel with mechanical middle click.',
    btn_wheel_cat: 'Front',

    // ActionPicker
    picker_factory_badge: 'Factory Default',
    picker_custom_badge: 'Custom',
    picker_active_badge: '✓ Active',
    picker_assign_hint: 'Assign',
    picker_applying: 'Applying configuration in mx-daemon...',

    // ActionPicker Categories
    cat_all: 'All',
    cat_nav: 'Navigation',
    cat_prod: 'Productivity',
    cat_hw: 'Hardware',

    // Preset Actions
    act_super_label: 'Super / Meta (Activities)',
    act_super_desc: 'Opens the desktop overview in GNOME / KDE.',
    act_ws_right_label: 'Workspace: Right (Ctrl + Alt + →)',
    act_ws_right_desc: 'Switches to the virtual workspace on the right.',
    act_ws_left_label: 'Workspace: Left (Ctrl + Alt + ←)',
    act_ws_left_desc: 'Switches to the virtual workspace on the left.',
    act_alttab_label: 'Window Switcher (Alt + Tab)',
    act_alttab_desc: 'Cycles between active applications.',
    act_copy_label: 'Copy (Ctrl + C)',
    act_copy_desc: 'Copies the selected text or file to the clipboard.',
    act_paste_label: 'Paste (Ctrl + V)',
    act_paste_desc: 'Pastes the current clipboard contents.',
    act_mute_label: 'Mute / Unmute Audio',
    act_mute_desc: 'Toggles system audio mute state.',
    act_playpause_label: 'Play / Pause Media',
    act_playpause_desc: 'Global media playback control (Spotify, browsers).',
    act_none_label: 'Disable Button',
    act_none_desc: 'Emits no uinput action when clicked.',
    act_default_label: 'Factory Default (Native Click / Navigation)',
    act_default_desc: 'Restores native hardware control (Back/Forward in browsers, middle click).',

    // HealthModal
    modal_title: 'Internal Telemetry & Diagnostics',
    modal_daemon_state: 'Daemon State',
    modal_uptime: 'Uptime',
    modal_hidpp_device: 'HID++ Device',
    modal_device_none: 'None',
    modal_hidraw_node: 'hidraw Node',
    modal_divert_epoch: 'Divert Epoch',
    modal_reconnections: 'Reconnections',
    modal_queue_dropped: 'Event Queue / Dropped',
    modal_consecutive_failures: 'Consecutive Failures',
    modal_ram_title: 'RAM Memory Usage',
    modal_ram_rss_tag: '(Resident / RSS)',
    modal_ram_daemon: 'Daemon (Service)',
    modal_ram_gui: 'Interface (GUI)',
    modal_last_error: 'Last Recorded Error:',
    modal_empty: 'No telemetry data retrieved yet.',
    modal_querying: 'Querying...',
    modal_refresh: 'Refresh',
    modal_copied: '✓ Copied',
    modal_copy_btn: '📋 Copy Diagnostics',
    modal_autostart_title: 'System Autostart',
    modal_autostart_desc: 'Starts mx-daemon in the background when powering on or logging in.',
    modal_autostart_enabled: 'Enabled (Active on every reboot)',
    modal_autostart_disabled: 'Disabled (Manual)',
    modal_autostart_btn_enable: 'Enable',
    modal_autostart_btn_disable: 'Disable',
    modal_close: 'Close',
  },
};

export const locale = writable(detectInitialLocale());

/**
 * Cambia el idioma activo y lo persiste en localStorage.
 * @param {'es' | 'en'} newLocale
 */
export function setLocale(newLocale) {
  const valid = newLocale === 'en' ? 'en' : 'es';
  locale.set(valid);

  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      window.localStorage.setItem(STORAGE_KEY, valid);
    } catch (_) {}
  }

  if (typeof document !== 'undefined' && document.documentElement) {
    document.documentElement.lang = valid;
  }
}

/**
 * Traduce una clave para un locale específico.
 * @param {'es' | 'en'} lang
 * @param {string} key
 * @returns {string}
 */
export function translate(lang, key) {
  const dict = DICTIONARY[lang] || DICTIONARY.es;
  return dict[key] !== undefined ? dict[key] : (DICTIONARY.es[key] || key);
}

/**
 * Store derivado que devuelve una función traductora reactiva `$t('clave')`.
 */
export const t = derived(locale, ($locale) => {
  return (key) => translate($locale, key);
});

/**
 * Definiciones de los 4 botones físicos traducidas reactivamente según `$locale`.
 */
export function getLocalizedButtons(lang) {
  const tr = (k) => translate(lang, k);
  return [
    {
      cid: 0x00FD,
      id: 'svg-btn-top',
      name: tr('btn_top_name'),
      description: tr('btn_top_desc'),
      category: tr('btn_top_cat'),
    },
    {
      cid: 0x0056,
      id: 'svg-btn-forward',
      name: tr('btn_forward_name'),
      description: tr('btn_forward_desc'),
      category: tr('btn_forward_cat'),
    },
    {
      cid: 0x0053,
      id: 'svg-btn-back',
      name: tr('btn_back_name'),
      description: tr('btn_back_desc'),
      category: tr('btn_back_cat'),
    },
    {
      cid: 0x0052,
      id: 'svg-btn-wheel',
      name: tr('btn_wheel_name'),
      description: tr('btn_wheel_desc'),
      category: tr('btn_wheel_cat'),
    },
  ];
}

export const localizedButtons = derived(locale, ($locale) => getLocalizedButtons($locale));

/**
 * Obtiene el nombre legible y localizado de la acción asignada a un CID.
 */
export function getLocalizedActionSummary(cid, mapping, lang) {
  const tr = (k) => translate(lang, k);
  if (!mapping || !mapping[cid]) return tr('viewer_system_default');
  const act = mapping[cid];
  if (act.type === 'SingleKey') {
    if (act.key === 'KEY_LEFTMETA' || act.key === 'SUPER' || act.key === 'META') return tr('viewer_super_activities');
    if (act.key === 'KEY_MUTE') return tr('viewer_mute');
    if (act.key === 'KEY_PLAYPAUSE') return tr('viewer_play_pause');
    return act.key;
  }
  if (act.type === 'KeyCombo') {
    const mods = Array.isArray(act.modifiers) ? act.modifiers : [];
    if (act.key === 'KEY_RIGHT' && mods.includes('KEY_LEFTCTRL') && mods.includes('KEY_LEFTALT')) {
      return tr('viewer_workspace_right');
    }
    if (act.key === 'KEY_LEFT' && mods.includes('KEY_LEFTCTRL') && mods.includes('KEY_LEFTALT')) {
      return tr('viewer_workspace_left');
    }
    if (act.key === 'KEY_TAB' && mods.includes('KEY_LEFTALT') && mods.length === 1) {
      return tr('viewer_alttab');
    }
    if (act.key === 'KEY_C' && mods.includes('KEY_LEFTCTRL') && mods.length === 1) {
      return tr('viewer_copy');
    }
    if (act.key === 'KEY_V' && mods.includes('KEY_LEFTCTRL') && mods.length === 1) {
      return tr('viewer_paste');
    }
    return `${mods.join('+')}+${act.key}`.replace(/KEY_/g, '');
  }
  if (act.type === 'MouseButton') return act.button.replace('BTN_', tr('viewer_mouse_btn_prefix'));
  if (act.type === 'None') return tr('viewer_disabled');
  if (act.type === 'SystemDefault') return tr('viewer_system_default');
  return tr('viewer_custom');
}
