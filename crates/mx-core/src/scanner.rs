use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DetectedHidDevice {
    pub path: PathBuf,
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub is_logitech: bool,
    pub is_mx_vertical: bool,
}

pub fn scan_hidraw_devices() -> Vec<DetectedHidDevice> {
    let mut devices = Vec::new();
    let sys_hidraw = PathBuf::from("/sys/class/hidraw");

    if let Ok(entries) = fs::read_dir(sys_hidraw) {
        for entry in entries.flatten() {
            let hidraw_name = entry.file_name();
            let uevent_path = entry.path().join("device/uevent");
            let dev_path = PathBuf::from("/dev").join(&hidraw_name);

            if let Ok(content) = fs::read_to_string(&uevent_path) {
                let mut name = String::from("Desconocido");
                let mut vendor_id = 0u16;
                let mut product_id = 0u16;

                for line in content.lines() {
                    if let Some(val) = line.strip_prefix("HID_NAME=") {
                        name = val.trim().to_string();
                    } else if let Some(val) = line.strip_prefix("HID_ID=") {
                        let parts: Vec<&str> = val.split(':').collect();
                        if parts.len() >= 3 {
                            vendor_id = u16::from_str_radix(parts[1], 16).unwrap_or(0);
                            product_id = u16::from_str_radix(parts[2], 16).unwrap_or(0);
                        }
                    }
                }

                let is_logitech = vendor_id == 0x046d;
                let is_mx_vertical = is_logitech && (product_id == 0xb020 || name.to_lowercase().contains("mx vertical"));

                devices.push(DetectedHidDevice {
                    path: dev_path,
                    name,
                    vendor_id,
                    product_id,
                    is_logitech,
                    is_mx_vertical,
                });
            }
        }
    }

    devices
}
