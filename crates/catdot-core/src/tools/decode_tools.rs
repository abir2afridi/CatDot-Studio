//! apktool adapters (Part F4/Phase 3): decode a real APK into editable text
//! (Android manifest + resources) and rebuild it back. Processes spawn ONLY
//! here — always via the detected JDK (`java -jar`) with hard timeouts and
//! bounded output. The jar itself is pinned in `toolchain/manifest.json`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::errors::{AppError, ErrorCode};
use crate::tools::build_tools::run_timeout;
use crate::tools::detect::detect_toolchain;

/// Locate the pinned apktool jar (dev download cache or explicit env). The
/// manifest pin (version + sha256) is the source of truth.
pub fn apktool_jar() -> Result<PathBuf, AppError> {
    if let Ok(p) = std::env::var("CATDOT_APKTOOL_JAR") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Ok(p);
        }
    }
    // Relative to the working dir (dev runs + tests; the installer will move
    // this into the app data dir in a later slice).
    let local = PathBuf::from("toolchain")
        .join("downloads")
        .join("apktool.jar");
    if local.is_file() {
        return std::fs::canonicalize(&local).map_err(|_| {
            AppError::new(
                ErrorCode::ToolNotFound,
                "apktool jar unreadable",
                "What happened: toolchain/downloads/apktool.jar exists but cannot be read. What to do: re-run the apktool install from System check.",
            )
        });
    }
    Err(AppError::new(
        ErrorCode::ToolNotFound,
        "apktool not installed",
        "What happened: apktool (decode/rebuild tool) is not installed. What to do: run the apktool install in System check (pinned download with checksum).",
    ))
}

fn java_exe() -> Result<PathBuf, AppError> {
    let t = detect_toolchain();
    match t.java.path.as_deref().filter(|p| Path::new(p).is_file()) {
        Some(p) => Ok(PathBuf::from(p)),
        None => Err(AppError::new(
            ErrorCode::ToolNotFound,
            "java not found",
            "What happened: no JDK/JRE found. What to do: install a JDK 17+ and re-run System check (apktool needs java).",
        )),
    }
}

/// Decode `apk` into `out_dir` (fully text-editable manifest + resources).
pub fn decode(apk: &Path, out_dir: &Path) -> Result<u64, AppError> {
    let jar = apktool_jar()?;
    let java = java_exe()?;
    if !apk.is_file() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "apk missing",
            "What happened: there is no APK to decode. What to do: import an APK first.",
        ));
    }
    let _ = std::fs::remove_dir_all(out_dir);
    let run = run_timeout(
        &java,
        &[
            "-jar",
            &jar.to_string_lossy(),
            "d",
            "-f",
            "-o",
            &out_dir.to_string_lossy(),
            &apk.to_string_lossy(),
        ],
        out_dir.parent().or_else(|| out_dir.ancestors().nth(2)),
        Duration::from_secs(600),
    )
    .ok_or_else(|| {
        AppError::new(
            ErrorCode::Internal,
            "apktool decode timed out",
            "What happened: decoding took too long (600 s cap). What to do: retry; for huge APKs this can be slow on the first run (apktool may fetch its framework cache once).",
        )
    })?;
    if run.code != 0 || !out_dir.join("AndroidManifest.xml").is_file() {
        let head: String = run.out.lines().take(3).collect::<Vec<_>>().join(" | ");
        return Err(AppError::new(
            ErrorCode::Internal,
            format!("apktool decode failed: {head}"),
            "What happened: apktool could not decode this APK (often protection or an unsupported format). What to do: read the detail; manual text patching still works for plain-text entries.",
        ));
    }
    fix_empty_resource_package(out_dir)?;
    let files = count_files(out_dir);
    Ok(files)
}

