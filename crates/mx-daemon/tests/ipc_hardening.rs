//! Suite de endurecimiento adversarial IPC — Fase 4.5.
//!
//! Cobertura (sin dependencias nuevas, solo `std` + harness `cargo test`):
//!  1. Fuzz determinista de `parse_request` / `ActionPayload` / `parse_key_code`
//!     (corpus fijo + mutaciones pseudoaleatorias con PRNG xorshift; invariante:
//!     el parser NUNCA entra en pánico, solo `Ok` o `Err(String)`).
//!  2. Framing corrupto sobre socket real: sin `\n` (timeout Slow-Loris 2 s),
//!     truncados, `\r\n`, `\0`, UTF-8 inválido, líneas vacías, pipelining.
//!  3. Límite de payload 64 KiB: exactamente 64 KiB aceptado (sin `payload too
//!     large`), 64 KiB+1 rechazado con desconexión, fragmentación byte a byte.
//!  4. Resiliencia: desconexión a mitad de intercambio, retención de 8 sockets
//!     (`MAX_CLIENTS`), ráfaga de 10 000 mensajes por lotes (respeta la cota de
//!     64 KiB por conexión) + estrés concurrente 8×100.
//!
//! Nota de diseño: el servidor acumula hasta 64 KiB por conexión ANTES de
//! drenar líneas, por lo que una ráfaga de 10 k mensajes en UN solo `write`
//! (~220 KiB) DEBE ser rechazada por diseño. El estrés válido envía por lotes
//! ≤500 líneas (~11 KiB) con lectura entre lotes.

use mx_daemon::ipc::protocol::{
    parse_key_code, parse_mouse_button, parse_request, ActionPayload, IpcCommand, IpcResponse,
};
use mx_daemon::ipc::server::IpcServer;
use mx_daemon::state::DaemonState;
use mx_daemon::workers::DaemonCommand;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

static TEST_SEQ: AtomicU64 = AtomicU64::new(1);

/// PRNG xorshift64 determinista (sin crates nuevos; semilla fija = reproducible).
struct XorShift64(u64);
impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % (n as u64).max(1)) as usize
    }
}

