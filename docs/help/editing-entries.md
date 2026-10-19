---
id: editing-entries
title: Editing a Compressed Entry
anchors: editor, patches, revert, diff
phase: 3
---

Opening a file from inside the APK, changing it, and saving turns your change into a reversible patch. This article covers the editor, reverting, the Diff tab, and the Change History section.

## Open an entry in the editor

After importing (see [getting-started](getting-started.md)), the **APK Explorer** sidebar lists every entry in the archive as a tree. Click a file to open it in a tab with breadcrumbs and the Monaco editor: find and replace, multi-cursor, XML syntax tokens, and hover all work offline.

The tree marks entries that can be opened as text by extension (`xml`, `txt`, `json`, `md`, `smali`, `html`, `js`, `csv`). Entries that are binary or not valid UTF-8 - for example the compiled `AndroidManifest.xml` inside a built APK - refuse to open with a clear error such as "entry is not UTF-8 text". That is expected; see [apk-structure](apk-structure.md) for what needs decoding first. Text entries are capped at 8 MiB.

## Saving creates a reversible patch

Edit the text, then press **Save** or `Ctrl+S` (or palette command "Save active tab"). You will see "Edit saved (applied on export)". Nothing is written into the APK yet: the workspace stores a patch (target entry path plus new content) in `patches.json`, keyed to the SHA-256 of the imported original. Until you export or build, the archive on disk is untouched.

An open tab shows a dot for unsaved changes, and a patched entry is marked **edited** in the editor header. The pending-edit counters update across the app.

## Revert one file or all of them

- **One file:** in the editor, **Revert edit** (enabled only for patched entries) drops that patch and restores the original content.
- **Per file in the list:** in Change History, hover a path in **Pending edits** and click its **Revert**.
- **Everything:** the Explorer's **Revert all edits** button (also in the command palette) clears all patches at once. Your workspace returns to the pristine imported state; previously exported or signed files stay until you rebuild.

## Review changes in the Diff tab

The bottom panel's **Diff** tab shows a side-by-side comparison of the committed content and your current draft. Without a draft it stays disabled and explains why ("Edit a file to preview a before/after diff here"). The palette command "Show pending-edit diff" jumps straight to it.

## Change History

The **Change History** section at the bottom of the Explorer lists every pending edit (click a path to reopen it) and your checkpoints. It is the same data as the patch manager - one honest list of what is currently changed.

Related: [checkpoints](checkpoints.md), [apk-structure](apk-structure.md), [build-and-sign](build-and-sign.md)
