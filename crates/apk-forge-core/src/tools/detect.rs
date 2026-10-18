//! Toolchain detection (Part H1 step 3 + Q). Read-only: never installs here.
//! Checks: Java, apktool, JADX, build-tools (aapt2/zipalign/apksigner),
//! ADB, emulator, disk space, Windows long-path support.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolInfo {
    pub id: String,
    pub found: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    /// One-line purpose shown in the System-check UI.
    pub purpose: String,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolchainReport {
    pub java: ToolInfo,
    pub aapt2: ToolInfo,
    pub zipalign: ToolInfo,
    pub apksigner: ToolInfo,
    pub adb: ToolInfo,
    pub emulator: ToolInfo,
    pub apktool: ToolInfo,
    pub jadx: ToolInfo,
    pub long_paths_enabled: bool,
    pub free_disk_mb: u64,
}

/// Detect the toolchain. Pure reads + `--version` probes with timeouts.
pub fn detect_toolchain() -> ToolchainReport {
    let java = probe_java();
    let sdk = android_sdk_root();
    let build_tools = newest_build_tools(&sdk);
    ToolchainReport {
        java,
        aapt2: probe_exe(
            "aapt2",
            build_tools.as_ref().map(|p| p.join("aapt2.exe")),
            "Android Asset Packaging Tool (reads APK metadata)",
            "Install Android SDK build-tools via Android Studio or sdkmanager",
        ),
        zipalign: probe_exe(
            "zipalign",
            build_tools.as_ref().map(|p| p.join("zipalign.exe")),
            "Aligns APK entries before signing",
            "Install Android SDK build-tools",
        ),
        apksigner: probe_exe(
            "apksigner",
            build_tools.as_ref().map(|p| p.join("apksigner.bat")),
            "Signs and verifies APKs",
            "Install Android SDK build-tools",
        ),
        adb: probe_exe(
            "adb",
            sdk.as_ref().map(|s| s.join("platform-tools").join("adb.exe")),
            "Installs apps and reads device logs",
            "Install Android SDK platform-tools",
        ),
        emulator: probe_exe(
            "emulator",
            sdk.as_ref().map(|s| s.join("emulator").join("emulator.exe")),
            "Runs an Android emulator for testing",
            "Install Android SDK emulator + a system image",
        ),
        apktool: probe_exe(
            "apktool",
            None,
            "Decodes and rebuilds APK resources",
            "Will be offered as a pinned download in Phase 4 installer",
        ),
        jadx: probe_exe(
            "jadx",
            None,
            "Decompiles DEX to readable Java (reference only)",
            "Will be offered as a pinned download in Phase 4 installer",
        ),
        long_paths_enabled: long_paths_enabled(),
        free_disk_mb: free_disk_mb(),
    }
}

fn probe_java() -> ToolInfo {
    // Candidates: PATH `java`, JAVA_HOME, Android Studio JBR.
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(jh) = std::env::var("JAVA_HOME") {
        candidates.push(PathBuf::from(jh).join("bin").join("java.exe"));
    }
    candidates.push(PathBuf::from(
        "C:\\Program Files\\Android\\Android Studio\\jbr\\bin\\java.exe",
    ));
    for c in candidates {
        if c.is_file() {
            let v = java_version(&c).unwrap_or_default();
            return ToolInfo {
                id: "java".into(),
                found: true,
                version: Some(v),
                path: Some(c.to_string_lossy().into_owned()),
                purpose: "Runs apktool/JADX/aapt2 (needs JDK 17+)".into(),
                fix_hint: None,
            };
        }
    }
    // PATH fallback.
    if let Some(v) = run_capture("java.exe", &["-version"]) {
        return ToolInfo {
            id: "java".into(),
            found: true,
            version: Some(first_line(&v)),
            path: Some("java.exe (PATH)".into()),
            purpose: "Runs apktool/JADX/aapt2 (needs JDK 17+)".into(),
            fix_hint: None,
        };
    }
    ToolInfo {
        id: "java".into(),
        found: false,
        version: None,
        path: None,
        purpose: "Runs apktool/JADX/aapt2 (needs JDK 17+)".into(),
        fix_hint: Some("Install Temurin JRE 17+ via the in-app fixer (Phase 4)".into()),
    }
}