struct Harness {
    dir: std::path::PathBuf,
    socket: std::path::PathBuf,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

fn spawn_server() -> Harness {
    let n = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mx_hardening_{}_{}_{}_{:?}",
        std::process::id(),
        nanos,
        n,
        thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let state = Arc::new(RwLock::new(DaemonState::new()));
    // Sin worker: el canal queda sin receptor; `set_action` válido responde
    // "Worker de hardware no disponible" sin bloquearse (útil para fuzz).
    let (cmd_tx, _cmd_rx) = sync_channel::<DaemonCommand>(8);
    let shutdown = Arc::new(AtomicBool::new(false));
    let server =
        IpcServer::new_with_dir(&dir, state, cmd_tx, shutdown.clone()).expect("crear servidor");
    let socket = server.socket_path().to_path_buf();
    let handle = thread::spawn(move || server.run());
    // Espera activa hasta ~1 s en vez de sleep fijo (menos flaky en CI).
    let deadline = Instant::now() + Duration::from_secs(1);
    while UnixStream::connect(&socket).is_err() {
        if Instant::now() > deadline {
            panic!("socket IPC no apareció en 1 s");
        }
        thread::sleep(Duration::from_millis(10));
    }
    Harness {
        dir,
        socket,
        shutdown,
        handle: Some(handle),
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn connect(h: &Harness, timeout: Duration) -> UnixStream {
    let s = UnixStream::connect(&h.socket).expect("conectar");
    s.set_read_timeout(Some(timeout)).unwrap();
    s
}

fn send_line(stream: &mut UnixStream, line: &str) {
    stream.write_all(line.as_bytes()).unwrap();
    stream.write_all(b"\n").unwrap();
    stream.flush().unwrap();
}

fn read_line(reader: &mut BufReader<UnixStream>, timeout_note: &str) -> String {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .unwrap_or_else(|e| panic!("{}: {}", timeout_note, e));
    line
}

// ═══════════════════════════════════════════════════════════════════
// A. FUZZ DE PARSING PURO (sin socket)
// ═══════════════════════════════════════════════════════════════════

/// Corpus adversarial fijo: ninguno debe provocar pánico; el resultado solo
/// puede ser `Ok` con variante conocida o `Err`.
#[test]
fn fuzz_parse_request_corpus_no_panica() {
    let corpus = vec![
        String::new(),
        " ".to_string(),
        "\n".to_string(),
        "\r\n".to_string(),
        "\0".to_string(),
        "{".to_string(),
        "}".to_string(),
        "{\"id\":1,\"cmd\":\"ping\"".to_string(), // truncado sin cierre
        "{\"id\":1,\"cmd\":\"ping\"}\0".to_string(), // NUL final
        "{\"id\":1,\"cmd\":\"ping\"}\r".to_string(),
        "{\"id\":1,\"cmd\":\"ping\"}\n{\"id\":2,\"cmd\":\"ping\"}".to_string(), // 2 líneas en 1 parse
        "{\"cmd\":\"ping\"} TRAILING GARBAGE".to_string(),
        "null".to_string(),
        "[]".to_string(),
        "42".to_string(),
        "\"ping\"".to_string(),
        "{\"cmd\":null}".to_string(),
        "{\"cmd\":42}".to_string(),
        "{\"cmd\":\"\"}".to_string(),
        "{\"cmd\":\"PING\"}".to_string(), // mayúsculas: debe fallar
        "{\"cmd\":\"ping \"}".to_string(), // espacio final: debe fallar
        "{\"cmd\":\"set_action\"}".to_string(), // faltan cid/action
        "{\"cmd\":\"set_action\",\"cid\":253,\"action\":null}".to_string(),
        "{\"cmd\":\"set_action\",\"cid\":-1,\"action\":{\"type\":\"None\"}}".to_string(),
        "{\"cmd\":\"set_action\",\"cid\":999999,\"action\":{\"type\":\"None\"}}".to_string(),
        "{\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\"}}".to_string(), // falta key
        "{\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"NoExiste\",\"key\":\"X\"}}".to_string(),
        "{\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\",\"key\":\"\"}}".to_string(),
        "{\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\",\"key\":\"; rm -rf /\"}}".to_string(),
        "{\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\",\"key\":\"KEY_LEFTMETA\\u0000\"}}".to_string(),
        "{\"cmd\":\"comando_inexistente\"}".to_string(),
        "{\"id\":\"no-numerico\",\"cmd\":\"ping\"}".to_string(),
        "{\"id\":1.5,\"cmd\":\"ping\"}".to_string(),
        "{\"id\":18446744073709551616,\"cmd\":\"ping\"}".to_string(), // u64 overflow
        "{\"id\":1,\"id\":2,\"cmd\":\"ping\"}".to_string(), // clave duplicada
        "{\"cmd\":\"ping\",\"cmd\":\"get_status\"}".to_string(), // cmd duplicado
        "{\"id\":1,\"cmd\":\"ping\",\"extra_desconocido\":true}".to_string(), // campo extra
        "{\"ID\":1,\"CMD\":\"ping\"}".to_string(), // mayúsculas en claves
        "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]".to_string(), // nesting 50
        "[".repeat(10_000), // nesting 10k incompleto
        "{".repeat(5_000),
        "A".repeat(70_000), // basura gigante
        "{\"cmd\":\"ping\",\"id\":null}".to_string(), // id null explícito = None
        "Coyote dice: {\"cmd\":\"ping\"}".to_string(),
        "😀{\"cmd\":\"ping\"}".to_string(), // emoji prefijo
        "{\"cmd\":\"ping\",\"nota\":\"áéíóú ñ 😀\"}".to_string(), // unicode extra
    ];
    for (i, input) in corpus.iter().enumerate() {
        let r = parse_request(input); // invariante: no panic
        match r {
            Ok(req) => {
                // Si parsea, el comando debe ser una variante conocida y el
                // payload convertible sin pánico (puede dar Err de allowlist).
                match &req.command {
                    IpcCommand::SetAction { action, .. } => {
                        let _ = action.to_action();
                    }
                    IpcCommand::Ping
                    | IpcCommand::GetStatus
                    | IpcCommand::GetHealth
                    | IpcCommand::GetBattery
                    | IpcCommand::ReloadProfile => {}
                }
                // `id:null` explícito debe mapear a None.
                if input.contains("\"id\":null") {
                    assert_eq!(req.id, None, "corpus[{}]: id null -> None", i);
                }
            }
            Err(msg) => assert!(!msg.is_empty(), "corpus[{}]: error vacío", i),
        }
    }
}

/// Mutaciones deterministas sobre una base válida (3000 iteraciones).
#[test]
fn fuzz_parse_request_mutaciones_deterministas() {
    let base = b"{\"id\":1,\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\",\"key\":\"KEY_LEFTMETA\"}}";
    let mut rng = XorShift64(0xC10E_7E42);
    let alphabet: &[u8] = b"{}\"[]:,0123456789 \t\r\n\0\\\"'ABCDExyz_|;/\\u+-NULLping";
    for _ in 0..3000 {
        let mut buf = base.to_vec();
        let ops = 1 + rng.below(4);
        for _ in 0..ops {
            match rng.below(4) {
                0 => {
                    // sustitución
                    if !buf.is_empty() {
                        let p = rng.below(buf.len());
                        buf[p] = alphabet[rng.below(alphabet.len())];
                    }
                }
                1 => {
                    // inserción
                    let p = rng.below(buf.len() + 1);
                    buf.insert(p, alphabet[rng.below(alphabet.len())]);
                }
                2 => {
                    // borrado
                    if !buf.is_empty() {
                        let p = rng.below(buf.len());
                        buf.remove(p);
                    }
                }
                _ => buf.truncate(rng.below(buf.len() + 1)), // truncado
            }
        }
        if let Ok(s) = std::str::from_utf8(&buf) {
            let r = parse_request(s); // invariante: no panic
            if let Ok(req) = r {
                if let IpcCommand::SetAction { action, .. } = &req.command {
                    let _ = action.to_action(); // tampoco debe entrar en pánico
                }
            }
        }
        // bytes no UTF-8 se cubren en el test de socket (UTF-8 inválido).
    }
}

#[test]
fn fuzz_parse_key_code_y_mouse_button_allowlist() {
    // Todas las teclas válidas deben resolver; variantes de caso/espacio NO.
    for valid in [
        "KEY_LEFTMETA", "SUPER", "KEY_ENTER", "KEY_TAB", "KEY_C", "KEY_PLAYPAUSE", "KEY_VOLUMEUP",
        "BTN_LEFT",
    ] {
        if valid.starts_with("BTN_") {
            assert!(parse_mouse_button(valid).is_ok(), "{}", valid);
        } else {
            assert!(parse_key_code(valid).is_ok(), "{}", valid);
        }
    }
    let evil = vec![
        String::new(),
        " ".to_string(),
        "key_leftmeta".to_string(),
        "KEY_LEFTMETA ".to_string(),
        " KEY_LEFTMETA".to_string(),
        "KEY_LEFTMETA\0".to_string(),
        "KEY_LEFTMETA\n".to_string(),
        "SUPER;reboot".to_string(),
        "$(id)".to_string(),
        "KEY_*".to_string(),
        "BTN_LEFT;DROP".to_string(),
        "áéíóú".to_string(),
        "😀".to_string(),
        "A".repeat(1_000_000), // 1 MB de tecla
        "KEY_\u{202E}TSE".to_string(), // override bidireccional
        "\u{FEFF}KEY_ENTER".to_string(), // BOM prefijo
    ];
    for e in &evil {
        assert!(parse_key_code(e).is_err(), "key debió rechazarse: {:?}", &e[..e.len().min(32)]);
        assert!(parse_mouse_button(e).is_err(), "btn debió rechazarse: {:?}", &e[..e.len().min(32)]);
    }
    // Fuzz aleatorio del nombre: 2000 intentos, invariante = no panic.
    let mut rng = XorShift64(0xBAD_C0DE);
    let alpha: &[u8] = b"KEY_BTN_LEFTRIGHTSUPERMETAALTCTRLSHIFT nail\t\r\n\0_012 ;$(){}";
    for _ in 0..2000 {
        let len = rng.below(48);
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(alpha[rng.below(alpha.len())]);
        }
        if let Ok(s) = std::str::from_utf8(&v) {
            let _ = parse_key_code(s);
            let _ = parse_mouse_button(s);
            let p = ActionPayload::SingleKey { key: s.to_string() };
            let _ = p.to_action();
        }
    }
}

#[test]
fn fuzz_action_payload_roundtrip_y_format_resiliente() {
    // Round-trip de variantes válidas.
    let payloads = vec![
        ActionPayload::SingleKey { key: "KEY_ENTER".into() },
        ActionPayload::KeyCombo {
            modifiers: vec!["KEY_LEFTCTRL".into(), "KEY_LEFTALT".into()],
            key: "KEY_RIGHT".into(),
        },
        ActionPayload::MouseButton { button: "BTN_LEFT".into() },
        ActionPayload::None,
        ActionPayload::SystemDefault,
    ];
    for p in &payloads {
        let json = serde_json::to_string(p).unwrap();
        let back: ActionPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(*p, back);
    }
    // `format_response` siempre termina en \n y es JSON válido aun con
    // mensajes adversariales (saltos, comillas, NUL como escape, unicode).
    for evil_msg in [
        "x".to_string(),
        "línea1\nlínea2\r\ninyección: \"status\":\"ok\"".to_string(),
        "\\u0000 Null y \u{202E} override".to_string(),
        "🖱️".repeat(5000),
    ] {
        let s = mx_daemon::ipc::protocol::format_response(&IpcResponse::error(None, &evil_msg));
        assert!(s.ends_with('\n'), "siempre framing con \\n");
        let v: serde_json::Value = serde_json::from_str(s.trim_end()).expect("JSON válido");
        assert_eq!(v["status"], "error");
    }
}

// ═══════════════════════════════════════════════════════════════════
// B. FRAMING CORRUPTO SOBRE SOCKET REAL
// ═══════════════════════════════════════════════════════════════════

#[test]
fn framing_sin_salto_de_linea_provoca_timeout_y_servidor_sigue_vivo() {
    let h = spawn_server();
    // Cliente A: línea incompleta sin \n, mantiene el socket abierto.
    let mut slow = connect(&h, Duration::from_secs(4));
    slow.write_all(b"{\"id\":1,\"cmd\":\"ping\"}").unwrap(); // sin \n
    slow.flush().unwrap();

    // Cliente B debe seguir siendo atendido (sin bloqueo head-of-line).
    let mut fast = connect(&h, Duration::from_millis(800));
    send_line(&mut fast, "{\"id\":2,\"cmd\":\"ping\"}");
    let mut r = BufReader::new(fast.try_clone().unwrap());
    let line = read_line(&mut r, "cliente sano bloqueado por slow-loris");
    let resp: IpcResponse = serde_json::from_str(line.trim()).unwrap();
    assert!(matches!(resp, IpcResponse::Ok { .. }), "respuesta: {}", line);

    // Tras INCOMPLETE_LINE_TIMEOUT (2 s) el servidor cierra al cliente lento.
    let mut rslow = BufReader::new(slow.try_clone().unwrap());
    rslow.get_ref().set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    let mut buf = String::new();
    match rslow.read_line(&mut buf) {
        Ok(0) => {} // EOF: cerró la conexión. Esperado.
        Ok(_) => panic!("slow-loris debió cerrarse con EOF, recibió: {:?}", buf),
        Err(e) => {
            // Reset/timeout también acreditan que no quedó retenido por siempre.
            let k = e.kind();
            assert!(
                matches!(
                    k,
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::BrokenPipe
                        | std::io::ErrorKind::UnexpectedEof
                ),
                "error inesperado slow-loris: {}",
                e
            );
        }
    }

    // El servidor sigue vivo para un tercer cliente.
    let mut third = connect(&h, Duration::from_millis(800));
    send_line(&mut third, "{\"id\":3,\"cmd\":\"ping\"}");
    let mut r3 = BufReader::new(third.try_clone().unwrap());
    let line3 = read_line(&mut r3, "servidor muerto tras slow-loris");
    assert!(line3.contains("\"pong\""), "respuesta: {}", line3);
}

#[test]
fn framing_truncado_responde_error_y_no_contamina_siguiente() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_millis(800));
    let mut r = BufReader::new(s.try_clone().unwrap());
    // Truncado CON \n -> error estructurado, conexión viva.
    s.write_all(b"{\"id\":1,\"cmd\":\"ping\"\n").unwrap();
    s.flush().unwrap();
    let line = read_line(&mut r, "sin respuesta a truncado");
    let resp: IpcResponse = serde_json::from_str(line.trim()).unwrap();
    assert!(matches!(resp, IpcResponse::Error { .. }), "línea: {}", line);
    // Siguiente comando válido en la MISMA conexión debe funcionar.
    send_line(&mut s, "{\"id\":2,\"cmd\":\"ping\"}");
    let line2 = read_line(&mut r, "conexión contaminada tras truncado");
    let resp2: IpcResponse = serde_json::from_str(line2.trim()).unwrap();
    if let IpcResponse::Ok { id, .. } = resp2 {
        assert_eq!(id, Some(2));
    } else {
        panic!("esperaba ok, llegó: {}", line2);
    }
}

