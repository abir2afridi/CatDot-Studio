//! APK inspection (Phase 2 slice): structured AndroidManifest analysis via
//! `aapt2 dump xmltree`. No decode needed — this reads the real manifest contents
//! straight out of any APK, and IT is also the badging ground-truth helper
//! used by build verification.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};
use crate::tools::build_tools::run_timeout;
use crate::tools::detect::detect_toolchain;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ManifestComponent {
    /// element kind: activity | service | receiver | provider
    pub kind: String,
    pub name: Option<String>,
    pub exported: Option<String>,
    #[ts(type = "string[]")]
    pub actions: Vec<String>,
    #[ts(type = "string[]")]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ManifestInfo {
    pub package: Option<String>,
    pub label: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<String>,
    pub min_sdk: Option<String>,
    pub target_sdk: Option<String>,
    #[ts(type = "string[]")]
    pub permissions: Vec<String>,
    pub components: Vec<ManifestComponent>,
}

#[derive(Debug, Default)]
struct Elem {
    name: String,
    attrs: HashMap<String, String>,
    children: Vec<Elem>,
}

/// Run `aapt2 dump xmltree <apk> --file AndroidManifest.xml` and fold it into
/// a structured ManifestInfo.
pub fn manifest_info(apk: &Path) -> Result<ManifestInfo, AppError> {
    let t = detect_toolchain();
    let aapt2 = t
        .aapt2
        .path
        .as_ref()
        .filter(|p| Path::new(p).is_file())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::ToolNotFound,
                "aapt2 not installed",
                "What happened: aapt2 (the Android build inspector) was not found. What to do: install the Android SDK build tools and run System check.",
            )
        })?;
    let run = run_timeout(
        Path::new(aapt2),
        &[
            "dump",
            "xmltree",
            &apk.to_string_lossy(),
            "--file",
            "AndroidManifest.xml",
        ],
        None,
        Duration::from_secs(120),
    )
    .ok_or_else(|| {
        AppError::new(
            ErrorCode::Internal,
            "aapt2 xmltree timed out",
            "What happened: manifest analysis took too long. What to do: retry.",
        )
    })?;
    if run.code != 0 || run.out.is_empty() {
        return Err(AppError::new(
            ErrorCode::Internal,
            format!("aapt2 xmltree failed: {}", run.out.lines().next().unwrap_or("")),
            "What happened: the manifest could not be analyzed (APK may be corrupt). What to do: re-import the APK, then retry.",
        ));
    }
    Ok(manifest_from_xmltree(&run.out))
}

/// Parse the text tree (used by the offline fixture test too). Attribute keys
/// keep a short `android:` prefix; values are unquoted.
pub fn manifest_from_xmltree(out: &str) -> ManifestInfo {
    let mut stack: Vec<(usize, Elem)> = Vec::new();
    let mut root_children: Vec<Elem> = Vec::new();
    for line in out.lines() {
        let stripped = line.trim_start();
        let indent = line.len() - stripped.len();
        let level = indent / 2;
        if let Some(rest) = stripped.strip_prefix("E: ") {
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string();
            attach_or_push(
                &mut stack,
                level,
                Elem {
                    name,
                    ..Elem::default()
                },
                &mut root_children,
            );
        } else if let Some(rest) = stripped.strip_prefix("A: ") {
            if let Some((k, v)) = split_attr(rest) {
                let key = if k.starts_with("http://schemas.android.com/apk/res/android:") {
                    format!(
                        "android:{}",
                        k.trim_start_matches("http://schemas.android.com/apk/res/android:")
                    )
                } else {
                    k
                };
                if let Some(top) = stack.last_mut() {
                    top.1.attrs.insert(key, v);
                }
            }
        }
    }
    // Close any remaining open elements (manifest is always still here).
    while let Some((_, e)) = stack.pop() {
        if let Some(top) = stack.last_mut() {
            top.1.children.push(e);
        } else {
            root_children.push(e);
        }
    }
    extract_from(&root_children)
}

fn attach_or_push(stack: &mut Vec<(usize, Elem)>, level: usize, e: Elem, root: &mut Vec<Elem>) {
    while let Some((l, _)) = stack.last() {
        if *l < level {
            break;
        }
        let (_, parent) = stack.pop().expect("pop guard");
        if let Some(top) = stack.last_mut() {
            top.1.children.push(parent);
        } else {
            root.push(parent);
        }
    }
    stack.push((level, e));
}

fn split_attr(rest: &str) -> Option<(String, String)> {
    let eq = rest.find('=')?;
    let mut key = rest[..eq].trim().to_string();
    // aapt2 appends the resource identifier: `…:versionCode(0x0101021b)`.
    if key.ends_with(')') {
        if let Some(i) = key.rfind('(') {
            key.truncate(i);
        }
    }
    let raw = rest[eq + 1..].trim();
    let value = if raw.starts_with('"') {
        raw.trim_start_matches('"')
            .split('"')
            .next()
            .unwrap_or_default()
            .to_string()
    } else {
        raw.split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string()
    };
    Some((key, value))
}

