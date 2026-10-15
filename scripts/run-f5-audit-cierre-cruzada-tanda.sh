#!/usr/bin/env bash
# ==============================================================================
# run-f5-audit-cierre-cruzada-tanda.sh — Auditoría de Cierre Cruzada (Roles Rotados)
# ==============================================================================
# Subagentes con Roles Rotados:
#   1. luna (gpt-6-luna): Asume rol de Haiku -> Metadatos Control, shlibdeps, CI/CD y RPM
#   2. muse (muse-spark-1.3-contributor): Asume rol de DeepSeek -> Arquitectura FHS, scripts e instalación
#   3. deepseek (deepseek-v4.1-flash): Asume rol de Muse -> Kill-Gates, Zero Dead Buttons y tiempos de HW
#   4. haiku (claude-haiku-5-5): Asume rol de Luna -> Estándares FreeDesktop, inspección empírica y arnés
#
# Propuesta adicional obligatoria para todos:
#   - Evaluar el nombre del proyecto y proponer entre 3 y 5 nombres ideales para el repositorio en GitHub.
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

NOMBRE_TANDA="audit_cierre_f5_cruzada"
SALIDA_DIR="$REPO/.superpowers/$NOMBRE_TANDA"
LOGS="$SALIDA_DIR/logs"
PROMPTS_DIR="$SALIDA_DIR/prompts"
REPORTS_DIR="$SALIDA_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "luna|opencode-go/gpt-6-luna|Metadatos Control, shlibdeps y CI/CD GitHub Actions (Rol ex-Haiku)"
  "muse|opencode-go/muse-spark-1.3-contributor|Arquitectura FHS y Scripts de Mantenedor (Rol ex-DeepSeek)"
  "deepseek|opencode-go/deepseek-v4.1-flash|Kill-Gate Zero Dead Buttons y Señales Hardware (Rol ex-Muse)"
  "haiku|opencode-go/claude-haiku-5-5|Estándares FreeDesktop e Inspección Empírica (Rol ex-Luna)"
)

echo "=============================================================================="
echo " INICIANDO AUDITORÍA CRUZADA DE CIERRE FASE 5 (ROLES ROTADOS + NOMBRES GITHUB)"
echo " Salida: $SALIDA_DIR"
echo " Artefacto a evaluar: target/mx-options_0.1.0-1_amd64.deb"
echo "=============================================================================="

# 1. Generar prompts especializados para cada auditor con rol rotado

# LUNA (Rol ex-Haiku: Metadatos, Dependencias y CI/CD)
cat > "$PROMPTS_DIR/luna.md" << 'EOF'
Eres Luna (gpt-6-luna). En esta sesión rotas de rol y asumes la especialidad de Metadatos Debian, Gestión de Dependencias y CI/CD en GitHub Actions.
Audita la suite de packaging en `master`:
- `scripts/package-deb.sh` (cálculo de dependencias con `dpkg-shlibdeps`, compatibilidad t64, inclusión de `procps, libayatana-appindicator3-1, udev, systemd`, versión dinámica y nombre de paquete).
- `.github/workflows/release.yml` (paso `npm ci`, `components: clippy`, caché de Rust/npm, `permissions: contents: write` acotado al release, sumas SHA256).
- `packaging/rpm/mx-options.spec` (dependencias Fedora incluyendo `procps-ng`, sección `%files` con los 5 binarios).
- `LICENSE` y `packaging/common/copyright` (licencia GPL-2.0 oficial DEP-5).

Analiza e informa:
1. ¿El archivo DEBIAN/control generado en `target/mx-options_0.1.0-1_amd64.deb` cumple todas las directivas Debian?
2. ¿El workflow de GitHub Actions está 100% blindado y listo para compilar sin fallos en un runner limpio?
3. Revisa la consistencia entre los metadatos de Debian y Fedora.

REQUERIMIENTO ESPECIAL — Nombres de Repositorio para GitHub:
Evalúa el nombre actual `mx-options-linux`. Propón entre 3 y 5 nombres creativos, concisos y memorables para el repositorio en GitHub, justificando por qué resuenan bien en la comunidad Linux (considerando que reemplaza Logitech Options/Options+ para el MX Vertical y ratones MX en Rust+Svelte/Tauri v2 sin root).

Emite tu dictamen técnico con hallazgos, propuestas de nombres y visto bueno final.
EOF

# MUSE (Rol ex-DeepSeek: Arquitectura FHS y Scripts)
cat > "$PROMPTS_DIR/muse.md" << 'EOF'
Eres Muse (muse-spark-1.3-contributor). En esta sesión rotas de rol y asumes la especialidad de Arquitectura de Sistemas Linux, Jerarquía FHS y Scripts de Mantenedor Debian/RPM.
Audita la suite de packaging en `master`:
- `packaging/deb/{postinst,prerm,postrm}` y `packaging/common/`.
- `packaging/rpm/mx-options.spec` (secciones `%post`, `%preun`, `%postun`).
- `scripts/package-deb.sh` y el árbol FHS en staging.

Analiza e informa:
1. ¿Los scripts de mantenedor son 100% seguros, idempotentes y sin posibilidad de bloquear `dpkg` o `rpm`?
2. ¿La migración de udev en `postinst` des-sombrea limpiamente cualquier regla previa en `/etc/udev/rules.d/70-mx-options.rules`?
3. ¿La unidad systemd en `/usr/lib/systemd/user/mx-options.service` arranca limpia con `ExecStart=/usr/bin/mx-daemon` y sin directivas inválidas como `systemctl --global daemon-reload`?
4. ¿Existe algún riesgo de permisos, modos o conflictos FHS en los 5 binarios y activos?