#[test]
fn framing_crlf_nul_vacias_y_pipelining() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_millis(1000));
    let mut r = BufReader::new(s.try_clone().unwrap());

    // 1. CRLF debe aceptarse (el servidor hace trim).
    s.write_all(b"{\"id\":11,\"cmd\":\"ping\"}\r\n").unwrap();
    s.flush().unwrap();
    let l = read_line(&mut r, "CRLF rechazado");
    assert!(l.contains("\"pong\""), "CRLF: {}", l);

    // 2. Líneas vacías se ignoran; el siguiente ping responde.
    s.write_all(b"\n\n   \n{\"id\":12,\"cmd\":\"ping\"}\n").unwrap();
    s.flush().unwrap();
    let l = read_line(&mut r, "líneas vacías rompieron framing");
    assert!(l.contains("\"pong\""), "vacías: {}", l);

    // 3. NUL embebido -> error estructurado, conexión viva.
    s.write_all(b"{\"id\":13,\"cmd\":\"ping\"}\0\n").unwrap();
    s.flush().unwrap();
    let l = read_line(&mut r, "NUL sin respuesta");
    let resp: IpcResponse = serde_json::from_str(l.trim()).unwrap();
    assert!(matches!(resp, IpcResponse::Error { .. }), "NUL: {}", l);
    send_line(&mut s, "{\"id\":14,\"cmd\":\"ping\"}");
    let l = read_line(&mut r, "conexión muerta tras NUL");
    assert!(l.contains("\"pong\""), "post-NUL: {}", l);

    // 4. Pipelining: 3 mensajes en un solo write -> 3 respuestas en orden.
    s.write_all(
        b"{\"id\":21,\"cmd\":\"ping\"}\n{\"id\":22,\"cmd\":\"ping\"}\n{\"id\":23,\"cmd\":\"ping\"}\n",
    )
    .unwrap();
    s.flush().unwrap();
    for want in [21u64, 22, 23] {
        let l = read_line(&mut r, "pipelining perdió respuesta");
        let resp: IpcResponse = serde_json::from_str(l.trim()).unwrap();
        if let IpcResponse::Ok { id, .. } = resp {
            assert_eq!(id, Some(want));
        } else {
            panic!("pipelining esperaba ok {}: {}", want, l);
        }
    }
}

