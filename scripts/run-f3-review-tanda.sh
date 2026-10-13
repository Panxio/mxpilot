#!/usr/bin/env bash
# scripts/run-f3-review-tanda.sh
# Lanza la tanda de revisión crítica de 4 subagentes sobre Fase 3:
#   - haiku (claude-haiku-5-5)
#   - luna (gpt-6-luna)
#   - deepseek (deepseek-v4.1-flash)
#   - muse (muse-spark-1.3-contributor)
#
# Cumple con AGENTS.md, docs/SUBAGENTES.md y lecciones aprendidas de La Fábrica:
#   - systemd-run --user --scope -p MemoryMax=2G
#   - Entrada redirigida </dev/null
#   - Concurrencia acotada (2 procesos simultáneos para proteger RAM)
#   - Monitor en vivo de estado, tiempo y tamaño de salida
#   - Extracción de respuestas en Markdown estructurado

set -uo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
REVIEW_DIR="$REPO/.superpowers/review_f3"
LOGS="$REVIEW_DIR/logs"
PROMPTS_DIR="$REVIEW_DIR/prompts"
REPORTS_DIR="$REVIEW_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

# Definición de los 4 modelos y sus roles
AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura y Sistemas Linux (Unix sockets, SO_PEERCRED, concurrencia liviana sin Tokio, señales, locks)"
  "haiku|opencode-go/claude-haiku-5-5|Métricas, Criterios de Aceptación y Rigor SDD/TDD (latencia p95, memoria < 15MB, arnés TDD IPC, casos borde)"
  "muse|opencode-go/muse-spark-1.3-contributor|Kill-Gates, Observabilidad y Telemetría (salud del daemon, mx-diag, logging, recuperación de fallos, rollback)"
  "luna|opencode-go/gpt-6-luna|Verificaciones Empíricas y Compatibilidad Linux (systemd --user, XDG_RUNTIME_DIR, suspend/resume, multi-distro)"
)

echo "=== INICIANDO TANDA DE REVISIÓN FASE 3 (4 SUBAGENTES) ==="
echo "Directorio de logs: $LOGS"
echo "Directorio de reportes: $REPORTS_DIR"
echo ""

# Generar prompts especializados para cada subagente
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prompt_file="$PROMPTS_DIR/$alias.md"
  cat <<EOF > "$prompt_file"
# REVISIÓN TÉCNICA CRÍTICA Y MEJORA CONTINUA: FASE 3 (mx-daemon & Socket IPC)

Eres el subagente especialista: **$alias** ($model).
Tu rol en esta revisión: **$role**.

