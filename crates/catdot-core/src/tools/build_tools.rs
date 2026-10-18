//! Build/align/sign adapters (Part F5, Phase 2/4 slice). Processes spawn ONLY
//! here, always from verified toolchain paths (see `detect`). Every step is a
//! real subprocess with a hard timeout and captured stderr.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};
use crate::tools::detect::detect_toolchain;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct SignStep {
    pub step: String,
    pub exit_code: i32,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct BuildReport {
    pub unsigned_apk: Option<String>,
    pub aligned_apk: Option<String>,
    pub signed_apk: Option<String>,
    pub keystore_created: bool,
    /// Real `zipalign -c` check result (`aligned` step).
    pub aligned_verified: bool,
    /// Real `apksigner verify` result (`signed` step).
    pub signature_verified: bool,
    pub steps: Vec<SignStep>,
}

fn tool_err(step: &str, msg: impl Into<String>) -> AppError {
    AppError::new(
        ErrorCode::BuildFailed,
        format!("{step}: {}", msg.into()),
        format!(
            "What happened: the {step} step failed. What to do: open System Logs, copy the error and check the toolchain."
        ),
    )
}

struct Run {
    code: i32,
    out: String,
}

/// Run `exe` with `args` under a hard timeout (poll + kill tree). Output is
/// bounded to ~8KB via the reader threads; tools that print megabytes may
/// block their tail â€” acceptable for zipalign/apksigner (small outputs).
fn run_timeout(exe: &Path, args: &[&str], cwd: Option<&Path>, timeout: Duration) -> Option<Run> {
    use std::io::Read;
    let mut c = std::process::Command::new(exe);
    c.args(args);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    c.env("JAVA_HOME", std::env::var("JAVA_HOME").unwrap_or_default());
    c.env("SystemRoot", std::env::var("SystemRoot").unwrap_or_default());
    c.stdout(std::process::Stdio::piped());
    c.stderr(std::process::Stdio::piped());
    let mut child = c.spawn().ok()?;
    let mut so = child.stdout.take();
    let mut se = child.stderr.take();
    let t_out = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(p) = so.as_mut() {
            let _ = p.take(64 * 1024).read_to_string(&mut s);
        }
        s
    });
    let t_err = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(p) = se.as_mut() {
            let _ = p.take(64 * 1024).read_to_string(&mut s);
        }
        s
    });
    let deadline = std::time::Instant::now() + timeout;
    let code = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.code().unwrap_or(-1),
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None; // timed out
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    };
    let out = t_out.join().unwrap_or_default();
    let err = t_err.join().unwrap_or_default();
    Some(Run {
        code,
        out: format!("{err}{out}").chars().take(8_000).collect(),
    })
}

fn run(exe: &Path, args: &[&str], cwd: Option<&Path>, timeout: Duration) -> Option<Run> {
    run_timeout(exe, args, cwd, timeout)
}

