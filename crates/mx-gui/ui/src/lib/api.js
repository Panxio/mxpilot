import { invoke } from '@tauri-apps/api/core';

/**
 * Obtiene el estado funcional del daemon y ratón conectado.
 * @returns {Promise<import('./types').StatusData>}
 */
export async function getStatus() {
  return await invoke('get_status');
}

/**
 * Obtiene la telemetría interna y métricas de salud del daemon.
 */
export async function getHealth() {
  return await invoke('get_health');
}

/**
 * Consulta la batería en tiempo real a través del daemon.
 */
export async function getBattery() {
  return await invoke('get_battery');
}

/**
 * Asigna una acción a un botón (CID) tras validación en allowlist.
 * @param {number} cid
 * @param {object} action
 */
export async function setAction(cid, action) {
  return await invoke('set_action', { cid, action });
}

/**
 * Invoca systemctl para levantar el servicio de usuario de mx-daemon.
 */
export async function startDaemonService() {
  return await invoke('start_daemon_service');
}

/**
 * Lee el uso de memoria VmRSS (en KB) del proceso propio de la GUI.
 */
export async function getGuiMemory() {
  return await invoke('get_gui_memory');
}
