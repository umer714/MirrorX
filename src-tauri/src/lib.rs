use serde::Serialize;
use std::{
    net::{IpAddr, SocketAddr},
    process::{Child, Command},
    str::FromStr,
    sync::Mutex,
};
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AndroidDevice {
    serial: String,
    state: String,
    model: Option<String>,
    product: Option<String>,
    device: Option<String>,
}

struct ScrcpySession(Mutex<Option<Child>>);

fn valid_serial(serial: &str) -> bool {
    !serial.is_empty()
        && serial.len() <= 256
        && serial.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-'))
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


fn wireless_endpoint(host: &str, port: u16) -> Result<String, String> {
    let ip = IpAddr::from_str(host.trim())
        .map_err(|_| "Enter the phone's valid IPv4 or IPv6 address.".to_string())?;
    if port == 0 {
        return Err("Enter a valid Android wireless debugging port.".to_string());
    }
    Ok(SocketAddr::new(ip, port).to_string())
}

#[tauri::command]
fn pair_android_wireless(host: String, port: u16, pairing_code: String) -> Result<String, String> {
    let endpoint = wireless_endpoint(&host, port)?;
    let code = pairing_code.trim();
    if code.len() != 6 || !code.chars().all(|character| character.is_ascii_digit()) {
        return Err("Android pairing code must contain exactly 6 digits.".to_string());
    }
    let output = Command::new("adb")
        .args(["pair", endpoint.as_str(), code])
        .output()
        .map_err(|error| format!("ADB wireless pairing could not start: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(if stdout.is_empty() { "Android wireless pairing completed. Connect using the separate connection port shown on your phone.".to_string() } else { stdout })
}

#[tauri::command]
fn connect_android_wireless(host: String, port: u16) -> Result<String, String> {
    let endpoint = wireless_endpoint(&host, port)?;
    let output = Command::new("adb")
        .args(["connect", endpoint.as_str()])
        .output()
        .map_err(|error| format!("ADB wireless connection could not start: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    if stdout.to_ascii_lowercase().contains("failed") {
        return Err(stdout);
    }
    Ok(if stdout.is_empty() { "ADB wireless connection request completed. Scan devices to verify the connection.".to_string() } else { stdout })
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

#[tauri::command]
fn start_scrcpy(serial: String, session: State<'_, ScrcpySession>) -> Result<String, String> {
    if !valid_serial(&serial) {
        return Err("Invalid Android device identifier.".to_string());
    }

    let state = Command::new("adb")
        .args(["-s", serial.as_str(), "get-state"])
        .output()
        .map_err(|error| format!("Could not verify the device with ADB: {error}"))?;
    if !state.status.success() || String::from_utf8_lossy(&state.stdout).trim() != "device" {
        return Err("This device is not authorized or connected. Check the phone and scan again.".to_string());
    }

    let mut active = session.0.lock().map_err(|_| "Could not access the scrcpy session state.".to_string())?;
    if let Some(child) = active.as_mut() {
        match child.try_wait() {
            Ok(None) => return Err("A MirrorX scrcpy session is already running. Stop it before starting another.".to_string()),
            Ok(Some(_)) => { *active = None; }
            Err(error) => return Err(format!("Could not inspect the existing scrcpy session: {error}")),
        }
    }

    let child = Command::new("scrcpy")
        .args(["--serial", serial.as_str()])
        .spawn()
        .map_err(|error| format!("scrcpy could not be started. Install scrcpy and make sure it is available in PATH. Details: {error}"))?;
    *active = Some(child);
    Ok("scrcpy launched. Its mirroring window should open separately.".to_string())
}

#[tauri::command]
fn stop_scrcpy(session: State<'_, ScrcpySession>) -> Result<String, String> {
    let mut active = session.0.lock().map_err(|_| "Could not access the scrcpy session state.".to_string())?;
    if let Some(mut child) = active.take() {
        match child.try_wait() {
            Ok(Some(_)) => Ok("The scrcpy process has already exited.".to_string()),
            Ok(None) => {
                child.kill().map_err(|error| format!("Could not stop scrcpy: {error}"))?;
                child.wait().map_err(|error| format!("Could not finish stopping scrcpy: {error}"))?;
                Ok("scrcpy stopped.".to_string())
            }
            Err(error) => Err(format!("Could not inspect scrcpy before stopping: {error}")),
        }
    } else {
        Ok("No MirrorX scrcpy session is running.".to_string())
    }
}

#[tauri::command]
fn scrcpy_status(session: State<'_, ScrcpySession>) -> Result<bool, String> {
    let mut active = session.0.lock().map_err(|_| "Could not access the scrcpy session state.".to_string())?;
    if let Some(child) = active.as_mut() {
        match child.try_wait() {
            Ok(None) => Ok(true),
            Ok(Some(_)) => { *active = None; Ok(false) }
            Err(error) => Err(format!("Could not inspect scrcpy: {error}")),
        }
    } else {
        Ok(false)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(ScrcpySession(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            list_android_devices,
            pair_android_wireless,
            connect_android_wireless,
            start_scrcpy,
            stop_scrcpy,
            scrcpy_status
        ])
        .build(tauri::generate_context!())
        .expect("error while building MirrorX");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let session = app_handle.state::<ScrcpySession>();
            if let Ok(mut active) = session.0.lock() {
                if let Some(mut child) = active.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            };
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{parse_adb_devices, valid_serial, wireless_endpoint};

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

    #[test]
    fn accepts_common_adb_serial_formats() {
        assert!(valid_serial("ABC123"));
        assert!(valid_serial("192.168.1.20:5555"));
        assert!(valid_serial("emulator-5554"));
    }

    #[test]
    fn validates_wireless_ip_and_port() {
        assert_eq!(wireless_endpoint("192.168.1.12", 37123).unwrap(), "192.168.1.12:37123");
        assert!(wireless_endpoint("not-an-ip", 37123).is_err());
        assert!(wireless_endpoint("192.168.1.12", 0).is_err());
    }

    #[test]
    fn rejects_empty_or_shell_metacharacter_serials() {
        assert!(!valid_serial(""));
        assert!(!valid_serial("phone; shutdown"));
        assert!(!valid_serial("phone && whoami"));
    }
}
