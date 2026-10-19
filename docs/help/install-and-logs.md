---
id: install-and-logs
title: Devices, Install, and Logs
anchors: adb, device-install, logcat, usb-debugging
phase: 4
---

CatDot Studio installs the signed APK on a real device or emulator with `adb`, and reads device logs back. This article covers picking a target device, installing, the USB debugging prompt, and the Device logs tab.

## Pick a target device

Open the **Devices** page from the activity bar. The page runs a real `adb devices -l` (auto-refresh plus a manual **Refresh** button) and lists everything attached, with state chips: **ready**, **unauthorized**, or **offline**. Click **Use this device** to select a target; the page header then shows `Target: <serial>`. With no explicit selection, actions go to the first ready device (`Target: first ready device`).

If nothing is attached you get an honest "No devices or emulators are connected" plus the hint to start an emulator or plug in a phone with USB debugging enabled - never a fake success.

## Install the signed APK

Click **Install signed APK** on a device row, or **Install on device** on the Project Map, or the install step in Guided Mode, or the palette command. All of them run `adb install -r` of the `signed.apk` from your last build.

Honest error states you may see:

- **No signed build yet** - build first; the app tells you to run **Build - align - sign**.
- **No device connected** - start an emulator or connect a phone, then refresh.
- **Unauthorized** - your computer has not been trusted yet; accept the prompt on the device (below).
- **Not ready / offline** - the device is still booting; wait and retry.
- **INSTALL_FAILED_UPDATE_INCOMPATIBLE** (signatures do not match) - the installed app was signed with a different key. Uninstalling the old app usually fixes it; its data will be lost.
- **INSTALL_FAILED_VERSION_DOWNGRADE**, **INSTALL_FAILED_NO_MATCHING_ABIS**, **INSTALL_FAILED_OLDER_SDK** - version or architecture mismatches, each with its own plain-language explanation.

## Enable USB debugging

On the phone: Settings -> About phone -> tap **Build number** seven times to unlock Developer options, then Settings -> Developer options -> enable **USB debugging**. Connect the cable and watch for the **Allow USB debugging?** RSA fingerprint prompt - tap **Allow** (optionally "always from this computer"). Until you accept, the device shows as `unauthorized` and installs are refused. Emulators need no prompt.

## Read device logs

The **Device** tab in the bottom panel (or **Open device logs** on the Devices page) shows a bounded dump from `adb logcat -d` - recent lines only, with a **Lines** limit and a manual **Refresh**. It is not a live tail: press Refresh after reproducing the problem. No device attached, an empty buffer, and timeouts are reported as such.

Related: [build-and-sign](build-and-sign.md), [toolchain-setup](toolchain-setup.md), [getting-started](getting-started.md)
