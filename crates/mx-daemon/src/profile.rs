use crate::ipc::protocol::{ActionPayload, TOP_BUTTON_CID};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const PROFILE_VERSION: u32 = 1;
pub const PROFILE_DIR_NAME: &str = "mx-options";
pub const PROFILE_FILE_NAME: &str = "profile.json";

/// Estructura de serialización del perfil de usuario persistente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    pub version: u32,
    pub mapping: HashMap<u16, ActionPayload>,
}

impl Default for UserProfile {
    fn default() -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(
            TOP_BUTTON_CID,
            ActionPayload::SingleKey {
                key: "KEY_LEFTMETA".to_string(),
            },
        );
        Self {
            version: PROFILE_VERSION,
            mapping,
        }
    }
}

/// Obtiene el directorio base de configuración según el estándar XDG.
pub fn default_profile_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join(PROFILE_DIR_NAME);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join(PROFILE_DIR_NAME);
    }
    PathBuf::from("/tmp").join(PROFILE_DIR_NAME)
}

/// Obtiene la ruta canónica del archivo profile.json.
pub fn default_profile_path() -> PathBuf {
    default_profile_dir().join(PROFILE_FILE_NAME)
}

/// Carga el perfil desde la ruta por defecto del sistema.
/// Si el archivo no existe o está corrupto, retorna el mapeo seguro por defecto sin fallar (Pilar 4).
pub fn load_profile() -> HashMap<u16, ActionPayload> {
    let path = default_profile_path();
    load_profile_from_path(&path).unwrap_or_else(|e| {
        log::info!("Perfil en {}: {}. Usando mapeo por defecto.", path.display(), e);
        UserProfile::default().mapping
    })
}

/// Persiste el mapa de botones atómicamente en la ruta por defecto.
pub fn save_profile(mapping: &HashMap<u16, ActionPayload>) -> Result<(), String> {
    let path = default_profile_path();
    save_profile_to_path(&path, mapping)
}

/// Carga y deserializa el perfil desde una ruta específica.
pub fn load_profile_from_path(path: &Path) -> Result<HashMap<u16, ActionPayload>, String> {
    if !path.exists() {
        return Err("Archivo de perfil no existe".into());
    }
    let data = fs::read_to_string(path).map_err(|e| format!("Error leyendo perfil: {}", e))?;
    let profile: UserProfile =
        serde_json::from_str(&data).map_err(|e| format!("Perfil malformado o incompatible: {}", e))?;
    Ok(profile.mapping)
}

/// Guarda atómicamente el perfil mediante escritura en archivo temporal + fsync + rename POSIX (Invariante 2).
pub fn save_profile_to_path(path: &Path, mapping: &HashMap<u16, ActionPayload>) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Ruta de perfil inválida sin directorio padre".to_string())?;

    fs::create_dir_all(parent)
        .map_err(|e| format!("No se pudo crear directorio {}: {}", parent.display(), e))?;

    let profile = UserProfile {
        version: PROFILE_VERSION,
        mapping: mapping.clone(),
    };
    let json_bytes =
        serde_json::to_vec_pretty(&profile).map_err(|e| format!("Error serializando perfil: {}", e))?;

    let tmp_path = parent.join(format!(".profile.json.tmp.{}", std::process::id()));
    {
        let mut f = File::create(&tmp_path)
            .map_err(|e| format!("Error creando archivo temporal {}: {}", tmp_path.display(), e))?;
        f.write_all(&json_bytes)
            .map_err(|e| format!("Error escribiendo archivo temporal: {}", e))?;
        f.flush()
            .map_err(|e| format!("Error en flush de archivo temporal: {}", e))?;
        f.sync_all()
            .map_err(|e| format!("Error en sync_all de archivo temporal: {}", e))?;
    }

    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("Error renombrando perfil atómico: {}", e));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let def = UserProfile::default();
        assert_eq!(def.version, PROFILE_VERSION);
        assert_eq!(def.mapping.len(), 1);
        assert_eq!(
            def.mapping.get(&TOP_BUTTON_CID),
            Some(&ActionPayload::SingleKey {
                key: "KEY_LEFTMETA".to_string()
            })
        );
    }

    #[test]
    fn test_save_load_roundtrip() {
        let tmp_dir = std::env::temp_dir().join(format!("mx_profile_test_{}", std::process::id()));
        let path = tmp_dir.join("profile.json");

        let mut mapping = HashMap::new();
        mapping.insert(
            TOP_BUTTON_CID,
            ActionPayload::SingleKey {
                key: "KEY_C".to_string(),
            },
        );
        mapping.insert(
            0x0056,
            ActionPayload::KeyCombo {
                modifiers: vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string()],
                key: "KEY_RIGHT".to_string(),
            },
        );

        save_profile_to_path(&path, &mapping).expect("Guardado debe ser exitoso");
        let loaded = load_profile_from_path(&path).expect("Carga debe ser exitosa");

        assert_eq!(loaded, mapping);

        // Limpieza
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_corrupt_file_recovery() {
        let tmp_dir = std::env::temp_dir().join(format!("mx_corrupt_test_{}", std::process::id()));
        let path = tmp_dir.join("profile.json");
        fs::create_dir_all(&tmp_dir).unwrap();
        fs::write(&path, b"{\"incompleto\": true").unwrap();

        let res = load_profile_from_path(&path);
        assert!(res.is_err(), "Debe detectar archivo corrupto");

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_atomic_overwrite() {
        let tmp_dir = std::env::temp_dir().join(format!("mx_overwrite_test_{}", std::process::id()));
        let path = tmp_dir.join("profile.json");

        let mut m1 = HashMap::new();
        m1.insert(
            TOP_BUTTON_CID,
            ActionPayload::SingleKey {
                key: "KEY_A".to_string(),
            },
        );
        save_profile_to_path(&path, &m1).unwrap();

        let mut m2 = HashMap::new();
        m2.insert(
            TOP_BUTTON_CID,
            ActionPayload::SingleKey {
                key: "KEY_B".to_string(),
            },
        );
        save_profile_to_path(&path, &m2).unwrap();

        let loaded = load_profile_from_path(&path).unwrap();
        assert_eq!(loaded, m2);

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
