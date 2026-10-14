#!/usr/bin/env bash
# scripts/run-f4-audit-cierre-tanda.sh
# Lanza la tanda de auditoría colegiada de CIERRE de Fase 4 con 4 subagentes:
#   - deepseek (deepseek-v4.1-flash)
#   - haiku (claude-haiku-5-5)
#   - muse (muse-spark-1.3-contributor)
#   - luna (gpt-6-luna)
#
# Cumple con AGENTS.md, docs/SUBAGENTES.md y lecciones aprendidas:
#   - systemd-run --user --scope -p MemoryMax=2G
#   - Entrada redirigida </dev/null
#   - Concurrencia acotada (2 procesos simultáneos para proteger RAM)
#   - Monitor en vivo de estado, tiempo y tamaño de salida
#   - Extracción de respuestas en Markdown estructurado

set -uo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
AUDIT_DIR="$REPO/.superpowers/audit_cierre_f4"
LOGS="$AUDIT_DIR/logs"
PROMPTS_DIR="$AUDIT_DIR/prompts"
REPORTS_DIR="$AUDIT_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura Backend Rust Tauri v2 (cliente IPC nonblocking poll, spawn_blocking, Zero Hang UI, single-instance)"
  "haiku|opencode-go/claude-haiku-5-5|Rigor Pilar 7 y Métricas Cuantitativas (36 tests pasando, 0 clippy, binario 11MB vs 15MB, análisis PSS/RSS WebKitGTK)"
  "muse|opencode-go/muse-spark-1.3-contributor|Experiencia de Usuario Honesta y Observabilidad (máquina de estados 4 vías, HealthModal, reconciliación set_action)"
  "luna|opencode-go/gpt-6-luna|Integración Desktop Linux y Wayland (AppIndicator tray, centrado multimonitor, systemctl start, desktop entry)"
)

echo "=== INICIANDO TANDA DE AUDITORÍA DE CIERRE DE FASE 4 (4 SUBAGENTES) ==="
echo "Directorio de logs: $LOGS"
echo "Directorio de reportes: $REPORTS_DIR"
echo ""

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prompt_file="$PROMPTS_DIR/$alias.md"
  cat <<EOF > "$prompt_file"
# AUDITORÍA DE CIERRE Y ENTREGA DE FASE 4: GUI Tauri v2 & System Tray (mx-gui)

Eres el subagente especialista: **$alias** ($model).
Tu rol en esta auditoría de entrega: **$role**.

