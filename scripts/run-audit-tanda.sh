#!/usr/bin/env bash
# scripts/run-audit-tanda.sh
# Lanza la tanda de auditoría de 7 subagentes sobre Fase 0 y Fase 1.
# Cumple con AGENTS.md, docs/SUBAGENTES.md y lecciones aprendidas de La Fábrica:
#   - systemd-run --user --scope -p MemoryMax=2G
#   - Entrada redirigida </dev/null
#   - Concurrencia acotada (2-3 procesos simultáneos para cuidar RAM)
#   - Monitor en vivo de estado, tiempo y tamaño de salida
#   - Extracción de respuestas en Markdown

set -uo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
LOGS="$REPO/.superpowers/logs"
AUDIT_DIR="$REPO/.superpowers/audit_f0_f1"
PROMPTS_DIR="$AUDIT_DIR/prompts"
REPORTS_DIR="$AUDIT_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

# Definición de los 7 modelos y sus roles según docs/SUBAGENTES.md
AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura y Sistemas Linux (udev, uinput, IPC, sockets, gestión de procesos)"
  "haiku|opencode-go/claude-haiku-5-5|Métricas, Criterios de Aceptación y Rigor SDD/TDD (latencia, memoria, cobertura, casos borde)"
  "mimo|opencode-go/mimo-v2.6-flash|Análisis de Features HID++ y Protocolo de Hardware (0x1B04, 0x2201, BLE vs Unifying, tramas)"
  "muse|opencode-go/muse-spark-1.3-contributor|Kill-Gates, Observabilidad y Telemetría (recuperación de fallos, rollback, diagnóstico mx-diag)"
  "qwen|opencode-go/qwen3.8-flash|Ciclo de Vida y Robustez Bluetooth (reconexión BLE, suspend/resume de SO, hotplug, sleep)"
  "glm|opencode-go/glm-5.3-flash|Rendimiento, Bounds de Memoria y Escalabilidad (concurrencia, consumo RAM, transición a daemon)"
  "luna|opencode-go/gpt-6-luna|Verificaciones Empíricas y Compatibilidad Linux (supuestos no probados, multi-distro, portabilidad)"
)

echo "=== INICIANDO TANDA DE AUDITORÍA F0 & F1 (7 SUBAGENTES) ==="
echo "Directorio de logs: $LOGS"
echo "Directorio de reportes: $REPORTS_DIR"
echo ""

# Generar prompts especializados para cada subagente
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prompt_file="$PROMPTS_DIR/$alias.md"
  cat <<EOF > "$prompt_file"
# AUDITORÍA TÉCNICA CRÍTICA: FASE 0 Y FASE 1 — MX OPTIONS LINUX

Eres el subagente especialista: **$alias** ($model).
Tu rol en esta auditoría: **$role**.

