# Especificación F6 — Pulido Final, Soporte Bilingüe (i18n), Empaquetado RPM y Limpieza del Repositorio

> **Fase:** Fase 6 (Pulido Final Pre-Release)  
> **Perfil:** tauri-app (Rust Core + Svelte/Tailwind UI)  
> **Objetivo:** Preparar el repositorio oficial `Panxio/mxpilot` para su primer release público `v0.1.0` con paridad total de distribución (.deb + .rpm), soporte bilingüe (ES/EN) en GUI y documentación, y un repositorio limpio sin ruido de workflows de agentes, siguiendo el estándar de calidad de `Panxio/lekto`.

---

## 1. Alcance y Objetivos de la Fase 6

### 1.1 Empaquetado RPM Oficial (Fedora / RHEL / openSUSE)
- **Problema:** Actualmente el CI de GitHub solo construye y distribuye paquetes `.deb`. Los usuarios de Fedora, Red Hat Enterprise Linux y distribuciones basadas en RPM no cuentan con un instalador binario listo para usar.
- **Solución:**
  1. Proveer un script local determinista `scripts/package-rpm.sh` que compile los binarios con optimizaciones de release, empaquete los archivos FHS correspondientes y genere el `.rpm` canónico con `rpmbuild`.
  2. Actualizar el workflow de CI en `.github/workflows/release.yml` para compilar y empaquetar tanto `.deb` como `.rpm` en paralelo, adjuntando ambos artefactos y sus sumas SHA256 al release de GitHub.
  3. Actualizar `packaging/rpm/mx-options.spec` con la URL oficial del repositorio (`https://github.com/Panxio/mxpilot`).
- **Invariante:** El script de mantenedor `%preun` del paquete RPM debe cumplir la garantía **Zero Dead Buttons** (restauración del firmware al desinstalar, matando limpiamente el daemon antes de eliminar archivos).

---

### 1.2 Limpieza y Descontaminación del Repositorio
- **Problema:** El repositorio contiene scripts internos de tandas de subagentes (`scripts/run-f*-tanda.sh`), bitácoras de brainstorming y protocolos de gobernanza (`docs/SUBAGENTES.md`, `docs/BRAINSTORMING*`, `hooks/`) que corresponden al método de construcción de los agentes y no al producto final de cara al usuario.
- **Solución (Inspirada en `Panxio/lekto`):**
  1. Desacoplar del seguimiento de git (`git rm --cached`) todo el andamiaje interno de orquestación de subagentes.
  2. Preservar las herramientas locales en un directorio ignorado por git (ej. `.internal/` o `.factory/`) para no perder capacidad de ejecución local de subagentes si se requiere.
  3. Mantener en el árbol público únicamente:
     - Código fuente de producción (`crates/*`).
     - Activos visuales y de escritorio (`assets/*`, iconos).
     - Archivos de empaquetado FHS (`packaging/*`).
     - Scripts limpios de ayuda al usuario (`scripts/install-*.sh`, `scripts/package-*.sh`, `scripts/audit-packages.sh`).
     - Especificaciones técnicas de diseño (`specs/*`).
     - Licencia oficial GPL-2.0 (`LICENSE`).
     - Manifiestos de dependencias (`Cargo.toml`, `Cargo.lock`).
     - Flujos de GitHub Actions (`.github/*`).

---

### 1.3 Documentación Bilingüe Estructurada al Estilo Lekto
- **Problema:** El `README.md` actual contiene tablas de avance de desarrollo y le falta una estructura clara orientada al usuario final, además de carecer de versión en inglés.
- **Solución:**
  1. Crear `README.md` (Español) con enlace superior `[English](README.en.md)`.
  2. Crear `README.en.md` (Inglés) con enlace superior `[Español](README.md)`.
  3. Estructura canónica basada en `Panxio/lekto`:
     - Título e insignia de versión / licencia.
     - Resumen ejecutivo breve y directo.
     - Blueprint visual y capturas de la aplicación.
     - Sección de **Rendimiento y arquitectura** (Rust puro, Svelte, consumo < 3 MB RAM daemon, tamaño de paquetes).
     - Sección de **Funciones** (Divert de botones, uinput a nivel de kernel, bandeja del sistema, interfaz bilingüe).
     - **Requisitos del sistema** (Linux, udev, systemd, soporte hidraw).
     - **Instalación rápida** (.deb para Ubuntu/Debian/Mint, .rpm para Fedora/RHEL).

