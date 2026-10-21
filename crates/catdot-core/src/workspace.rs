//! APK workspace engine (Part F1â€“F4, Phase 2 slice): import â†’ inspect â†’ patch
//! â†’ export. Reads the original APK via the `zip` crate. F0 rule: the original
//! APK is never modified â€” import copies it into the workspace, every export
//! writes a NEW file. No apktool/JADX needed for these flows.

use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use ts_rs::TS;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
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

pub const MAX_SEARCH_HITS: usize = 200;
const MAX_SEARCH_ENTRY_BYTES: u64 = 1_500_000;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ApkSearchHit {
    pub entry_path: String,
    pub line: u32,
    pub column: u32,
    pub snippet: String,
    pub patched: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ApkSearchResult {
    pub hits: Vec<ApkSearchHit>,
    pub truncated: bool,
    pub scanned_entries: usize,
}

pub const MAX_REPLACE_ENTRIES: usize = 100;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ApkReplaceResult {
    pub changed_entries: Vec<String>,
    pub replacements: usize,
    pub truncated: bool,
}

/// Non-serialized companion: the new patch contents produced by a replace.
#[derive(Debug)]
pub struct ApkReplaceOutcome {
    pub result: ApkReplaceResult,
    pub updates: Vec<Patch>,
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
/// report the archive contents. The source must be a ZIP by magic bytes
/// (`PK\x03\x04`) — detection is by content, not extension (Part F1).
pub fn import_apk(source: &Path, workspace_root: &Path) -> Result<ApkReport, AppError> {
    std::fs::create_dir_all(workspace_root)?;
    let meta =
        std::fs::metadata(source).map_err(|e| read_err(format!("cannot read source: {e}")))?;
    if meta.len() > MAX_APK_BYTES {
        return Err(read_err("source APK exceeds the 2 GiB limit"));
    }
    if !source.is_file() {
        return Err(read_err("source is not a regular file"));
    }
    reject_non_zip(source)?;
    let target = workspace_root.join("original.apk");
    std::fs::copy(source, &target).map_err(|e| read_err(format!("copy failed: {e}")))?;
    report_of(&target, "original.apk")
}

/// Reject files whose leading bytes are not the local-file-header ZIP magic.
/// The check runs BEFORE anything is copied, so a bad file never reaches the
/// workspace.
fn reject_non_zip(source: &Path) -> Result<(), AppError> {
    let mut f =
        std::fs::File::open(source).map_err(|e| read_err(format!("cannot open source: {e}")))?;
    let mut magic = [0u8; 4];
    let n = f
        .read(&mut magic)
        .map_err(|e| read_err(format!("cannot read source: {e}")))?;
    if n == 4 && magic == [0x50, 0x4B, 0x03, 0x04] {
        Ok(())
    } else {
        Err(read_err(
            "file is not a valid APK (missing ZIP magic; allowed types: .apk APK)",
        ))
    }
}

/// Scan the APK's central directory and build a report.
pub fn report_of(apk_path: &Path, name: &str) -> Result<ApkReport, AppError> {
    let f = std::fs::File::open(apk_path).map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let total_size = f
        .metadata()
        .map_err(|e| read_err(format!("metadata: {e}")))?
        .len();
    let mut ar = ZipArchive::new(f).map_err(|e| read_err(format!("not a valid APK/ZIP: {e}")))?;
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

/// SHA-256 of a file, hex-encoded. Streams in 1 MiB chunks so a 2 GiB APK
/// never loads into memory (Part F1: "stream, don't load whole file").
pub fn sha256_file(p: &Path) -> Result<String, AppError> {
    let f = std::fs::File::open(p).map_err(|e| read_err(format!("open failed: {e}")))?;
    let mut r = BufReader::with_capacity(1024 * 1024, f);
    let mut h = Sha256::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let n = r
            .read(&mut chunk)
            .map_err(|e| read_err(format!("read failed: {e}")))?;
        if n == 0 {
            break;
        }
        h.update(&chunk[..n]);
    }
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
    let f = std::fs::File::open(apk_path).map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let mut ar = ZipArchive::new(f).map_err(|e| read_err(format!("bad APK: {e}")))?;
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

/// Search all text entries (original APK + pending patches win) for `query`.
/// Binary and oversized entries are skipped silently; `scanned_entries` says
/// how many readable entries were actually searched (honest limits).
pub fn search_entries(
    apk_path: &Path,
    patches: &[Patch],
    query: &str,
    case_sensitive: bool,
) -> Result<ApkSearchResult, AppError> {
    let needle = query.trim();
    if needle.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "search query is empty",
            "What happened: the search box is empty. What to do: type at least one character.",
        ));
    }
    let needle_lower = needle.to_lowercase();
    let patched: HashMap<&str, &str> = patches
        .iter()
        .map(|p| (p.entry_path.as_str(), p.content.as_str()))
        .collect();

    let f = std::fs::File::open(apk_path).map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let mut ar = ZipArchive::new(f).map_err(|e| read_err(format!("bad APK: {e}")))?;

    let mut hits = Vec::new();
    let mut scanned_entries = 0usize;
    let mut truncated = false;

    for i in 0..ar.len() {
        let mut entry = ar
            .by_index(i)
            .map_err(|e| read_err(format!("bad APK entry #{i}: {e}")))?;
        if entry.is_dir() {
            continue;
        }
        let path = entry.name().to_string();
        let content = if let Some(text) = patched.get(path.as_str()) {
            scanned_entries += 1;
            (*text).to_string()
        } else {
            if entry.size() > MAX_SEARCH_ENTRY_BYTES {
                continue;
            }
            let mut text = String::with_capacity(entry.size() as usize);
            if entry.read_to_string(&mut text).is_err() {
                continue; // binary / non-UTF-8: not searchable as text
            }
            scanned_entries += 1;
            text
        };
        let is_patched = patched.contains_key(path.as_str());

        for (idx, line) in content.lines().enumerate() {
            let (found, col) = if case_sensitive {
                match line.find(needle) {
                    Some(at) => (true, line[..at].chars().count() + 1),
                    None => (false, 0),
                }
            } else {
                let lower = line.to_lowercase();
                match lower.find(&needle_lower) {
                    Some(at) => (true, lower[..at].chars().count() + 1),
                    None => (false, 0),
                }
            };
            if !found {
                continue;
            }
            hits.push(ApkSearchHit {
                entry_path: path.clone(),
                line: (idx + 1) as u32,
                column: col as u32,
                snippet: line.trim().chars().take(200).collect(),
                patched: is_patched,
            });
            if hits.len() >= MAX_SEARCH_HITS {
                truncated = true;
                break;
            }
        }
        if truncated {
            break;
        }
    }

    Ok(ApkSearchResult {
        hits,
        truncated,
        scanned_entries,
    })
}