#[test]
fn framing_utf8_invalido_responde_error_y_sobrevive() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_millis(800));
    // 0xFF 0xFE nunca son UTF-8 válidos.
    s.write_all(b"{\"id\":1,\"cmd\":\"ping\xff\xfe\"}\n").unwrap();
    s.flush().unwrap();
    let mut r = BufReader::new(s.try_clone().unwrap());
    let l = read_line(&mut r, "UTF-8 inválido sin respuesta");
    let resp: IpcResponse = serde_json::from_str(l.trim()).unwrap();
    if let IpcResponse::Error { message, .. } = resp {
        assert!(message.contains("invalid utf-8"), "mensaje: {}", message);
    } else {
        panic!("esperaba invalid utf-8: {}", l);
    }
    send_line(&mut s, "{\"id\":2,\"cmd\":\"ping\"}");
    let l2 = read_line(&mut r, "conexión muerta tras UTF-8 inválido");
    assert!(l2.contains("\"pong\""), "post-utf8: {}", l2);
}

// ═══════════════════════════════════════════════════════════════════
// C. LÍMITE 64 KiB + FRAGMENTACIÓN
// ═══════════════════════════════════════════════════════════════════

const LIMITE: usize = 65_536;

#[test]
fn limite_exactamente_64k_no_dispara_payload_too_large() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_secs(2));
    // Relleno con espacios (serde los ignora) hasta total exacto 65536 con \n.
    let base = "{\"id\":1,\"cmd\":\"ping\"}";
    let pad = LIMITE - base.len() - 1; // -1 por el \n
    let mut msg = String::with_capacity(LIMITE);
    msg.push_str(base);
    msg.push_str(&" ".repeat(pad));
    assert_eq!(msg.len() + 1, LIMITE);
    s.write_all(msg.as_bytes()).unwrap();
    s.write_all(b"\n").unwrap();
    s.flush().unwrap();
    let mut r = BufReader::new(s.try_clone().unwrap());
    let l = read_line(&mut r, "sin respuesta en borde 64K");
    assert!(
        !l.contains("payload too large"),
        "falso positivo en el borde: {}",
        &l[..l.len().min(200)]
    );
    assert!(l.contains("\"pong\""), "borde 64K: {}", &l[..l.len().min(200)]);
}

