use mx_daemon::ipc::server::IpcServer;
use mx_daemon::ipc::IpcResponse;
use mx_daemon::state::DaemonState;
use mx_daemon::workers::DaemonCommand;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[test]
fn test_ipc_server_ping_and_status() {
    let temp_dir = std::env::temp_dir().join(format!("mx_ipc_test_ping_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let state = Arc::new(RwLock::new(DaemonState::new()));
    let (cmd_tx, _cmd_rx) = sync_channel::<DaemonCommand>(8);
    let shutdown = Arc::new(AtomicBool::new(false));

    let server = IpcServer::new_with_dir(&temp_dir, state, cmd_tx, shutdown.clone()).expect("Debe crear servidor IPC");
    let socket_path = server.socket_path().to_path_buf();

    let srv_handle = thread::spawn(move || {
        server.run();
    });

    // Esperar a que el socket esté listo
    thread::sleep(Duration::from_millis(50));

    // Conectar cliente UnixStream
    let mut stream = UnixStream::connect(&socket_path).expect("Debe conectar al socket");
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();

    // 1. Enviar Ping
    stream.write_all(b"{\"id\":10,\"cmd\":\"ping\"}\n").unwrap();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Ok { id, data } = resp {
        assert_eq!(id, Some(10));
        assert_eq!(data.unwrap(), serde_json::json!("pong"));
    } else {
        panic!("Respuesta inesperada para ping: {}", line);
    }

    // 2. Enviar GetHealth
    line.clear();
    stream
        .write_all(b"{\"id\":20,\"cmd\":\"get_health\"}\n")
        .unwrap();
    reader.read_line(&mut line).unwrap();

    let health_resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Ok { id, data } = health_resp {
        assert_eq!(id, Some(20));
        let val = data.unwrap();
        assert_eq!(val["state"], "Disconnected");
        assert!(val["rss_kb"].as_u64().unwrap() > 0);
    } else {
        panic!("Respuesta inesperada para health: {}", line);
    }

    // 3. Enviar JSON malformado -> respuesta estructurada de error
    line.clear();
    stream.write_all(b"ESTO NO ES UN JSON\n").unwrap();
    reader.read_line(&mut line).unwrap();

    let err_resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Error { message, .. } = err_resp {
        assert!(message.contains("Error deserializando JSON"));
    } else {
        panic!("Se esperaba error estructurado");
    }

    // Apagar servidor
    shutdown.store(true, Ordering::SeqCst);
    let _ = srv_handle.join();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ipc_payload_limit_defence() {
    let temp_dir = std::env::temp_dir().join(format!("mx_ipc_test_limit_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let state = Arc::new(RwLock::new(DaemonState::new()));
    let (cmd_tx, _cmd_rx) = sync_channel::<DaemonCommand>(8);
    let shutdown = Arc::new(AtomicBool::new(false));

    let server = IpcServer::new_with_dir(&temp_dir, state, cmd_tx, shutdown.clone()).expect("Debe crear servidor IPC");
    let socket_path = server.socket_path().to_path_buf();

    let srv_handle = thread::spawn(move || {
        server.run();
    });

    thread::sleep(Duration::from_millis(50));

    let mut stream = UnixStream::connect(&socket_path).expect("Debe conectar");
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();

    // Enviar más de 64 KB sin salto de línea
    let oversized = vec![b'A'; 70_000];
    let _ = stream.write_all(&oversized);

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let _ = reader.read_line(&mut line);

    let resp: IpcResponse = serde_json::from_str(&line).unwrap_or(IpcResponse::Error {
        id: None,
        message: "EOF".to_string(),
    });

    if let IpcResponse::Error { message, .. } = resp {
        assert!(message.contains("payload too large"), "Mensaje inesperado: {}", message);
    } else {
        panic!("Se esperaba error de payload too large");
    }

    shutdown.store(true, Ordering::SeqCst);
    let _ = srv_handle.join();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ipc_set_action_atomicity_and_validation() {
    let temp_dir = std::env::temp_dir().join(format!("mx_ipc_test_action_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let state = Arc::new(RwLock::new(DaemonState::new()));
    let (cmd_tx, cmd_rx) = sync_channel::<DaemonCommand>(8);
    let shutdown = Arc::new(AtomicBool::new(false));

    let server = IpcServer::new_with_dir(&temp_dir, state.clone(), cmd_tx, shutdown.clone()).expect("Debe crear servidor IPC");
    let socket_path = server.socket_path().to_path_buf();

    let srv_handle = thread::spawn(move || {
        server.run();
    });

    thread::sleep(Duration::from_millis(50));

    let mut stream = UnixStream::connect(&socket_path).expect("Debe conectar");
    stream.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // 1. CID inválido
    stream.write_all(b"{\"id\":1,\"cmd\":\"set_action\",\"cid\":39321,\"action\":{\"type\":\"SingleKey\",\"key\":\"KEY_ENTER\"}}\n").unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Error { id, message } = resp {
        assert_eq!(id, Some(1));
        assert!(message.contains("no es reprogramable"));
    } else {
        panic!("Esperaba error por CID inválido");
    }

    // 2. Tecla inválida
    line.clear();
    stream.write_all(b"{\"id\":2,\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\",\"key\":\"TECLA_FANTASMA\"}}\n").unwrap();
    reader.read_line(&mut line).unwrap();
    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Error { id, message } = resp {
        assert_eq!(id, Some(2));
        assert!(message.contains("Tecla no permitida o desconocida"));
    } else {
        panic!("Esperaba error por tecla inválida");
    }

    // 3. Simular fallo de hardware y verificar ATOMICIDAD (state.mapping no debe mutar)
    let worker_thread = thread::spawn(move || {
        if let Ok(DaemonCommand::SetAction { reply, .. }) = cmd_rx.recv_timeout(Duration::from_millis(500)) {
            let _ = reply.send(Err("Hardware error simulado".to_string()));
        }
    });

    line.clear();
    stream.write_all(b"{\"id\":3,\"cmd\":\"set_action\",\"cid\":253,\"action\":{\"type\":\"SingleKey\",\"key\":\"KEY_ENTER\"}}\n").unwrap();
    reader.read_line(&mut line).unwrap();
    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Error { id, message } = resp {
        assert_eq!(id, Some(3));
        assert!(message.contains("Hardware error simulado"));
    } else {
        panic!("Esperaba error de hardware");
    }

    worker_thread.join().unwrap();

    // Invariante de atomicidad: mapping para 253 debe seguir siendo KEY_LEFTMETA, no KEY_ENTER!
    {
        let s = state.read().unwrap();
        let current_map = s.mapping.get(&253).unwrap();
        if let mx_daemon::ipc::ActionPayload::SingleKey { key } = current_map {
            assert_eq!(key, "KEY_LEFTMETA", "Violación de atomicidad: el mapeo mutó pese a error de hardware");
        } else {
            panic!("Mapeo inesperado");
        }
    }

    shutdown.store(true, Ordering::SeqCst);
    let _ = srv_handle.join();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ipc_reload_profile_and_battery() {
    let temp_dir = std::env::temp_dir().join(format!("mx_ipc_test_misc_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let state = Arc::new(RwLock::new(DaemonState::new()));
    let (cmd_tx, cmd_rx) = sync_channel::<DaemonCommand>(8);
    let shutdown = Arc::new(AtomicBool::new(false));

    let server = IpcServer::new_with_dir(&temp_dir, state.clone(), cmd_tx, shutdown.clone()).expect("Debe crear servidor IPC");
    let socket_path = server.socket_path().to_path_buf();

    let srv_handle = thread::spawn(move || {
        server.run();
    });

    let worker_handle = thread::spawn(move || {
        while let Ok(cmd) = cmd_rx.recv_timeout(Duration::from_millis(500)) {
            match cmd {
                DaemonCommand::RefreshBattery { reply } => {
                    let _ = reply.send(Err("Dispositivo no conectado".to_string()));
                }
                DaemonCommand::SetAction { reply, .. } => {
                    let _ = reply.send(Ok(()));
                }
                DaemonCommand::Shutdown => break,
            }
        }
    });

    thread::sleep(Duration::from_millis(50));

    let mut stream = UnixStream::connect(&socket_path).expect("Debe conectar");
    stream.set_read_timeout(Some(Duration::from_millis(1500))).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // 1. reload_profile devuelve error honesto informando planificación en Fase 4
    stream.write_all(b"{\"id\":100,\"cmd\":\"reload_profile\"}\n").unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Error { id, message } = resp {
        assert_eq!(id, Some(100));
        assert!(message.contains("Phase 4"));
    } else {
        panic!("Esperaba error honesto para reload_profile");
    }

    // 2. get_battery sin batería en estado -> worker responde error -> error
    line.clear();
    stream.write_all(b"{\"id\":101,\"cmd\":\"get_battery\"}\n").unwrap();
    reader.read_line(&mut line).unwrap();
    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Error { id, message } = resp {
        assert_eq!(id, Some(101));
        assert!(message.contains("Battery information not available"));
    } else {
        panic!("Esperaba error cuando no hay batería");
    }

    // 3. get_battery con caché reciente -> respuesta inmediata ok
    {
        let mut s = state.write().unwrap();
        s.battery_level = Some(85);
        s.battery_status = Some("Discharging".to_string());
        s.battery_last_updated = Some(std::time::Instant::now());
    }

    line.clear();
    stream.write_all(b"{\"id\":102,\"cmd\":\"get_battery\"}\n").unwrap();
    reader.read_line(&mut line).unwrap();
    let resp: IpcResponse = serde_json::from_str(&line).unwrap();
    if let IpcResponse::Ok { id, data } = resp {
        assert_eq!(id, Some(102));
        let bat = data.unwrap();
        assert_eq!(bat["percentage"], 85);
        assert_eq!(bat["status"], "Discharging");
    } else {
        panic!("Esperaba batería en caché");
    }

    shutdown.store(true, Ordering::SeqCst);
    let _ = srv_handle.join();
    let _ = worker_handle.join();
    let _ = std::fs::remove_dir_all(&temp_dir);
}