/// Replace an exact, case-sensitive needle in every readable text entry
/// (pending patches win as the base). Every change becomes a reversible patch;
/// limits keep a single bulk edit bounded and honest (`truncated`).
pub fn replace_in_entries(
    apk_path: &Path,
    patches: &[Patch],
    needle: &str,
    replacement: &str,
) -> Result<ApkReplaceOutcome, AppError> {
    if needle.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "replace target is empty",
            "What happened: there is nothing to replace. What to do: enter the text to find.",
        ));
    }
    if needle.len() > 512 || replacement.len() > 4096 {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "search/replace text is too long",
            "What happened: the search text must be ≤512 bytes and the replacement ≤4096 bytes. What to do: shorten them.",
        ));
    }
    let patched: HashMap<&str, &str> = patches
        .iter()
        .map(|p| (p.entry_path.as_str(), p.content.as_str()))
        .collect();

    let f = std::fs::File::open(apk_path).map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let mut ar = ZipArchive::new(f).map_err(|e| read_err(format!("bad APK: {e}")))?;

    let mut updates = Vec::new();
    let mut changed_entries = Vec::new();
    let mut replacements = 0usize;
    let mut truncated = false;
    let total_entries = ar.len();

    for i in 0..total_entries {
        let mut entry = ar
            .by_index(i)
            .map_err(|e| read_err(format!("bad APK entry #{i}: {e}")))?;
        if entry.is_dir() {
            continue;
        }
        let path = entry.name().to_string();
        if updates.iter().any(|u: &Patch| u.entry_path == path) {
            continue;
        }
        let content = if let Some(text) = patched.get(path.as_str()) {
            (*text).to_string()
        } else {
            if entry.size() > MAX_SEARCH_ENTRY_BYTES {
                continue;
            }
            let mut text = String::with_capacity(entry.size() as usize);
            if entry.read_to_string(&mut text).is_err() {
                continue;
            }
            text
        };
        let count = content.matches(needle).count();
        if count == 0 {
            continue;
        }
        let next = content.replace(needle, replacement);
        replacements += count;
        changed_entries.push(path.clone());
        updates.push(Patch {
            entry_path: path,
            content: next,
        });
        if changed_entries.len() >= MAX_REPLACE_ENTRIES {
            truncated = i + 1 < total_entries;
            break;
        }
    }

    Ok(ApkReplaceOutcome {
        result: ApkReplaceResult {
            changed_entries,
            replacements,
            truncated,
        },
        updates,
    })
}

