# Lecciones Aprendidas: Iconografía, System Tray e Integración de Escritorio en Linux (Tauri v2 + GNOME)

> **Documento de Lecciones Aprendidas de Ingeniería y Factoría de Software**  
> **Proyecto:** MX Options Linux (`mx-options-linux`)  
> **Entorno:** Ubuntu 24.04 LTS / GNOME Shell 46+ (Wayland / X11)  
> **Stack:** Tauri v2 (Rust + WebKitGTK) + Svelte + Ayatana AppIndicator + FreeDesktop / XDG  
> **Fecha de Consolidación:** 2026-10-14  

---

## 1. Contexto y Propósito

Durante el desarrollo de la Fase 4 (Interfaz Gráfica de Usuario y Desktop Shell) de `mx-options-linux`, nos enfrentamos a desafíos recurrentes en la visualización de iconos en el ecosistema Linux moderno (GNOME Dash, Ubuntu Dock, System Tray / AppIndicator y ventanas de WebKitGTK).

Este documento compila de forma sistemática los **defectos encontrados, sus causas raíces técnicas y las soluciones definitivas verificadas**, con el objetivo de servir como playbook exportable a la factoría de agentes para cualquier proyecto futuro basado en **Tauri + Rust + Linux Desktop**.

---

## 2. Problemas Encontrados y Soluciones Definitivas

### Defecto 1: El icono aparece "casi negro" o apagado en GNOME Dash y Ubuntu Dock

#### Síntoma Observado por el Usuario
Al presionar `Super` para buscar la aplicación o al anclarla al Dock izquierdo de Ubuntu, el contenedor del icono se visualizaba como un bloque oscuro/negro con apenas unos reflejos verdes tenues en los bordes.

#### Causa Raíz Técnica
1. **Fondo del Dock en GNOME:** El tema oscuro nativo de Ubuntu y GNOME Shell utiliza un fondo semi-transparente de tono `#1e1e1e` a `#111111`.
2. **Contraste Cromático Insuficiente:** La propuesta inicial utilizaba un fondo squircle esmeralda oscuro `#052617` o `#0f2d1e`. El ratio de contraste relativo entre `#052617` y el fondo `#1e1e1e` es de apenas **1.35:1** (por debajo del mínimo recomendado WCAG de 3:1 para elementos gráficos).
3. **Escalado y Filtros Ópticos:** GNOME aplica un filtro de contraste y suavizado a los lanzadores. Un fondo de bajo contraste se fusiona visualmente con el fondo del panel, haciendo que el ojo humano lo perciba como negro puro.

#### Solución Definitiva (Regla Óptica S8)
- **Base Sólida Vibrante:** Establecer el fondo del squircle en un color cromáticamente activo con contraste $\ge 4.5:1$ contra `#1e1e1e`. En nuestro caso, verde esmeralda vibrante `#059669` (contraste **5.2:1**).
- **Monograma de Alto Contraste:** Elementos gráficos interiores en blanco puro `#ffffff` (contraste **4.5:1** contra `#059669`).
- **Geometría Squircle:** Utilizar un radio de esquina proporcional de `rx="112"` sobre un viewBox de `512x512` (equivalente al 21.8% del ancho).

```svg
<!-- assets/icon-proposals/opcion-2-esmeralda-vibrante.svg -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <rect width="512" height="512" rx="112" ry="112" fill="#059669"/>
  <g fill="#ffffff">
    <!-- Monograma o isotipo vectorial plano -->
  </g>
</svg>
```

---

### Defecto 2: Glifos tipográficos empastados / solapados a escalas pequeñas

#### Síntoma Observado
En resoluciones de lanzador (32x32, 48x48) o en el indicador del Dash, las letras 'M' y 'X' del monograma se tocaban y superponían en el centro, pareciendo una sola mancha distorsionada.

#### Causa Raíz Técnica
1. **Falta de Kerning Óptico para Submuestreo:** Un espaciado entre letras adecuado en un canvas de 512px (ej. 4px a 8px) se convierte en menos de medio píxel al reducir a 48px o 32px. El algoritmo de interpolación bicúbica funde los bordes antialiasing de ambas letras.
2. **Efectos Skeuomórficos y Sombras:** El uso de filtros gaussianos (`feGaussianBlur`), sombras arrojadas y biseles de degradado añade "ruido" que ensucia los canales alfa al reducir de tamaño.

#### Solución Definitiva
- **Separación Mínima Garantizada:** Dejar al menos **20px a 24px** de separación horizontal entre glifos en un canvas de 512x512.
- **Trazos Planos y Robustos:** Usar grosores de trazo ortogonales de al menos **38px a 40px** en 512x512 para que a 32x32 correspondan al menos a 2.5 píxeles reales limpios.
- **Pureza Vectorial:** Cero sombras arrojadas, cero filtros SVG borrosos dentro de iconos de sistema. El contraste debe ser generado por el color y la geometría, no por sombras simuladas.

