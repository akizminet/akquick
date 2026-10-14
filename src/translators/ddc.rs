use std::fs::{self, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

const I2C_SLAVE: libc::c_ulong = 0x0703;
const DDC_I2C_ADDR: libc::c_ulong = 0x37;
const VCP_BRIGHTNESS: u8 = 0x10;

/// Resolves a DRM connector name (e.g. "DP-1", "HDMI-A-2") to its corresponding `/dev/i2c-N` path
pub fn find_i2c_device_for_connector(connector: &str) -> Option<PathBuf> {
    let drm_base = Path::new("/sys/class/drm");
    if let Ok(entries) = fs::read_dir(drm_base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Matches "card0-DP-1", "card1-DP-1", etc.
            if let Some((_, conn_name)) = name.split_once('-') {
                if conn_name == connector {
                    let ddc_path = entry.path().join("ddc");
                    if let Ok(target) = fs::canonicalize(&ddc_path) {
                        if let Some(i2c_name) = target.file_name() {
                            let dev_path = PathBuf::from("/dev").join(i2c_name);
                            if dev_path.exists() {
                                return Some(dev_path);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Reads monitor brightness (0..100) via DDC/CI over I2C
pub fn read_ddc_brightness(dev_path: &Path) -> Option<u32> {
    use std::io::{Read, Write};

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(dev_path)
        .ok()?;

    let fd = file.as_raw_fd();
    unsafe {
        if libc::ioctl(fd, I2C_SLAVE, DDC_I2C_ADDR) < 0 {
            return None;
        }
    }

    // DDC/CI Get VCP request: dest 0x51, length 0x82, cmd 0x01, opcode 0x10, checksum
    let chk = 0x6e ^ 0x51 ^ 0x82 ^ 0x01 ^ VCP_BRIGHTNESS;
    let req = [0x51, 0x82, 0x01, VCP_BRIGHTNESS, chk];

    let mut mut_file = file;
    mut_file.write_all(&req).ok()?;

    // Allow the monitor hardware 40ms to process VCP request
    thread::sleep(Duration::from_millis(40));

    let mut buf = [0u8; 12];
    let n = mut_file.read(&mut buf).ok()?;
    if n >= 11 && buf[2] == 0x02 && buf[3] == 0x00 {
        let max_val = ((buf[6] as u32) << 8) | (buf[7] as u32);
        let cur_val = ((buf[8] as u32) << 8) | (buf[9] as u32);
        if max_val > 0 {
            return Some(((cur_val as f64 / max_val as f64) * 100.0).round() as u32);
        }
    }

    None
}

/// Sets monitor brightness (0..100) via DDC/CI over I2C
pub fn write_ddc_brightness(dev_path: &Path, percent: u32) -> Result<(), std::io::Error> {
    use std::io::Write;

    let file = OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(dev_path)?;

    let fd = file.as_raw_fd();
    unsafe {
        if libc::ioctl(fd, I2C_SLAVE, DDC_I2C_ADDR) < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    let val = percent.min(100) as u16;
    let val_hi = (val >> 8) as u8;
    let val_lo = (val & 0xff) as u8;
    let chk = 0x6e ^ 0x51 ^ 0x84 ^ 0x03 ^ VCP_BRIGHTNESS ^ val_hi ^ val_lo;
    let packet = [0x51, 0x84, 0x03, VCP_BRIGHTNESS, val_hi, val_lo, chk];

    let mut mut_file = file;
    mut_file.write_all(&packet)?;
    Ok(())
}

/// Get brightness for a given monitor name (trying DDC/CI first, then sysfs /sys/class/backlight)
pub fn get_monitor_brightness(connector: &str) -> Option<u32> {
    if let Some(i2c_dev) = find_i2c_device_for_connector(connector) {
        if let Some(pct) = read_ddc_brightness(&i2c_dev) {
            return Some(pct);
        }
    }

    // Fallback: Linux kernel sysfs /sys/class/backlight (e.g. laptop internal screens)
    crate::translators::sysfs::read_backlight_percent()
}

struct BrightnessCmd {
    connector: String,
    percent: u32,
}

static DDC_CMD_TX: LazyLock<Sender<BrightnessCmd>> = LazyLock::new(|| {
    let (tx, rx) = channel::<BrightnessCmd>();
    thread::Builder::new()
        .name("ddc-worker".into())
        .spawn(move || {
            while let Ok(cmd) = rx.recv() {
                let mut latest_map = std::collections::HashMap::new();
                latest_map.insert(cmd.connector, cmd.percent);

                // Drain queue to coalesce rapid slider events
                while let Ok(next) = rx.try_recv() {
                    latest_map.insert(next.connector, next.percent);
                }

                for (connector, percent) in latest_map {
                    if let Some(i2c_dev) = find_i2c_device_for_connector(&connector) {
                        let _ = write_ddc_brightness(&i2c_dev, percent);
                    } else {
                        // Fallback to sysfs/logind backlight
                        crate::translators::sysfs::set_backlight_percent(percent);
                    }
                }
            }
        })
        .expect("Failed to start ddc worker thread");
    tx
});

/// Non-blocking, coalesced setter for display brightness
pub fn set_monitor_brightness(connector: &str, percent: u32) {
    let _ = DDC_CMD_TX.send(BrightnessCmd {
        connector: connector.to_string(),
        percent,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_i2c_device() {
        if Path::new("/sys/class/drm").exists() {
            let dp = find_i2c_device_for_connector("DP-1");
            if let Some(ref path) = dp {
                assert!(path.exists());
            }
        }
    }
}
