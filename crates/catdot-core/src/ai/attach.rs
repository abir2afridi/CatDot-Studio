//! Read user-picked files/folders as bounded plain-text chat context.
//!
//! Hard caps keep prompts bounded; binaries, oversize files and budget
//! exhaustion are reported through `ChatAttachment::note` instead of being
//! silently dropped. The Tauri command is a thin wrapper over
//! [`read_attachments`].

use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, ErrorCode};

use super::ChatAttachment;

pub const MAX_FILE_BYTES: u64 = 128 * 1024;
pub const MAX_TOTAL_BYTES: usize = 512 * 1024;
pub const MAX_FILES: usize = 40;
pub const MAX_DEPTH: usize = 3;

const SKIP_DIRS: &[&str] = &["node_modules", "target", "dist", "build", ".git", ".gradle"];

fn is_text_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "xml"
            | "json"
            | "txt"
            | "md"
            | "smali"
            | "java"
            | "kt"
            | "properties"
            | "yml"
            | "yaml"
            | "toml"
            | "cfg"
            | "gradle"
            | "pro"
            | "sh"
            | "bat"
            | "ps1"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "css"
            | "scss"
            | "html"
            | "csv"
            | "ini"
            | "conf"
            | "c"
            | "h"
            | "cpp"
            | "rs"
            | "py"
    )
}

fn floor_char_boundary(text: &str, mut index: usize) -> usize {
    if index >= text.len() {
        return text.len();
    }
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn read_text_file(path: &Path, display_name: String, total: &mut usize) -> ChatAttachment {
    let bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let mut attachment = ChatAttachment {
        path: path.to_string_lossy().into_owned(),
        name: display_name,
        bytes,
        text: None,
        truncated: false,
        note: None,
    };
    if bytes > MAX_FILE_BYTES {
        attachment.note = Some(format!(
            "skipped: {} KB exceeds the {} KB per-file limit",
            bytes / 1024,
            MAX_FILE_BYTES / 1024
        ));
        return attachment;
    }
    if *total >= MAX_TOTAL_BYTES {
        attachment.note = Some("skipped: attachment budget exhausted".into());
        return attachment;
    }
    match fs::read(path) {
        Ok(raw) => match String::from_utf8(raw) {
            Ok(text) => {
                let remaining = MAX_TOTAL_BYTES.saturating_sub(*total);
                if text.len() > remaining {
                    let cut = floor_char_boundary(&text, remaining);
                    *total += cut;
                    attachment.text = Some(text[..cut].to_string());
                    attachment.truncated = true;
                    attachment.note = Some("truncated: attachment budget exhausted".into());
                } else {
                    *total += text.len();
                    attachment.text = Some(text);
                }
            }
            Err(_) => {
                attachment.note = Some("skipped: not valid UTF-8 text".into());
            }
        },
        Err(e) => {
            attachment.note = Some(format!("skipped: could not read ({e})"));
        }
    }
    attachment
}

fn collect_file(
    path: &Path,
    display_name: String,
    out: &mut Vec<ChatAttachment>,
    total: &mut usize,
) {
    if out.len() >= MAX_FILES {
        return;
    }
    if !is_text_extension(path) {
        let bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        out.push(ChatAttachment {
            path: path.to_string_lossy().into_owned(),
            name: display_name,
            bytes,
            text: None,
            truncated: false,
            note: Some("skipped: binary or unsupported file type".into()),
        });
        return;
    }
    out.push(read_text_file(path, display_name, total));
}

fn collect_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<ChatAttachment>,
    total: &mut usize,
) {
    if depth > MAX_DEPTH || out.len() >= MAX_FILES {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        if out.len() >= MAX_FILES {
            return;
        }
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if dir_name.starts_with('.') || SKIP_DIRS.contains(&dir_name) {
                continue;
            }
            collect_dir(root, &path, depth + 1, out, total);
        } else {
            collect_file(&path, name, out, total);
        }
    }
}

