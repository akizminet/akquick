use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;
use serde::Deserialize;

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Clone, Deserialize)]
pub struct HyprMonitor {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub focused: bool,
    pub width: i64,
    pub height: i64,
    pub refreshRate: f64,
}

fn hyprland_socket_path() -> Option<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok().or_else(|| {
        let hypr_dir = PathBuf::from(&runtime_dir).join("hypr");
        if let Ok(entries) = std::fs::read_dir(hypr_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    return Some(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        None
    })?;

    let p = PathBuf::from(runtime_dir)
        .join("hypr")
        .join(sig)
        .join(".socket.sock");

    if p.exists() {
        Some(p)
    } else {
        None
    }
}

/// Send a raw command string to Hyprland's Unix socket and receive response
pub fn query_hyprland(command: &str) -> Option<String> {
    let path = hyprland_socket_path()?;
    let mut stream = UnixStream::connect(path).ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(100))).ok()?;
    stream.set_write_timeout(Some(Duration::from_millis(100))).ok()?;

    let cmd_wire = if command.ends_with('\n') {
        command.to_string()
    } else {
        format!("{}\n", command)
    };

    stream.write_all(cmd_wire.as_bytes()).ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    Some(response)
}

/// Fetch list of monitors from Hyprland via Unix socket
pub fn get_monitors() -> Vec<HyprMonitor> {
    if let Some(json_str) = query_hyprland("j/monitors") {
        if let Ok(monitors) = serde_json::from_str::<Vec<HyprMonitor>>(&json_str) {
            return monitors;
        }
    }
    Vec::new()
}

/// Get focused monitor name from Hyprland via Unix socket
pub fn get_focused_monitor_name() -> Option<String> {
    let monitors = get_monitors();
    monitors.into_iter().find(|m| m.focused).map(|m| m.name)
}
