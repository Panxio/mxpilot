#!/usr/bin/env python3
# ==============================================================================
# f45-ipc-attacks.py — Arnés de Ethical Hacking local para mx-daemon (Fase 4.5)
# ==============================================================================
# Prueba la superficie de ataque del Unix Domain Socket de mx-daemon contra
# una instancia EN EJECUCIÓN. Solo envía payloads inválidos o de consulta:
# NO reprograma hardware (ningún set_action válido llega al firmware).
#
# Uso:   python3 scripts/security/f45-ipc-attacks.py
#        echo $?    # 0 = todas las defensas verificadas (T8 documenta el hallazgo)
#
# Árbitro objetivo: imprime PASS/FAIL por test. T8 es un HALLAZGO conocido
# (agotamiento de slots por conexiones ociosas), se marca PASS cuando la
# debilidad se reproduce (evidencia), no cuando la defensa funciona.
# ==============================================================================
import os
import socket
import sys
import time

SOCK = os.path.join(
    os.environ.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}"),
    "mx-options.sock",
)

results = []


def conn():
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(3)
    s.connect(SOCK)
    return s


def recv_line(s):
    try:
        return s.recv(4096).decode("utf-8", "replace").strip()
    except socket.timeout:
        return "<TIMEOUT/NO-RESPONSE>"
    except OSError as e:
        return f"<ERR {e}>"


def alive():
    try:
        s = conn()
        s.sendall(b'{"id":1,"cmd":"ping"}\n')
        r = recv_line(s)
        s.close()
        return '"pong"' in r
    except OSError:
        return False


def T(name, passed, detail):
    results.append((name, "PASS" if passed else "FAIL", detail))


def main():
    if not os.path.exists(SOCK):
        print(f"[!] Socket no encontrado en {SOCK}. ¿Está mx-daemon activo?")
        return 2

    # T1 — JSON malformado -> error estructurado, sin panic
    s = conn()
    s.sendall(b"NOT JSON AT ALL\n")
    r = recv_line(s)
    s.close()
    T("T1 JSON malformado -> error estructurado", '"status":"error"' in r, r[:80])

    # T2 — CID fuera de allowlist
    s = conn()
    s.sendall(
        b'{"id":2,"cmd":"set_action","cid":48879,'
        b'"action":{"type":"SingleKey","key":"KEY_ENTER"}}\n'
    )
    r = recv_line(s)
    s.close()
    T("T2 CID 0xBEEF rechazado", '"status":"error"' in r and "reprogramable" in r, r[:90])

    # T3 — Tecla fuera de allowlist
    s = conn()
    s.sendall(
        b'{"id":3,"cmd":"set_action","cid":253,'
        b'"action":{"type":"SingleKey","key":"KEY_RM_RF"}}\n'
    )
    r = recv_line(s)
    s.close()
    T("T3 tecla no permitida rechazada", '"status":"error"' in r and "no permitida" in r, r[:90])

    # T4 — Payload > 64 KB sin terminador
    s = conn()
    s.sendall(b"A" * 70000)
    r = recv_line(s)
    s.close()
    T("T4 payload >64KB rechazado", "payload too large" in r, r[:80])

    # T5 — UTF-8 inválido en el framing
    s = conn()
    s.sendall(b"\xff\xfe\x00 invalid \n")
    r = recv_line(s)
    s.close()
    T("T5 UTF-8 inválido -> error controlado", "invalid utf-8" in r, r[:80])

    # T6 — Inyección CRLF: no debe corromper el framing
    s = conn()
    s.sendall(b'{"id":6,"cmd":"ping"}\r\n\r\n')
    r = recv_line(s)
    s.close()
    T("T6 CRLF tolerado sin corromper framing", '"status"' in r, r[:80])

    # T7 — Ráfaga de 200 pings (orden y correlación)
    ok = 0
    s = conn()
    for i in range(200):
        s.sendall(f'{{"id":{i},"cmd":"ping"}}\n'.encode())
        if '"pong"' in recv_line(s):
            ok += 1
    s.close()
    T("T7 200 pings seguidos respondidos", ok == 200, f"{ok}/200")

    # T8 — HALLAZGO: 8 conexiones ociosas (0 bytes) retienen slots >2s
    idle = []
    for _ in range(8):
        try:
            idle.append(conn())
        except OSError:
            break
    time.sleep(3.2)  # > INCOMPLETE_LINE_TIMEOUT (2 s)
    try:
        s9 = conn()
        s9.sendall(b'{"id":99,"cmd":"ping"}\n')
        r9 = recv_line(s9)
        s9.close()
    except OSError as e:
        r9 = f"<connect failed {e}>"
    finding = "too many clients" in r9
    T("T8 [HALLAZGO] 8 conexiones ociosas agotan slots", finding, r9[:80])
    for x in idle:
        x.close()
    time.sleep(0.5)

    # T9 — El daemon sigue vivo tras toda la batería
    live = alive()
    T("T9 daemon vivo tras 200+ ataques", live, f"ping_ok={live}")

    # ---- Reporte ----
    print(f"{'TEST':<52} {'RES':<5} DETAIL")
    print("-" * 112)
    for name, res, detail in results:
        print(f"{name:<52} {res:<5} {detail}")
    print("-" * 112)
    fails = [n for n, r, _ in results if r == "FAIL"]
    passed = len(results) - len(fails)
    print(f"RESUMEN: {passed}/{len(results)} PASS" + ((" | FALLOS: " + ", ".join(fails)) if fails else ""))
    return 0 if not fails else 1


if __name__ == "__main__":
    sys.exit(main())