/// Export a NEW APK at `out_path` with `patches` applied. Every patch target
/// must exist in the APK (add-file flows land later). Returns the exported
/// file's SHA-256. Unpatched entries are copied byte-for-byte; each entry's
/// original compression method is preserved (Stored stays Stored — required
/// so `resources.arsc` can remain uncompressed+aligned for targetSdk ≥ 30).
/// Patch targets are validated BEFORE any output is produced, so a failed
/// export never leaves a partial file behind.
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
    let unique: HashMap<&str, &Patch> =
        patches.iter().map(|p| (p.entry_path.as_str(), p)).collect();
    if unique.len() != patches.len() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "duplicate patch target",
            "What happened: two edits target the same file. What to do: keep one edit per file.",
        ));
    }

    let f_in =
        std::fs::File::open(source_apk).map_err(|e| read_err(format!("cannot open APK: {e}")))?;
    let mut ar = ZipArchive::new(f_in).map_err(|e| read_err(format!("bad APK: {e}")))?;

    // Pre-validate every patch target against the archive before writing.
    for name in unique.keys() {
        if ar.by_name(name).is_err() {
            return Err(unknown_entry(name));
        }
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write entries in original order; patch targets get new bytes.
    let buf = Vec::new();
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(buf));
    for i in 0..ar.len() {
        let (path, method, is_dir) = {
            let e = ar
                .by_index_raw(i)
                .map_err(|e| read_err(format!("bad entry: {e}")))?;
            (e.name().to_string(), e.compression(), e.is_dir())
        };
        match unique.get(path.as_str()) {
            Some(p) => {
                let opts: SimpleFileOptions =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
                zw.start_file(path.clone(), opts)
                    .map_err(|e| read_err(format!("write entry: {e}")))?;
                std::io::copy(&mut p.content.as_bytes(), &mut zw)
                    .map_err(|e| read_err(format!("write failed: {e}")))?;
            }
            None => {
                let opts: SimpleFileOptions =
                    SimpleFileOptions::default().compression_method(method);
                if is_dir {
                    zw.add_directory(path.clone(), opts)
                        .map_err(|er| read_err(format!("write dir: {er}")))?;
                } else {
                    zw.start_file(path.clone(), opts)
                        .map_err(|e| read_err(format!("write entry: {e}")))?;
                    let mut e = ar
                        .by_index(i)
                        .map_err(|err| read_err(format!("copy failed: {err}")))?;
                    std::io::copy(&mut e, &mut zw)
                        .map_err(|err| read_err(format!("copy failed: {err}")))?;
                }
            }
        }
    }
    let bytes = zw
        .finish()
        .map_err(|e| read_err(format!("finish: {e}")))?
        .into_inner();
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

