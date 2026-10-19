//! Structured edit transformations (Part F4): pure text transforms over the
//! apktool-decoded project files. Each op is a small, unit-tested function;
//! the desktop commands apply them and record the result as ordinary
//! reversible patches.

use regex::Regex;

/// Set/replace an `android:<attr>` attribute on the `<application>` tag in
/// the decoded manifest (literal value). Returns the patched manifest.
pub fn set_application_attr(manifest: &str, attr: &str, value: &str) -> String {
    let pat = format!("android:{}=\"([^\"]*)\"", regex::escape(attr));
    if let Ok(re) = Regex::new(&pat) {
        if re.is_match(manifest) {
            let replacement = format!("android:{attr}=\"{}\"", escape(value));
            return re.replace(manifest, replacement.as_str()).into_owned();
        }
    }
    insert_application_attr(manifest, attr, value)
}

/// Set/replace a `<string name="X">` value in decoded res/values/strings.xml.
pub fn set_string_entry(strings_xml: &str, name: &str, value: &str) -> String {
    let pat = format!("<string name=\"{}\">([^<]*)</string>", regex::escape(name));
    if let Ok(re) = Regex::new(&pat) {
        if re.is_match(strings_xml) {
            let replacement = format!("<string name=\"{}\">{}</string>", name, escape(value));
            return re.replace(strings_xml, replacement.as_str()).into_owned();
        }
    }
    // Insert before </resources> (or as the sole entry in an empty file).
    let entry = format!("    <string name=\"{}\">{}</string>\n", name, escape(value));
    match strings_xml.rfind("</resources>") {
        Some(i) => format!("{}{}{}", &strings_xml[..i], entry, &strings_xml[i..]),
        None => format!("<resources>\n{}</resources>\n", entry),
    }
}

/// Extract the raw `android:label` of `<application>`; Some("@res/x") means a
/// string-resource reference, Some("literal") a literal, None = attribute
/// absent.
pub fn application_label(manifest: &str) -> Option<String> {
    let re = Regex::new("android:label=\"([^\"]*)\"").ok()?;
    re.captures(manifest).map(|c| c[1].to_string())
}

/// Append `<uses-permission android:name="p"/>` when absent. Returns whether
/// the manifest changed.
pub fn add_permission(manifest: &str, perm: &str) -> Option<String> {
    if manifest.contains(&format!("android:name=\"{}\"", escape(perm))) {
        return None;
    }
    let entry = format!("    <uses-permission android:name=\"{}\"/>\n", escape(perm));
    manifest
        .rfind("</manifest>")
        .map(|i| format!("{}{}{}", &manifest[..i], entry, &manifest[i..]))
}

/// Remove the `<uses-permission android:name="p"/>` element. Returns whether
/// the manifest changed.
pub fn remove_permission(manifest: &str, perm: &str) -> Option<String> {
    let re = Regex::new(&format!(
        "\\s*<uses-permission[^>]*android:name=\"{}\"[^>]*/>\\s*\n",
        regex::escape(perm)
    ))
    .ok()?;
    if !re.is_match(manifest) {
        return None;
    }
    Some(re.replace(manifest, "").into_owned())
}

/// Replace versionCode / versionName attributes on the manifest tag.
pub fn set_version(manifest: &str, code: u32, name: Option<&str>) -> String {
    let m = set_top_attr(manifest, "versionCode", &code.to_string());
    match name {
        Some(n) => set_top_attr(&m, "versionName", n),
        None => m,
    }
}

fn set_top_attr(manifest: &str, attr: &str, value: &str) -> String {
    let pat = format!("android:{}=\"([^\"]*)\"", regex::escape(attr));
    if let Ok(re) = Regex::new(&pat) {
        if re.is_match(manifest) {
            let replacement = format!("android:{attr}=\"{}\"", escape(value));
            return re.replace(manifest, replacement.as_str()).into_owned();
        }
    }
    insert_top_attr(manifest, attr, value)
}