---

### Defecto 3: System Tray (Ayatana / AppIndicator) muestra tres puntos (`...`) o falla en cargar

#### Síntoma Observado
Al arrancar la aplicación, el icono en la barra superior de GNOME no aparecía o mostraba tres puntos suspensivos (`...`), impidiendo ver la identidad de la aplicación.

#### Causa Raíz Técnica
1. **Profundidad de Color de ImageMagick:** Al rasterizar SVG a PNG mediante `convert` en sistemas Linux modernos, ImageMagick (compilado típicamente en modo Q16) produce por defecto imágenes PNG con **16 bits por canal de color** (64 bits por píxel: 16-bit Red, 16-bit Green, 16-bit Blue, 16-bit Alpha).
2. **Incompatibilidad de `libayatana-appindicator`:** La biblioteca `libayatana-appindicator3` y el subsistema de pixbuf de GTK/GDK en ciertos compositores esperan buffers estándar RGBA de **8 bits por canal** (32 bits por píxel). Al recibir un PNG de 64 bpp, la deserialización de pixbuf falla silenciosamente y la bandeja cae en el icono de fallback (`...`).

#### Solución Definitiva
Forzar explícitamente profundidad de 8 bits y formato `png32:` en todos los comandos de rasterización:

```bash
# ❌ INCORRECTO (puede generar 64 bpp con ImageMagick Q16):
convert icon.svg 32x32.png

#  CORRECTO (garantiza 32 bpp / 8-bit RGBA compatible con Ayatana):
convert -background none -density 384 -resize 32x32 -depth 8 icon.svg png32:32x32.png
```

---

### Defecto 4: GNOME Shell no refleja los iconos nuevos tras modificarlos

#### Síntoma Observado
A pesar de haber reemplazado los archivos PNG y SVG en disco, GNOME seguía mostrando el icono viejo o el icono por defecto de ventana.

#### Causa Raíz Técnica
1. **Caché Mmap de XDG:** GNOME Shell lee los iconos desde un archivo binario indexado en memoria (`~/.local/share/icons/hicolor/icon-theme.cache`). Modificar los archivos en el árbol de carpetas no invalida automáticamente el mapeo en memoria de GNOME Shell.
2. **Jerarquía Hicolor Incompleta:** Si el tema no encuentra el tamaño exacto solicitado por el monitor (ej. 48x48 o 64x64 en pantallas con escalado fraccional al 125%), intenta buscar el SVG escalable. Si falta `scalable/apps/`, recurre a iconos del sistema genéricos.

#### Solución Definitiva
1. **Desplegar la jerarquía completa de Hicolor:**
   - `~/.local/share/icons/hicolor/16x16/apps/<app>.png`
   - `~/.local/share/icons/hicolor/24x24/apps/<app>.png`
   - `~/.local/share/icons/hicolor/32x32/apps/<app>.png`
   - `~/.local/share/icons/hicolor/48x48/apps/<app>.png`
   - `~/.local/share/icons/hicolor/64x64/apps/<app>.png`
   - `~/.local/share/icons/hicolor/128x128/apps/<app>.png`
   - `~/.local/share/icons/hicolor/256x256/apps/<app>.png`
   - `~/.local/share/icons/hicolor/512x512/apps/<app>.png`
   - `~/.local/share/icons/hicolor/scalable/apps/<app>.svg`
2. **Actualizar la caché explícitamente:**
   ```bash
   gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor
   ```

---

### Defecto 5: Disociación de Ventana en el Dock (`StartupWMClass`)

#### Síntoma Observado
Al abrir la aplicación, el icono en el Dock aparecía con el icono genérico gris o creaba un lanzador duplicado sin icono.

#### Causa Raíz Técnica
GNOME Shell empareja las ventanas de Wayland/X11 con sus archivos `.desktop` mediante la propiedad `WM_CLASS` de la ventana. En Tauri (WebKitGTK), el `WM_CLASS` por defecto es el nombre del binario compilado (ej. `mx-gui`). Si el `.desktop` no tiene `StartupWMClass=mx-gui`, GNOME no puede enlazar la ventana al lanzador.

#### Solución Definitiva
Configurar en `~/.local/share/applications/<app>.desktop`:
```ini
[Desktop Entry]
Name=MX Options Linux
Exec=/home/panxio/.local/bin/mx-gui
Icon=mx-options
Terminal=false
Type=Application
Categories=Utility;Settings;HardwareSettings;
StartupWMClass=mx-gui
StartupNotify=true
```

---

### Defecto 6: Pérdida de botones nativos del ratón al desviar hardware (HID++ 2.0)

