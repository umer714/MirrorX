use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AndroidDevice {
    serial: String,
    state: String,
    model: Option<String>,
    product: Option<String>,
    device: Option<String>,
}

fn parse_adb_devices(output: &str) -> Vec<AndroidDevice> {
    output.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices attached") { return None; }
        let mut fields = line.split_whitespace();
        let serial = fields.next()?;
        let state = fields.next()?;
        let mut model = None;
        let mut product = None;
        let mut device = None;
        for field in fields {
            if let Some((key, value)) = field.split_once(':') {
                match key {
                    "model" => model = Some(value.replace('_', " ")),
                    "product" => product = Some(value.to_string()),
                    "device" => device = Some(value.to_string()),
                    _ => {}
                }
            }
        }
        Some(AndroidDevice {
            serial: serial.to_string(), state: state.to_string(),
            model, product, device,
        })
    }).collect()
}

#[tauri::command]
fn list_android_devices() -> Result<Vec<AndroidDevice>, String> {
    let output = Command::new("adb").args(["devices", "-l"]).output().map_err(|error| {
        format!("ADB could not be started. Install Android Platform Tools and make sure adb is available in PATH. Details: {error}")
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("ADB exited with status {}.", output.status)
        } else { format!("ADB error: {stderr}") });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_adb_devices(&stdout))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_android_devices])
        .run(tauri::generate_context!())
        .expect("error while running MirrorX");
}

#[cfg(test)]
mod tests {
    use super::parse_adb_devices;

    #[test]
    fn parses_authorized_and_unauthorized_devices() {
        let output = "List of devices attached\nABC123 device product:pixel device:pixel model:Pixel_7\nXYZ789 unauthorized usb:1-2\n";
        let devices = parse_adb_devices(output);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].serial, "ABC123");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[0].model.as_deref(), Some("Pixel 7"));
        assert_eq!(devices[1].state, "unauthorized");
    }

    #[test]
    fn empty_adb_output_returns_no_devices() {
        assert!(parse_adb_devices("List of devices attached\n\n").is_empty());
    }
}