Tu tarea es realizar una AUDITORÍA EN MODO LECTURA (READ-ONLY) estricta sobre la implementación de las Fases 0 y 1 de \`mx-options-linux\`.
BAJO NINGUNA CIRCUNSTANCIA DEBES MODIFICAR O CREAR ARCHIVOS DE CÓDIGO EN EL REPOSITORIO.
Tu única salida debe ser tu informe técnico de auditoría estructurado en Markdown.

## Contexto y Alcance del Proyecto
El proyecto busca reemplazar Logitech Options/Options+ en Linux de forma ligera, nativa y robusta, soportando el Logitech MX Vertical (046d:b020) vía Bluetooth y extensible a receptores Unifying/Bolt.
Se implementaron y probaron en hardware real dos fases:
1. **Fase 0 (Protocolo HID++ y Sonda):** \`crates/mx-core\`, \`crates/mx-probe\`, \`specs/F0-mx-probe.md\`. Handshake, feature discovery, lectura de batería, verificación de feature \`0x1B04\` (Button Diversion) y \`0x2201\` (DPI).
2. **Fase 1 (Button Diversion y Emulación uinput):** \`crates/mx-uinput\`, \`crates/mx-cli\`, \`specs/F1-divert-uinput.md\`, \`rules/99-mx-options.rules\`. Desvío del botón superior DPI (CID \`0x00FD\`), recepción asíncrona de eventos HID++, inyección uinput (ej. \`KEY_LEFTMETA\`) y guardián RAII para restaurar el botón al salir.

## Archivos a Evaluar en el Repositorio
Examina usando tus herramientas de lectura:
- \`specs/F0-mx-probe.md\` y \`specs/F1-divert-uinput.md\`
- \`crates/mx-core/src/\` (\`frame.rs\`, \`device.rs\`, \`hidraw.rs\`, \`features.rs\`, \`scanner.rs\`, \`transport.rs\`, \`lib.rs\`)
- \`crates/mx-probe/src/main.rs\`, \`crates/mx-probe/src/diag.rs\`
- \`crates/mx-uinput/src/\` (\`lib.rs\`, \`device.rs\`, \`action.rs\`)
- \`crates/mx-cli/src/main.rs\`
- \`rules/99-mx-options.rules\`
- \`STATUS.md\`

## Tu Misión Específica de Auditoría
Realiza una evaluación crítica, aguda y sin complacencias desde la óptica de tu especialidad (**$role**):

1. **Veredicto Técnico:** (APROBADO CON OBSERVACIONES / RIESGO MODERADO / RIESGO ALTO).
2. **Puntos Fuertes Identificados:** Qué decisiones de arquitectura e implementación son acertadas y sólidas.
3. **Críticas Agudas y Puntos Ciegos:**
   - ¿Qué pasa si el ratón se apaga o pierde conexión Bluetooth mientras está desviado?
   - ¿Qué pasa si el proceso recibe \`SIGKILL\` o se cierra violentamente (el Drop de RAII no corre)?
   - ¿Cómo convive el uinput con sesiones Wayland vs X11 y múltiples monitores?
   - ¿Hay cuellos de botella en la lectura síncrona/bloqueante de \`/dev/hidraw\` antes de migrar a daemon asíncrono con Tokio?
   - ¿El reporte largo de 20 bytes en Bluetooth LE es seguro para todos los transportes o necesitará bifurcación cuando se use dongle Unifying?
   - ¿Las reglas udev y permisos (\`plugdev\` vs \`uaccess\`) son 100% portables en systemd moderno?
4. **Propuestas Concretas de Mejora Continua para la Fase 2 (Daemon / Sockets IPC) y Fase 3:**
   - Recomendaciones técnicas precisas para evitar regresiones o bloqueos en producción.

Sé conciso, técnico y exigente. Entrega tu reporte completo en formato Markdown.
EOF
done

echo "Prompts generados para los 7 agentes en $PROMPTS_DIR."
echo ""

# Función para lanzar un agente
lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="audit-$alias"

  date +%s > "$LOGS/$log_prefix.start_ts"
  date +%H:%M:%S > "$LOGS/$log_prefix.inicio"
  echo "EJECUTANDO" > "$LOGS/$log_prefix.status"

  # Ejecución con límite de RAM y timeout
  systemd-run --user --scope -p MemoryMax=2G \
    timeout 10m opencode run "$(cat "$prompt_file")" \
      --standalone \
      --model "$model" \
      --auto \
      --print-logs \
      --log-level info \
      --format json \
      > "$LOGS/$log_prefix.json" 2> "$LOGS/$log_prefix.err" < /dev/null

  local code=$?
  echo "$code" > "$LOGS/$log_prefix.exit"
  date +%H:%M:%S > "$LOGS/$log_prefix.fin"
  date +%s > "$LOGS/$log_prefix.end_ts"

  if [[ $code -eq 0 ]]; then
    echo "COMPLETADO" > "$LOGS/$log_prefix.status"
  else
    echo "FALLIDO (exit $code)" > "$LOGS/$log_prefix.status"
  fi
}

export -f lanzar_agente
export LOGS PROMPTS_DIR

# Ejecutar con concurrencia máxima de 2 agentes a la vez (para proteger RAM y evitar saturación)
MAX_JOBS=2
echo "Lanzando agentes con concurrencia máxima = $MAX_JOBS..."

pids=()
aliases=()

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"

  # Esperar si se alcanza la concurrencia máxima
  while [[ $(jobs -r -p | wc -l) -ge $MAX_JOBS ]]; do
    sleep 2
  done

  echo "--> Despachando agente: $alias ($model)..."
  lanzar_agente "$alias" "$model" &
  pids+=("$!")
  aliases+=("$alias")
done

echo ""
echo "Todos los agentes han sido despachados. Entrando al monitor en vivo..."
echo ""

# Monitor en vivo
todos_terminados=false
while [[ "$todos_terminados" == "false" ]]; do
  todos_terminados=true
  clear_output=""
  now=$(date +%s)

  echo "==================== ESTADO DE SUBAGENTES ===================="
  printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
  echo "--------------------------------------------------------------"

  for item in "${AGENTES[@]}"; do
    IFS="|" read -r alias model role <<< "$item"
    status_file="$LOGS/audit-$alias.status"
    start_ts_file="$LOGS/audit-$alias.start_ts"
    json_file="$LOGS/audit-$alias.json"

    status="PENDIENTE"
    elapsed="0s"
    size="0 B"

    if [[ -f "$status_file" ]]; then
      status=$(cat "$status_file")
    fi

    if [[ -f "$start_ts_file" ]]; then
      start_ts=$(cat "$start_ts_file")
      if [[ "$status" == "EJECUTANDO" ]]; then
        elapsed="$(( now - start_ts ))s"
        todos_terminados=false
      elif [[ -f "$LOGS/audit-$alias.end_ts" ]]; then
        end_ts=$(cat "$LOGS/audit-$alias.end_ts")
        elapsed="$(( end_ts - start_ts ))s"
      fi
    else
      todos_terminados=false
    fi

    if [[ -f "$json_file" ]]; then
      bytes=$(wc -c < "$json_file" 2>/dev/null || echo 0)
      if [[ $bytes -ge 1048576 ]]; then
        size="$(awk "BEGIN {printf \"%.1f MB\", $bytes/1048576}")"
      elif [[ $bytes -ge 1024 ]]; then
        size="$(awk "BEGIN {printf \"%.1f KB\", $bytes/1024}")"
      else
        size="${bytes} B"
      fi
    fi

    printf "%-10s %-25s %-15s %-10s %-12s\n" "$alias" "${model:0:24}" "$status" "$elapsed" "$size"
  done

  echo "=============================================================="
  if [[ "$todos_terminados" == "false" ]]; then
    sleep 5
  fi
done

echo ""
echo "=== TODOS LOS AGENTES HAN FINALIZADO ==="
echo ""

# Extracción de Markdown desde JSON
echo "Extrayendo reportes individuales en $REPORTS_DIR..."
python3 - << 'PYEOF'
import os, json, glob

repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) if '__file__' in globals() else os.getcwd()
logs_dir = os.path.join(repo, ".superpowers", "logs")
reports_dir = os.path.join(repo, ".superpowers", "audit_f0_f1", "reports")
os.makedirs(reports_dir, exist_ok=True)

aliases = ["deepseek", "haiku", "mimo", "muse", "qwen", "glm", "luna"]

for alias in aliases:
    json_path = os.path.join(logs_dir, f"audit-{alias}.json")
    out_md = os.path.join(reports_dir, f"{alias}.md")
    
    if not os.path.exists(json_path):
        print(f"[{alias}] Log JSON no encontrado.")
        continue
        
    texts = []
    try:
        with open(json_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    # Detectar evento de texto
                    if data.get("type") == "text" and "part" in data:
                        t = data["part"].get("text", "")
                        if t:
                            texts.append(t)
                except json.JSONDecodeError:
                    pass
        
        full_text = "\n".join(texts).strip()
        if full_text:
            with open(out_md, "w", encoding="utf-8") as out_f:
                out_f.write(full_text)
            print(f"[{alias}] Reporte guardado exitosamente: {out_md} ({len(full_text)} caracteres)")
        else:
            print(f"[{alias}] ADVERTENCIA: No se extrajo texto del log JSON.")
    except Exception as e:
        print(f"[{alias}] Error procesando JSON: {e}")

PYEOF

echo ""
echo "Tanda de auditoría completada."