#### Síntoma Observado
Al deshabilitar un botón lateral en la UI o asignarle una acción, se perdía la navegación web estándar de Atrás / Adelante en navegadores (Chrome, Firefox).

#### Causa Raíz Técnica
En ratones Logitech con protocolo HID++ 2.0 (Feature `0x1B04` Special Keys), cuando un botón se marca con `divert = true`, el firmware del ratón **suprime el reporte HID estándar** (`BTN_SIDE`, `BTN_EXTRA`, `BTN_MIDDLE`). Si el daemon no traduce el evento o lo mapea a `None`, el sistema operativo nunca recibe los clics nativos.

#### Solución Definitiva
1. **Comando de Des-Desvío (`divert = false`):** Cuando el usuario selecciona "Función de Fábrica", el daemon envía a hardware `set_cid_divert(cid, false)` y elimina el botón de la lista de desviados. El hardware del ratón vuelve a emitir eventos HID estándar directamente al kernel.
2. **Acción Canónica `SystemDefault`:** Diseñar un tipo de payload `SystemDefault` tanto en el protocolo IPC Rust como en la interfaz de usuario, garantizando reconciliación honesta con el estado del firmware.

---

### Defecto 7: Mismatch en la comparación de acciones (Orden de claves JSON)

#### Síntoma Observado
Al seleccionar una acción en la UI, el daemon la aplicaba con éxito, pero la UI no mostraba el badge "✓ Activo" porque la comparación de objetos fallaba.

#### Causa Raíz Técnica
Rust `serde_json` serializa estructuras con claves en orden variable según la declaración (`{"key":"...","type":"..."}` vs `{"type":"...","key":"..."}`). En JavaScript, la comparación por string o por claves directas fallaba. Asimismo, los modificadores en `KeyCombo` podían llegar en orden inverso (`['KEY_LEFTALT', 'KEY_LEFTCTRL']` vs `['KEY_LEFTCTRL', 'KEY_LEFTALT']`).

#### Solución Definitiva
Construir un comparador semántico canónico (`isActionEqual` en `actions.js`):
- Compara modificadores utilizando conjuntos (`Set`), insensible al orden y deduplicando.
- Normaliza alias de teclas (`SUPER` $\rightarrow$ `KEY_LEFTMETA`, `CTRL` $\rightarrow$ `KEY_LEFTCTRL`).
- Compara propiedades individuales independientemente del orden de serialización JSON.

---

## 3. Guía de Rasterización Estándar para la Factoría

Para automatizar la generación de assets en futuros proyectos Tauri + Linux, utilizar este script de referencia:

```bash
#!/usr/bin/env bash
set -euo pipefail

SVG_SRC="crates/mx-gui/icons/mx-options.svg"
DEST_ICONS="crates/mx-gui/icons"
HICOLOR="$HOME/.local/share/icons/hicolor"

echo "🎨 [1/3] Rasterizando variantes PNG 8-bit RGBA (png32)..."
for size in 16 24 32 48 64 128 256 512; do
  convert -background none -density 384 -resize "${size}x${size}" -depth 8 "$SVG_SRC" "png32:${DEST_ICONS}/${size}x${size}.png"
done

cp "${DEST_ICONS}/256x256.png" "${DEST_ICONS}/128x128@2x.png"
cp "${DEST_ICONS}/512x512.png" "${DEST_ICONS}/icon.png"

echo "🖼️  [2/3] Generando ICO multi-resolución..."
convert "${DEST_ICONS}/16x16.png" "${DEST_ICONS}/32x32.png" "${DEST_ICONS}/48x48.png" \
        "${DEST_ICONS}/64x64.png" "${DEST_ICONS}/128x128.png" "${DEST_ICONS}/256x256.png" \
        "${DEST_ICONS}/icon.ico"

echo "📦 [3/3] Instalando en árbol XDG Hicolor y actualizando caché..."
for size in 16 24 32 48 64 128 256 512; do
  mkdir -p "${HICOLOR}/${size}x${size}/apps"
  cp "${DEST_ICONS}/${size}x${size}.png" "${HICOLOR}/${size}x${size}/apps/mx-options.png"
done
mkdir -p "${HICOLOR}/scalable/apps"
cp "$SVG_SRC" "${HICOLOR}/scalable/apps/mx-options.svg"

gtk-update-icon-cache -f -t "$HICOLOR"
echo "✅ Proceso completado exitosamente."
```

---

## 4. Conclusión

La fidelidad visual en Linux requiere comprender la interacción entre el compositor (GNOME Shell / Wayland), el subsistema de temas de iconos FreeDesktop (XDG Hicolor), el motor de renderizado de Tauri (WebKitGTK) y las limitaciones de formato de `libayatana-appindicator`.

Aplicar estos principios garantiza interfaces que lucen nativas, nítidas, contrastadas y perfectamente integradas desde el primer despliegue.