Tu tarea es realizar una AUDITORÍA EN MODO LECTURA (READ-ONLY) estricta sobre la implementación final de la Fase 4 en \`mx-options-linux\`.
BAJO NINGUNA CIRCUNSTANCIA DEBES MODIFICAR O CREAR ARCHIVOS DE CÓDIGO EN EL REPOSITORIO.
Tu única salida debe ser tu informe técnico de auditoría estructurado en Markdown con veredicto claro y recomendaciones antes de proceder a la Fase 5 (Empaquetado .deb).

## Resumen del Entregable
Se ha implementado por completo la aplicación gráfica \`mx-gui\` en Tauri v2:
1. **Backend Rust Desacoplado (\`crates/mx-gui/src/\`):**
   - \`client.rs\`: Cliente IPC bounded con conexión no bloqueante (\`libc::poll\`, deadline 500 ms), timeouts FAST (1.5 s) y SLOW (2.5 s), límite de lectura \`take(64 KB + 1)\` y correlación de IDs atómica.
   - \`commands.rs\`: Comandos \`#[tauri::command]\` asíncronos delegando a \`spawn_blocking\` para garantizar que el event loop de GTK jamás se congele (Zero Hang UI).
   - \`poller.rs\`: Hilo nativo desacoplado (\`mx-gui-poller\`) con cadencia dinámica (10 s visible / 60 s en tray) que emite eventos tipados.
   - \`tray.rs\`: System Tray fallible con \`libayatana-appindicator\`, icono embebido (\`include_image!\`), menú contextual y cierre a la bandeja.
   - \`main.rs\`: Instancia única (\`tauri-plugin-single-instance\`), centrado garantizado (\`window.center()\`) para entornos multimonitor.
2. **Frontend Reactivo (\`crates/mx-gui/ui/\`):**
   - Svelte 5 + Tailwind v4 + Vite.
   - \`MouseViewer.svelte\`: Blueprint vectorial técnico industrial 57° (**Gemini 4**) con hotspots interactivos (Top CID \`0x00FD\`, Forward \`0x0056\`, Back \`0x0053\`, Wheel \`0x0052\`), brillo reactivo y accesibilidad por teclado.
   - \`ActionPicker.svelte\`: Catálogo de acciones predefinidas estilo Libadwaita con estado en vuelo y reconciliación ante fallos.
   - \`BatteryBadge.svelte\`: Nivel dinámico con atenuación y etiqueta *stale-while-revalidate*.
   - \`HealthModal.svelte\`: Telemetría completa de \`get_health\` más VmRSS de la GUI.
3. **Métricas Empíricas Reales (Pilar 7):**
   - Binario release compilado: **11 MB stripped** (< 15 MB) ✅
   - Daemon en segundo plano: **2.3 MB a 3.3 MB RSS** (< 15 MB) ✅
   - Tests unitarios e integración: **36/36 tests pasando** (100% verde) ✅
   - Clippy: **0 warnings** con \`--all-targets -- -D warnings\` ✅
   - Memoria GUI: **~230 MB PSS** (sobrecarga intrínseca de WebKitGTK / JavaScriptCore / Mesa en Linux).

## Archivos Críticos a Auditar
- \`specs/F4-gui-tauri.md\`
- \`crates/mx-gui/Cargo.toml\` y \`crates/mx-gui/tauri.conf.json\`
- \`crates/mx-gui/src/client.rs\`
- \`crates/mx-gui/src/commands.rs\`
- \`crates/mx-gui/src/poller.rs\`
- \`crates/mx-gui/src/tray.rs\`
- \`crates/mx-gui/src/main.rs\`
- \`crates/mx-gui/tests/client_mock_test.rs\`
- \`crates/mx-gui/ui/src/App.svelte\`
- \`crates/mx-gui/ui/src/lib/MouseViewer.svelte\`
- \`crates/mx-gui/ui/src/lib/ActionPicker.svelte\`
- \`crates/mx-gui/ui/src/lib/stores.js\`
- \`scripts/measure-gui.sh\`

## Tu Estructura de Informe Requerida
1. **Evaluación de Cumplimiento:** Análisis contra los requisitos de tu especialidad.
2. **Puntos Fuertes Identificados:** Aspectos destacados del código y diseño.
3. **Puntos Ciegos o Hallazgos:** Riesgos latentes, casos borde o áreas de atención.
4. **Recomendaciones de Mejora:** Sugerencias concretas para la Fase 5 (Empaquetado) o mantenimiento.
5. **Veredicto Final:** [APROBADO PARA CIERRE DE FASE 4 / APROBADO CON OBSERVACIONES / BLOQUEADO].
EOF
done

echo "Prompts generados para los 4 agentes en $PROMPTS_DIR."
echo ""

lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="audit-$alias"

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

declare -A prev_status
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prev_status["$alias"]="INICIANDO"
done

echo "[$(date +%H:%M:%S)] Monitor iniciado. Monitoreando cambios de estado en vivo..."
todos_terminados=false
last_heartbeat=0

while [[ "$todos_terminados" == "false" ]]; do
  todos_terminados=true
  now=$(date +%s)
  running_count=0
  running_list=()

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
        running_count=$((running_count + 1))
        running_list+=("$alias")
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

    # Emitir evento en tiempo real únicamente ante cambio de estado
    if [[ "${prev_status[$alias]}" != "$status" ]]; then
      echo "[$(date +%H:%M:%S)] [EVENTO] Agente '$alias': ${prev_status[$alias]} -> $status (tiempo: $elapsed, salida: $size)"
      prev_status["$alias"]="$status"
    fi
  done

  # Latido discreto cada 30 segundos si hay agentes en ejecución
  if (( now - last_heartbeat >= 30 )) && [[ "$todos_terminados" == "false" ]]; then
    running_str=$(IFS=", "; echo "${running_list[*]}")
    echo "[$(date +%H:%M:%S)] [LATIDO] $running_count/${#AGENTES[@]} agentes en ejecución ($running_str)..."
    last_heartbeat=$now
  fi

  if [[ "$todos_terminados" == "false" ]]; then
    sleep 5
  fi
done

echo ""
echo "==================== RESUMEN CONSOLIDADO DE AUDITORÍA F4 ===================="
printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
echo "----------------------------------------------------------------------------"
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  status=$(cat "$LOGS/audit-$alias.status" 2>/dev/null || echo "DESCONOCIDO")
  start_ts=$(cat "$LOGS/audit-$alias.start_ts" 2>/dev/null || echo 0)
  end_ts=$(cat "$LOGS/audit-$alias.end_ts" 2>/dev/null || echo "$start_ts")
  elapsed="$(( end_ts - start_ts ))s"
  json_file="$LOGS/audit-$alias.json"
  size="0 B"
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
echo "============================================================================"

echo ""
echo "=== TODOS LOS AGENTES HAN FINALIZADO ==="
echo ""

echo "Extrayendo reportes individuales en $REPORTS_DIR..."
python3 - "$REPO" "$AUDIT_DIR" << 'PYEOF'
import sys, os, json

repo = sys.argv[1]
audit_dir = sys.argv[2]
logs_dir = os.path.join(audit_dir, "logs")
reports_dir = os.path.join(audit_dir, "reports")
os.makedirs(reports_dir, exist_ok=True)

aliases = ["deepseek", "haiku", "muse", "luna"]

for alias in aliases:
    json_path = os.path.join(logs_dir, f"audit-{alias}.json")
    out_md = os.path.join(reports_dir, f"{alias}.md")
    
    if not os.path.exists(json_path):
        print(f"[{alias}] Log JSON no encontrado en: {json_path}")
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
echo "Tanda de auditoría de cierre F4 completada exitosamente."
