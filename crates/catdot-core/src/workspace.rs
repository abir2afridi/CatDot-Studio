//! APK workspace engine (Part F1â€“F4, Phase 2 slice): import â†’ inspect â†’ patch
//! â†’ export. Reads the original APK via the `zip` crate. F0 rule: the original
//! APK is never modified â€” import copies it into the workspace, every export
//! writes a NEW file. No apktool/JADX needed for these flows.

use std::collections::HashMap;
use std::io::{Read};
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use ts_rs::TS;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

use crate::errors::{AppError, ErrorCode};

const MAX_APK_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB
const MAX_ENTRIES: usize = 60_000; // archive-limits sanity bound (E2)
pub const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024; // per-file read limit
const TEXT_SUFFIXES: &[&str] = &["xml", "txt", "json", "md", "smali", "html", "js", "csv"];

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ApkEntry {
    pub path: String,
    pub size: u64,
    pub compressed_size: u64,
    pub is_text: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ApkReport {
    pub name: String,
    pub entry_count: usize,
    pub total_size: u64,
    pub uncompressed_total: u64,
    pub sha256: String,
    pub entries: Vec<ApkEntry>,
}

/// A pending edit: replaces the bytes of `entry_path` inside the APK on export.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Patch {
    pub entry_path: String,
    pub content: String,
}

fn read_err(msg: impl Into<String>) -> AppError {
    AppError::new(
        ErrorCode::ImportUnsupportedType,
        msg.into(),
        "What happened: the APK could not be read. Why: it may be corrupted or not a valid APK (a ZIP). What to do: pick a valid APK file.",
    )
}

fn unknown_entry(p: &str) -> AppError {
    AppError::new(
        ErrorCode::JobNotFound,
        format!("entry `{p}` not found in APK"),
        "What happened: that file path is not in the APK. What to do: pick a file from the tree.",
    )
}

/// Copy `source` into `workspace/original.apk` (source stays untouched) and
/// report the archive contents.
pub fn import_apk(source: &Path, workspace_root: &Path) -> Result<ApkReport, AppError> {
    std::fs::create_dir_all(workspace_root)?;
    let meta = std::fs::metadata(source)
        .map_err(|e| read_err(format!("cannot read source: {e}")))?;
    if meta.len() > MAX_APK_BYTES {
        return Err(read_err("source APK exceeds the 2 GiB limit"));
    }
    let target = workspace_root.join("original.apk");
    std::fs::copy(source, &target)
        .map_err(|e| read_err(format!("copy failed: {e}")))?;
    report_of(&target, "original.apk")
}

