#!/usr/bin/env bash
# ==============================================================================
# run-f4-plan-mejora-tanda.sh — Tanda Colegiada: Revisión Crítica del Plan de Mejora UI/UX F4
# ==============================================================================
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

SALIDA_DIR="$REPO/.superpowers/f4_plan_mejora"
LOGS="$SALIDA_DIR/logs"
PROMPTS_DIR="$SALIDA_DIR/prompts"
REPORTS_DIR="$SALIDA_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura y Sistemas"
  "haiku|opencode-go/claude-haiku-5-5|Métricas y Criterios de Aceptación"
  "muse|opencode-go/muse-spark-1.3-contributor|UI/UX y Telemetría"
  "luna|opencode-go/gpt-6-luna|Integración Linux y Escritorio"
)

echo "=============================================================================="
echo " INICIANDO TANDA COLEGIADA: REVISIÓN CRÍTICA DEL PLAN DE MEJORA F4"
echo " Directorio: $SALIDA_DIR"
echo "=============================================================================="

# 1. Generar prompts para los 4 agentes
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prompt_file="$PROMPTS_DIR/$alias.md"

  cat > "$prompt_file" << EOF
Eres el auditor especializado **$alias** con rol de **$role** en el proyecto \`mx-options-linux\`.
Tu tarea es realizar una **revisión crítica y constructiva del Plan de Mejora UI/UX e Integración de Escritorio de la Fase 4** antes de su implementación en código.

## Contexto de la Prueba Manual E2E Realizada por el Usuario Humano
El usuario ejecutó la aplicación \`mx-gui\` y el servicio \`mx-daemon\` en su entorno real (Ubuntu GNOME Wayland / X11, 3 monitores, Logitech MX Vertical en \`/dev/hidraw2\`) y reportó los siguientes 5 hallazgos empíricos:

1. **Geometría y Maquetación de la Ventana:**
   - La ventana actual se desplegó en un tamaño pequeño (\`760x540 px\`) y el contenido se apiló verticalmente (blueprint SVG arriba, tarjetas abajo) en lugar de lado a lado.
   - *Causa raíz técnica:* \`App.svelte\` tenía \`grid grid-cols-1 md:grid-cols-2\`. Como el breakpoint \`md:\` de Tailwind requiere 768 px y la ventana fija mide 760 px (\`760 < 768\`), se activó la vista móvil apilada.
2. **Asignación de Acciones vs Indicador 'Activo':**
   - El usuario asignó la acción "Super / Meta" al botón superior. Al pulsar físicamente el botón en el mouse real, la acción se ejecutó en el escritorio GNOME (abrió Actividades), demostrando que el driver kernel uinput y el IPC funcionan al 100%.
   - Sin embargo, en la interfaz visual nunca apareció la etiqueta "Activo", quedándose marcada en "Deshabilitar Botón".
   - *Causa raíz técnica:* \`isActionCurrent\` comparaba \`JSON.stringify(current) === JSON.stringify(presetPayload)\`. El daemon serializa \`{"key":"KEY_LEFTMETA","type":"SingleKey"}\` mientras el preset JS tiene \`{"type":"SingleKey","key":"KEY_LEFTMETA"}\`. Por diferente orden de claves, la igualdad de strings falló.
3. **Icono en el Lanzador de Aplicaciones (GNOME Shell Dash y Dock de Ubuntu):**
   - Al buscar la app en GNOME con \`Super\` y en el dock lateral, aparece un icono gris de engranaje genérico en vez del icono de la app.
   - *Causa raíz técnica:* El archivo \`.desktop\` tiene \`Icon=mx-options\`, pero el icono solo se instaló en una resolución PNG (\`128x128\`), faltando el SVG escalable y las resoluciones estándar (\`48x48\`, \`64x64\`, \`256x256\`), y no se ejecutó \`gtk-update-icon-cache\`.
   - *Preferencia visual del usuario:* Fondo en cuadrado redondeado (squircle) con degradado suave en **verde oscuro técnico**, y tipografía **MX** de alto contraste.
4. **System Tray (Panel Superior de GNOME):**
   - El indicador mostró correctamente el menú y \`Batería: 90% (Cargando)\` tras el parche de esta mañana.
5. **Nombre del Botón de Telemetría:**
   - El botón superior derecho actualmente se titula "Salud"; el usuario prefiere "Diagnóstico" (o "Telemetría").
6. **Posicionamiento Multimonitor:**
   - Actualmente usa \`window.center()\`, centrando en el monitor principal. El usuario consultó si puede desplegarse en el monitor donde está el foco/cursor.

---

## El Plan de Mejora Propuesto Actual
- **Paso 1: Asset del Icono:** Crear icono SVG oficial y resoluciones PNG (\`48\`, \`64\`, \`128\`, \`256\`) con degradado suave verde oscuro y monograma MX. Registrar en \`~/.local/share/icons/hicolor/\` y regenerar caché con \`gtk-update-icon-cache\`.
- **Paso 2: Geometría y Layout:** Aumentar ventana a \`960x620 px\` por defecto, \`resizable: true\` con mínimo \`840x520 px\`. Modificar \`App.svelte\` para forzar \`grid-cols-2\` en escritorio.
- **Paso 3: Comparador Canónico de Acciones:** Reemplazar \`JSON.stringify\` por función canónica semántica que compare tipo y campos específicos (\`key\`, \`modifiers\`, \`button\`) inmune al orden de claves de serialización.
- **Paso 4: Encabezado:** Renombrar botón "Salud" a "Diagnóstico".
- **Paso 5: Recompilar, reinstalar y verificar.**

---

## Tu Misión Crítica
Analiza este plan desde tu especialidad (**$role**):
1. **¿Qué puntos ciegos, riesgos o casos borde tiene este plan?**
2. **¿Qué mejoras concretas y constructivas propones para enriquecer el plan?**
   - Si eres *deepseek* (Arquitectura): ¿cómo manejar el redimensionamiento de ventana en Tauri v2 sin romper la relación de aspecto del SVG, y qué considerar respecto a multimonitor/Wayland?
   - Si eres *haiku* (Métricas/Aceptación): ¿cuáles deben ser los casos de prueba exactos del comparador canónico y los criterios medibles para dar por cerrada la UX?
   - Si eres *muse* (UI/UX): ¿cómo asegurar que el diseño verde oscuro se sienta premium y no desentone con Libadwaita? ¿Cómo optimizar el SVG y la lista de acciones para el nuevo espacio 960x620?
   - Si eres *luna* (Integración Linux): ¿cuáles son los requisitos exactos de FreeDesktop para iconos de GNOME Shell / Wayland (dimensiones, symlinks, permisos, WM_CLASS), y cómo asegurar que el dock y Alt+Tab muestren el icono nuevo?
3. **Veredicto del Plan:** [PLAN APROBADO / PLAN APROBADO CON MODIFICACIONES / PLAN INSUFICIENTE].

Sé riguroso, directo y de alto nivel técnico.
EOF
done

echo "Prompts generados para los 4 agentes en $PROMPTS_DIR."
echo ""

# 2. Función de despacho desacoplada
lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="plan-$alias"

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

# 3. Monitor basado en eventos (anti-loop)
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
    status_file="$LOGS/plan-$alias.status"
    start_ts_file="$LOGS/plan-$alias.start_ts"
    json_file="$LOGS/plan-$alias.json"

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
      elif [[ -f "$LOGS/plan-$alias.end_ts" ]]; then
        end_ts=$(cat "$LOGS/plan-$alias.end_ts")
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
  status=$(cat "$LOGS/plan-$alias.status" 2>/dev/null || echo "DESCONOCIDO")
  start_ts=$(cat "$LOGS/plan-$alias.start_ts" 2>/dev/null || echo 0)
  end_ts=$(cat "$LOGS/plan-$alias.end_ts" 2>/dev/null || echo "$start_ts")
  elapsed="$(( end_ts - start_ts ))s"
  json_file="$LOGS/plan-$alias.json"
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
    json_path = os.path.join(logs_dir, f"plan-{ag}.json")
    report_path = os.path.join(reports_dir, f"{ag}.md")
    
    if not os.path.exists(json_path):
        print(f"[-] No existe JSON para {ag}")
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
                    # OpenCode format handling
                    if isinstance(obj, dict):
                        if "type" in obj and obj.get("type") == "text":
                            extracted_text.append(obj.get("content", ""))
                        elif "content" in obj and isinstance(obj["content"], str):
                            extracted_text.append(obj["content"])
                        elif "message" in obj and isinstance(obj["message"], dict):
                            parts = obj["message"].get("content", [])
                            if isinstance(parts, list):
                                for p in parts:
                                    if isinstance(p, dict) and p.get("type") == "text":
                                        extracted_text.append(p.get("text", ""))
                                    elif isinstance(p, str):
                                        extracted_text.append(p)
                            elif isinstance(parts, str):
                                extracted_text.append(parts)
                        elif "text" in obj:
                            extracted_text.append(obj.get("text", ""))
                except Exception:
                    continue
    except Exception as e:
        print(f"[!] Error leyendo {json_path}: {e}")
        
    full_content = "\n".join(extracted_text).strip()
    if full_content:
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write(full_content)
        print(f"[+] Reporte extraído para {ag}: {len(full_content)} caracteres -> {report_path}")
    else:
        # Fallback a err log si hubo fallo
        err_path = os.path.join(logs_dir, f"plan-{ag}.err")
        if os.path.exists(err_path) and os.path.getsize(err_path) > 0:
            with open(err_path, 'r', encoding='utf-8') as ef, open(report_path, 'w', encoding='utf-8') as rf:
                rf.write(f"# Error en ejecución de {ag}\n\n```\n" + ef.read() + "\n```")
            print(f"[!] Reportado error para {ag} desde .err")
        else:
            print(f"[-] Sin contenido para {ag}")

print("Extracción completada.")
PYEOF