/// Locate build-tools executables (zipalign/apksigner + keytool from the JBR).
pub fn tool_paths() -> Result<(PathBuf, PathBuf, PathBuf), AppError> {
    let t = detect_toolchain();
    let missing = |name, fix: &str| {
        AppError::new(
            ErrorCode::ToolNotFound,
            format!("{name} not found"),
            format!(
                "What happened: {name} is not installed. What to do: {fix}"
            ),
        )
    };
    let zipalign = t.zipalign.path.as_ref().map(PathBuf::from).ok_or_else(|| {
        missing("zipalign", "install Android SDK build-tools")
    })?;
    let apksigner = t.apksigner.path.as_ref().map(PathBuf::from).ok_or_else(|| {
        missing("apksigner", "install Android SDK build-tools")
    })?;
    // keytool: sibling of the detected java (JBR ships it).
    let java_path = t.java.path.clone().unwrap_or_default();
    let java_dir = PathBuf::from(&java_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let keytool = java_dir.join("keytool.exe");
    if !keytool.is_file() {
        // JAVA_HOME fallback
        if let Ok(jh) = std::env::var("JAVA_HOME") {
            let alt = PathBuf::from(jh).join("bin").join("keytool.exe");
            if alt.is_file() {
                return Ok((zipalign, apksigner, alt));
            }
        }
        return Err(missing("keytool", "install a JDK (17+)"));
    }
    Ok((zipalign, apksigner, keytool))
}

/// Ensure a debug keystore exists (auto-created once with a fixed alias).
/// Passphrase is knowledge-embedded for the debug keystore only (release keystores must never hardcode secrets; Phase 4 adds the keystore manager).
pub fn ensure_keystore(data_dir: &Path, keytool: &Path) -> Result<(PathBuf, String, String), AppError> {
    std::fs::create_dir_all(data_dir.join("keys"))?;
    let ks = data_dir.join("keys").join("catdot-debug.jks");
    const STORE_PASS: &str = "catdot-local";
    const ALIAS: &str = "catdot";
    if ks.is_file() {
        return Ok((ks, STORE_PASS.into(), ALIAS.into()));
    }
    let r = run(
        keytool,
        &[
            "-genkeypair",
            "-v",
            "-keystore",
            ks.to_string_lossy().as_ref(),
            "-alias",
            ALIAS,
            "-keyalg",
            "RSA",
            "-keysize",
            "2048",
            "-validity",
            "10000",
            "-storepass",
            STORE_PASS,
            "-keypass",
            STORE_PASS,
            "-dname",
            "CN=CatDot Studio Debug",
        ],
        None,
        Duration::from_secs(60),
    )
    .ok_or_else(|| tool_err("keystore", "keytool did not start"))?;
    if r.code != 0 {
        return Err(tool_err("keystore", r.out));
    }
    Ok((ks, STORE_PASS.into(), ALIAS.into()))
}

/// Align `unsigned` via zipalign (4-byte), then sign with apksigner.
/// Writes `aligned` and `signed` â€” original file is untouched.
pub fn align_and_sign(
    unsigned: &Path,
    aligned: &Path,
    signed: &Path,
    workdir: &Path,
    keystore: &Path,
    store_pass: &str,
    alias: &str,
) -> Result<BuildReport, AppError> {
    let mut report = BuildReport {
        unsigned_apk: Some(unsigned.to_string_lossy().into_owned()),
        aligned_apk: None,
        signed_apk: None,
        keystore_created: keystore.is_file(),
        aligned_verified: false,
        signature_verified: false,
        steps: Vec::new(),
    };
    let (zipalign, apksigner, _keytool) = tool_paths()?;
    let mut steps = Vec::new();

    // 1) zipalign -f 4 in out
    let r = run(
        &zipalign,
        &["-f", "4", unsigned.to_string_lossy().as_ref(), aligned.to_string_lossy().as_ref()],
        Some(workdir),
        Duration::from_secs(300),
    )
    .ok_or_else(|| tool_err("zipalign", "zipalign did not start"))?;
    steps.push(SignStep {
        step: "zipalign".into(),
        exit_code: r.code,
        detail: r.out.trim().to_string(),
    });
    if r.code != 0 {
        for s in &steps {
            push_log_error("build", &s.step, &s.detail);
        }
        return Err(tool_err("zipalign", r.out));
    }
    report.aligned_apk = Some(aligned.to_string_lossy().into_owned());

    // 1b) zipalign -c 4 aligned — verify alignment, not just run the tool.
    let r = run(
        &zipalign,
        &["-c", "4", aligned.to_string_lossy().as_ref()],
        Some(workdir),
        Duration::from_secs(120),
    )
    .ok_or_else(|| tool_err("zipalign-check", "zipalign did not start"))?;
    steps.push(SignStep {
        step: "zipalign-check".into(),
        exit_code: r.code,
        detail: r.out.trim().to_string(),
    });
    if r.code != 0 {
        for s in &steps {
            push_log_error("build", &s.step, &s.detail);
        }
        return Err(tool_err("zipalign-check", r.out));
    }
    report.aligned_verified = true;

    // 2) apksigner sign --ks ... --out signed aligned
    let ks_arg = format!("--ks={}", keystore.to_string_lossy());
    let ks_pass = format!("--ks-pass=pass:{store_pass}");
    let ks_alias = format!("--ks-key-alias={alias}");
    let out_arg = format!("--out={}", signed.to_string_lossy());
    let r = run(
        &apksigner,
        &[
            "sign",
            &ks_arg,
            &ks_pass,
            &ks_alias,
            &out_arg,
            aligned.to_string_lossy().as_ref(),
        ],
        Some(workdir),
        Duration::from_secs(300),
    )
    .ok_or_else(|| tool_err("apksigner", "apksigner did not start"))?;
    steps.push(SignStep {
        step: "apksigner-sign".into(),
        exit_code: r.code,
        detail: r.out.trim().to_string(),
    });
    if r.code != 0 {
        report.steps = steps;
        for s in &report.steps {
            push_log_error("build", &s.step, &s.detail);
        }
        return Err(tool_err("apksigner", r.out));
    }
    report.signed_apk = Some(signed.to_string_lossy().into_owned());

    // 3) apksigner verify signed
    let r = run(
        &apksigner,
        &["verify", signed.to_string_lossy().as_ref()],
        Some(workdir),
        Duration::from_secs(120),
    )
    .ok_or_else(|| tool_err("apksigner-verify", "apksigner did not start"))?;
    steps.push(SignStep {
        step: "apksigner-verify".into(),
        exit_code: r.code,
        detail: r.out.trim().to_string(),
    });
    if r.code != 0 {
        for s in &steps {
            push_log_error("build", &s.step, &s.detail);
        }
        return Err(tool_err("apksigner-verify", r.out));
    }
    report.signature_verified = true;
    report.steps = steps;
    Ok(report)
}

/// Push a build failure into the tracing stream (System Logs panel).
fn push_log_error(source: &str, step: &str, detail: &str) {
    if detail.is_empty() {
        return;
    }
    for line in detail.lines().take(6) {
        tracing::error!("[{source}/{step}] {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_paths_reports_missing_tools_structurally() {
        // On machines without the SDK this must fail with ToolNotFound, not panic.
        match tool_paths() {
            Ok((z, a, k)) => {
                assert!(z.is_file(), "zipalign path should exist");
                assert!(a.is_file(), "apksigner path should exist");
                assert!(k.is_file(), "keytool path should exist");
            }
            Err(e) => assert_eq!(e.code, ErrorCode::ToolNotFound),
        }
    }

    #[test]
    fn ensure_keystore_creates_file_once() {
        // Requires a JDK/keytool; skip-verify on machines without one.
        let dir = std::env::temp_dir().join(format!("catdot-ks-{}", std::process::id()));
        let keytool = std::env::var("JAVA_HOME")
            .map(|j| PathBuf::from(j).join("bin").join("keytool.exe"))
            .unwrap_or_else(|_| PathBuf::from("keytool.exe"));
        if !keytool.is_file() {
            return; // UNVERIFIED-ENV: no JDK on this machine
        }
        let (ks, pass, alias) = ensure_keystore(&dir, &keytool).expect("keystore");
        assert!(ks.is_file());
        let (ks2, pass2, alias2) = ensure_keystore(&dir, &keytool).expect("keystore 2x");
        assert_eq!(ks, ks2);
        assert_eq!(pass, pass2);
        assert_eq!(alias, alias2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn align_sign_verify_end_to_end() {
        // Real pipeline: aapt2-built APK â†’ zipalign â†’ apksigner sign â†’ verify.
        // Skips (UNVERIFIED-ENV) when aapt2/android.jar/build-tools are absent.
        let t = detect_toolchain();
        let Some(aapt2) = t.aapt2.path.as_ref().map(PathBuf::from) else { return };
        if !(aapt2.is_file()
            && t.apksigner.found
            && t.zipalign.found
            && t.java.found)
        {
            return; // UNVERIFIED-ENV
        }
        let android_jar = find_android_jar();
        let Some(android_jar) = android_jar else { return };
        let dir = std::env::temp_dir().join(format!("catdot-build-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        // Minimal manifest + aapt2 link â†’ real (unsigned) APK.
        let manifest = dir.join("AndroidManifest.xml");
        std::fs::write(
            &manifest,
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.forge.test"
    android:versionCode="1"
    android:versionName="1.0">
    <uses-sdk android:minSdkVersion="24" android:targetSdkVersion="31"/>
</manifest>
"#,
        )
        .unwrap();
        let unsigned = dir.join("testapp.apk");
        let run_r = run_timeout(
            &aapt2,
            &[
                "link",
                "--manifest",
                manifest.to_string_lossy().as_ref(),
                "-I",
                android_jar.to_string_lossy().as_ref(),
                "-o",
                unsigned.to_string_lossy().as_ref(),
            ],
            Some(&dir),
            std::time::Duration::from_secs(120),
        )
        .expect("aapt2 ran");
        assert_eq!(run_r.code, 0, "aapt2 link failed: {}", run_r.out);
        assert!(unsigned.is_file());

        // Build pipeline on the real APK.
        let (ks, pass, alias) = ensure_keystore(&dir, &t.java.path.as_ref().map(PathBuf::from).unwrap().parent().unwrap().join("keytool.exe")).expect("keystore");
        let aligned = dir.join("aligned.apk");
        let signed = dir.join("signed.apk");
        let rep =
            align_and_sign(&unsigned, &aligned, &signed, &dir, &ks, &pass, &alias)
                .expect("align+sign");
        assert!(aligned.is_file());
        assert!(signed.is_file());
        assert!(rep.steps.iter().all(|s| s.exit_code == 0), "all steps exit 0");
        assert!(rep.aligned_verified, "zipalign -c must pass");
        assert!(rep.signature_verified, "apksigner verify must pass");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Find the newest platforms/android-X/android.jar (UNVERIFIED-ENV â†’ None).
    fn find_android_jar() -> Option<PathBuf> {
        let t = detect_toolchain();
        let bt = t.apksigner.path.as_ref()?;
        // apksigner(.bat) lives in build-tools/<ver>/ â†’ SDK root is 3 levels up.
        let p = PathBuf::from(bt);
        let sdk = p.parent().and_then(|d| d.parent()).and_then(|d| d.parent())?;
        let mut plats: Vec<(u32, PathBuf)> = Vec::new();
        for e in std::fs::read_dir(sdk.join("platforms")).ok()? {
            let e = e.ok()?;
            let p = e.path().join("android.jar");
            let ver = e
                .file_name()
                .to_string_lossy()
                .rsplit('-')
                .next_back()
                .and_then(|s| s.parse::<u32>().ok());
            if let Some(v) = ver {
                if p.is_file() {
                    plats.push((v, p));
                }
            }
        }
        plats.sort_by_key(|(v, _)| *v);
        plats.pop().map(|(_, p)| p)
    }
}
