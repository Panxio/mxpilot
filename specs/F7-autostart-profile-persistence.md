# Especificación Técnica (SDD) — Fase 7: Persistencia de Perfil en Disco y Autoarranque Desatendido

> **Capa:** `mx-daemon`, `mx-gui`, `packaging`  
> **Perfil:** `tauri-app`  
> **Versión objetivo:** `v0.1.1`  
> **Fecha:** 2026-10-18  

---

## 1. Problema y Contexto

En la versión `v0.1.0` de MX Pilot:
1. **Pérdida del servicio al reiniciar:** Al hacer clic en "Iniciar Servicio" en la GUI, se invoca `systemctl --user start mx-options.service`. El comando `start` únicamente activa la unidad para la sesión viva actual; al reiniciar el laptop o cerrar sesión, systemd no la reactiva porque la unidad nunca fue habilitada (`enable`).
2. **Amnesia de personalizaciones (Perfiles volátiles):** `DaemonState::new()` inicializa los mapeos en RAM únicamente con el botón superior (`TOP_BUTTON_CID -> KEY_LEFTMETA`). Los cambios de botones vía `set_action` viven exclusivamente en la memoria del daemon. Si el servicio arranca automáticamente tras un reinicio sin abrir la GUI, las personalizaciones de botones laterales o rueda no existen.
3. **Restricción de Sandbox en systemd:** `mx-options.service` utiliza `ProtectHome=read-only`. Cualquier intento del daemon de guardar configuración en `$HOME/.config/` fallará con `EROFS` a menos que se perfore quirúrgicamente el sandbox con `ConfigurationDirectory=mx-options` y `ReadWritePaths=%h/.config/mx-options`.

---

## 2. Invariantes del Sistema (Pilar 2)

- **Invariante 1 (Zero Dead Buttons / Fail-Safe):** Si el archivo `profile.json` no existe, está corrupto o tiene sintaxis inválida, `mx-daemon` NUNCA debe entrar en pánico ni abortar su inicio. Debe emitir un warning y aplicar el mapeo por defecto (`TOP_BUTTON_CID -> Super`).
- **Invariante 2 (Escritura Atómica POSIX):** La persistencia en disco se realiza escribiendo en un archivo temporal (`.profile.json.tmp.<pid>`), forzando `sync_all()` al descriptor y renombrando atómicamente con `rename`. No puede ocurrir corrupción o truncado a 0 bytes por corte abrupto de energía.
- **Invariante 3 (Hardware Confirmation First):** `set_action` solo persiste a disco si el firmware confirmó `Ok(())`. Operaciones fallidas de hardware jamás alteran el perfil persistente.
- **Invariante 4 (Permisos Canónicos 0600):** El archivo de perfil reside en `$XDG_CONFIG_HOME/mx-options/profile.json` y se guarda con permisos de lectura/escritura exclusivos para el usuario (`UMask=0177`).
- **Invariante 5 (Zero UI Hang):** Las invocaciones a `systemctl` desde la GUI se realizan en hilos bloqueantes dedicados (`spawn_blocking`) con timeouts estrictos para no congelar el bucle de eventos GTK.
- **Invariante 6 (Sandbox Integrity):** `ProtectHome=read-only` se mantiene intacto para todo `/home`, concediendo acceso de escritura única y exclusivamente a `%h/.config/mx-options`.

---

## 3. Arquitectura del Perfil (`crates/mx-daemon/src/profile.rs`)

### 3.1 Estructura del Perfil
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub version: u32,
    pub mapping: HashMap<u16, ActionPayload>,
}
```

### 3.2 Rutas de Acceso
- Directorio: `$XDG_CONFIG_HOME/mx-options` (fallback: `$HOME/.config/mx-options`).
- Archivo: `$XDG_CONFIG_HOME/mx-options/profile.json`.

### 3.3 Funciones Principales
- `load_profile() -> HashMap<u16, ActionPayload>`: Carga y valida el archivo. Retorna el mapa recuperado o el default si hay error/ausencia.
- `save_profile(mapping: &HashMap<u16, ActionPayload>) -> Result<(), String>`: Escritura atómica a disco.
- `load_profile_from_path(path: &Path) -> Result<HashMap<u16, ActionPayload>, String>`: Para pruebas unitarias.
- `save_profile_to_path(path: &Path, mapping: &HashMap<u16, ActionPayload>) -> Result<(), String>`: Para pruebas unitarias.

---

## 4. Protocolo IPC y Ciclo de Vida

1. **`SetAction`:**
   - Validar CID y payload contra allowlist.
   - Enviar a `hid_worker` -> firmware HID++.
   - Recibir respuesta del hardware.
   - Si `Ok(())`: actualizar `s.mapping` en RAM y llamar a `save_profile(&s.mapping)`.
   - Si `Err`: devolver error IPC y no tocar disco.
2. **`ReloadProfile`:**
   - Cargar `profile.json` desde disco.
   - Actualizar `s.mapping` en RAM.
   - Enviar `DaemonCommand::ReloadProfile` a `hid_worker`.
   - `hid_worker` ajusta los desvíos en el mouse (desvía los CIDs activos y libera los que ya no están).
   - Devolver `Ok("Perfil recargado")`.

---

## 5. Endurecimiento de systemd

En `packaging/common/mx-options.service` y `systemd/user/mx-options.service`:
```ini
ConfigurationDirectory=mx-options
ReadWritePaths=%h/.config/mx-options
```

---

## 6. Autoarranque en la GUI (`mx-gui`)

1. **`start_daemon_service`:** Cambiar de `["--user", "start", "..."]` a:
   `["--user", "enable", "--now", "mx-options.service"]`
2. **Nuevos Comandos Tauri:**
   - `is_daemon_service_enabled() -> Result<bool, String>`: Consulta `systemctl --user is-enabled mx-options.service`.
   - `set_daemon_service_autostart(enable: bool) -> Result<(), String>`: Ejecuta `systemctl --user enable` o `disable`.
3. **UI en Svelte:**
   - Selector en `HealthModal.svelte` para consultar y conmutar el estado de autoarranque con el sistema.
   - Textos internacionalizados en ES y EN con paridad 100%.

---

## 7. Empaquetado

En `packaging/common/50-mx-options.preset` (o `default.target.wants`):
- Proporcionar preset de usuario `/usr/lib/systemd/user-preset/50-mx-options.preset` con:
  ```ini
  enable mx-options.service
  ```
- Mantener compatibilidad estricta con el arnés de auditoría `scripts/audit-packages.sh` (modos `0644`, propietario `root:root`).
