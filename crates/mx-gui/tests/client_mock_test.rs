use mx_daemon::ipc::protocol::{
    ActionPayload, IpcCommand, IpcRequest, IpcResponse, StatusData, TOP_BUTTON_CID,
};
use mx_gui::client::{ClientError, DaemonClient, MAX_IPC_RESPONSE_BYTES};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::thread;
use tempfile::tempdir;

#[test]
fn test_socket_not_found_returns_daemon_not_running() {
    let dir = tempdir().expect("tempdir");
    let missing_sock = dir.path().join("missing.sock");
    let client = DaemonClient::with_socket_path(missing_sock);

    let res = client.ping();
    match res {
        Err(ClientError::DaemonNotRunning(_)) => {}
        other => panic!("Esperado DaemonNotRunning, obtenido {:?}", other),
    }
}

#[test]
fn test_ping_success_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("test_ping.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");

        let req: IpcRequest = serde_json::from_str(&line).expect("parse req");
        assert_eq!(req.command, IpcCommand::Ping);

        let resp = IpcResponse::ok(req.id, "pong");
        let mut out = serde_json::to_vec(&resp).expect("serialize");
        out.push(b'\n');
        stream.write_all(&out).expect("write");
    });

    let client = DaemonClient::with_socket_path(sock_path);
    let pong = client.ping().expect("ping debe ser exitoso");
    assert_eq!(pong, "pong");

    handle.join().expect("join");
}

#[test]
fn test_payload_exceeding_64kb_is_rejected() {
    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("test_large.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");

        // Enviar respuesta gigante superior a 64 KB sin salto de línea
        let huge_data = "X".repeat(MAX_IPC_RESPONSE_BYTES + 1024);
        let _ = stream.write_all(huge_data.as_bytes());
        let _ = stream.flush();
    });

    let client = DaemonClient::with_socket_path(sock_path);
    let res = client.ping();
    match res {
        Err(ClientError::PayloadTooLarge) => {}
        other => panic!("Esperado PayloadTooLarge, obtenido {:?}", other),
    }

    handle.join().expect("join");
}

#[test]
fn test_premature_eof_detected() {
    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("test_eof.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        // Cierra la conexión de inmediato sin responder nada
        drop(stream);
    });

    let client = DaemonClient::with_socket_path(sock_path);
    let res = client.ping();
    match res {
        Err(ClientError::ProtocolError(msg)) => {
            assert!(msg.contains("EOF") || msg.contains("Reset") || msg.contains("cerrada"));
        }
        other => panic!("Esperado ProtocolError por cierre prematuro/EOF, obtenido {:?}", other),
    }

    handle.join().expect("join");
}

#[test]
fn test_id_mismatch_rejected() {
    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("test_mismatch.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");

        let req: IpcRequest = serde_json::from_str(&line).expect("parse req");
        let wrong_id = req.id.unwrap_or(1) + 999;
        let resp = IpcResponse::ok(Some(wrong_id), "pong");
        let mut out = serde_json::to_vec(&resp).expect("serialize");
        out.push(b'\n');
        stream.write_all(&out).expect("write");
    });

    let client = DaemonClient::with_socket_path(sock_path);
    let res = client.ping();
    match res {
        Err(ClientError::ProtocolError(msg)) => {
            assert!(msg.contains("no coincide"));
        }
        other => panic!("Esperado ProtocolError por ID mismatch, obtenido {:?}", other),
    }

    handle.join().expect("join");
}

#[test]
fn test_get_status_and_health_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let sock_path = dir.path().join("test_status.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone"));

        // Atender get_status
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");
        let req: IpcRequest = serde_json::from_str(&line).expect("parse req");
        assert_eq!(req.command, IpcCommand::GetStatus);

        let status_data = StatusData {
            connected: true,
            device_name: Some("Logitech MX Vertical".to_string()),
            device_path: Some("/dev/hidraw3".to_string()),
            battery_level: Some(20),
            battery_status: Some("Discharging".to_string()),
            diverted_cids: vec![TOP_BUTTON_CID],
            mapping: HashMap::new(),
        };
        let resp = IpcResponse::ok(req.id, status_data);
        let mut out = serde_json::to_vec(&resp).expect("serialize");
        out.push(b'\n');
        stream.write_all(&out).expect("write");
    });

    let client = DaemonClient::with_socket_path(sock_path);
    let status = client.get_status().expect("get_status");
    assert!(status.connected);
    assert_eq!(status.device_name.as_deref(), Some("Logitech MX Vertical"));
    assert_eq!(status.battery_level, Some(20));

    handle.join().expect("join");
}

#[test]
fn test_set_action_validates_allowlist_locally() {
    let client = DaemonClient::with_socket_path(PathBuf::from("/dev/null"));

    // CID inválido (no en allowlist)
    let bad_action = ActionPayload::SingleKey {
        key: "KEY_LEFTMETA".to_string(),
    };
    let res = client.set_action(0x9999, bad_action);
    assert!(res.is_err());
    match res {
        Err(ClientError::ProtocolError(msg)) => {
            assert!(msg.contains("no es reprogramable"));
        }
        other => panic!("Esperado ProtocolError por CID inválido, obtenido {:?}", other),
    }

    // Tecla inválida (no en allowlist evdev)
    let hacker_action = ActionPayload::SingleKey {
        key: "KEY_MALICIOUS_SHELL".to_string(),
    };
    let res = client.set_action(TOP_BUTTON_CID, hacker_action);
    assert!(res.is_err());
}
