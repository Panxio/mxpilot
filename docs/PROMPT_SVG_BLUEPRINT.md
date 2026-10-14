# Guía y Prompt de Generación: Blueprint Vectorial SVG (Logitech MX Vertical)

> **Destino:** Para ser utilizado en modelos avanzados de diseño/razonamiento visual (Claude 3.5/3.7 Sonnet / Opus, Gemini 2.0/3.8 Flash, Antigravity 4.0).  
> **Objetivo:** Generar un diagrama vectorial técnico SVG puro, minimalista y de alta precisión del mouse **Logitech MX Vertical**, visto en **perfil lateral ergonómico izquierdo (vista del pulgar)**.  

---

## 1. Características Técnicas del SVG Requerido

Para que el SVG se integre de forma nativa en la interfaz Tauri/Svelte de `mx-options-linux`, debe cumplir con las siguientes especificaciones:

| Parámetro | Requisito Técnico |
|---|---|
| **Formato** | Solo código `<svg>...</svg>` limpio, sin etiquetas HTML externas. |
| **ViewBox** | `viewBox="0 0 360 320"` (proporción óptima ~9:8). |
| **Escalabilidad** | `width="100%" height="100%"` con `vector-effect="non-scaling-stroke"` en trazos principales. |
| **Estilo Visual** | Blueprint técnico / Wireframe moderno industrial: líneas limpias (1.5px a 2px), degradados monocromáticos suaves para el volumen, sin fotos ni renders 3D pesados. |
| **Tema Adaptativo** | Debe utilizar variables semánticas CSS (`var(--border)`, `var(--card)`, `var(--foreground)`) o colores neutros basados en la paleta Slate (`#0f172a`, `#1e293b`, `#475569`, `#94a3b8`) para adaptarse a temas oscuros y claros. |
| **Elementos Identificables (`id`)** | Cada botón interactivo debe estar agrupado en una etiqueta `<g>` o `<path>` con su ID exacto para poder vincular los eventos `onclick` de Tauri/Svelte. |

---

## 2. Mapa de Componentes y Anatomía del Mouse (Basado en la Foto Oficial)

El perfil lateral izquierdo del **Logitech MX Vertical** presenta 5 elementos clave:

1. **Inclinación Ergonómica de 57° (Silueta General):**
   - Rampa frontal izquierda inclinada a ~57° subiendo desde la base hasta la cúspide.
   - Ala inferior derecha (apoyo ergonómico para el pulgar) que se extiende horizontalmente en la base.
2. **Placa Metálica Superior y Conmutador de Velocidad / DPI (Componente #4):**
   - Placa de aluminio ovalada/biselada en la cúspide del ratón.
   - Botón ovalado ubicado en el extremo superior de la placa: **Botón Superior (DPI / Kill-Gate, CID `0x00FD`)**. `id="svg-btn-top"`.
3. **Botones Laterales del Pulgar (Componente #5):**
   - Ranura en forma de cápsula sobre el descanso del pulgar conteniendo dos botones horizontales:
     - Botón delantero: **Adelante (Forward, CID `0x0056`)**. `id="svg-btn-forward"`.
     - Botón trasero: **Atrás (Back, CID `0x0053`)**. `id="svg-btn-back"`.
   - Pequeño LED indicador circular inmediatamente a la izquierda de la ranura.
4. **Rampa Frontal y Rueda de Desplazamiento (Componentes #1 y #2):**
   - Rueda de scroll física sobresaliendo sutilmente en el perfil frontal superior: **Clic Central (Wheel, CID `0x0052`)**. `id="svg-btn-wheel"`.
   - Línea sutil de separación de los botones primarios (clic izquierdo/derecho).
5. **Estrías y Textura Ergonómica (Componentes #6 y #7):**
   - Serie de 6 a 8 estrías curvas suaves que fluyen desde la zona central del pulgar hacia el faldón inferior derecho, aportando profundidad y realismo táctil sin saturar el gráfico.
6. **Puerto USB-C y Base (Componentes #3 y #8):**
   - Ranura frontal inferior USB-C y línea base plana.

---

## 3. Prompt Copiable para Usar en la Interfaz Web (Opus / Antigravity / Sonnet)

Puedes copiar y pegar el siguiente bloque tal cual en el modelo:

```text
Actúa como un diseñador industrial senior experto en gráficos vectoriales SVG técnicos para aplicaciones de escritorio en Linux (estilo GNOME Libadwaita / Blueprint moderno).

Tu tarea es generar un archivo SVG de alta precisión técnica, elegante y limpio del mouse ergonómico Logitech MX Vertical, visto en PERFIL LATERAL IZQUIERDO (vista del pulgar, mirando hacia la izquierda).

Requisitos indispensables:
1. Perspectiva y Forma:
   - Vista lateral fiel a la ergonomía del MX Vertical: ángulo ergonómico de 57 grados en la rampa frontal izquierda.
   - Ala curva extendida en la parte inferior derecha que forma el descanso del pulgar.
   - Cúspide rematada por una placa metálica/aluminio biselada ligeramente inclinada.
2. Botones y Elementos Interactivos (con IDs específicos):
   - id="svg-btn-top": Botón ovalado de conmutación de DPI situado sobre la placa metálica de la cúspide (CID 0x00FD).
   - id="svg-btn-wheel": Rueda de scroll que asoma en el perfil frontal superior izquierdo (CID 0x0052).
   - id="svg-btn-forward": Botón lateral delantero en la cápsula del pulgar (CID 0x0056).
   - id="svg-btn-back": Botón lateral trasero en la cápsula del pulgar (CID 0x0053).
   - id="svg-thumb-ridges": Grupo de 6-8 líneas curvas tenues que representan las estrías de agarre del pulgar.
   - id="svg-led": Pequeño punto circular indicador de estado junto a los botones del pulgar.
   - id="svg-usbc": Ranura pequeña del puerto USB-C en la punta frontal inferior.
3. Dimensiones y Código:
   - viewBox="0 0 360 320".
   - Utiliza exclusivamente trazos vectoriales y gradientes SVG suaves (definidos en <defs>).
   - Paleta de color técnico: tonos oscuros para el chasis (#0f172a, #1e293b), acento metálico suave para la placa superior (#94a3b8, #cbd5e1), y trazos contrastados en azul/índigo suave (#60a5fa, #818cf8) en los botones para destacarlos como interactivos.
   - NO incluyas logos comerciales protegidos (como el logo "logi").
   - Devuelve ÚNICAMENTE el bloque de código <svg>...</svg> sin texto introductorio ni explicaciones.
```
