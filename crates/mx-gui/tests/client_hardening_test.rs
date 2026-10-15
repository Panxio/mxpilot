//! Endurecimiento del cliente IPC (`mx-gui`) — Fase 4.5.
//!
//! El cliente ya acota con `take(64 KiB + 1)`, valida `\n` y correlaciona IDs.
//! Estos tests fijan ese comportamiento contra un mock adversario:
//! truncados, gigantes, JSON inválido, NUL, ID mismatch, respuestas lentas
//! (timeout) y ráfaga secuencial. Sin dependencias nuevas.

use mx_daemon::ipc::protocol::{IpcCommand, IpcRequest, IpcResponse};
use mx_gui::client::{ClientError, DaemonClient};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn unique_sock(dir: &tempfile::TempDir, name: &str) -> std::path::PathBuf {
    dir.path().join(format!(
        "{}_{}_{}.sock",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn respuesta_truncada_sin_newline_tras_cierre_es_protocol_error() {
    let dir = tempdir().unwrap();
    let sock = unique_sock(&dir, "trunc");
    let listener = UnixListener::bind(&sock).unwrap();
    let h = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut r = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        // Parcial sin \n y cierre inmediato.
        stream.write_all(b"{\"status\":\"ok\",\"data\":\"pon").unwrap();
        stream.flush().unwrap();
    });
    let client = DaemonClient::with_socket_path(&sock);
    match client.ping() {
        Err(ClientError::ProtocolError(msg)) => {
            assert!(msg.contains("truncada") || msg.contains("EOF") || msg.contains("cerrada"), "msg: {}", msg)
        }
        other => panic!("esperado ProtocolError, llegó {:?}", other),
    }
    h.join().unwrap();
}

#[test]
fn respuesta_gigante_con_newline_es_payload_too_large() {
    let dir = tempdir().unwrap();
    let sock = unique_sock(&dir, "huge");
    let listener = UnixListener::bind(&sock).unwrap();
    let h = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut r = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        // JSON válido pero > 64 KiB + \n.
        let big = "Y".repeat(mx_gui::client::MAX_IPC_RESPONSE_BYTES + 512);
        let resp = format!("{{\"status\":\"ok\",\"data\":\"{}\"}}\n", big);
        stream.write_all(resp.as_bytes()).unwrap();
        stream.flush().unwrap();
    });
    let client = DaemonClient::with_socket_path(&sock);
    match client.ping() {
        Err(ClientError::PayloadTooLarge) => {}
        other => panic!("esperado PayloadTooLarge, llegó {:?}", other),
    }
    h.join().unwrap();
}

#[test]
fn respuesta_json_invalido_y_nul_son_protocol_error() {
    for (name, payload) in [
        ("badjson", b"ESTO NO ES JSON\n".to_vec()),
        ("nul", b"{\"status\":\"ok\",\"data\":\"a\0b\"}\n".to_vec()),
        ("vacia", b"\n".to_vec()), // línea vacía -> serde falla
    ] {
        let dir = tempdir().unwrap();
        let sock = unique_sock(&dir, name);
        let listener = UnixListener::bind(&sock).unwrap();
        let h = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut r = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            r.read_line(&mut line).unwrap();
            stream.write_all(&payload).unwrap();
            stream.flush().unwrap();
        });
        let client = DaemonClient::with_socket_path(&sock);
        match client.ping() {
            Err(ClientError::ProtocolError(_)) => {}
            other => panic!("{}: esperado ProtocolError, llegó {:?}", name, other),
        }
        h.join().unwrap();
    }
}