fn extract_from(root: &[Elem]) -> ManifestInfo {
    let mut info = ManifestInfo {
        package: None,
        label: None,
        version_name: None,
        version_code: None,
        min_sdk: None,
        target_sdk: None,
        permissions: Vec::new(),
        components: Vec::new(),
    };
    let Some(manifest) = root.iter().find(|e| e.name == "manifest") else {
        return info;
    };
    info.package = manifest.attrs.get("package").cloned();
    info.version_name = manifest.attrs.get("android:versionName").cloned();
    info.version_code = manifest.attrs.get("android:versionCode").cloned();
    for pchild in &manifest.children {
        if pchild.name == "uses-sdk" {
            info.min_sdk = pchild.attrs.get("android:minSdkVersion").cloned();
            info.target_sdk = pchild.attrs.get("android:targetSdkVersion").cloned();
        }
        if pchild.name == "uses-permission" {
            if let Some(p) = pchild.attrs.get("android:name") {
                info.permissions.push(p.clone());
            }
        }
        if pchild.name == "application" {
            info.label = pchild.attrs.get("android:label").cloned();
            for child in &pchild.children {
                if matches!(
                    child.name.as_str(),
                    "activity" | "service" | "receiver" | "provider"
                ) {
                    let mut comp = ManifestComponent {
                        kind: child.name.clone(),
                        name: child.attrs.get("android:name").cloned(),
                        exported: child.attrs.get("android:exported").cloned(),
                        actions: Vec::new(),
                        categories: Vec::new(),
                    };
                    for csub in &child.children {
                        if csub.name == "intent-filter" {
                            for isub in &csub.children {
                                if isub.name == "action" {
                                    if let Some(a) = isub.attrs.get("android:name") {
                                        comp.actions.push(a.clone());
                                    }
                                }
                                if isub.name == "category" {
                                    if let Some(a) = isub.attrs.get("android:name") {
                                        comp.categories.push(a.clone());
                                    }
                                }
                            }
                        }
                    }
                    info.components.push(comp);
                }
            }
        }
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_aapt2_xmltree_fixture() {
        // Captured from `aapt2 dump xmltree` on a real signed APK built by the
        // pipeline (manifest package com.forge.test, label CatForge X).
        let raw = include_str!("../../../../tests/fixtures/tool-output/sample-xmltree.txt");
        let m = manifest_from_xmltree(raw);
        assert_eq!(m.package.as_deref(), Some("com.forge.test"));
        assert_eq!(m.version_name.as_deref(), Some("1.0"));
        assert_eq!(m.version_code.as_deref(), Some("1"));
        assert_eq!(m.min_sdk.as_deref(), Some("24"));
        assert_eq!(m.target_sdk.as_deref(), Some("31"));
        assert_eq!(m.label.as_deref(), Some("CatForge X"));
        assert!(m.permissions.is_empty());
        assert!(m.components.is_empty()); // the fixture app has no components
    }

    #[test]
    fn parses_components_and_permissions() {
        let raw = r#"N: android=http://schemas.android.com/apk/res/android (line=2)
  E: manifest (line=2)
    A: package="com.demo"
    A: http://schemas.android.com/apk/res/android:versionName(0x0101021c)="2.3"
      E: uses-permission (line=0)
        A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.permission.INTERNET"
      E: application (line=4)
        A: http://schemas.android.com/apk/res/android:label(0x01010001)="Demo"
        E: activity (line=5)
          A: http://schemas.android.com/apk/res/android:name(0x01010003)="com.demo.MainActivity"
          A: http://schemas.android.com/apk/res/android:exported(0x01010003)="true"
          E: intent-filter (line=6)
            E: action (line=7)
              A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.intent.action.MAIN"
            E: category (line=8)
              A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.intent.category.LAUNCHER"
"#;
        let m = manifest_from_xmltree(raw);
        assert_eq!(
            m.permissions,
            vec!["android.permission.INTERNET".to_string()]
        );
        assert_eq!(m.components.len(), 1);
        let c = &m.components[0];
        assert_eq!(c.kind, "activity");
        assert_eq!(c.name.as_deref(), Some("com.demo.MainActivity"));
        assert_eq!(c.exported.as_deref(), Some("true"));
        assert_eq!(
            m.components[0].actions,
            vec!["android.intent.action.MAIN".to_string()]
        );
        assert!(m.components[0]
            .categories
            .contains(&"android.intent.category.LAUNCHER".to_string()));
        assert_eq!(m.label.as_deref(), Some("Demo"));
    }
}