---

### 1.4 Soporte Bilingüe Nativo en la GUI (i18n Español / Inglés)
- **Problema:** La interfaz gráfica actual solo presenta textos en español. Usuarios angloparlantes o con sistemas en inglés ven textos en español sin opción de cambio.
- **Solución:**
  1. Crear tienda reactiva de internacionalización (`crates/mx-gui/ui/src/lib/i18n.js`).
  2. Detección automática del idioma predeterminado del sistema (`navigator.language`). Si empieza con `es`, selecciona `es`; de lo contrario, `en`.
  3. Selector visual de idioma (pill `ES | EN`) en la barra superior de la ventana principal.
  4. Persistencia en `localStorage.getItem('mxpilot_locale')`.
  5. Diccionario exhaustivo de cadenas de traducción para:
     - Estados del ratón y daemon ("Conectado" / "Connected", "Desconectado" / "Disconnected").
     - Batería y tooltips.
     - Identificadores de los 4 botones físicos del MX Vertical:
       - Top Button (DPI) / Botón Superior (DPI).
       - Middle Button (Scroll) / Rueda de Scroll.
       - Forward Button / Botón Adelante.
       - Back Button / Botón Atrás.
     - Acciones y categorías del ActionPicker:
       - Categorías: Navegación, Sistema, Teclas Especiales, Mouse, Personalizado.
       - Acciones: Workspace Left/Right, Window Switcher, Copy, Paste, Undo, Redo, Mute, System Default, etc.
     - Modal de telemetría y diagnósticos:
       - Título, métricas de hardware, consumo de memoria RAM residente, botones de actualización y copia.
     - Textos del System Tray en Rust / Tauri (menú contextual).

---

## 2. Invariantes Técnicos (Pilar 2)
1. **Zero Dead Buttons:** La desinstalación de cualquier paquete (.deb o .rpm) debe restaurar de inmediato el comportamiento nativo del hardware.
2. **Cero Dependencias Pesadas de Runtime:** No agregar librerías externas de i18n pesadas en npm. Un store ligero de Svelte con mapeo de claves es 100% suficiente (< 2 KB) y respeta el perfil ultraliviano.
3. **Consistencia Visual:** El selector de idioma debe integrarse armoniosamente en el header junto al botón de Telemetría sin desbordar el layout de 960x620.
4. **Verificación Automatizada:** Los tests unitarios existentes (`npm test`, `cargo test`) deben seguir pasando al 100%, y deben añadirse aserciones para las claves del diccionario de traducción.

---

## 3. Dictamen de Auditoría Técnica (Gemini 4.0) y Resoluciones Implementadas

### 3.1 ¿Existen inconsistencias o riesgos en el empaquetado dual RPM / DEB en GitHub Actions?
- **Hallazgo:** Construir un RPM binario desde un runner `ubuntu-24.04` (o Fedora) a partir de ejecutables Rust pre-compilados y *stripped* presenta tres riesgos clásicos en `rpmbuild`:
  1. Generación fallida de paquetes `-debuginfo` o enlaces `.build-id` duplicados en `/usr/lib/.build-id/`.
  2. El generador automático `find-requires` en un host Debian puede inyectar símbolos específicos de glibc/Debian incompatibles con la base de datos RPM de Fedora/openSUSE.
  3. Desalineación de versión cuando el tag de Git avanza y el `.spec` mantiene una versión estática.
- **Resolución Aplicada:**
  - Incorporadas directivas `%global debug_package %{nil}`, `%define _build_id_links none` y `AutoReqProv: no` en [`packaging/rpm/mx-options.spec`](../packaging/rpm/mx-options.spec), declarando explícitamente las dependencias nativas de Fedora/RHEL (`webkit2gtk4.1`, `gtk3`, `libayatana-appindicator-gtk3`, `systemd`, `systemd-udev`, `procps-ng`).
  - Parametrizadas las macros `%{?!pkg_version:0.1.0}` y `%{?!pkg_release:1}` en el `.spec`, alimentadas dinámicamente por [`scripts/package-rpm.sh`](../scripts/package-rpm.sh) desde `Cargo.toml` o `${GITHUB_REF_NAME#v}`.
  - Unificadas todas las URLs de metadatos (`Homepage`, `URL`, `Source`) a `https://github.com/Panxio/mxpilot`.
  - Extendido el arnés [`scripts/audit-packages.sh`](../scripts/audit-packages.sh) para auditar tanto `.deb` como `.rpm` (tamaño `< 15 MB`, propietarios `root:root`, modos `0755/0644`, cero fugas de `/home/panxio`, scripts de mantenedor y dependencias).

