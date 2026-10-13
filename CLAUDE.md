# AGENTS.md — MX Options Linux

> Reglas de trabajo. Núcleo invariable (calidad primero) + perfil del
> proyecto. Genérico para Claude Code, Cursor, Gemini CLI, Codex, agy.
> Perfil: tauri-app · Creado: 2026-10-13

---

<!-- ═══════════════════════════════════════════════════════════════════════
     CAPA 1 — NÚCLEO INVARIABLE · NO EDITAR POR PROYECTO
     Destilado del razonamiento de un modelo de alta capacidad (Fable 5) +
     filosofía "lazy senior dev". Validado experimentalmente. La calidad va
     primero: este bloque se incrusta TAL CUAL en el AGENTS.md de cada proyecto.
     ═══════════════════════════════════════════════════════════════════════ -->

> Empieza SIEMPRE por el Pilar 0. La REGLA DE ORO manda sobre todo lo demás.

---

## REGLA DE ORO — Integridad de la verificación (prioridad máxima)

Si una prueba, un caso esperado, una especificación o un dato necesario NO se
cumple o NO está disponible, está PROHIBIDO falsearlo para aparentar que todo
funciona. Esto incluye: modificar la prueba para que pase, inventar datos que
no se entregaron (tasas, credenciales, constantes), o declarar éxito sobre algo
que no se verificó realmente.

Ante un bloqueo de este tipo, solo hay dos caminos válidos:
1. El fallo es de TU código → arréglalo hasta cumplir la especificación original.
2. Falta un dato, hay una contradicción, o la spec es imposible tal como está →
   DETENTE ANTES DE PRODUCIR EL ARTEFACTO y escala: describe el bloqueo en 2-3
   líneas, ofrece 2-3 alternativas con sus trade-offs, y espera la decisión.

