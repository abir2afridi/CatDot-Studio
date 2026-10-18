//! Path sandbox (Part E1).
//! Every UI/AI-requested filesystem operation must go through
//! [`resolve_in_project`], which rejects escapes before touching disk.

use std::path::{Component, Path, PathBuf};

use crate::errors::{AppError, ErrorCode};

/// Reserved Windows device names (case-insensitive, without extension).
const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Maximum relative path length we accept before canonicalization.
/// Windows MAX_PATH is 260 without long-path support; we allow longer
/// but validate, since the manifest enables long paths.
pub const MAX_REL_LEN: usize = 1024;

/// Resolve a user/model-supplied relative path inside `project_root`.
///
/// Rejects: absolute paths, drive letters, `..`, UNC, device paths,
/// alternate data streams (`:`), reserved names, trailing dots/spaces,
/// NUL bytes, and (on existing paths) symlink/junction escapes.
pub fn resolve_in_project(project_root: &Path, rel: &str) -> Result<PathBuf, AppError> {
    if rel.is_empty() {
        return Err(AppError::new(
            ErrorCode::PathTraversalDenied,
            "empty relative path",
            "What happened: an empty file path was supplied. What to do: pick a file inside the project.",
        ));
    }
    if rel.len() > MAX_REL_LEN {
        return Err(denied(format!("relative path too long (>{MAX_REL_LEN})")));
    }
    if rel.contains('\0') {
        return Err(denied("NUL byte in path"));
    }
    // Normalize backslashes for inspection; reject ADS + device paths early.
    let norm = rel.replace('\\', "/");
    if norm.contains(':') {
        return Err(denied("drive letters and ADS (`:`) are not allowed"));
    }
    if norm.starts_with("//") || norm.starts_with("\\\\") {
        return Err(denied("UNC paths are not allowed"));
    }
    if norm.starts_with('/') {
        return Err(denied("absolute paths are not allowed"));
    }
    let p = Path::new(&norm);
    if p.is_absolute() {
        return Err(denied("absolute paths are not allowed"));
    }

    let mut out = PathBuf::new();
    let mut depth: i32 = 0;
    for comp in p.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => {
                return Err(denied("absolute/prefixed paths are not allowed"));
            }
            Component::ParentDir => {
                return Err(denied("`..` escapes the project"));
            }
            Component::CurDir => {}
            Component::Normal(seg) => {
                let s = seg.to_string_lossy();
                validate_segment(&s)?;
                out.push(seg);
                depth += 1;
            }
        }
    }
    if depth == 0 {
        return Err(denied("path resolves to project root itself"));
    }

    let candidate = project_root.join(&out);
    // If the path (or a parent) already exists, canonicalize and verify containment
    // to catch symlink/junction escapes (E1: after resolving symlinks).
    if let Some(existing_ancestor) = nearest_existing_ancestor(&candidate) {
        let canon_ancestor = existing_ancestor
            .canonicalize()
            .map_err(|e| denied(format!("cannot canonicalize ancestor: {e}")))?;
        let canon_root = project_root
            .canonicalize()
            .map_err(|e| denied(format!("cannot canonicalize project root: {e}")))?;
        if !canon_ancestor.starts_with(&canon_root) {
            return Err(denied("symlink/junction escapes project root"));
        }
        // If the full candidate exists, check it too.
        if candidate.exists() {
            if let Ok(canon) = candidate.canonicalize() {
                // Allow the root itself only if rel was "." â€” already rejected above.
                if !canon.starts_with(&canon_root) {
                    return Err(denied("path escapes project root via symlink"));
                }
                return Ok(canon);
            }
        }
    }
    Ok(candidate)
}

fn denied(reason: impl Into<String>) -> AppError {
    let reason = reason.into();
    AppError::new(
        ErrorCode::PathTraversalDenied,
        format!("path denied: {reason}"),
        "What happened: that file path is outside the project. Why: only files inside the project workspace are allowed. What to do: pick a file inside the project.",
    )
    .with_hint(reason)
    .with_help("projects-storage")
}

fn validate_segment(seg: &str) -> Result<(), AppError> {
    if seg.is_empty() || seg == "." || seg == ".." {
        return Err(denied("empty or dot segment"));
    }
    if seg.ends_with('.') || seg.ends_with(' ') {
        return Err(denied(format!("segment `{seg}` has trailing dot/space")));
    }
    // Reserved names: compare stem before first '.'.
    let stem = seg.split('.').next().unwrap_or(seg);
    if RESERVED_NAMES
        .iter()
        .any(|r| r.eq_ignore_ascii_case(stem))
    {
        return Err(denied(format!("reserved device name `{seg}`")));
    }
    if seg.chars().any(|c| c.is_control()) {
        return Err(denied("control character in segment"));
    }
    Ok(())
}

fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut cur: &Path = path;
    loop {
        if cur.exists() {
            return Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) if p.as_os_str().is_empty() => return None,
            Some(p) => cur = p,
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_root(name: &str) -> PathBuf {
        let mut d = std::env::temp_dir();
        d.push(format!("catdot-pathtest-{name}-{}", std::process::id()));
        let _ = fs::create_dir_all(&d);
        d
    }

    #[test]
    fn accepts_normal_relative_paths() {
        let root = tmp_root("ok");
        let r = resolve_in_project(&root, "workspace/decoded/AndroidManifest.xml").unwrap();
        assert!(r.starts_with(&root));
        let r2 = resolve_in_project(&root, "smali/com/example/App.smali").unwrap();
        assert!(r2.starts_with(&root));
    }

    #[test]
    fn rejects_traversal_and_absolute() {
        let root = tmp_root("deny");
        for bad in [
            "../evil.apk",
            "a/../../b",
            "/etc/passwd",
            "C:/Windows/notepad.exe",
            "C:\\Windows\\x",
            r"\\server\share",
            "//server/share",
            "\\\\?\\C:\\x",
            "file:stream",
            "a:b/c",
            "CON",
            "con.txt",
            "NUL",
            "COM1.log",
            "dir/trailing./f",
            "dir/trailing /f",
            "",
            ".",
        ] {
            assert!(
                resolve_in_project(&root, bad).is_err(),
                "should reject `{bad}`"
            );
        }
    }

    #[test]
    fn rejects_backslash_drive_and_unc() {
        let root = tmp_root("unc");
        assert!(resolve_in_project(&root, "D:\\evil").is_err());
        assert!(resolve_in_project(&root, "\\\\.\\pipe").is_err());
    }
}
