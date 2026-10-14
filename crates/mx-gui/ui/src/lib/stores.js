import { writable, derived } from 'svelte/store';

/**
 * Estados de la máquina honesta de 4 vías:
 * - 'Connecting': Iniciando o reconectando.
 * - 'Connected': Daemon activo Y ratón conectado.
 * - 'DaemonDown': Socket inaccesible o servicio detenido.
 * - 'DeviceDisconnected': Daemon activo pero ratón apagado / fuera de alcance.
 */
export const uiState = writable('Connecting');

export const statusData = writable({
  connected: false,
  device_name: null,
  device_path: null,
  battery_level: null,
  battery_status: null,
  diverted_cids: [],
  mapping: {},
});

export const healthData = writable(null);
export const guiMemoryKb = writable(0);
export const showHealthModal = writable(false);

// Botón seleccionado actualmente en el visualizador (por defecto: Botón Superior 0x00FD)
export const selectedCid = writable(0x00FD);

// Estado de petición en vuelo para reconciliación honesta
export const isPendingAction = writable(false);

// Mensajes de error u advertencia visibles
export const errorMessage = writable(null);

// Timestamp de última actualización de datos
export const lastUpdated = writable(null);

export const BUTTON_DEFINITIONS = [
  {
    cid: 0x00FD,
    id: 'svg-btn-top',
    name: 'Botón Superior (DPI / Kill-Gate)',
    description: 'Ubicado en la cresta metálica de 57°. Controla acciones globales o atajos rápidos.',
    category: 'Cúspide',
  },
  {
    cid: 0x0056,
    id: 'svg-btn-forward',
    name: 'Botón Lateral Adelante',
    description: 'Botón superior del pulgar para avance de navegación.',
    category: 'Pulgar',
  },
  {
    cid: 0x0053,
    id: 'svg-btn-back',
    name: 'Botón Lateral Atrás',
    description: 'Botón inferior del pulgar para retroceso de navegación.',
    category: 'Pulgar',
  },
  {
    cid: 0x0052,
    id: 'svg-btn-wheel',
    name: 'Rueda Central (Clic Central)',
    description: 'Rueda de scroll texturizada con clic mecánico.',
    category: 'Frontal',
  },
];