Tu tarea es realizar una REVISIÓN EN MODO LECTURA (READ-ONLY) estricta sobre la especificación y arquitectura de la Fase 3 de \`mx-options-linux\`.
BAJO NINGUNA CIRCUNSTANCIA DEBES MODIFICAR O CREAR ARCHIVOS DE CÓDIGO EN EL REPOSITORIO.
Tu única salida debe ser tu informe técnico de revisión estructurado en Markdown con ideas críticas y constructivas de mejora continua.

## Contexto del Proyecto
El proyecto implementa una suite para Linux que reemplaza Logitech Options/Options+ para el mouse Logitech MX Vertical (046d:b020).
Fases 0 y 1 ya fueron completadas y validadas en hardware real:
- Protocolo HID++ 2.0/4.5, lectura de batería, Button Diversion en CID 0x00FD (\`mx-core\`).
- Emulación uinput (\`mx-uinput\`) y CLI interactivo (\`mx-cli\`).
- Paquete de parches de robustez H1-H7 aplicados: salida limpia en desconexión, guardián RAII con reconciliación al inicio, semántica Down/Up pura sin sleeps bloqueantes.

## Documentos y Código a Evaluar
Examina detenidamente con tus herramientas de lectura:
- \`specs/F3-daemon-ipc.md\` (Especificación técnica recién redactada para el Daemon y Socket IPC).
- \`specs/F1-divert-uinput.md\` y \`specs/F0-mx-probe.md\` (Especificaciones previas).
- \`docs/AUDITORIA_FASE_0_1.md\` (Hallazgos y lecciones de la auditoría colegiada previa).
- \`STATUS.md\` (Secciones 1 a 8).
- Código actual en \`crates/mx-core/src/\`, \`crates/mx-uinput/src/\`, \`crates/mx-cli/src/\`.

## Tu Misión Específica desde tu Especialidad (**$role**)
Proporciona una revisión aguda, constructiva y rigurosa:

1. **Evaluación de la Especificación (\`specs/F3-daemon-ipc.md\`):**
   - ¿El protocolo IPC (JSON framing por newline, \`SO_PEERCRED\`, límite 64 KB) es robusto y seguro?
   - ¿La máquina de estados (Disconnected -> Connecting -> Reconciling -> Active) tiene lagunas o transiciones no cubiertas?
   - ¿El modelo de concurrencia sin Tokio (hilo lector poll + canal mpsc + despachador uinput) es óptimo para la meta de RAM < 15 MB, o introduce riesgos de contención / bloqueo?
2. **Casos Borde y Puntos Ciegos no Previstos:**
   - ¿Qué pasa si un cliente IPC envía un comando a medias y se desconecta bruscamente?
   - ¿Qué pasa si el sistema se suspende (sleep to RAM) y al despertar BlueZ reasigna un nuevo nodo \`/dev/hidrawX\`?
   - ¿Qué pasa con el socket y el lock file ante un reinicio forzado o corte de energía?
   - ¿Cómo se comporta \`systemd --user\` en entornos sin sesión gráfica o en logins SSH?
3. **Propuestas de Mejora Concreta para el Plan de Implementación:**
   - 2 a 3 mejoras técnicas específicas, con justificación clara y accionable, para blindar la implementación de \`crates/mx-daemon\` antes de escribir el código.

Sé conciso, técnico y exigente. Entrega tu reporte completo en formato Markdown.
EOF
done

echo "Prompts generados para los 4 agentes en $PROMPTS_DIR."
echo ""

# Función para lanzar un agente
lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="review-$alias"

  date +%s > "$LOGS/$log_prefix.start_ts"
  date +%H:%M:%S > "$LOGS/$log_prefix.inicio"
  echo "EJECUTANDO" > "$LOGS/$log_prefix.status"

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

MAX_JOBS=2
echo "Lanzando agentes con concurrencia máxima = $MAX_JOBS..."

pids=()
aliases=()

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"

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

todos_terminados=false
while [[ "$todos_terminados" == "false" ]]; do
  todos_terminados=true
  now=$(date +%s)

  echo "==================== ESTADO DE REVISIÓN F3 ===================="
  printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
  echo "--------------------------------------------------------------"

  for item in "${AGENTES[@]}"; do
    IFS="|" read -r alias model role <<< "$item"
    status_file="$LOGS/review-$alias.status"
    start_ts_file="$LOGS/review-$alias.start_ts"
    json_file="$LOGS/review-$alias.json"

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
      elif [[ -f "$LOGS/review-$alias.end_ts" ]]; then
        end_ts=$(cat "$LOGS/review-$alias.end_ts")
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

echo "Extrayendo reportes individuales en $REPORTS_DIR..."
python3 - << 'PYEOF'
import os, json

repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) if '__file__' in globals() else os.getcwd()
logs_dir = os.path.join(repo, ".superpowers", "review_f3", "logs")
reports_dir = os.path.join(repo, ".superpowers", "review_f3", "reports")
os.makedirs(reports_dir, exist_ok=True)

aliases = ["deepseek", "haiku", "muse", "luna"]

for alias in aliases:
    json_path = os.path.join(logs_dir, f"review-{alias}.json")
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
echo "Tanda de revisión F3 completada exitosamente."