REQUERIMIENTO ESPECIAL — Nombres de Repositorio para GitHub:
Evalúa el nombre actual `mx-options-linux`. Propón entre 3 y 5 nombres atractivos y distintivos para el repositorio en GitHub (estilo proyectos Open Source de Linux como Solaar, Piper, libratbag, logiops), destacando su ligereza, control de hardware y espíritu de sistema.

Emite tu dictamen técnico con hallazgos, propuestas de nombres y visto bueno final.
EOF

# DEEPSEEK (Rol ex-Muse: Zero Dead Buttons y Hardware Safe)
cat > "$PROMPTS_DIR/deepseek.md" << 'EOF'
Eres DeepSeek (deepseek-v4.1-flash). En esta sesión rotas de rol y asumes la especialidad de Kill-Gates, Señalización de Procesos y Convivencia de Hardware ("Zero Dead Buttons").
Audita la suite de packaging en `master`:
- `packaging/deb/prerm` y sección `%preun` de `packaging/rpm/mx-options.spec`.
- La lógica de apagado limpio del daemon en `crates/mx-daemon/src/main.rs` y `crates/mx-daemon/src/workers.rs`.
- El arnés de verificación `scripts/audit-packages.sh`.

Analiza e informa:
1. Invariante Zero Dead Buttons: ¿Garantizan `prerm` y `%preun` que ante `apt remove mx-options` o `dnf remove mx-options` el ratón Logitech MX Vertical recupera su función de hardware nativa (DPI switch) y desactiva el divert antes de borrar los binarios?
2. Tiempos y señales: ¿Es adecuado el presupuesto de 2.5 s (`sleep 0.1` x 25), la señal `SIGTERM` y el fallback `SIGKILL`? ¿Qué pasa si el daemon tarda más de 2.5 s?
3. Distinción de ciclo de vida: ¿Verificas que en actualización (`upgrade`) NO se envíe señal de parada ni se altere el hardware?
4. Cobertura del arnés: ¿Las 4 aserciones sobre `prerm` en `scripts/audit-packages.sh` son suficientes para evitar falsos verdes?

REQUERIMIENTO ESPECIAL — Nombres de Repositorio para GitHub:
Evalúa el nombre actual `mx-options-linux`. Propón entre 3 y 5 nombres concisos, técnicos y elegantes para el repositorio en GitHub, resaltando la seguridad de hardware, la precisión del protocolo HID++ y la filosofía de drivers modernos en Linux.

Emite tu dictamen técnico con hallazgos, propuestas de nombres y visto bueno final.
EOF

# HAIKU (Rol ex-Luna: FreeDesktop, Inspección Empírica y Arnés)
cat > "$PROMPTS_DIR/haiku.md" << 'EOF'
Eres Claude Haiku (claude-haiku-5-5). En esta sesión rotas de rol y asumes la especialidad de Estándares FreeDesktop / XDG, Integración con el Escritorio e Inspección Empírica de Artefactos.
Audita el paquete `.deb` real generado en `target/mx-options_0.1.0-1_amd64.deb` y el arnés `scripts/audit-packages.sh`:

Analiza e informa:
1. Inspección empírica sobre `target/mx-options_0.1.0-1_amd64.deb`:
   - Tamaño del paquete (meta < 15 MB).
   - Propietarios (`root/root`) y permisos canónicos (`0755` y `0644`).
   - Presencia de los 5 binarios (`mx-daemon`, `mx-gui`, `mx-diag`, `mx-cli`, `mx-probe`).
   - Lanzador `/usr/share/applications/mx-options.desktop` (`StartupWMClass=mx-gui`, `Exec=/usr/bin/mx-gui`, categorías correctas).
   - Jerarquía completa de iconos Hicolor (16x16 a 512x512 y SVG escalable).
   - Cero cadenas residuales de desarrollo (`/home/panxio`) en archivos y strings de ejecutables.
2. Arnés `scripts/audit-packages.sh`: ¿Es riguroso y cubre todos los puntos críticos del paquete?
3. ¿Cumple la suite con los estándares XDG FreeDesktop para integrarse limpiamente en GNOME, KDE, Cinnamon y gestores Wayland/X11?

REQUERIMIENTO ESPECIAL — Nombres de Repositorio para GitHub:
Evalúa el nombre actual `mx-options-linux`. Propón entre 3 y 5 nombres frescos, memorables y con gancho para la comunidad de desarrolladores y usuarios de Linux en GitHub, destacando la experiencia de escritorio, la elegancia de la GUI y la autonomía frente al software privativo de Windows/macOS.

Emite tu dictamen técnico con hallazgos, propuestas de nombres y visto bueno final.
EOF

# 2. Función de despacho desacoplada con aislamiento systemd-run
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
export LOGS PROMPTS_DIR REPO

# 3. Despacho con concurrencia máxima = 2
MAX_JOBS=2
echo "Despachando agentes de auditoría cruzada con concurrencia máxima = $MAX_JOBS..."

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  while [[ $(jobs -r -p | wc -l) -ge $MAX_JOBS ]]; do
    sleep 2
  done
  echo "--> Despachando: $alias ($model) con rol [$role]..."
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
echo " RESUMEN FINAL CONSOLIDADO — AUDITORÍA CRUZADA"
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

agentes = ["luna", "muse", "deepseek", "haiku"]

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

print("Extracción completada exitosamente.")
PYEOF
