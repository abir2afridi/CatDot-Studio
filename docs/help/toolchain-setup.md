---
id: toolchain-setup
title: Toolchain and System Check
anchors: system-check, platform-tools, build-tools, jdk
phase: 0/4
---

CatDot Studio does not bundle the Android SDK: it detects the tools already on your machine and uses them. The **System check** page is where you see what was found and what is missing.

## What System Check detects

Open **System check** from the activity bar (or the palette) and press **Detect tools**. Detection is read-only - nothing is installed, changed, or downloaded. You get a row per tool with its status (**Found** / **Not found**), version, exact path, a one-line purpose, and a **How to fix** hint when absent:

- **java** - runs apktool, JADX, and aapt2; needs a JDK 17 or newer.
- **aapt2** - reads APK metadata (used for summaries and by the build pipeline).
- **zipalign** - aligns APK entries before signing.
- **apksigner** - signs and verifies APKs.
- **adb** - installs apps and reads device logs.
- **emulator** - runs an Android virtual device.
- **apktool** - decodes and rebuilds APK resources (decode is planned, see below).
- **jadx** - decompiles DEX to readable Java (reference only).

The page also reports Windows long-path support.

## Where the tools come from

Build-tools (`aapt2`, `zipalign`, `apksigner`) and `adb`/`emulator` are located in your Android SDK: the app checks `ANDROID_SDK_ROOT`, then `ANDROID_HOME`, then the default Windows location `%LOCALAPPDATA%\Android\Sdk`. For build-tools it picks the **newest installed version**; for `adb` it uses `platform-tools\adb.exe`. A PATH fallback covers `adb`, `apktool`, and `jadx`.

Getting an SDK: install Android Studio (its SDK manager downloads platform-tools and build-tools), or use `sdkmanager` from command-line tools and accept Google's license. A consent-based in-app installer with checksums is planned for Phase 4 - today detection only reports; it never installs silently.

## The JDK note

Signing needs `keytool`, which lives next to `java`. The detector looks at `JAVA_HOME`, then Android Studio's bundled JBR, then PATH. Any JDK 17+ works. Without a JDK, **Build - align - sign** fails with a "keytool not found" error telling you to install a JDK - there is no silent fallback.

## Which tools you need for what

- **Import, edit, export:** no external tools at all - the workspace engine handles them.
- **Build - align - sign:** `zipalign`, `apksigner`, `keytool`.
- **Install and device logs:** `adb`.
- **apktool / JADX decode and smali editing:** planned, not yet available (Phase 3) - System Check reports their presence, but no feature calls them yet.

Related: [build-and-sign](build-and-sign.md), [install-and-logs](install-and-logs.md), [apk-structure](apk-structure.md)
