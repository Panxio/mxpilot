#!/usr/bin/env bash
# Alarma de contexto para Antigravity (AGY). Adaptación de ~/.claude/hooks/hook-contexto.sh
# (origen: zeroFeeBank S-25/S-27). Mismos umbrales: 160k alarma, 200k techo.
#
# Evento: PreInvocation (antes de CADA llamada al modelo; mejor que PreToolUse porque
# avisa también en turnos sin herramientas). Entrada: JSON camelCase por stdin.
# Salida: {"injectSteps":[{"ephemeralMessage": "..."}]} o {} .
#
# Contexto = input_tokens + cache_read_tokens de la ULTIMA respuesta del modelo
# (PLANNER_RESPONSE) en transcript_full.jsonl. Se usa la última y no el máximo porque
# si AGY compacta el contexto, el valor real baja.
#
# NO BLOQUEA por defecto (igual que la versión Claude). CTX_BLOQUEA=1 lo convierte en
# corte duro con el hook PreToolUse (modo "deny"), ver hooks.json.
set -uo pipefail
: "${CTX_AVISO:=200000}"   # techo
: "${CTX_ALARMA:=160000}"  # alarma
: "${CTX_BLOQUEA:=0}"      # 1 = en el techo, PreToolUse deniega todo salvo editar STATUS.md
: "${CTX_MODO:=invocation}" # invocation | tool  (lo fija hooks.json)
export CTX_AVISO CTX_ALARMA CTX_BLOQUEA CTX_MODO

exec python3 -c '
import sys, os, json, glob

aviso   = int(os.environ["CTX_AVISO"])
alarma  = int(os.environ["CTX_ALARMA"])
bloquea = int(os.environ["CTX_BLOQUEA"])
modo    = os.environ["CTX_MODO"]

try:
    data = json.load(sys.stdin)
except Exception:
    print("{}"); sys.exit(0)

# Umbrales por modelo: Anthropic (cuota chica) 160k/200k; Gemini (ventana amplia) mas altos.
# Modelo desconocido -> conservador (160k/200k).
modelo = (data.get("modelName") or "").lower()
if "gemini" in modelo:
    alarma = int(os.environ.get("CTX_ALARMA_GEMINI", 600000))
    aviso  = int(os.environ.get("CTX_AVISO_GEMINI", 800000))

ruta = data.get("transcriptPath") or ""
cid  = data.get("conversationId") or ""
if not (ruta and os.path.isfile(ruta)) and cid:
    # transcriptPath puede apuntar al compacto o a otra raiz: busca el completo por id
    cand = glob.glob(os.path.expanduser(
        f"~/.gemini/antigravity*/brain/{cid}/.system_generated/logs/transcript_full.jsonl"))
    ruta = cand[0] if cand else ruta
if ruta.endswith("transcript.jsonl"):
    full = ruta[:-len("transcript.jsonl")] + "transcript_full.jsonl"
    if os.path.isfile(full):
        ruta = full

ctx = 0
try:
    with open(ruta, encoding="utf-8", errors="replace") as fh:
        for linea in fh:
            try: r = json.loads(linea)
            except ValueError: continue
            if r.get("type") != "PLANNER_RESPONSE": continue
            v = (r.get("input_tokens") or 0) + (r.get("cache_read_tokens") or 0)
            if v > 0: ctx = v
except Exception:
    print("{}"); sys.exit(0)

k = ctx // 1000
if ctx >= aviso:
    txt = (f"[CUOTA] contexto {k}k ≥ {aviso//1000}k: AVISO FINAL (no es corte). Si la tarea actual "
           f"puede terminar pronto, termínala; si no, guarda el estado en STATUS.md/relevo y "
           f"sugiere al usuario abrir una sesión nueva en un punto natural.")
    nivel = "techo"
elif ctx >= alarma:
    txt = (f"[CUOTA] contexto {k}k ≥ {alarma//1000}k: ALARMA, no corte. Empieza el cierre "
           f"documental (STATUS.md) sin dejar de trabajar; no empieces trabajo nuevo.")
    nivel = "alarma"
else:
    print("{}"); sys.exit(0)

if modo == "tool":
    # Solo actua el corte duro, y solo si el humano lo activo.
    if nivel == "techo" and bloquea:
        args = json.dumps((data.get("toolCall") or {}).get("args") or {})
        if "STATUS" in args:
            print("{}")
        else:
            print(json.dumps({"decision": "deny", "reason": txt}, ensure_ascii=False))
    else:
        print("{}")
else:
    print(json.dumps({"injectSteps": [{"ephemeralMessage": txt}]}, ensure_ascii=False))
'
