---
id: apk-structure
title: APK Anatomy - What Is Inside and What You Can Edit
anchors: apk-anatomy, dex, resources, manifest
phase: 2
---

An APK looks complicated, but it is just a ZIP archive with a specific layout. Knowing what each part is tells you what you can safely change today and what needs decoding tools first.

## What an APK actually is

Open any APK in the Explorer tree and you will see the standard parts:

- **`classes.dex`** - compiled Dalvik bytecode: the app's logic. Multiple `classesN.dex` files can exist.
- **`resources.arsc`** - the compiled resource table: string IDs, colors, and the mapping from `res/` references to values. Stored uncompressed so it can be memory-mapped.
- **`AndroidManifest.xml`** - the app's manifest (package name, components, permissions, version) in Android's **compiled binary XML** format, not plain text.
- **`res/`** - resources: layouts, drawables, values. In a built APK these are also compiled (binary XML, compiled PNGs).
- **`assets/`** - raw files the app reads at runtime (fonts, configs, HTML, JSON). Kept as-is - this is the friendliest place to make text changes.
- **`lib/`** - native `.so` libraries per ABI (`arm64-v8a`, `armeabi-v7a`, `x86_64`).
- **`META-INF/`** - the signature (`MANIFEST.MF`, `CERT.SF`, `CERT.RSA`) that makes the APK installable.

## What is editable as text today

The editor opens any entry that is valid UTF-8 text under 8 MiB. Entries are marked as text by extension: `xml`, `txt`, `json`, `md`, `smali`, `html`, `js`, `csv`. In practice that means:

- text files in **`assets/`** (JSON, HTML, JS, plain text),
- text files under **`META-INF/`** that are not the binary signature blocks,
- and any other genuinely textual entry.

Saving creates a reversible patch; nothing changes until you export or build - see [editing-entries](editing-entries.md).

If you click a binary entry you get a clear error such as "entry is not UTF-8 text" instead of mojibake. That includes `AndroidManifest.xml` and `res/` XML in a *built* APK: they are compiled binary XML even though they end in `.xml`.

## What needs decoding first

These parts cannot be edited as text in the current release:

- **Binary manifest and resources** - need `apktool` decode/encode.
- **`resources.arsc` and compiled resources** - need resource decoding.
- **`classes.dex`** - needs decompiling (JADX, for reading) and smali tooling to change.

apktool/JADX decode plus smali editing is **planned (Phase 3)** and currently disabled - Guided Mode and the feature list show it with an honest "Planned" badge. The structured **manifest analyzer** is likewise not yet available; this article is the anatomy reference until it lands.

## Reading structure in the app

The Explorer tree shows every entry with folder hierarchy; the APK report shows entry count, total and uncompressed size, and the SHA-256 of the original. Use it to confirm what you are about to touch.

Related: [editing-entries](editing-entries.md), [getting-started](getting-started.md), [toolchain-setup](toolchain-setup.md)
