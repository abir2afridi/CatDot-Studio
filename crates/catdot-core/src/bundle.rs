//! Source-media resolution (Phase 3): a plain APK is used directly; an XAPK
//! bundle is not an APK, so its base APK is extracted (cached, read-only copy)
//! and becomes the mod target. The imported original stays untouched.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, ErrorCode};

#[derive(Debug)]
pub struct SourceApk {
    pub path: PathBuf,
    /// `Some(root entry name)` when the source came from an XAPK bundle.
    pub base_entry: Option<String>,
    pub size: u64,
    /// True when this call extracted the base (false for cache hits/plain APK).
    pub extracted: bool,
}

fn zip_err(msg: impl Into<String>) -> AppError {
    AppError::new(
        ErrorCode::ImportUnsupportedType,
        msg.into(),
        "What happened: this file is not a readable APK/XAPK zip. What to do: import a valid .apk or .xapk file.",
    )
}

/// Resolve the installable APK to decode/build. `cache_key` must be filename
/// safe (the import SHA-256 is used by callers).
pub fn resolve_source_apk(
    container: &Path,
    cache_dir: &Path,
    cache_key: &str,
) -> Result<SourceApk, AppError> {
    let f = std::fs::File::open(container)
        .map_err(|e| zip_err(format!("cannot open imported file: {e}")))?;
    let mut ar = zip::ZipArchive::new(f).map_err(|e| zip_err(format!("bad APK/XAPK: {e}")))?;

    let mut has_root_manifest = false;
    let mut best: Option<(String, u64, usize)> = None;
    for i in 0..ar.len() {
        let entry = ar
            .by_index(i)
            .map_err(|e| zip_err(format!("bad entry #{i}: {e}")))?;
        if entry.name() == "AndroidManifest.xml" {
            has_root_manifest = true;
        }
        if !entry.is_dir()
            && !entry.name().contains('/')
            && entry.name().to_ascii_lowercase().ends_with(".apk")
        {
            let size = entry.size();
            if best.as_ref().is_none_or(|(_, s, _)| size > *s) {
                best = Some((entry.name().to_string(), size, i));
            }
        }
    }

    if has_root_manifest {
        let size = std::fs::metadata(container)
            .map_err(|e| zip_err(format!("cannot stat imported file: {e}")))?
            .len();
        return Ok(SourceApk {
            path: container.to_path_buf(),
            base_entry: None,
            size,
            extracted: false,
        });
    }

    let Some((name, size, index)) = best else {
        return Err(AppError::new(
            ErrorCode::ImportUnsupportedType,
            "no base APK found in bundle",
            "What happened: this XAPK has no root-level APK to decode. What to do: import the app's base APK directly.",
        ));
    };

    let key: String = cache_key
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(16)
        .collect();
    if key.is_empty() {
        return Err(zip_err("invalid cache key"));
    }
    let out = cache_dir.join(format!("base-{key}.apk"));
    if out.is_file() && std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0) == size {
        return Ok(SourceApk {
            path: out,
            base_entry: Some(name),
            size,
            extracted: false,
        });
    }

    let mut entry = ar
        .by_index(index)
        .map_err(|e| zip_err(format!("cannot read base APK entry: {e}")))?;
    let tmp = out.with_extension("apk.tmp");
    {
        let mut w = std::fs::File::create(&tmp)
            .map_err(|e| zip_err(format!("cannot write base APK: {e}")))?;
        std::io::copy(&mut entry, &mut w)
            .map_err(|e| zip_err(format!("cannot extract base APK: {e}")))?;
        w.flush()
            .map_err(|e| zip_err(format!("cannot flush base APK: {e}")))?;
    }
    std::fs::rename(&tmp, &out)
        .map_err(|e| zip_err(format!("cannot finalize base APK: {e}")))?;
    Ok(SourceApk {
        path: out,
        base_entry: Some(name),
        size,
        extracted: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zip_with(tag: &str, files: &[(&str, &[u8])]) -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!(
            "catdot-bundle-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|t| t.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&base).unwrap();
        let path = base.join("input.zip");
        let f = std::fs::File::create(&path).unwrap();
        let mut w = zip::ZipWriter::new(f);
        for (name, data) in files {
            w.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap();
        (base, path)
    }

    #[test]
    fn plain_apk_is_used_directly() {
        let (dir, p) = zip_with("apk", &[("AndroidManifest.xml", b"<manifest/>")]);
        let src = resolve_source_apk(&p, &dir, "abc123").expect("resolve");
        assert_eq!(src.path, p);
        assert!(src.base_entry.is_none());
        assert!(!src.extracted);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn xapk_extracts_and_caches_largest_root_apk() {
        let (dir, p) = zip_with(
            "xapk",
            &[
                ("manifest.json", b"{\"xapk_version\":2}"),
                ("config.en.apk", b"small"),
                ("com.app.base.apk", b"the-largest-base-apk"),
                ("assets/not_a_root.apk", b"nested-must-not-win"),
            ],
        );
        let src = resolve_source_apk(&p, &dir, "sha256abcdef012345").expect("resolve");
        assert_eq!(src.base_entry.as_deref(), Some("com.app.base.apk"));
        assert!(src.extracted);
        assert_eq!(std::fs::read(&src.path).unwrap(), b"the-largest-base-apk");
        assert!(src.path.starts_with(&dir) && src.path != p);

        let again = resolve_source_apk(&p, &dir, "sha256abcdef012345").expect("cached");
        assert!(!again.extracted);
        assert_eq!(again.path, src.path);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundle_without_base_apk_fails_honestly() {
        let (dir, p) = zip_with("nobase", &[("manifest.json", b"{}")]);
        let err = resolve_source_apk(&p, &dir, "k").expect_err("no base");
        assert_eq!(err.code, ErrorCode::ImportUnsupportedType);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
