# Catálogo y Protocolo de Subagentes — MX Options Linux

Este catálogo reúne la lista verificada de subagentes y modelos probados en la sesión preparatoria (`e3641738-eb0c-4c7a-8ef8-67bdb2465ed4`) y consolidados bajo las reglas de La Fábrica (`factory/playbook/LANZAR-SUBAGENTES.md`).

---

## 1. Catálogo Verificado de Modelos

### OpenCode Go (vía `scripts/run-dev.sh oc` o `run-subagent.sh`)

| Modelo (ID con prefijo) | Perfil y Mejor Uso | Desempeño y Notas de la Sesión Previa |
|---|---|---|
| `opencode-go/deepseek-v4.1-flash` | **Arquitectura y Sistemas Linux** | **Mejor evaluado (1er lugar).** Decisiones firmes y sin ambigüedad: daemon `systemd --user`, reglas udev con `TAG+="uaccess"`, prevención de sockets huérfanos, IPC seguro (`SO_PEERCRED`), prohibición de `Exec` arbitrario. |
| `opencode-go/claude-haiku-5-5` | **Métricas y Criterios de Aceptación** | **Excelente (2do lugar).** Formuló los mejores criterios medibles: p95 latencia < 20 ms, RAM total < 80 MB, 20 reconexiones BT, ±10% batería vs Solaar. Plan F0–F5 reutilizable. |
| `opencode-go/mimo-v2.6-flash` | **Análisis de Features HID++** | Bueno. Señaló con precisión que la feature `0x2201` (DPI) puede no estar expuesta por firmware en el MX Vertical y propuso pruebas de pulsaciones sin duplicados (≥99%). |
| `opencode-go/muse-spark-1.3` | **Kill-Gates y Telemetría** | Rápido (89 s). Propuso la herramienta `mx-diag --dump` desde el día 1, verificación de licencia GPL-2.0 y convivencia con Solaar. |
| `opencode-go/qwen3.8-flash` | **Ciclo de Vida y Robustez** | Identificó que el 40% del esfuerzo real está en la robustez de Bluetooth y ciclo de vida del daemon (suspensión/wake-up). Propuso usar arte visual propio en vez de assets de Logitech. |
| `opencode-go/glm-5.3-flash` | **Planes de Trabajo y Estimaciones** | Detallado. **Precaución:** requiere límite de memoria (`MemoryMax=2G`) porque tiende a consumir RAM excesiva. |
| `opencode-go/gpt-6-luna` | **Verificaciones Empíricas** | Cautela con afirmaciones de "100% compatibilidad" hasta no verificar con firmware real. |

### Antigravity / Gemini (vía `scripts/run-dev.sh agy` o CLI agy)

| Modelo | Perfil y Mejor Uso | Notas |
|---|---|---|
| `gemini-3.8-flash-high` | **Desarrollador de tareas paralelas** | Excelente para tareas aisladas, generación de boilerplate, tests y módulos auxiliares sin consumir la cuota de modelos Claude u OpenCode. |
| `gemini-3.8-flash` | **Lecturas rápidas y validaciones** | Rápido y económico para revisiones de sintaxis o documentación. |

---

## 2. Reglas de Seguridad para Lanzar Subagentes

Siguiendo las lecciones pagadas de `factory/playbook/LANZAR-SUBAGENTES.md`:

1. **Intrada redirigida obligatoria (`< /dev/null`):**
   * Sin esto, OpenCode espera stdin y muere con timeout (exit 124).
2. **Límite de memoria estricto (`systemd-run`):**
   * Todos los subagentes deben lanzarse bajo `systemd-run --user --scope -p MemoryMax=2G` para evitar que un bucle infinito o leak consuma la memoria del sistema.
3. **Logs únicos por corrida:**
   * Nunca sobrescribir un log anterior; el script aborta si el log ya existe.
4. **Worktree aislado:**
   * El subagente trabaja en su propio git worktree (`git worktree add ../mx-wt-<nombre> -b <rama>`). No toca el directorio de trabajo del orquestador.
