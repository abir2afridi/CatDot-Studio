//! Device adapters (Part F7, Phase 4 slice): adb device listing, signed-APK
//! install, and a bounded logcat dump. Processes spawn ONLY here — always via
//! the detected toolchain `adb`, with hard timeouts and bounded output.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};
use crate::tools::build_tools::run_timeout;
use crate::tools::detect::detect_toolchain;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DeviceInfo {
    pub serial: String,
    /// adb state: device | unauthorized | offline | ...
    pub state: String,
    pub model: Option<String>,
}

fn adb_err(msg: impl Into<String>, user: impl Into<String>) -> AppError {
    AppError::new(ErrorCode::DeviceOffline, msg, user)
}

/// Locate adb from the toolchain (PATH fallback included by detect).
pub fn adb_path() -> Result<PathBuf, AppError> {
    let t = detect_toolchain();
    let adb = t.adb.path;
    match adb {
        Some(p) if Path::new(&p).is_file() => Ok(PathBuf::from(p)),
        _ => Err(AppError::new(
            ErrorCode::ToolNotFound,
            "adb not found",
            "What happened: the Android platform-tools (adb) are not installed. What to do: install them from the Android SDK, then run System check.",
        )),
    }
}

/// Parse `adb devices -l` output (headers/comments ignored).
pub fn parse_devices(out: &str) -> Vec<DeviceInfo> {
    let mut list = Vec::new();
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(serial) = parts.next() else { continue };
        let Some(state) = parts.next() else { continue };
        let model = line
            .split_whitespace()
            .find_map(|tok| tok.strip_prefix("model:"))
            .map(|m| m.replace('_', " "));
        list.push(DeviceInfo {
            serial: serial.to_string(),
            state: state.to_string(),
            model,
        });
    }
    list
}

/// List attached devices/emulators (`adb devices -l`).
pub fn list_devices() -> Result<Vec<DeviceInfo>, AppError> {
    let adb = adb_path()?;
    let run = run_timeout(&adb, &["devices", "-l"], None, Duration::from_secs(10))
        .ok_or_else(|| adb_err("adb devices timed out", "What happened: adb did not answer. What to do: restart the adb server (`adb kill-server`) and retry."))?;
    if run.code != 0 {
        return Err(adb_err(
            format!("adb devices failed: {}", run.out.trim()),
            "What happened: adb could not list devices. What to do: check that platform-tools are properly installed in System check.",
        ));
    }
    Ok(parse_devices(&run.out))
}

fn require_device() -> Result<(PathBuf, DeviceInfo), AppError> {
    let adb = adb_path()?;
    let devices = list_devices()?;
    if devices.is_empty() {
        return Err(AppError::new(
            ErrorCode::DeviceOffline,
            "no device connected",
            "What happened: no device or emulator is connected. What to do: start an emulator or plug in a phone with USB debugging enabled, then check `adb devices`.",
        ));
    }
    if let Some(unauth) = devices.iter().find(|d| d.state == "unauthorized") {
        return Err(AppError::new(
            ErrorCode::DeviceUnauthorized,
            format!("device {} is unauthorized", unauth.serial),
            "What happened: the device has not authorized this computer. What to do: accept the USB-debugging prompt on the device screen.",
        ));
    }
    let dev = devices
        .into_iter()
        .find(|d| d.state == "device")
        .ok_or_else(|| {
            adb_err(
                "no ready device",
                "What happened: a device is connected but not ready. What to do: wait for it to finish booting and retry.",
            )
        })?;
    Ok((adb, dev))
}

/// Install a signed APK (`adb install -r`). Returns the device serial on success.
pub fn install_apk(apk: &Path) -> Result<String, AppError> {
    if !apk.is_file() {
        return Err(adb_err(
            "signed apk missing",
            "What happened: there is no signed APK to install yet. What to do: run Build · align · sign first.",
        ));
    }
    let (adb, dev) = require_device()?;
    let apk_str = apk.to_string_lossy().into_owned();
    let run = run_timeout(
        &adb,
        &["-s", &dev.serial, "install", "-r", &apk_str],
        None,
        Duration::from_secs(180),
    )
    .ok_or_else(|| {
        adb_err(
            "adb install timed out",
            "What happened: the install took too long and was stopped. What to do: check the device screen for prompts and retry.",
        )
    })?;
    let out = run.out.trim().to_string();
    if run.code == 0 && out.contains("Success") {
        return Ok(dev.serial);
    }
    let code = if out.contains("INSTALL_FAILED_UPDATE_INCOMPATIBLE")
        || out.contains("signatures do not match")
    {
        ErrorCode::InstallIncompatibleUpdate
    } else if out.contains("INSTALL_FAILED_VERSION_DOWNGRADE") {
        ErrorCode::InstallDowngrade
    } else if out.contains("INSTALL_FAILED_NO_MATCHING_ABIS") {
        ErrorCode::InstallNoMatchingAbis
    } else if out.contains("INSTALL_FAILED_OLDER_SDK") {
        ErrorCode::InstallOldSdk
    } else {
        ErrorCode::InstallFailed
    };
    Err(AppError::new(
        code,
        format!("install failed: {out}"),
        "What happened: the APK could not be installed on the device. What to do: read the error above — uninstalling the old app often fixes signature mismatch.",
    ))
}

/// Bounded `adb logcat -d -t <lines>` dump (recent entries, newest last).
pub fn logcat_dump(lines: u32) -> Result<String, AppError> {
    let (adb, dev) = require_device()?;
    let n = lines.clamp(1, 1000).to_string();
    let run = run_timeout(
        &adb,
        &["-s", &dev.serial, "logcat", "-d", "-t", &n],
        None,
        Duration::from_secs(20),
    )
    .ok_or_else(|| {
        adb_err(
            "logcat timed out",
            "What happened: reading device logs took too long. What to do: retry with fewer lines.",
        )
    })?;
    if run.code != 0 && run.out.trim().is_empty() {
        return Err(adb_err(
            "logcat failed",
            "What happened: the device log could not be read. What to do: check the device connection and retry.",
        ));
    }
    Ok(run.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_devices_l_output() {
        let out = "List of devices attached\n\
                   emulator-5554          device product:sdk_gphone64_x86_64 model:sdk_gphone64_x86_64 device:emu64x transport_id:1\n\
                   ABCD1234               unauthorized transport_id:2\n\
                   OFFLINE1               offline\n\n";
        let list = parse_devices(out);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].serial, "emulator-5554");
        assert_eq!(list[0].state, "device");
        assert_eq!(list[0].model.as_deref(), Some("sdk gphone64 x86 64"));
        assert_eq!(list[1].state, "unauthorized");
        assert_eq!(list[2].state, "offline");
        assert!(list[2].model.is_none());
    }

    #[test]
    fn parses_empty_output() {
        assert!(parse_devices("List of devices attached\n\n").is_empty());
        assert!(parse_devices("").is_empty());
        assert!(parse_devices(
            "* daemon not running; starting now at tcp:5037\nList of devices attached\n"
        )
        .is_empty());
    }
}
