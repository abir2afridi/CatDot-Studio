//! Decoded apktool project access (Phase 3): list text sources, read/save-safe
//! paths, and text search over editable smali/XML/resources. The decoded dir is
//! a working copy; edits are recorded as `decoded/<rel>` patches and applied to
//! a FRESH decode at build time (see `workspace::apply_decoded_patches`).

use std::path::{Path, PathBuf};

use crate::errors::{AppError, ErrorCode};
use crate::workspace::{ApkEntry, ApkSearchHit, ApkSearchResult, MAX_SEARCH_HITS, MAX_TEXT_BYTES};

const MAX_DECODED_ENTRIES: usize = 20_000;
/// Bound synchronous scan work so a huge smali tree cannot hang the UI thread.
const MAX_DECODED_SCAN_BYTES: usize = 64 * 1024 * 1024;
const TEXT_EXTENSIONS: &[&str] = &[
    "xml", "smali", "txt", "json", "md", "html", "js", "csv", "java", "kt", "properties", "pro",
    "cfg", "yml", "yaml", "gradle", "css",
];
const SKIP_DIRS: &[&str] = &["build", ".git", ".gradle", "original"];

fn invalid(msg: impl Into<String>, user: &str) -> AppError {
    AppError::new(ErrorCode::InvalidInput, msg.into(), user.to_string())
}

fn not_found(entry_path: &str) -> AppError {
    AppError::new(
        ErrorCode::JobNotFound,
        format!("decoded entry `{entry_path}` not found"),
        "What happened: that decoded source file does not exist. What to do: decode the APK, then pick a file from the Decoded sources tree.",
    )
}

