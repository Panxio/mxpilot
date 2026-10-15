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

## 3. Plan de Verificación y Auditoría con Gemini 4.0 (Pilar 5 & Pilar 6)
- Someter esta especificación y la base de código actual a una auditoría exhaustiva con el nuevo modelo `opencode-go/gemini-4-argon-contributor` (Gemini 4.0).
- Preguntas clave para la auditoría de Gemini 4.0:
  1. ¿Existen inconsistencias o riesgos en el empaquetado dual RPM / DEB en GitHub Actions?
  2. ¿Qué recomendaciones técnicas aporta para el script `%preun` y `%postun` en distribuciones basadas en RPM con SELinux activo?
  3. ¿Es óptima la arquitectura de i18n reactiva propuesta en Svelte sin sobrecargar el bundle?
  4. ¿Qué aspectos adicionales de UX o empaquetado deben ser considerados antes de sellar el tag `v0.1.0`?