/// apktool 3.0.x quirk: `resourcesInfo.packageName` can decode as empty, and
/// rebuild then passes an empty `--rename-resources-package` that crashes
/// aapt2 ("all packages being linked must have a name"). Deterministic fix:
/// fill it from the decoded manifest's `package=` attribute.
pub fn fix_empty_resource_package(out_dir: &Path) -> Result<(), AppError> {
    let yml = out_dir.join("apktool.yml");
    let Ok(text) = std::fs::read_to_string(&yml) else {
        return Ok(()); // no yml → nothing to fix
    };
    if !text.contains("resourcesInfo") {
        return Ok(());
    }
    let has_empty = text
        .lines()
        .any(|l| l.trim_start().starts_with("packageName:") && l.trim_end().ends_with(':'));
    if !has_empty {
        return Ok(());
    }
    let manifest = std::fs::read_to_string(out_dir.join("AndroidManifest.xml")).map_err(|e| {
        AppError::new(
            ErrorCode::Internal,
            e.to_string(),
            "What happened: the decoded manifest cannot be read. What to do: run decode again.",
        )
    })?;
    let pkg = manifest
        .lines()
        .find_map(|l| {
            l.split_whitespace().find_map(|tok| tok.strip_prefix("package=\""))
                .map(|v| v.trim_end_matches('"').to_string())
        })
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::Internal,
                "decoded manifest has no package attribute",
                "What happened: apktool.yml needs resourcesInfo.packageName but the decoded manifest has none. What to do: report this APK in an issue (qq).",
            )
        })?;
    let fixed: String = text
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("packageName:") && l.trim_end().ends_with(':') {
                format!("  packageName: {pkg}")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&yml, fixed.replace("\r\n", "\n")).map_err(|e| {
        AppError::new(
            ErrorCode::Internal,
            e.to_string(),
            "What happened: apktool.yml cannot be updated. What to do: check disk space and retry.",
        )
    })?;
    Ok(())
}

/// Rebuild a decoded directory into an unsigned APK.
pub fn rebuild(dir: &Path, out_apk: &Path) -> Result<(), AppError> {
    let jar = apktool_jar()?;
    let java = java_exe()?;
    if !dir.join("AndroidManifest.xml").is_file() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "decoded tree missing",
            "What happened: there is no decoded project to rebuild. What to do: run apktool decode first (any structured edit will do it automatically).",
        ));
    }
    let _ = std::fs::remove_file(out_apk);
    let run = run_timeout(
        &java,
        &[
            "-jar",
            &jar.to_string_lossy(),
            "b",
            &dir.to_string_lossy(),
            "-o",
            &out_apk.to_string_lossy(),
        ],
        dir.parent(),
        Duration::from_secs(600),
    )
    .ok_or_else(|| {
        AppError::new(
            ErrorCode::Internal,
            "apktool rebuild timed out",
            "What happened: rebuilding took too long (600 s cap). What to do: retry; check the decoded edits for broken XML.",
        )
    })?;
    if run.code != 0 || !out_apk.is_file() {
        let head: String = run.out.lines().take(5).collect::<Vec<_>>().join(" | ");
        return Err(AppError::new(
            ErrorCode::Internal,
            format!("apktool rebuild failed: {head}"),
            "What happened: apktool could not rebuild the project (usually a malformed decoded edit). What to do: check the last edits (Change history) and revert the suspect checkpoint.",
        ));
    }
    Ok(())
}