/// Read the picked paths into chat context. Files are read as UTF-8 text up to
/// the caps above; folders are walked depth-limited and filtered to text
/// extensions. Returns one entry per considered file.
pub fn read_attachments(paths: &[String]) -> Result<Vec<ChatAttachment>, AppError> {
    if paths.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "no attachment paths",
            "What happened: no file or folder was selected. What to do: pick at least one item.",
        ));
    }
    let mut out: Vec<ChatAttachment> = Vec::new();
    let mut total = 0usize;
    for raw in paths {
        let path = PathBuf::from(raw);
        let meta = fs::metadata(&path).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("attachment stat failed: {raw}: {e}"),
                "What happened: an attachment could not be opened. What to do: pick an existing file or folder.",
            )
        })?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment")
            .to_string();
        if meta.is_dir() {
            collect_dir(&path, &path, 0, &mut out, &mut total);
        } else {
            collect_file(&path, name, &mut out, &mut total);
        }
        if out.len() >= MAX_FILES || total >= MAX_TOTAL_BYTES {
            break;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        let mut file = fs::File::create(path).expect("create");
        file.write_all(bytes).expect("write");
    }

    fn paths(list: &[&Path]) -> Vec<String> {
        list.iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn rejects_empty_selection() {
        let err = read_attachments(&[]).expect_err("must fail");
        assert_eq!(err.code, ErrorCode::InvalidInput);
    }

    #[test]
    fn reads_text_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("AndroidManifest.xml");
        write(&file, b"<manifest />");
        let out = read_attachments(&paths(&[&file])).expect("read");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text.as_deref(), Some("<manifest />"));
        assert_eq!(out[0].name, "AndroidManifest.xml");
        assert!(!out[0].truncated);
    }

    #[test]
    fn marks_binary_and_oversize_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let binary = dir.path().join("classes.dex");
        write(&binary, &[0xff, 0xfe, 0x00, 0x01]);
        let huge = dir.path().join("big.txt");
        write(&huge, &vec![b'a'; (MAX_FILE_BYTES + 10) as usize]);
        let out = read_attachments(&paths(&[&binary, &huge])).expect("read");
        assert_eq!(out.len(), 2);
        assert!(out[0].text.is_none());
        assert!(out[0].note.as_deref().unwrap_or("").contains("unsupported"));
        assert!(out[1].text.is_none());
        assert!(out[1].note.as_deref().unwrap_or("").contains("per-file limit"));
    }

    #[test]
    fn walks_folders_depth_limited_and_skips_noise() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("res");
        write(&root.join("values/strings.xml"), b"<resources />");
        write(&root.join("node_modules/ignored.txt"), b"nope");
        write(&root.join("a/b/c/d/deep.txt"), b"too deep");
        let out = read_attachments(&paths(&[&root])).expect("read");
        let names: Vec<&str> = out.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"values/strings.xml"));
        assert!(!names.iter().any(|n| n.contains("node_modules")));
        assert!(!names.iter().any(|n| n.contains("deep.txt")));
    }

    #[test]
    fn truncates_on_total_budget() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut files = Vec::new();
        for i in 0..10 {
            let file = dir.path().join(format!("f{i}.txt"));
            write(&file, &vec![b'x'; 100 * 1024]);
            files.push(file);
        }
        let refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
        let out = read_attachments(&paths(&refs)).expect("read");
        let total: usize = out
            .iter()
            .filter_map(|a| a.text.as_ref().map(String::len))
            .sum();
        assert!(total <= MAX_TOTAL_BYTES);
        assert!(out.iter().any(|a| a.truncated || a.note.is_some()));
    }

    #[test]
    fn utf8_boundary_truncation_is_safe() {
        let text = "বাংলা";
        let cut = floor_char_boundary(text, 4);
        assert!(text.is_char_boundary(cut));
        assert_eq!(&text[..cut], "ব");
    }
}