No basta con marcar el hueco y seguir (ej: poner datos de ejemplo "para que los
reemplaces"). Si el dato es imprescindible para que el resultado sea correcto,
DETENTE antes de escribir el archivo, no después. Un entregable con datos
inventados —aunque estén etiquetados como provisionales— produce resultados con
apariencia de correctos que es justo lo que esta regla previene.

Límite de la regla: las decisiones de implementación (qué estructura, qué
locator, qué patrón) son tu dominio autónomo. Qué se prueba, qué cuenta como
éxito, y qué datos son reales: eso no se inventa.

---

## Pilar 0 — Compuerta: ¿cuánto código merece esta tarea?

Antes de escribir nada, clasifica recorriendo esta escalera; detente en el
primer peldaño que aplique:

1. ¿Esto necesita existir?            → no: no lo escribas (YAGNI).
2. ¿La librería estándar lo resuelve? → úsala, no la reimplementes.
3. ¿Hay una función nativa de la plataforma? → úsala.
4. ¿Una dependencia ya instalada lo hace? → úsala.
5. ¿Se resuelve en una línea?          → una línea.
6. Solo entonces: escribe lo mínimo que funcione.

Según dónde te detuviste:
- **Tarea TRIVIAL o ya resuelta** (peldaños 1-5): entrega la solución mínima.
  Nada de invariantes ni arneses para algo trivial. Marca atajos deliberados con
  `// atajo: <ruta de mejora>` y termina.
- **Tarea con ESTADO, lógica no trivial, o sistemas acoplados** (peldaño 6 con
  casos borde reales): aplica los Pilares 1-7.

### Línea roja (ambos caminos)
La pereza nunca recorta: validación en fronteras de confianza, manejo de pérdida
de datos, seguridad y accesibilidad. Eso jamás es "código de más".

---

## Pilares de rigor (solo para tareas no triviales)

1. **Especifica antes de codear.** Comportamiento completo en prosa + TODAS las
   constantes con su valor y una frase que justifique cada una.
2. **Declara los invariantes** que deben ser siempre verdaderos (sin NaN, límites
   respetados, estado acotado). Define "éxito" antes de implementar.
3. **Planifica el orden de ejecución** del pipeline y por qué ese orden.
4. **Anticipa los casos borde** antes de que ocurran (primer/último elemento,
   estados vacíos, entradas malformadas, cambios simultáneos).
5. **Construye un arnés de verificación**; no confíes en "se ve bien". Stubea el
   entorno pesado y corre la lógica aislada. Los casos de prueba se fijan ANTES
   y NO se tocan después (ver REGLA DE ORO).
6. **Ante un obstáculo, diagnostica la causa**; no adivines. Causa raíz → 2+
   soluciones → elige y justifica. Nada de prueba-y-error a ciegas.
7. **Cierra el bucle: mide contra una meta numérica** y reporta el número real,
   incluidos los fallos. "Funciona" no es métrica.

---

## Nota sobre adherencia (hallazgo validado)
Estas reglas se siguen mejor cuanto más capaz es el modelo. Un modelo potente
las honra incluso bajo la presión del entorno de "entregar algo ya". Un modelo
pequeño las lee pero el sesgo a la acción puede ganarles, sobre todo la REGLA DE
ORO de detenerse a escalar. Si el agente que ejecuta es pequeño, no confíes solo
en este texto: añade un control estructural (hook/gate) que bloquee la entrega
cuando falte un dato imprescindible.

<!-- ═══════════════════ FIN CAPA 1 — NÚCLEO INVARIABLE ═══════════════════ -->

---

<!-- ═══════════════════════════════════════════════════════════════════════
     CAPA 2 — PERFIL: TAURI APP (escritorio / multiplataforma)
     Para apps Tauri: backend Rust + frontend web, empaquetado liviano.
     Se concatena DESPUÉS del núcleo. Hereda todo lo del perfil genérico
     salvo lo que aquí se especialice.
     ═══════════════════════════════════════════════════════════════════════ -->

## Perfil del proyecto: TAURI APP

### Contexto del stack
- **Backend:** Rust (core de Tauri). Aquí va la lógica de sistema, IO de
  archivos, comandos `#[tauri::command]`.
- **Frontend:** web (HTML/CSS/JS o framework liviano). Vista, no lógica pesada.
- **Filosofía Tauri:** lo pesado y lo sensible vive en Rust; el frontend es
  delgado. Esto NO es Electron: el peso y el consumo de recursos importan.
- **Prioridades declaradas del producto:** minimalista, personalizable,
  escalable, liviano, eficiente en recursos del PC. Toda decisión de diseño se
  juzga contra estas cinco. Agregar peso o dependencias pesadas viola el perfil.

### Pilar 0 aplicado a Tauri (qué NO escribir)
- ¿Lo resuelve una API nativa de Tauri (`@tauri-apps/api`)? → úsala, no un crate
  externo. (ej: diálogos de archivo, fs, path, eventos).
- ¿Lo resuelve `std` de Rust? → úsalo antes de traer un crate.
- Cada dependencia (crate o npm) suma peso al binario final. Justifícala en una
  frase (Pilar 1) o no entra. "Por si acaso" no es justificación.

### Frontera de confianza Rust ↔ Frontend (LÍNEA ROJA del núcleo)
- TODO lo que cruza de frontend a un `#[tauri::command]` se valida en Rust. El
  frontend no es confiable como fuente de verdad.
- Rutas de archivo del usuario: validar que existan, manejar permisos denegados,
  manejar archivos que desaparecen entre el listado y la lectura (TOCTOU).
- Nunca exponer un comando que escriba/borre sin validar la ruta destino.

### Metodología
- **SDD:** cada comando Tauri nuevo se especifica antes (entrada, salida, errores
  posibles, qué pasa si el archivo no existe / está corrupto / es enorme).
- **TDD en Rust:** la lógica del core (parseo MD, manejo de archivos, estado) se
  prueba con `#[cfg(test)]` aislada del runtime de Tauri (Pilar 5: stubea el
  entorno pesado). El render del frontend NO necesita TDD; se valida visual.
- **BDD:** desactivado salvo que la app crezca a flujos multi-paso de usuario.

### Estructura sugerida del repo
```
.
├── AGENTS.md / CLAUDE.md / STATUS.md
├── specs/                      # specs SDD por comando/feature
├── src-tauri/                  # backend Rust
│   ├── Cargo.toml
│   ├── tauri.conf.json         # config: ventana, permisos, bundle
│   └── src/
│       ├── main.rs
│       ├── commands/           # un módulo por grupo de comandos
│       └── core/               # lógica pura testeable (parseo, estado)
│           └── tests/          # arnés Rust (Pilar 5), aislado del runtime
├── src/                        # frontend
│   ├── index.html
│   ├── main.js
│   └── styles/
└── package.json
```

### Invariantes típicos de una app Tauri (Pilar 2)
- El binario final no supera un tamaño objetivo definido (fíjalo como número y
  mídelo — Pilar 7). Si no hay número, no hay "liviano", hay deseo.
- Ningún comando Tauri puede colgar la UI: IO pesado va asíncrono o en hilo.
- El estado de la app es acotado: no crece sin límite al abrir archivos.
- Errores de Rust se propagan al frontend como `Result` tipado, nunca panic.

### Gates de Git específicos
- `cargo build` y `cargo test` deben pasar antes de cerrar rama (reporta el
  número de tests, Pilar 7).
- `cargo clippy` sin warnings nuevos.
- Si el bundle crece respecto a la rama base, justifícalo o no se mergea.

### Guante de restricciones (ver `playbook/RESTRICCIONES.md` para la doctrina)

El guante concreto de un proyecto Rust + web. Se declara al empezar; lo que no está acá no es
compuerta.

| Restricción | Comando | Compuerta |
|---|---|---|
| Tests unitarios de la lógica pura | `cargo test` | **dura** — rojo bloquea |
| Lint | `cargo clippy -- -D warnings` | **dura** — 0 warnings nuevos |
| Tipos / compilación completa | `cargo build --release` | **dura** |
| Vulnerabilidades de dependencias | `cargo audit` | **dura** — 0, o excepción con **fecha de caducidad** (F-039) |
| **Puntuación de mutación** del núcleo | `cargo mutants --in-place -- --lib` | **meta numérica declarada**, por lote |
| Cobertura | `cargo llvm-cov` | **tablero, NUNCA compuerta** (ver abajo) |
| Peso del binario y del bundle | medido sobre el artefacto empaquetado | tendencia; se justifica si crece |

🔴 **La cobertura no es compuerta, y no es una preferencia de estilo:** una compuerta de
cobertura se satisface con tests sin una sola aserción. Eso produce exactamente el verde falso
que este kit existe para evitar. Se mira como tendencia; **quien decide si un test verifica
algo es el mutation testing.**

#### La capa que el guante NO cubre (y por eso no es opcional)

Ninguna de las filas de arriba puede ver un defecto del **artefacto renderizado**. En este
perfil eso incluye, con casos reales detrás:

- **Maquetación en el motor web real.** Un test de presencia (`¿existe el nodo?`) no detecta un
  fallo de posición. Hay que medir **dónde quedó pintado**, y en **WebKitGTK**, no en Chromium.
- **El icono dentro del paquete**, no el SVG fuente.
- **El artefacto producido**: abrir el `.deb`/`.msi` y leer su contenido; imprimir a PDF y leer
  el papel con `pdftotext`.
- **Cada sistema operativo del CI, con el ojo** — no solo "compiló en verde".
- **Un documento que ejercite todo a la vez**, versionado en el repo y mirado con el ojo en
  cada plataforma nueva. Es el arnés más barato que existe y no hay que diseñarlo como test.

#### Handoff de dos pasadas
Toda tarea delegada declara que **la primera entrega no se acepta**: antes de darla por
terminada, el ejecutor vuelve a recorrer los requisitos citando la evidencia de cada uno, y
recién entonces entrega. Es el control estructural contra `exit 0` ≠ completo.

### Meta numérica de cierre (Pilar 7 — define ANTES)
Para el lector MD minimalista, ejemplos de metas medibles (ajusta según el caso):
- Tamaño del binario empaquetado: objetivo `< X MB`.
- Tiempo de apertura de un .md de tamaño típico: objetivo `< Y ms`.
- RAM en reposo con un archivo abierto: objetivo `< Z MB`.
> Si estos números no están definidos al empezar, ESCALA (Regla de Oro): no se
> puede declarar "eficiente y liviano" sin una meta contra la cual medir.

### Orquestación multi-agente (opcional — tu experimento)
Si usas Claude Code como orquestador/juez y `agy` (Antigravity CLI) como
developer, el puente es por ARCHIVOS, no por terminal:
- `agy` en modo no-interactivo: `agy -p "<tarea>"` con stdout redirigido a un log.
- Workaround del bug de stdout en subproceso sin TTY: envolver con
  `script -qec 'agy -p "..."' /dev/null`.
- `agy --headless --approve <policy>` dentro de un sandbox para que no se cuelgue
  esperando confirmación al escribir archivos.
- Claude Code lee el log, juzga contra el núcleo + este perfil, y redirige.
- El grading cruzado (un agente juzga al otro) NO reemplaza el arnés externo:
  el arnés sigue siendo el árbitro objetivo (Pilar 5 + Regla de Oro).


---

### Gobernanza de Tokens y Delegación a Subagentes

#### 1. Alarma de Contexto y Gestión de Tokens
- El entorno cuenta con un hook activo globalmente (`~/.gemini/config/hooks.json` -> `~/.gemini/config/hooks/hook-contexto-agy.sh`).
- **160k tokens:** ALARMA. No iniciar unidades nuevas de código; comenzar el cierre documental actualizando `STATUS.md`.
- **200k tokens:** AVISO FINAL / TECHO. Guardar cambios, consolidar estado y sugerir al usuario rotar de sesión en un punto natural.

#### 2. Delegación a Subagentes
- Para tareas independientes o intensivas, utilizar el lanzador `scripts/run-dev.sh` (con `systemd-run` y límite de 2 GB de RAM) hacia los modelos verificados en `docs/SUBAGENTES.md`:
  - `opencode-go/deepseek-v4.1-flash`: Módulos de Linux, udev, sockets IPC y arquitectura.
  - `opencode-go/claude-haiku-5-5`: Criterios de aceptación, specs y descomposición en fases.
  - `gemini-3.8-flash-high` (vía agy): Tareas de soporte y código complementario sin quemar cuota de Claude.
- **Regla de oro de auditoría:** Nunca aceptar el autoinforme del subagente. Árbitro objetivo (`cargo test`, `evtest`, `clippy`), revisión de `diff` y evidencia antes de integrar.