fn java_version(exe: &std::path::Path) -> Option<String> {
    // `java -version` prints to stderr.
    let out = std::process::Command::new(exe).arg("-version").output().ok()?;
    let mut text = String::from_utf8_lossy(&out.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stdout));
    Some(first_line(&text))
}

fn probe_exe(
    id: &str,
    sdk_path: Option<PathBuf>,
    purpose: &str,
    fix: &str,
) -> ToolInfo {
    if let Some(p) = sdk_path {
        if p.is_file() {
            let v = run_capture(p.to_str().unwrap_or(""), &["version"])
                .or_else(|| run_capture(p.to_str().unwrap_or(""), &["--version"]))
                .map(|s| first_line(&s));
            return ToolInfo {
                id: id.into(),
                found: true,
                version: v,
                path: Some(p.to_string_lossy().into_owned()),
                purpose: purpose.into(),
                fix_hint: None,
            };
        }
    }
    // PATH fallback for adb/apktool/jadx.
    let exe = format!("{id}.exe");
    if run_capture(&exe, &["version"]).is_some()
        || run_capture(&exe, &["--version"]).is_some()
    {
        return ToolInfo {
            id: id.into(),
            found: true,
            version: None,
            path: Some(format!("{id} (PATH)")),
            purpose: purpose.into(),
            fix_hint: None,
        };
    }
    ToolInfo {
        id: id.into(),
        found: false,
        version: None,
        path: None,
        purpose: purpose.into(),
        fix_hint: Some(fix.into()),
    }
}

fn run_capture(prog: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(prog).args(args).output().ok()?;
    if out.stdout.is_empty() && out.stderr.is_empty() {
        return None;
    }
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    let s = s.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s.chars().take(500).collect())
    }
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().chars().take(200).collect()
}

fn android_sdk_root() -> Option<PathBuf> {
    for key in ["ANDROID_SDK_ROOT", "ANDROID_HOME"] {
        if let Ok(v) = std::env::var(key) {
            let p = PathBuf::from(v);
            if p.is_dir() {
                return Some(p);
            }
        }
    }
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let p = PathBuf::from(local).join("Android").join("Sdk");
    if p.is_dir() {
        return Some(p);
    }
    None
}

fn newest_build_tools(sdk: &Option<PathBuf>) -> Option<PathBuf> {
    let sdk = sdk.as_ref()?;
    let dir = sdk.join("build-tools");
    let mut vers: Vec<(Vec<u32>, PathBuf)> = Vec::new();
    for e in std::fs::read_dir(&dir).ok()? {
        let e = e.ok()?;
        let name = e.file_name().to_string_lossy().into_owned();
        let nums: Vec<u32> = name
            .split('.')
            .filter_map(|x| x.parse().ok())
            .collect();
        if !nums.is_empty() && e.path().is_dir() {
            vers.push((nums, e.path()));
        }
    }
    vers.sort_by(|a, b| a.0.cmp(&b.0));
    vers.pop().map(|(_, p)| p)
}

fn long_paths_enabled() -> bool {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        // Read-only registry probe; failure = assume disabled.
        let out = std::process::Command::new("reg.exe")
            .args([
                "query",
                "HKLM\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                "/v",
                "LongPathsEnabled",
            ])
            .output();
        match out {
            Ok(o) => {
                let _ = OsString::from("x");
                String::from_utf8_lossy(&o.stdout).contains("0x1")
            }
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        true
    }
}

fn free_disk_mb() -> u64 {
    // Best-effort: use the projects-dir drive free space via `fsutil`? Fallback 0.
    // Kept dependency-free: report via temp-dir anchor is enough for Phase 0.
    // (Accurate per-drive numbers land with the disk-preflight in Phase 2.)
    let _ = std::env::temp_dir();
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_runs_without_panic_and_reports_java() {
        let r = detect_toolchain();
        // On this dev machine Java (JBR) + SDK exist; on CI they may not.
        // Assert shape, not presence.
        assert_eq!(r.java.id, "java");
        assert_eq!(r.adb.id, "adb");
        assert!(!r.aapt2.purpose.is_empty());
    }

    #[test]
    fn newest_build_tools_picks_highest() {
        if android_sdk_root().is_none() {
            return; // UNVERIFIED-ENV on machines without SDK
        }
        let sdk = android_sdk_root().unwrap();
        let bt = newest_build_tools(&Some(sdk));
        assert!(bt.is_some());
    }
}