#[test]
fn id_mismatch_en_ok_es_protocol_error_pero_en_error_es_daemon_error() {
    // Caso 1: Ok con ID erróneo -> ProtocolError (correlación estricta).
    let dir = tempdir().unwrap();
    let sock = unique_sock(&dir, "mismatch_ok");
    let listener = UnixListener::bind(&sock).unwrap();
    let h = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut r = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        let req: IpcRequest = serde_json::from_str(&line).unwrap();
        let resp = IpcResponse::ok(Some(req.id.unwrap_or(1) + 999), "pong");
        let mut out = serde_json::to_vec(&resp).unwrap();
        out.push(b'\n');
        stream.write_all(&out).unwrap();
    });
    let client = DaemonClient::with_socket_path(&sock);
    match client.ping() {
        Err(ClientError::ProtocolError(msg)) => assert!(msg.contains("no coincide"), "msg: {}", msg),
        other => panic!("esperado ProtocolError, llegó {:?}", other),
    }
    h.join().unwrap();

    // Caso 2: Error con ID erróneo -> DaemonError (tolerante, solo warn).
    let dir2 = tempdir().unwrap();
    let sock2 = unique_sock(&dir2, "mismatch_err");
    let listener2 = UnixListener::bind(&sock2).unwrap();
    let h2 = thread::spawn(move || {
        let (mut stream, _) = listener2.accept().unwrap();
        let mut r = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        let req: IpcRequest = serde_json::from_str(&line).unwrap();
        let resp = IpcResponse::error(Some(req.id.unwrap_or(1) + 999), "fallo simulado");
        let mut out = serde_json::to_vec(&resp).unwrap();
        out.push(b'\n');
        stream.write_all(&out).unwrap();
    });
    let client2 = DaemonClient::with_socket_path(&sock2);
    match client2.ping() {
        Err(ClientError::DaemonError(msg)) => assert!(msg.contains("fallo simulado"), "msg: {}", msg),
        other => panic!("esperado DaemonError, llegó {:?}", other),
    }
    h2.join().unwrap();
}

#[test]
fn respuesta_lenta_supera_timeout_rapido() {
    let dir = tempdir().unwrap();
    let sock = unique_sock(&dir, "slow");
    let listener = UnixListener::bind(&sock).unwrap();
    let h = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut r = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        // FAST_READ_TIMEOUT = 1000 ms para ping; dormir 1800 ms lo supera.
        thread::sleep(Duration::from_millis(1800));
        let req: IpcRequest = serde_json::from_str(&line).unwrap();
        let resp = IpcResponse::ok(req.id, "pong-tarde");
        let mut out = serde_json::to_vec(&resp).unwrap();
        out.push(b'\n');
        let _ = stream.write_all(&out);
    });
    let client = DaemonClient::with_socket_path(&sock);
    match client.ping() {
        Err(ClientError::Timeout(_)) => {}
        other => panic!("esperado Timeout, llegó {:?}", other),
    }
    h.join().unwrap();
}

#[test]
fn rafaga_secuencial_50_pings_sin_perdidas() {
    let dir = tempdir().unwrap();
    let sock = unique_sock(&dir, "burst");
    let listener = UnixListener::bind(&sock).unwrap();
    // Mock multi-shot: atiende 50 conexiones seguidas (una por send_command).
    let h = thread::spawn(move || {
        for _ in 0..50 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut r = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if r.read_line(&mut line).is_err() {
                return;
            }
            let req: Result<IpcRequest, _> = serde_json::from_str(&line);
            let resp = match req {
                Ok(q) => IpcResponse::ok(q.id, "pong"),
                Err(e) => IpcResponse::error(None, format!("mock: {}", e)),
            };
            let mut out = serde_json::to_vec(&resp).unwrap();
            out.push(b'\n');
            let _ = stream.write_all(&out);
        }
    });
    let client = DaemonClient::with_socket_path(&sock);
    let t0 = Instant::now();
    for _ in 0..50 {
        let pong = client.ping().expect("ping en ráfaga");
        assert_eq!(pong, "pong");
        // El cliente valida que el comando serializado sea ping (sanidad).
        let _ = IpcCommand::Ping;
    }
    assert!(t0.elapsed() < Duration::from_secs(20), "ráfaga 50 muy lenta");
    h.join().unwrap();
}
