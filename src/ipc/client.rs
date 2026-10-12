use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use crate::ipc::protocol::{socket_path, IpcCommand};

pub fn send_command(cmd: &IpcCommand) -> Result<String, std::io::Error> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)?;
    stream.set_read_timeout(Some(Duration::from_millis(200)))?;
    stream.set_write_timeout(Some(Duration::from_millis(200)))?;

    let wire = cmd.to_wire_string();
    stream.write_all(wire.as_bytes())?;
    let _ = stream.shutdown(Shutdown::Write);

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

pub fn try_query(cmd: &IpcCommand) -> Option<String> {
    send_command(cmd).ok()
}

pub fn try_handle_cli(args: &[String]) -> Option<glib::ExitCode> {
    let is_daemon = args.iter().any(|a| a == "--daemon" || a == "-d");
    if is_daemon {
        return None;
    }

    let cmd = if args.iter().any(|a| a == "--status" || a == "-s" || a == "--status-full") {
        Some(IpcCommand::StatusFull)
    } else if args.iter().any(|a| a == "--status-json") {
        Some(IpcCommand::StatusJson)
    } else if args.iter().any(|a| a == "--status-icons") {
        Some(IpcCommand::StatusIcons)
    } else if args.iter().any(|a| a == "--status-wifi") {
        Some(IpcCommand::StatusWifi)
    } else if args.iter().any(|a| a == "--status-bt" || a == "--status-bluetooth") {
        Some(IpcCommand::StatusBt)
    } else if args.iter().any(|a| a == "--status-net" || a == "--status-speed") {
        Some(IpcCommand::StatusNet)
    } else if args.iter().any(|a| a == "--status-audio") {
        Some(IpcCommand::StatusAudio)
    } else if args.iter().any(|a| a == "--status-mic") {
        Some(IpcCommand::StatusMic)
    } else if args.iter().any(|a| a == "--toggle" || a == "-t") {
        Some(IpcCommand::Toggle)
    } else if let Some(pos) = args.iter().position(|a| a == "--page" || a == "-p") {
        let page = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("main");
        Some(IpcCommand::OpenPage(page.to_string()))
    } else {
        None
    };

    if let Some(command) = cmd {
        if let Some(resp) = try_query(&command) {
            let trimmed = resp.trim();
            if !trimmed.is_empty() {
                println!("{}", trimmed);
            }
            return Some(glib::ExitCode::SUCCESS);
        }
    }

    None
}
