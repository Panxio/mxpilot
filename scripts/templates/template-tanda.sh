#!/usr/bin/env bash
# ==============================================================================
# template-tanda.sh — Plantilla Oficial de Tandas Multi-Agente
# ==============================================================================
# Propósito:
#   Ejecutar tandas concurrentes de subagentes (OpenCode / agy) con:
#   1. Aislamiento de memoria estricto vía systemd-run (2GB max)
#   2. Redirección de entrada obligatoria (< /dev/null) para evitar cuelgues
#   3. Monitor en vivo no-invasivo basado en eventos (cero bucles visuales repetitivos)
#   4. Tabla resumen consolidada al finalizar
#   5. Extracción determinista de reportes markdown a disco
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO"

NOMBRE_TANDA="${1:-tanda_ejemplo}"
SALIDA_DIR="$REPO/.superpowers/$NOMBRE_TANDA"
LOGS="$SALIDA_DIR/logs"
PROMPTS_DIR="$SALIDA_DIR/prompts"
REPORTS_DIR="$SALIDA_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

# Lista de agentes: "alias|modelo|rol"
AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura y Sistemas"
  "haiku|opencode-go/claude-haiku-5-5|Métricas y Aceptación"
  "muse|opencode-go/muse-spark-1.3-contributor|Kill-Gates y Telemetría"
  "luna|opencode-go/gpt-6-luna|Verificación Empírica"
)

echo "=============================================================================="
echo " INICIANDO TANDA MULTI-AGENTE: $NOMBRE_TANDA"
echo " Directorio: $SALIDA_DIR"
echo "=============================================================================="

# 1. Generar prompts específicos para cada agente si no existen
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prompt_file="$PROMPTS_DIR/$alias.md"
  if [[ ! -f "$prompt_file" ]]; then
    cat > "$prompt_file" << EOF
Eres el auditor especializado $alias ($role).
Analiza el código del repositorio según las reglas de AGENTS.md.
EOF
  fi
done

# 2. Función de despacho desacoplada
lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="audit-$alias"

  date +%s > "$LOGS/$log_prefix.start_ts"
  date +%H:%M:%S > "$LOGS/$log_prefix.inicio"
  echo "EJECUTANDO" > "$LOGS/$log_prefix.status"

  local code=0
  systemd-run --user --scope -p MemoryMax=2G \
    timeout 10m opencode run "$(cat "$prompt_file")" \
      --standalone \
      --model "$model" \
      --auto \
      --print-logs \
      --log-level info \
      --format json \
      > "$LOGS/$log_prefix.json" 2> "$LOGS/$log_prefix.err" < /dev/null || code=$?

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

# 3. Despacho con límite de concurrencia
MAX_JOBS=2
echo "Despachando agentes con concurrencia máxima = $MAX_JOBS..."

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  while [[ $(jobs -r -p | wc -l) -ge $MAX_JOBS ]]; do
    sleep 2
  done
  echo "--> Despachando: $alias ($model)..."
  lanzar_agente "$alias" "$model" &
done

# 4. Monitor no-invasivo basado en eventos
declare -A prev_status
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prev_status["$alias"]="INICIANDO"
done

todos_terminados=false
last_heartbeat=0
echo "[$(date +%H:%M:%S)] Monitor iniciado. Escuchando eventos de estado..."

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

    if [[ "${prev_status[$alias]}" != "$status" ]]; then
      echo "[$(date +%H:%M:%S)] [EVENTO] Agente '$alias': ${prev_status[$alias]} -> $status (${elapsed}, salida: ${size})"
      prev_status["$alias"]="$status"
    fi
  done

  if (( now - last_heartbeat >= 30 )) && [[ "$todos_terminados" == "false" ]]; then
    running_str=$(IFS=", "; echo "${running_list[*]}")
    echo "[$(date +%H:%M:%S)] [LATIDO] $running_count/${#AGENTES[@]} en ejecución ($running_str)..."
    last_heartbeat=$now
  fi

  if [[ "$todos_terminados" == "false" ]]; then
    sleep 5
  fi
done

echo ""
echo "=============================================================================="
echo " RESUMEN FINAL CONSOLIDADO"
echo "=============================================================================="
printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
echo "------------------------------------------------------------------------------"
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
echo "=============================================================================="

echo ""
echo "Extrayendo reportes individuales en $REPORTS_DIR..."
python3 - "$REPO" "$SALIDA_DIR" << 'PYEOF'
import sys, os, json

repo = sys.argv[1]
audit_dir = sys.argv[2]
logs_dir = os.path.join(audit_dir, "logs")
reports_dir = os.path.join(audit_dir, "reports")

agentes = ["deepseek", "haiku", "muse", "luna"]

for ag in agentes:
    json_path = os.path.join(logs_dir, f"audit-{ag}.json")
    report_path = os.path.join(reports_dir, f"{ag}.md")
    
    if not os.path.exists(json_path):
        continue
        
    extracted_text = []
    try:
        with open(json_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                    if isinstance(obj, dict):
                        if "part" in obj and isinstance(obj["part"], dict) and "text" in obj["part"]:
                            extracted_text.append(obj["part"]["text"])
                        elif "type" in obj and obj.get("type") == "text" and "content" in obj:
                            extracted_text.append(obj["content"])
                        elif "content" in obj and isinstance(obj["content"], str):
                            extracted_text.append(obj["content"])
                        elif "text" in obj:
                            extracted_text.append(obj["text"])
                except Exception:
                    continue
    except Exception as e:
        print(f"[!] Error leyendo {json_path}: {e}")
        
    full_content = "\n\n".join(extracted_text).strip()
    if full_content:
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write(full_content)
        print(f"[+] Reporte extraído para {ag}: {len(full_content)} caracteres -> {report_path}")
    else:
        err_path = os.path.join(logs_dir, f"audit-{ag}.err")
        if os.path.exists(err_path) and os.path.getsize(err_path) > 0:
            with open(err_path, 'r', encoding='utf-8') as ef, open(report_path, 'w', encoding='utf-8') as rf:
                rf.write(f"# Error en ejecución de {ag}\n\n```\n" + ef.read() + "\n```")
            print(f"[!] Reportado error para {ag} desde .err")

print("Extracción completada.")
PYEOF
