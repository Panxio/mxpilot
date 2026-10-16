use std::path::Path;
use std::process::Command;

fn main() {
    let dist_dir = Path::new("ui/dist");
    let index_file = dist_dir.join("index.html");

    if !index_file.exists() {
        // Intentar compilar el frontend si npm está disponible
        let npm_built = Command::new("npm")
            .args(["--prefix", "ui", "run", "build"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        // Si npm no está disponible o falló, crear un stub mínimo para permitir
        // que `cargo test --workspace` y `cargo check` compilen sin pánico del macro.
        if !npm_built && !index_file.exists() {
            let _ = std::fs::create_dir_all(dist_dir);
            let _ = std::fs::write(
                &index_file,
                "<!DOCTYPE html><html><head><title>MX Pilot</title></head><body>Stub for Cargo build</body></html>",
            );
        }
    }

    tauri_build::build();
}