fn count_files(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| {
                    if e.path().is_dir() {
                        count_files(&e.path())
                    } else {
                        1
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::build_tools::{align_and_sign, tool_paths};
    use crate::tools::detect::ToolchainReport;

    /// The heart-of-product round trip (Part H3): rename the app label in a
    /// newly decoded project, rebuild, zipalign+sign, verify — and confirm the
    /// new label via `aapt2 dump badging`. Skips (UNVERIFIED-ENV) when apktool
    /// jar / SDK / android.jar are absent.
    #[test]
    fn rename_label_rebuild_sign_verify_round_trip() {
        let t = detect_toolchain();
        let Ok(jar) = apktool_jar() else {
            return; // UNVERIFIED-ENV: apktool not installed
        };
        assert!(jar.is_file());
        let (Some(aapt2), true) = (
            t.aapt2.path.as_ref().map(PathBuf::from),
            t.apksigner.found && t.zipalign.found && t.java.found,
        ) else {
            return; // UNVERIFIED-ENV: no build-tools
        };
        // android.jar: newest platforms dir from the SDK layout.
        let android_jar = find_android_jar(&t);
        let Some(android_jar) = android_jar else {
            return;
        };

        let dir = std::env::temp_dir().join(format!("catdot-decode-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let manifest_src = dir.join("AndroidManifest.xml");
        std::fs::write(
            &manifest_src,
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.forge.test"
    android:versionCode="1"
    android:versionName="1.0">
    <uses-sdk android:minSdkVersion="24" android:targetSdkVersion="31"/>
    <application android:label="CatForge"/>
</manifest>
"#,
        )
        .unwrap();
        let unsigned = dir.join("testapp.apk");
        let link = run_timeout(
            &aapt2,
            &[
                "link",
                "--manifest",
                &manifest_src.to_string_lossy(),
                "-I",
                &android_jar.to_string_lossy(),
                "-o",
                &unsigned.to_string_lossy(),
            ],
            Some(&dir),
            Duration::from_secs(120),
        )
        .expect("aapt2 ran");
        assert_eq!(link.code, 0, "aapt2 link failed: {}", link.out);
        assert!(unsigned.is_file());

        // 1) decode → decoded manifest must contain the literal label.
        let decoded = dir.join("decoded");
        decode(&unsigned, &decoded).expect("decode");
        let dm = decoded.join("AndroidManifest.xml");
        assert!(dm.is_file());
        let dm_src = std::fs::read_to_string(&dm).unwrap();
        assert!(
            dm_src.contains("CatForge"),
            "literal label must survive decode"
        );

        // 2) "user edit": rename the label in the decoded tree.
        let patched = dm_src.replace("CatForge", "CatForge X");
        assert_ne!(patched, dm_src, "replacement must change something");
        std::fs::write(&dm, patched).unwrap();

        // 3) rebuild → zipalign+sign.
        let rebuilt = dir.join("rebuilt.apk");
        rebuild(&decoded, &rebuilt).expect("rebuild");
        assert!(rebuilt.is_file());
        let (zipalign, _apksigner, keytool) = tool_paths().expect("build tools");
        let (ks, pass, alias) =
            crate::tools::build_tools::ensure_keystore(&dir, &keytool).expect("keystore");
        let aligned = dir.join("aligned.apk");
        let signed = dir.join("signed.apk");
        let rep = align_and_sign(&rebuilt, &aligned, &signed, &dir, &ks, &pass, &alias)
            .expect("align+sign");
        assert!(rep.signature_verified, "apksigner verify must pass");
        assert!(rep.aligned_verified, "zipalign -c must pass");
        assert!(zipalign.is_file(), "zipalign path exists");

        // 4) ground truth: badging shows the new label.
        let badging = run_timeout(
            &aapt2,
            &["dump", "badging", &signed.to_string_lossy()],
            Some(&dir),
            Duration::from_secs(60),
        )
        .expect("aapt2 badging ran");
        assert!(
            badging.out.contains("application-label:'CatForge X'"),
            "badging must show the renamed label, got: {}",
            badging.out
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Find the newest platforms/android-X/android.jar (UNVERIFIED-ENV → None).
    fn find_android_jar(t: &ToolchainReport) -> Option<PathBuf> {
        let bt = t.apksigner.path.as_ref()?;
        let p = PathBuf::from(bt);
        let sdk = p.parent()?.parent()?.parent()?;
        let mut plats: Vec<(u32, PathBuf)> = Vec::new();
        for e in std::fs::read_dir(sdk.join("platforms")).ok()? {
            let e = e.ok()?;
            let p = e.path().join("android.jar");
            if !p.is_file() {
                continue;
            }
            let num: String = e
                .file_name()
                .to_string_lossy()
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            let Ok(n) = num.parse::<u32>() else { continue };
            plats.push((n, p));
        }
        plats.sort_by_key(|(n, _)| std::cmp::Reverse(*n));
        plats.into_iter().next().map(|(_, p)| p)
    }
}