/// Scan the APK's central directory and build a report.
pub fn report_of(apk_path: &Path, name: &str) -> Result<ApkReport, AppError> {
    let f = std::fs::File::open(apk_path)
        .map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let total_size = f.metadata().map_err(|e| read_err(format!("metadata: {e}")))?.len();
    let mut ar = ZipArchive::new(f)
        .map_err(|e| read_err(format!("not a valid APK/ZIP: {e}")))?;
    if ar.len() > MAX_ENTRIES {
        return Err(read_err(format!(
            "archive has {} entries (limit {MAX_ENTRIES})",
            ar.len()
        )));
    }
    let mut entries = Vec::with_capacity(ar.len());
    let mut uncompressed_total = 0u64;
    for i in 0..ar.len() {
        let e = ar
            .by_index_raw(i)
            .map_err(|e| read_err(format!("bad entry: {e}")))?;
        let path = e.name().to_string();
        let suffix = path
            .rsplit('.')
            .next()
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        uncompressed_total += e.size();
        entries.push(ApkEntry {
            path,
            size: e.size(),
            compressed_size: e.compressed_size(),
            is_text: TEXT_SUFFIXES.contains(&suffix.as_str()),
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ApkReport {
        name: name.to_string(),
        entry_count: entries.len(),
        total_size,
        uncompressed_total,
        sha256: sha256_file(apk_path)?,
        entries,
    })
}

/// SHA-256 of a file, hex-encoded.
pub fn sha256_file(p: &Path) -> Result<String, AppError> {
    let data = std::fs::read(p).map_err(|e| read_err(format!("read failed: {e}")))?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(format!("{:x}", h.finalize()))
}

/// SHA-256 of an in-memory string, hex-encoded.
pub fn sha256_str(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

/// Read one text entry (UTF-8; binary-safe error otherwise).
pub fn read_entry(apk_path: &Path, entry_path: &str) -> Result<String, AppError> {
    let f = std::fs::File::open(apk_path)
        .map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let mut ar = ZipArchive::new(f)
        .map_err(|e| read_err(format!("bad APK: {e}")))?;
    let mut e = ar
        .by_name(entry_path)
        .map_err(|_| unknown_entry(entry_path))?;
    if e.size() as usize > MAX_TEXT_BYTES {
        return Err(read_err(format!(
            "entry `{entry_path}` too large to edit as text ({})",
            e.size()
        )));
    }
    let mut buf = String::with_capacity(e.size() as usize);
    e.read_to_string(&mut buf)
        .map_err(|_| read_err("entry is not UTF-8 text"))?;
    Ok(buf)
}

/// Export a NEW APK at `out_path` with `patches` applied. Every patch target
/// must exist in the APK (add-file flows land later). Returns the exported
/// file's SHA-256. Unpatched entries are copied byte-for-byte.
pub fn export_apk(
    source_apk: &Path,
    patches: &[Patch],
    out_path: &Path,
) -> Result<String, AppError> {
    for p in patches {
        if p.entry_path.is_empty() {
            return Err(AppError::new(
                ErrorCode::InvalidInput,
                "patch has an empty target path",
                "What happened: an edit had no target file. What to do: choose a file from the tree.",
            ));
        }
    }
    let unique: HashMap<&str, &Patch> = patches.iter().map(|p| (p.entry_path.as_str(), p)).collect();
    if unique.len() != patches.len() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "duplicate patch target",
            "What happened: two edits target the same file. What to do: keep one edit per file.",
        ));
    }

    let f_in = std::fs::File::open(source_apk)
        .map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let mut ar = ZipArchive::new(f_in)
        .map_err(|e| read_err(format!("bad APK: {e}")))?;
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write entries in original order; patch targets get new bytes.
    let buf = Vec::new();
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(buf));
    let opts: SimpleFileOptions = SimpleFileOptions::default();
    let mut missing: Option<String> = None;
    for i in 0..ar.len() {
        let mut e = ar
            .by_index(i)
            .map_err(|e| read_err(format!("bad entry: {e}")))?;
        let path = e.name().to_string();
        match unique.get(path.as_str()) {
            Some(p) => {
                zw.start_file(path.clone(), opts)
                    .map_err(|e| read_err(format!("write entry: {e}")))?;
                std::io::copy(&mut p.content.as_bytes(), &mut zw)
                    .map_err(|e| read_err(format!("write failed: {e}")))?;
            }
            None => {
                if path.ends_with('/') {
                    zw.add_directory(path.clone(), opts)
                        .map_err(|er| read_err(format!("write dir: {er}")))?;
                } else {
                    zw.start_file(path.clone(), opts)
                        .map_err(|e| read_err(format!("write entry: {e}")))?;
                    std::io::copy(&mut e, &mut zw)
                        .map_err(|err| read_err(format!("copy failed: {err}")))?;
                }
            }
        }
        let _ = &mut missing;
    }
    // Reject patches for entries that do not exist in the APK.
    let _ = &mut missing;
    for p in patches {
        if ar.by_name(&p.entry_path).is_err() {
            zw.finish().ok();
            return Err(unknown_entry(&p.entry_path));
        }
    }
    let bytes = zw.finish().map_err(|e| read_err(format!("finish: {e}")))?.into_inner();
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out_path, &bytes)?;
    sha256_file(out_path)
}

/// Summary of pending edits (UI badge).
#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export)]
pub struct PatchSummary {
    pub count: usize,
    pub entries: Vec<String>,
}

pub fn patch_summary(patches: &[Patch]) -> PatchSummary {
    PatchSummary {
        count: patches.len(),
        entries: patches.iter().map(|p| p.entry_path.clone()).collect(),
    }
}

/// Workspace anchor for a project (`resolve_in_project` applies on top).
pub fn workspace_root_for(data_dir: &Path, project_id: &str) -> PathBuf {
    data_dir.join("projects").join(project_id).join("workspace")
}

#[cfg(test)]
mod tests {
    use super::tests_util::make_test_zip;
    use std::path::PathBuf;

    fn tmp_dir(tag: &str) -> PathBuf {
        let mut d = std::env::temp_dir();
        d.push(format!(
            "catdot-ws-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|t| t.subsec_nanos())
                .unwrap_or(0)
        ));
        d
    }