fn text_ext(name: &str) -> bool {
    name.rsplit_once('.')
        .map(|(_, ext)| TEXT_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Validate a `decoded/<rel>` key and return the sandboxed relative path.
pub fn rel_path(entry_path: &str) -> Result<String, AppError> {
    let Some(rel) = entry_path.strip_prefix("decoded/") else {
        return Err(invalid(
            format!("not a decoded path: {entry_path}"),
            "What happened: the file path is not inside the decoded project. What to do: pick a file from Decoded sources.",
        ));
    };
    let rel = rel.replace('\\', "/");
    let bad = rel.is_empty()
        || rel.split('/').any(|c| c == ".." || c.is_empty())
        || rel.ends_with('/')
        || rel.starts_with('/')
        || rel.split('/').next() == Some("build");
    if bad {
        return Err(invalid(
            format!("rejected decoded path: {rel}"),
            "What happened: that path is outside the editable decoded sources. What to do: pick a source file from the tree.",
        ));
    }
    Ok(rel)
}

/// Resolve an existing decoded source file (never created on demand).
pub fn file_of(decoded_dir: &Path, entry_path: &str) -> Result<PathBuf, AppError> {
    let rel = rel_path(entry_path)?;
    let file = decoded_dir.join(rel);
    if !file.is_file() {
        return Err(not_found(entry_path));
    }
    Ok(file)
}

/// List editable text sources as `decoded/<rel>` entries (bounded, sorted).
pub fn list_entries(decoded_dir: &Path) -> Result<Vec<ApkEntry>, AppError> {
    if !decoded_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk(decoded_dir, decoded_dir, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<ApkEntry>) -> Result<(), AppError> {
    if out.len() >= MAX_DECODED_ENTRIES {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk(root, &entry.path(), out)?;
            continue;
        }
        if !ft.is_file() || !text_ext(&name) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| invalid("decoded walk escaped root", "What happened: the decoded tree path was invalid. What to do: decode the APK again."))?
            .to_string_lossy()
            .replace('\\', "/");
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(ApkEntry {
            path: format!("decoded/{rel}"),
            size,
            compressed_size: size,
            is_text: true,
        });
        if out.len() >= MAX_DECODED_ENTRIES {
            return Ok(());
        }
    }
    Ok(())
}

/// Read one decoded text source (≤8 MiB, UTF-8).
pub fn read_entry(decoded_dir: &Path, entry_path: &str) -> Result<String, AppError> {
    let file = file_of(decoded_dir, entry_path)?;
    let size = std::fs::metadata(&file)?.len();
    if size as usize > MAX_TEXT_BYTES {
        return Err(invalid(
            format!("decoded entry too large ({size} bytes)"),
            "What happened: this decoded file is too large to edit as text. What to do: edit a smaller source file.",
        ));
    }
    std::fs::read_to_string(&file).map_err(|_| {
        invalid(
            format!("decoded entry is not UTF-8: {entry_path}"),
            "What happened: this file is binary, not editable text. What to do: pick a text source (smali/XML/etc.).",
        )
    })
}

/// Search decoded text sources (case toggle, 200-hit cap, honest count).
pub fn search(
    decoded_dir: &Path,
    query: &str,
    case_sensitive: bool,
) -> Result<ApkSearchResult, AppError> {
    let needle = query.trim();
    if needle.is_empty() {
        return Err(invalid(
            "search query is empty",
            "What happened: the search box is empty. What to do: type at least one character.",
        ));
    }
    let needle_lower = needle.to_lowercase();
    let entries = list_entries(decoded_dir)?;
    let mut hits = Vec::new();
    let mut scanned_entries = 0usize;
    let mut scanned_bytes = 0usize;
    let mut truncated = false;

    for entry in entries {
        let Ok(content) = read_entry(decoded_dir, &entry.path) else {
            continue;
        };
        scanned_entries += 1;
        scanned_bytes += content.len();
        if scanned_bytes > MAX_DECODED_SCAN_BYTES {
            truncated = true;
            break;
        }
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
                entry_path: entry.path.clone(),
                line: (idx + 1) as u32,
                column: col as u32,
                snippet: line.trim().chars().take(200).collect(),
                patched: false,
            });
            if hits.len() >= MAX_SEARCH_HITS {
                return Ok(ApkSearchResult {
                    hits,
                    truncated: true,
                    scanned_entries,
                });
            }
        }
    }

    Ok(ApkSearchResult {
        hits,
        truncated,
        scanned_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let mut d = std::env::temp_dir();
        d.push(format!(
            "catdot-decoded-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|t| t.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(d.join("res/values")).unwrap();
        std::fs::create_dir_all(d.join("smali/com/app")).unwrap();
        std::fs::create_dir_all(d.join("build/generated")).unwrap();
        std::fs::write(d.join("AndroidManifest.xml"), "<manifest/>").unwrap();
        std::fs::write(
            d.join("res/values/strings.xml"),
            "<resources>\n  <string>Hello world</string>\n</resources>",
        )
        .unwrap();
        std::fs::write(
            d.join("smali/com/app/Main.smali"),
            ".class public Lcom/app/Main;\n# hello from smali\n",
        )
        .unwrap();
        std::fs::write(d.join("build/generated/junk.xml"), "junk").unwrap();
        std::fs::write(d.join("icon.png"), "not text").unwrap();
        d
    }

    #[test]
    fn list_entries_skips_build_and_binary() {
        let dir = tmp_dir("list");
        let entries = list_entries(&dir).expect("list");
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "decoded/AndroidManifest.xml",
                "decoded/res/values/strings.xml",
                "decoded/smali/com/app/Main.smali",
            ]
        );
        assert!(entries.iter().all(|e| e.is_text));
        assert!(list_entries(&dir.join("missing")).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_entry_round_trip_and_path_guards() {
        let dir = tmp_dir("read");
        let text = read_entry(&dir, "decoded/smali/com/app/Main.smali").expect("read");
        assert!(text.contains("hello from smali"));
        assert!(read_entry(&dir, "decoded/../evil").is_err());
        assert!(read_entry(&dir, "decoded/build/generated/junk.xml").is_err());
        assert!(read_entry(&dir, "decoded/res/nope.xml").is_err());
        assert!(read_entry(&dir, "AndroidManifest.xml").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn search_decoded_is_case_aware_and_scanned_is_honest() {
        let dir = tmp_dir("search");
        let insensitive = search(&dir, "hello", false).expect("search");
        assert_eq!(insensitive.hits.len(), 2);
        assert_eq!(insensitive.scanned_entries, 3);
        assert!(insensitive.hits.iter().all(|h| h.entry_path.starts_with("decoded/")));

        let sensitive = search(&dir, "hello", true).expect("search");
        assert_eq!(sensitive.hits.len(), 1);
        assert_eq!(sensitive.hits[0].entry_path, "decoded/smali/com/app/Main.smali");
        let upper = search(&dir, "Hello", true).expect("search");
        assert_eq!(upper.hits.len(), 1);
        assert_eq!(upper.hits[0].entry_path, "decoded/res/values/strings.xml");
        assert!(search(&dir, " ", false).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