/// Insert an attribute on the <manifest> tag (used when apktool dropped it).
fn insert_top_attr(manifest: &str, attr: &str, value: &str) -> String {
    let Some(i) = manifest.find("<manifest") else {
        return manifest.to_string();
    };
    let after = &manifest[i..];
    let Some(end) = after.find('>') else {
        return manifest.to_string();
    };
    let ins = format!(" android:{attr}=\"{}\"", escape(value));
    let at = i + end;
    let mut out = String::with_capacity(manifest.len() + ins.len());
    out.push_str(&manifest[..at]);
    out.push_str(&ins);
    out.push_str(&manifest[at..]);
    out
}

fn insert_application_attr(manifest: &str, attr: &str, value: &str) -> String {
    let Some(i) = manifest.find("<application") else {
        return manifest.to_string();
    };
    let after = &manifest[i..];
    let Some(end) = after.find('>') else {
        return manifest.to_string();
    };
    let ins = format!(" android:{attr}=\"{}\"", escape(value));
    let at = i + end;
    let mut out = String::with_capacity(manifest.len() + ins.len());
    out.push_str(&manifest[..at]);
    out.push_str(&ins);
    out.push_str(&manifest[at..]);
    out
}

fn escape(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.forge.test"
    android:versionCode="1"
    android:versionName="1.0">
    <uses-sdk android:minSdkVersion="24" android:targetSdkVersion="31"/>
    <application android:label="Old Label">
        <activity android:name=".MainActivity"/>
    </application>
</manifest>
"#;

    #[test]
    fn label_replaces_literal_each_call() {
        let m = set_application_attr(MANIFEST, "label", "CatForge X");
        assert!(m.contains("android:label=\"CatForge X\""));
        assert!(!m.contains("Old Label"));
        // Re-running must stay idempotent, not double-insert.
        let m2 = set_application_attr(&m, "label", "CatForge X");
        assert_eq!(m2, m);
    }

    #[test]
    fn label_inserts_when_attribute_missing() {
        let m = "<manifest>\n<application>\n</application>\n</manifest>";
        let out = set_application_attr(m, "label", "New");
        assert!(out.contains("<application"));
        assert!(out.contains("android:label=\"New\""));
    }

    #[test]
    fn string_entry_replaces_and_inserts() {
        let s = "<resources>\n    <string name=\"app_name\">Old</string>\n</resources>\n";
        let out = set_string_entry(s, "app_name", "New");
        assert!(out.contains("<string name=\"app_name\">New</string>"));
        let empty = "<resources>\n</resources>\n";
        let out2 = set_string_entry(empty, "app_name", "New");
        assert!(out2.contains("<string name=\"app_name\">New</string>"));
        assert!(out2.contains("</resources>"));
    }

    #[test]
    fn label_reference_reaches_string_entry() {
        // Label stored as a @string reference: the manifest attr is reported,
        // the actual text lives in strings.xml (both are patched).
        let m = set_application_attr(MANIFEST, "label", "@string/app_name");
        assert!(m.contains("android:label=\"@string/app_name\""));
        let s = set_string_entry(
            "<resources>\n    <string name=\"app_name\">whatever</string>\n</resources>",
            "app_name",
            "CatForge X",
        );
        assert!(s.contains(">CatForge X<"));
    }

    #[test]
    fn permissions_add_once_and_remove() {
        let p = "android.permission.CAMERA";
        let added = add_permission(MANIFEST, p).expect("added");
        assert!(added.contains("<uses-permission android:name=\"android.permission.CAMERA\"/>"));
        assert!(
            add_permission(&added, p).is_none(),
            "second add must be a no-op"
        );
        let removed = remove_permission(&added, p).expect("removed");
        assert!(!removed.contains("CAMERA"));
        assert!(
            remove_permission(&MANIFEST, p).is_none(),
            "absent â†’ no change"
        );
    }

    #[test]
    fn version_replaces_attrs() {
        let m = set_version(MANIFEST, 7, Some("2.5-beta"));
        assert!(m.contains("android:versionCode=\"7\""));
        assert!(m.contains("android:versionName=\"2.5-beta\""));
        let m2 = set_version(&m, 8, None);
        assert!(m2.contains("android:versionCode=\"8\""));
        assert!(m2.contains("android:versionName=\"2.5-beta\""));
    }

    #[test]
    fn escaping_is_applied() {
        let m = set_application_attr(MANIFEST, "label", "A & B <c>");
        assert!(m.contains("A &amp; B &lt;c&gt;"));
    }
}