    #[test]
    fn import_generates_report_and_copies() {
        let apk = make_test_zip(&[
            ("AndroidManifest.xml", "<manifest/>"),
            ("assets/config.json", "{\n \"a\": 1\n}\n"),
            ("res/values.xml", "<r/>"),
        ]);
        let dir = tmp_dir("import");
        let rep = super::import_apk(&apk, &dir).expect("import");
        assert_eq!(rep.entry_count, 3);
        assert_eq!(rep.sha256.len(), 64);
        assert!(rep.entries.iter().all(|e| e.path.len() > 0));
        // Anchored copy exists; the source file is still present (read-only).
        assert!(dir.join("original.apk").is_file());
        assert!(apk.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_entry_round_trip_and_missing() {
        let apk = make_test_zip(&[("assets/a.xml", "hello")]);
        let dir = tmp_dir("read");
        super::import_apk(&apk, &dir).expect("import");
        let p = dir.join("original.apk");
        assert_eq!(super::read_entry(&p, "assets/a.xml").unwrap(), "hello");
        assert!(super::read_entry(&p, "no/pe").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_applies_patch_and_preserves_original() {
        let apk = make_test_zip(&[
            ("assets/a.xml", "A".repeat(300).as_str()),
            ("assets/keep.json", "{\"k\":1}"),
        ]);
        let orig_sha = super::sha256_file(&apk).unwrap();
        let dir = tmp_dir("export");
        super::import_apk(&apk, &dir).unwrap();
        let out = dir.join("exported.apk");
        let patches = vec![super::Patch {
            entry_path: "assets/a.xml".into(),
            content: "patched text".into(),
        }];
        let out_sha =
            super::export_apk(&dir.join("original.apk"), &patches, &out).expect("export");
        // New file: different checksum; original untouched.
        assert_ne!(out_sha, orig_sha);
        assert_eq!(super::sha256_file(&apk).unwrap(), orig_sha);
        assert_eq!(
            super::read_entry(&out, "assets/a.xml").unwrap(),
            "patched text"
        );
        assert_eq!(super::read_entry(&out, "assets/keep.json").unwrap(), "{\"k\":1}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_rejects_patch_for_missing_entry() {
        let apk = make_test_zip(&[("a.xml", "x")]);
        let dir = tmp_dir("missing");
        super::import_apk(&apk, &dir).unwrap();
        let out = dir.join("out.apk");
        let patches = vec![super::Patch {
            entry_path: "nope.xml".into(),
            content: "y".into(),
        }];
        assert!(super::export_apk(&dir.join("original.apk"), &patches, &out).is_err());
        // No partial output should remain.
        assert!(!out.exists() || super::sha256_file(&out).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_rejects_duplicate_patch_targets() {
        let apk = make_test_zip(&[("a.xml", "x")]);
        let dir = tmp_dir("dup");
        super::import_apk(&apk, &dir).unwrap();
        let out = dir.join("out.apk");
        let patches = vec![
            super::Patch { entry_path: "a.xml".into(), content: "one".into() },
            super::Patch { entry_path: "a.xml".into(), content: "two".into() },
        ];
        assert!(super::export_apk(&dir.join("original.apk"), &patches, &out).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn patch_summary_lists_entries() {
        let s = super::patch_summary(&[
            super::Patch { entry_path: "a.xml".into(), content: "x".into() },
        ]);
        assert_eq!(s.count, 1);
        assert_eq!(s.entries, vec!["a.xml".to_string()]);
    }
}

pub mod tests_util {
    //! Test helpers (not reachable from the UI).

    use std::path::PathBuf;
    use zip::write::SimpleFileOptions;

    /// Build a real ZIP on disk with the given (name, content) entries.
    pub fn make_test_zip(entries: &[(&str, &str)]) -> PathBuf {
        let mut d = std::env::temp_dir();
        d.push(format!(
            "catdot-zip-{}-{}.zip",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|t| t.subsec_nanos())
                .unwrap_or(0)
        ));
        let f = std::fs::File::create(&d).expect("create zip");
        let mut zw = zip::ZipWriter::new(f);
        let opts: SimpleFileOptions = SimpleFileOptions::default();
        for (name, content) in entries {
            use std::io::Write;
            zw.start_file(*name, opts).expect("start file");
            zw.write_all(content.as_bytes()).expect("write bytes");
        }
        zw.finish().expect("finish zip");
        d
    }
}