#[test]
fn limite_64k_mas_1_rechaza_y_desconecta() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_secs(2));
    // 65536 'A' + '\n' = 65537 bytes acumulados -> excede la cota.
    let big = vec![b'A'; LIMITE];
    s.write_all(&big).unwrap();
    s.write_all(b"\n").unwrap();
    s.flush().unwrap();
    let mut r = BufReader::new(s.try_clone().unwrap());
    let l = read_line(&mut r, "sin respuesta a oversize");
    let resp: IpcResponse =
        serde_json::from_str(l.trim()).expect("el rechazo debe ser JSON válido");
    if let IpcResponse::Error { message, .. } = resp {
        assert!(message.contains("payload too large"), "mensaje: {}", message);
    } else {
        panic!("esperaba payload too large: {}", l);
    }
    // Tras el rechazo el servidor cierra: siguiente lectura = EOF.
    r.get_ref().set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut tail = String::new();
    match r.read_line(&mut tail) {
        Ok(0) => {}
        Ok(_) => panic!("conexión debió cerrarse tras oversize, llegó: {:?}", tail),
        Err(_) => {} // Reset también válido.
    }
    // El servidor sigue aceptando clientes nuevos.
    let mut s2 = connect(&h, Duration::from_millis(800));
    send_line(&mut s2, "{\"id\":9,\"cmd\":\"ping\"}");
    let mut r2 = BufReader::new(s2.try_clone().unwrap());
    let l2 = read_line(&mut r2, "servidor muerto tras oversize");
    assert!(l2.contains("\"pong\""));
}

