---
id: getting-started
title: Importing Your First APK
anchors: import, drag-drop, command-palette, pending-edits
phase: 2
---

CatDot Studio works on a real APK file. This article gets one into the workspace, orients you with the Project Map and Guided Mode, and explains what "pending edits" means before you change anything.

## Three ways to import

- **File picker:** in the APK Explorer sidebar (or on the welcome view), click **Open APK...**. A native file dialog opens; pick any `.apk` file.
- **Drag and drop:** drag an APK file from Explorer onto the CatDot window. The sidebar shows the hint "Drag an APK file here, or type its path."
- **Command palette:** press `Ctrl+Shift+P` (or `Ctrl+K`) and choose **Import APK...**.

All three run the same real import operation. The file is checked for ZIP magic bytes first, so a wrong or corrupt file is rejected before anything is copied. On success the app copies your APK into the project workspace as `original.apk` and reports the entry count.

## Your original is never modified

Import is a copy. Your source file on disk stays untouched, and the workspace copy is treated as read-only for the whole session. The welcome view's **Original APK** card shows a **Read-only - never modified** badge, the file's SHA-256, its size, entry count, and a **Reveal in folder** button. Every change you make later is applied when exporting or building a *new* file.

## Orient with the Project Map

The **Project Map** strip on the welcome view walks you through the five real stages: **Import - Edit - Export - Sign - Install**. It is derived from actual workspace state, not a fixed list, and ends with a **Next:** coach that names the single most useful action right now ("Open an APK to start a project", "You have changes - build, align and sign the APK", and so on). Next to it, the **Build outputs** card lists the exported (unsigned) and signed APKs once they exist.

**Guided Mode** (graduation-cap icon in the activity bar, or palette command "Open Guided Mode") shows the same state as a checklist: Open an APK, Look inside, Make a change, Build, Install. Steps that are not implemented yet carry an honest **Planned - Phase N** badge instead of pretending to work.

## What "pending edits" means

A pending edit is a file you opened, changed, and saved that is stored as a reversible patch but **not yet written into an APK**. The counter appears in the status area, the Project Map, and Guided Mode (for example, "3 pending edits"). Edits are applied when you run **Export unsigned APK** or **Build - align - sign**, and you can discard them individually or all at once from the Explorer. The original APK stays byte-for-byte identical either way.

Related: [editing-entries](editing-entries.md), [build-and-sign](build-and-sign.md), [toolchain-setup](toolchain-setup.md)
