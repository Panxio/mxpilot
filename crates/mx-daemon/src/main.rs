use mx_daemon::ipc::IpcServer;
use mx_daemon::lock::SingletonLock;
use mx_daemon::state::DaemonState;
use mx_daemon::workers::{run_hid_worker, run_uinput_worker, DaemonCommand, InputEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, RwLock};
use std::thread;

static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

extern "C" fn sig_handler(_sig: libc::c_int) {
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("=== MX OPTIONS LINUX DAEMON (mx-daemon) ===");
    log::info!("Iniciando servicio de segundo plano para Logitech MX Vertical...");

    // Instalar manejadores de señales Unix (SIGINT, SIGTERM, SIGPIPE)
    unsafe {
        libc::signal(libc::SIGINT, sig_handler as *const () as usize);
        libc::signal(libc::SIGTERM, sig_handler as *const () as usize);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

    // Adquirir bloqueo singleton exclusivo con flock
    let _singleton_lock = match SingletonLock::acquire() {
        Ok(lock) => lock,
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    let state = Arc::new(RwLock::new(DaemonState::new()));

    // Canal uinput acotado (persiste durante toda la vida de main)
    let (input_tx, input_rx) = sync_channel::<InputEvent>(64);

    // Canal de comandos entre IPC y actor hid_worker
    let (cmd_tx, cmd_rx) = sync_channel::<DaemonCommand>(16);

    // Servidor IPC local Unix Domain Socket
    let ipc_server = match IpcServer::new(state.clone(), cmd_tx.clone(), shutdown.clone()) {
        Ok(srv) => srv,
        Err(e) => {
            log::error!("Fallo crítico iniciando servidor IPC: {}", e);
            std::process::exit(1);
        }
    };

    // 1. Lanzar worker uinput
    let shutdown_uinput = shutdown.clone();
    let uinput_handle = thread::spawn(move || {
        run_uinput_worker(input_rx, shutdown_uinput);
    });

    // 2. Lanzar worker HID++ (Actor de hardware de dueño único)
    let shutdown_hid = shutdown.clone();
    let state_hid = state.clone();
    let hid_handle = thread::spawn(move || {
        run_hid_worker(cmd_rx, input_tx, state_hid, shutdown_hid);
    });

    // 3. Hilo del servidor IPC
    let shutdown_ipc = shutdown.clone();
    let ipc_server_arc = Arc::new(ipc_server);
    let ipc_server_clone = ipc_server_arc.clone();
    let ipc_handle = thread::spawn(move || {
        ipc_server_clone.run();
    });

    log::info!("Todos los hilos trabajadores inicializados. Escuchando eventos...");

    // Bucle de supervisión de señales en el hilo principal
    while !SHUTDOWN_FLAG.load(Ordering::Relaxed) {
        thread::sleep(std::time::Duration::from_millis(100));
    }

    log::info!("Señal de terminación recibida. Iniciando parada limpia del daemon (presupuesto <= 2s)...");
    shutdown.store(true, Ordering::SeqCst);
    shutdown_ipc.store(true, Ordering::SeqCst);

    let _ = cmd_tx.send(DaemonCommand::Shutdown);

    // Esperar terminación ordenada de los hilos
    let _ = ipc_handle.join();
    let _ = hid_handle.join();
    let _ = uinput_handle.join();

    log::info!("mx-daemon finalizado limpiamente. Cero botones muertos.");
}