/// Apply decoded-tree text patches (`decoded/<rel-path>`) to a decoded project
/// directory. Every target must be an EXISTING file of the decoded tree (we do
/// not create new project files) and traversal (`..`) is rejected — same
/// sandbox rules as zip entries. Returns the applied count.
pub fn apply_decoded_patches(decoded_dir: &Path, patches: &[Patch]) -> Result<usize, AppError> {
    let mut applied = 0;
    for p in patches {
        let Some(rel) = p.entry_path.strip_prefix("decoded/") else {
            return Err(unknown_entry(&p.entry_path));
        };
        let rel = rel.replace('\\', "/");
        let bad = rel.split('/').any(|c| c == ".." || c.is_empty()) || rel.ends_with('/');
        if bad {
            return Err(AppError::new(
                ErrorCode::InvalidInput,
                format!("decoded patch path rejected: {rel}"),
                "What happened: a decoded edit has an unsafe target path. What to do: revert that edit in Change history.",
            ));
        }
        if rel.starts_with("build/") {
            return Err(AppError::new(
                ErrorCode::InvalidInput,
                "decoded build dir is not editable",
                "What happened: apktool's build/ directory is generated. What to do: edit the source files (res/, assets/, AndroidManifest.xml) instead.",
            ));
        }
        let file = decoded_dir.join(&rel);
        if !file.is_file() {
            return Err(unknown_entry(&p.entry_path));
        }
        std::fs::write(&file, &p.content)?;
        applied += 1;
    }
    Ok(applied)
}

/// The decoded-tree sub-patches of a patch set (`decoded/…` keys).
pub fn split_decoded_patches(patches: &[Patch]) -> (Vec<Patch>, Vec<Patch>) {
    let (dec, zip): (Vec<_>, Vec<_>) = patches
        .iter()
        .cloned()
        .partition(|p| p.entry_path.starts_with("decoded/"));
    (dec, zip)
}

/// Workspace anchor for a project (`resolve_in_project` applies on top).
pub fn workspace_root_for(data_dir: &Path, project_id: &str) -> PathBuf {
    data_dir.join("projects").join(project_id).join("workspace")
}

