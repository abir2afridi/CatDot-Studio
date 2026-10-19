---
id: build-and-sign
title: Build, Align, and Sign
anchors: zipalign, apksigner, debug-keystore, signing
phase: 4
---

**Build - align - sign** turns your patched workspace into an installable APK. Every step is a real Android tool invocation with captured output, and the app verifies its own work before calling the build good.

## What the pipeline does

Click **Build - align - sign** in the Explorer, the Project Map, Guided Mode, or the command palette. The app then runs, in order:

1. **Export** - writes a new `unsigned.apk` with your pending patches applied. Your original stays untouched.
2. **zipalign -f 4** - aligns uncompressed data in the archive on 4-byte boundaries so the Android runtime can memory-map resources efficiently.
3. **zipalign -c 4** - a real alignment *check*. The report only shows **Alignment verified** if this passes.
4. **apksigner sign** - signs the aligned APK with a keystore.
5. **apksigner verify** - confirms the signature. The report shows **Signature verified** only on success.

The bottom panel's **Build** tab lists every step with its exit code and captured output, plus the paths of the unsigned, aligned, and signed files. Failures also land in the **Logs** tab with the same plain-language error.

## The debug keystore

The first build automatically creates a debug keystore under the app data folder (`keys/catdot-debug.jks`, alias `catdot`) using the `keytool` that ships with your JDK. The passphrase is fixed and embedded - that is normal for a debug keystore and fine for apps you install on your own devices. A dedicated keystore manager for release keystores is planned for Phase 4; never treat the debug keystore as production material.

Because your APK is re-signed with a different key than the original, it cannot update the original installation from Google Play; Android treats them as different apps.

## Where the outputs land

All outputs are written into the project workspace next to `original.apk`: `exported.apk` (unsigned export), then `unsigned.apk`, `aligned.apk`, and `signed.apk`. The **Build outputs** card on the welcome view and the palette commands "Reveal original APK in folder" / "Reveal signed APK in folder" open the containing folder for you.

## Why verification matters

Android refuses to install an APK with a missing or broken signature, and misaligned archives cause runtime problems that are hard to debug. Verifying both steps here means an install failure is not a mystery.

## Install after build

With `signed.apk` present, use **Install on device** (Project Map, palette, or Devices page) to push it over adb - see [install-and-logs](install-and-logs.md).

Related: [install-and-logs](install-and-logs.md), [toolchain-setup](toolchain-setup.md), [checkpoints](checkpoints.md)