### 3.2 ¿Qué recomendaciones técnicas aporta para `%preun` y `%postun` en distribuciones RPM con SELinux activo?
- **Hallazgo:** En Fedora y RHEL con SELinux en modo `enforcing` (política `targeted`), `rpm-plugin-selinux` etiqueta automáticamente los archivos instalados en `/usr/bin`, `/usr/lib/udev/rules.d` y `/usr/lib/systemd/user` con sus contextos canónicos (`bin_t`, `udev_rules_t`, `systemd_unit_file_t`). Sin embargo, si cualquier comando dentro de `%post`, `%preun` o `%postun` devuelve un código distinto de cero (por ejemplo, si `pkill` es bloqueado por una política MLS personalizada o no encuentra procesos activos), `rpm`/`dnf` aborta la transacción, dejando paquetes huérfanos imposibles de desinstalar.
- **Resolución Aplicada:**
  - En `%preun`, la parada **Zero Dead Buttons** se ejecuta exclusivamente en desinstalación definitiva (`if [ "$1" = "0" ]; then ... fi`), preservando el servicio activo durante actualizaciones (`$1 >= 1`).
  - Todos los scriptlets (`%post`, `%preun`, `%postun`) están protegidos con `2>/dev/null || true` y finalizan explícitamente con `exit 0`.
  - Declarados permisos explícitos `%defattr(0644,root,root,0755)` y `%attr(0755,root,root)` para los 5 binarios en `%files`.

### 3.3 ¿Es óptima la arquitectura de i18n reactiva propuesta en Svelte sin sobrecargar el bundle?
- **Hallazgo:** Librerías externas como `svelte-i18n` o `i18next` añaden entre 15 KB y 45 KB minificados y dependencias transitivas innecesarias para un esquema de 2 idiomas (`es` / `en`). Además, un cambio de idioma en el frontend dejaría los textos del menú contextual del System Tray (manejado por Rust en `libayatana-appindicator`) en el idioma anterior si no se sincronizan ambas capas.
- **Resolución Aplicada:**
  - Implementado [`crates/mx-gui/ui/src/lib/i18n.js`](../crates/mx-gui/ui/src/lib/i18n.js) con stores nativos de Svelte (`writable` + `derived`), sumando apenas ~3 KB al bundle y **0 dependencias externas**.
  - Paridad del 100% verificada por test unitario sobre las **93 claves** de `DICTIONARY.es` y `DICTIONARY.en`.
  - Añadido el comando Tauri `set_locale` en [`crates/mx-gui/src/commands.rs`](../crates/mx-gui/src/commands.rs) y actualización reactiva en caliente de los `MenuItem` de [`crates/mx-gui/src/tray.rs`](../crates/mx-gui/src/tray.rs), sincronizando instantáneamente el menú de bandeja del sistema (`Mostrar/Show`, `Batería/Battery`, `Reiniciar Daemon/Restart Daemon`, `Salir/Quit`) al alternar el selector `ES | EN`.

### 3.4 ¿Qué aspectos adicionales de UX o empaquetado deben ser considerados antes de sellar el tag `v0.1.0`?
- **Higiene del árbol Git:** Se trasladaron los scripts de tandas de subagentes, bitácoras internas y hooks a `.internal/` (ignorado por Git) y se desacoplaron los esquemas autogenerados `crates/mx-gui/gen/schemas/`, logrando un árbol público idéntico en pureza a `Panxio/lekto`.
- **Tests de Frontend en CI:** Se incorporó el paso `npm --prefix crates/mx-gui/ui test` en `.github/workflows/release.yml` para que las 16 pruebas de acciones e internacionalización actúen como compuerta dura antes de empaquetar.