#[cfg(test)]
mod tests {
    use super::tests_util::make_test_zip;
    use crate::errors::ErrorCode;
    use sha2::Digest as _;
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
    fn search_entries_case_patch_overlay_and_limits() {
        use super::Patch;
        let apk = make_test_zip(&[
            (
                "AndroidManifest.xml",
                "<manifest>\n  <application android:label=\"Old Name\"/>\n</manifest>",
            ),
            (
                "res/values/strings.xml",
                "<string name=\"app_name\">Old Name</string>",
            ),
        ]);
        let dir = tmp_dir("search");
        super::import_apk(&apk, &dir).expect("import");
        let p = dir.join("original.apk");

        let insensitive = super::search_entries(&p, &[], "old name", false).expect("search");
        assert_eq!(insensitive.hits.len(), 2);
        assert_eq!(insensitive.scanned_entries, 2);
        assert_eq!(insensitive.hits[0].entry_path, "AndroidManifest.xml");
        assert_eq!(insensitive.hits[0].line, 2);
        assert!(insensitive.hits[0].snippet.contains("application"));
        assert!(!insensitive.hits[0].patched);

        let sensitive = super::search_entries(&p, &[], "old name", true).expect("search");
        assert_eq!(sensitive.hits.len(), 0);

        let patches = [Patch {
            entry_path: "AndroidManifest.xml".into(),
            content: "<manifest>\n  <application android:label=\"New Name\"/>\n</manifest>".into(),
        }];
        let patched = super::search_entries(&p, &patches, "New Name", true).expect("search");
        assert_eq!(patched.hits.len(), 1);
        assert!(patched.hits[0].patched);
        let stale = super::search_entries(&p, &patches, "Old Name", true).expect("search");
        assert_eq!(stale.hits.len(), 1);
        assert_eq!(stale.hits[0].entry_path, "res/values/strings.xml");

        let empty = super::search_entries(&p, &[], "   ", false).expect_err("empty query");
        assert_eq!(empty.code, ErrorCode::InvalidInput);

        let big = (0..250).map(|_| "needle").collect::<Vec<_>>().join("\n");
        let many = make_test_zip(&[("assets/many.txt", big.as_str())]);
        let capped = super::search_entries(&many, &[], "needle", true).expect("search");
        assert_eq!(capped.hits.len(), super::MAX_SEARCH_HITS);
        assert!(capped.truncated);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn replace_in_entries_overlays_patches_and_is_exact() {
        use super::Patch;
        let apk = make_test_zip(&[
            ("assets/a.xml", "Old Old"),
            ("assets/b.txt", "Old text"),
            ("assets/c.txt", "nothing here"),
        ]);
        let dir = tmp_dir("replace");
        super::import_apk(&apk, &dir).expect("import");
        let p = dir.join("original.apk");

        let patches = [Patch {
            entry_path: "assets/a.xml".into(),
            content: "Old newer".into(),
        }];
        let out = super::replace_in_entries(&p, &patches, "Old", "New").expect("replace");
        assert_eq!(out.result.replacements, 2);
        assert_eq!(out.result.changed_entries.len(), 2);
        assert!(!out.result.truncated);
        let a = out
            .updates
            .iter()
            .find(|u| u.entry_path == "assets/a.xml")
            .expect("a updated");
        assert_eq!(a.content, "New newer");
        let b = out
            .updates
            .iter()
            .find(|u| u.entry_path == "assets/b.txt")
            .expect("b updated");
        assert_eq!(b.content, "New text");

        let miss = super::replace_in_entries(&p, &[], "old", "new").expect("exact case");
        assert_eq!(miss.result.replacements, 0);
        assert!(miss.updates.is_empty());

        let empty = super::replace_in_entries(&p, &[], "", "x").expect_err("empty");
        assert_eq!(empty.code, ErrorCode::InvalidInput);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_decoded_patches_guards_paths() {
        use super::Patch;
        let dir = tmp_dir("dec-ap");
        std::fs::create_dir_all(dir.join("res/values")).unwrap();
        std::fs::write(dir.join("res/values/strings.xml"), "<resources/>").unwrap();
        std::fs::write(dir.join("AndroidManifest.xml"), "<manifest/>").unwrap();
        let patches = vec![
            Patch {
                entry_path: "decoded/AndroidManifest.xml".to_string(),
                content: "<manifest ok/>".to_string(),
            },
            Patch {
                entry_path: "decoded/res/values/strings.xml".to_string(),
                content: "<resources new/>".to_string(),
            },
        ];
        assert_eq!(super::apply_decoded_patches(&dir, &patches).unwrap(), 2);
        assert_eq!(
            std::fs::read_to_string(dir.join("AndroidManifest.xml")).unwrap(),
            "<manifest ok/>"
        );
        let evil = vec![Patch {
            entry_path: "decoded/../evil.txt".to_string(),
            content: "x".to_string(),
        }];
        assert_eq!(
            super::apply_decoded_patches(&dir, &evil).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        let build = vec![Patch {
            entry_path: "decoded/build/secret".to_string(),
            content: "x".to_string(),
        }];
        assert_eq!(
            super::apply_decoded_patches(&dir, &build).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        let missing = vec![Patch {
            entry_path: "decoded/res/nope.xml".to_string(),
            content: "x".to_string(),
        }];
        assert_eq!(
            super::apply_decoded_patches(&dir, &missing)
                .unwrap_err()
                .code,
            ErrorCode::JobNotFound
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn split_decoded_patches_partition() {
        use super::Patch;
        let all = vec![
            Patch {
                entry_path: "assets/a.txt".to_string(),
                content: "1".to_string(),
            },
            Patch {
                entry_path: "decoded/AndroidManifest.xml".to_string(),
                content: "m".to_string(),
            },
        ];
        let (dec, zip) = super::split_decoded_patches(&all);
        assert_eq!(dec.len(), 1);
        assert_eq!(zip.len(), 1);
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
        let out_sha = super::export_apk(&dir.join("original.apk"), &patches, &out).expect("export");
        // New file: different checksum; original untouched.
        assert_ne!(out_sha, orig_sha);
        assert_eq!(super::sha256_file(&apk).unwrap(), orig_sha);
        assert_eq!(
            super::read_entry(&out, "assets/a.xml").unwrap(),
            "patched text"
        );
        assert_eq!(
            super::read_entry(&out, "assets/keep.json").unwrap(),
            "{\"k\":1}"
        );
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
            super::Patch {
                entry_path: "a.xml".into(),
                content: "one".into(),
            },
            super::Patch {
                entry_path: "a.xml".into(),
                content: "two".into(),
            },
        ];
        assert!(super::export_apk(&dir.join("original.apk"), &patches, &out).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_rejects_non_zip_before_copying() {
        let dir = tmp_dir("notzip");
        let bad = dir.join("input.bin");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&bad, b"this is definitely not a ZIP archive").unwrap();
        let err = super::import_apk(&bad, &dir).expect_err("must reject non-zip");
        assert_eq!(err.code, ErrorCode::ImportUnsupportedType);
        // Nothing was copied into the workspace.
        assert!(!dir.join("original.apk").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sha256_file_streams_and_matches_small_hash() {
        let dir = tmp_dir("sha");
        std::fs::create_dir_all(&dir).unwrap();
        let big = dir.join("big.bin");
        std::fs::write(&big, vec![0xAB; 3 * 1024 * 1024 + 7]).unwrap();
        let streamed = super::sha256_file(&big).unwrap();
        // Same digest the eager-read path would produce.
        let eager = {
            let data = std::fs::read(&big).unwrap();
            let mut h = super::Sha256::new();
            h.update(&data);
            format!("{:x}", h.finalize())
        };
        assert_eq!(streamed, eager);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_preserves_compression_and_rejects_partial_output() {
        // 1) A stored (uncompressed) entry must stay stored after export.
        let apk = super::tests_util::make_test_zip_stored(&[("resources.txt", "x")]);
        let dir = tmp_dir("stored");
        super::import_apk(&apk, &dir).unwrap();
        let out = dir.join("out.apk");
        super::export_apk(&dir.join("original.apk"), &[], &out).expect("export");
        let f = std::fs::File::open(&out).unwrap();
        let mut ar = zip::ZipArchive::new(f).unwrap();
        let method = ar.by_name("resources.txt").unwrap().compression();
        assert_eq!(method, zip::CompressionMethod::Stored);
        let _ = std::fs::remove_dir_all(&dir);

        // 2) A bad patch target must not leave a partial output file.
        let dir2 = tmp_dir("partial");
        super::import_apk(&apk, &dir2).unwrap();
        let out2 = dir2.join("out2.apk");
        let patches = vec![super::Patch {
            entry_path: "no/such/entry.xml".into(),
            content: "y".into(),
        }];
        let err = super::export_apk(&dir2.join("original.apk"), &patches, &out2)
            .expect_err("must reject missing target");
        assert_eq!(err.code, ErrorCode::JobNotFound);
        assert!(!out2.exists(), "no partial output may remain");
        let _ = std::fs::remove_dir_all(&dir2);
    }

    #[test]
    fn patch_summary_lists_entries() {
        let s = super::patch_summary(&[super::Patch {
            entry_path: "a.xml".into(),
            content: "x".into(),
        }]);
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
        build_zip(
            entries,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
        )
    }

    /// Build a real ZIP whose entries are STORED (uncompressed) — mirrors
    /// resources.arsc in real APKs (targetSdk ≥ 30 requires Stored + aligned).
    pub fn make_test_zip_stored(entries: &[(&str, &str)]) -> PathBuf {
        build_zip(
            entries,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
        )
    }

    fn build_zip(entries: &[(&str, &str)], opts: SimpleFileOptions) -> PathBuf {
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
        for (name, content) in entries {
            use std::io::Write;
            zw.start_file(*name, opts).expect("start file");
            zw.write_all(content.as_bytes()).expect("write bytes");
        }
        zw.finish().expect("finish zip");
        d
    }
}