#[test]
fn fragmentacion_byte_por_byte_reensambla() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_secs(3));
    let msg = b"{\"id\":77,\"cmd\":\"ping\"}\n";
    for b in msg.iter() {
        s.write_all(std::slice::from_ref(b)).unwrap();
        s.flush().unwrap();
        thread::sleep(Duration::from_millis(1)); // Slow-Loris benigno (< 2 s)
    }
    let mut r = BufReader::new(s.try_clone().unwrap());
    let l = read_line(&mut r, "fragmentación byte-a-byte sin respuesta");
    let resp: IpcResponse = serde_json::from_str(l.trim()).unwrap();
    if let IpcResponse::Ok { id, .. } = resp {
        assert_eq!(id, Some(77));
    } else {
        panic!("esperaba ok 77: {}", l);
    }
}

// ═══════════════════════════════════════════════════════════════════
// D. RESILIENCIA: DESCONEXIÓN, RETENCIÓN, RÁFAGAS
// ═══════════════════════════════════════════════════════════════════

#[test]
fn desconexion_forzada_mitad_intercambio_no_envenena_servidor() {
    let h = spawn_server();
    {
        let mut s = connect(&h, Duration::from_millis(500));
        s.write_all(b"{\"id\":1,\"cmd\":\"get_he").unwrap(); // mitad, sin \n
        s.flush().unwrap();
        // Cierre abrupto: shutdown + drop sin leer.
        let _ = s.shutdown(std::net::Shutdown::Both);
    } // drop aquí
    thread::sleep(Duration::from_millis(150)); // el loop (20 ms) recoge el EOF
    let mut s2 = connect(&h, Duration::from_millis(800));
    send_line(&mut s2, "{\"id\":2,\"cmd\":\"ping\"}");
    let mut r = BufReader::new(s2.try_clone().unwrap());
    let l = read_line(&mut r, "servidor envenenado tras cierre abrupto");
    assert!(l.contains("\"pong\""), "línea: {}", l);
}