5. **Prompt a archivo cerrado:**
   * El prompt va en un archivo (ej. `.superpowers/<tarea>/prompt.md`), listando explícitamente los archivos editables y los prohibidos.
6. **Entrega sin commitear:**
   * El subagente entrega el código en el worktree sin hacer commit. El orquestador humano o juez audita el código y realiza el commit.

---

## 3. Guía de Ejecución

### Lanzar con OpenCode:
```bash
# Ejemplo: delegar una tarea a deepseek-v4.1-flash
OC_MODEL="opencode-go/deepseek-v4.1-flash" \
bash scripts/run-dev.sh oc /ruta/al/worktree .superpowers/tarea1/prompt.md tarea1-deepseek
```

### Lanzar con agy (Gemini Flash):
```bash
# Ejemplo: delegar a Gemini sin gastar cuota de Claude
AGY_MODEL="gemini-3.8-flash-high" \
bash scripts/run-dev.sh agy /ruta/al/worktree .superpowers/tarea1/prompt.md tarea1-agy
```

---

## 4. Auditoría de Entregas (La Regla de Oro)

> **"Un árbitro que no mira algo no dice 'no sé'. Dice VERDE."**

Nunca aceptar el autoinforme del modelo. Antes de integrar una entrega de un subagente:
1. Ejecutar el árbitro objetivo de la fase (ej: `cargo test`, `cargo clippy`, script de prueba con hardware).
2. Verificar `git diff --stat` para comprobar que no tocó archivos no autorizados.
3. Revisar el código fuente con el ojo crítico antes de hacer commit.

---

## 5. Patrón Antiloop y Buenas Prácticas para Scripts de Tandas Colegiadas

Para evitar errores de lanzamiento, loops visuales en la consola y saturación de logs en sesiones multi-agente:

1. **Monitor Basado en Eventos (Prevenir bucles visuales repetitivos):**
   - **PROHIBIDO:** Imprimir la tabla completa de agentes repetidamente en un `while sleep 5`. Tanto en consola interactiva como en logs, esto llena el buffer de la terminal con cientos de líneas idénticas aparentando un bucle infinito colgado.
   - **PATRÓN OBLIGATORIO:**
     - Usar un array asociativo `declare -A prev_status` para almacenar el estado previo de cada agente.
     - Emitir una línea compacta `[EVENTO]` **únicamente cuando un agente cambia de estado** (ej: `PENDIENTE -> EJECUTANDO` o `EJECUTANDO -> COMPLETADO`).
     - Emitir un `[LATIDO]` discreto de una sola línea cada 30 segundos si aún hay agentes corriendo.
     - Imprimir la tabla formateada completa **UNA SOLA VEZ** al finalizar todos los agentes.
2. **Prevención de Escapes Accidentales en Bash:**
   - Si un script de tanda se genera mediante un script o heredoc, verificar meticulosamente que variables como `$LOGS`, `$alias` o `$model` no queden precedidas por barras invertidas (`\$LOGS`), lo cual provoca errores fatales como `$LOGS/...: No such file or directory`.
3. **Plantilla Canónica Reutilizable:**
   - La implementación de referencia y canónica se encuentra versionada en:
     [`scripts/templates/template-tanda.sh`](file:///home/panxio/Projects/mx-options-linux/scripts/templates/template-tanda.sh).
   - Para cualquier nueva fase (Fase 5, Fase 6), clonar esta plantilla en vez de reescribir la lógica de despacho y sondeo desde cero.
4. **Extracción Determinista de Salidas JSON:**
   - Todo script de extracción Python ejecutado inline (`python3 - << 'PYEOF'`) debe recibir los directorios absolutos (`$REPO`, `$LOGS`, `$REPORTS`) explícitamente como argumentos de línea de comandos (`sys.argv`), evitando depender de `os.getcwd()` o `__file__`.