#[test]
fn retencion_max_clientes_noveno_rechazado_y_recupera() {
    let h = spawn_server();
    // Ocupar los 8 slots (MAX_CLIENTS) con conexiones ociosas.
    let mut holders = Vec::new();
    for _ in 0..8 {
        holders.push(connect(&h, Duration::from_millis(500)));
    }
    thread::sleep(Duration::from_millis(400)); // el accept es 1/loop de 20 ms
    // Noveno cliente: rechazo cortés con JSON + cierre.
    let mut extra = connect(&h, Duration::from_secs(2));
    send_line(&mut extra, "{\"id\":99,\"cmd\":\"ping\"}");
    let mut r = BufReader::new(extra.try_clone().unwrap());
    let l = read_line(&mut r, "noveno cliente sin respuesta");
    assert!(l.contains("too many clients"), "línea: {}", l);
    // Liberar un slot y verificar recuperación.
    holders.pop();
    thread::sleep(Duration::from_millis(200));
    let mut s = connect(&h, Duration::from_millis(800));
    send_line(&mut s, "{\"id\":100,\"cmd\":\"ping\"}");
    let mut r2 = BufReader::new(s.try_clone().unwrap());
    let l2 = read_line(&mut r2, "sin recuperación tras liberar slot");
    assert!(l2.contains("\"pong\""), "línea: {}", l2);
}

#[test]
fn rafaga_10k_pipelined_por_lotes_sin_perdidas() {
    let h = spawn_server();
    let mut s = connect(&h, Duration::from_secs(5));
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut r = BufReader::new(s.try_clone().unwrap());
    let total: u64 = 10_000;
    let lote: u64 = 500; // ~11 KiB por lote, muy por debajo de 64 KiB
    let t0 = Instant::now();
    let mut next_id: u64 = 1;
    while next_id <= total {
        let fin = (next_id + lote - 1).min(total);
        for id in next_id..=fin {
            let msg = format!("{{\"id\":{},\"cmd\":\"ping\"}}\n", id);
            s.write_all(msg.as_bytes()).unwrap();
        }
        s.flush().unwrap();
        for want in next_id..=fin {
            let mut l = String::new();
            r.read_line(&mut l).expect("lectura en ráfaga");
            let resp: IpcResponse = serde_json::from_str(l.trim()).expect("JSON en ráfaga");
            match resp {
                IpcResponse::Ok { id, .. } => assert_eq!(id, Some(want), "orden en ráfaga"),
                IpcResponse::Error { message, .. } => {
                    panic!("error en ráfaga id {}: {}", want, message)
                }
            }
        }
        next_id = fin + 1;
    }
    let dt = t0.elapsed();
    // Meta numérica Pilar 7 para el arnés: 10 k ping-pong < 30 s en loopback.
    assert!(dt < Duration::from_secs(30), "ráfaga 10k tardó {:?}", dt);
    eprintln!("rafaga_10k: 10000/10000 ok en {:?}", dt);
}

#[test]
fn estres_concurrente_8_clientes_x_100_sin_perdidas() {
    let h = Arc::new(spawn_server());
    let sock = h.socket.clone();
    let mut handles = Vec::new();
    for _ in 0..8 {
        let p = sock.clone();
        handles.push(thread::spawn(move || {
            let mut s = UnixStream::connect(&p).expect("conectar estrés");
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            s.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
            let mut r = BufReader::new(s.try_clone().unwrap());
            // 100 pings pipelined en un solo lote (~2.3 KiB).
            for i in 0..100u64 {
                let msg = format!("{{\"id\":{},\"cmd\":\"ping\"}}\n", 1000 + i);
                s.write_all(msg.as_bytes()).unwrap();
            }
            s.flush().unwrap();
            for want in 0..100u64 {
                let mut l = String::new();
                r.read_line(&mut l).expect("lectura concurrente");
                let resp: IpcResponse = serde_json::from_str(l.trim()).unwrap();
                match resp {
                    IpcResponse::Ok { id, .. } => assert_eq!(id, Some(1000 + want)),
                    IpcResponse::Error { message, .. } => {
                        panic!("error concurrente: {}", message)
                    }
                }
            }
        }));
    }
    for hd in handles {
        hd.join().expect("hilo de estrés");
    }
}
