---
id: checkpoints
title: Checkpoints and Change History
anchors: checkpoints, restore-point, snapshots, change-history
phase: 3
---

Checkpoints are named snapshots of your pending edits. They let you return to an earlier state of your work without touching the original APK. This article explains what a checkpoint stores, how to save one, what happens automatically before a build, and the rules that keep restores safe.

## What a checkpoint is

A checkpoint captures the **set of pending patches** at the moment you save it: which entries are changed and what their new content is, plus the SHA-256 of the imported original the patches belong to. Checkpoints are plain JSON files written to the `checkpoints/` folder inside your project workspace - they are deliberately simple snapshots, not a Git repository.

Each entry in the list shows a label (for example "Checkpoint saved (3 edits)"), the creation time, the number of edits, and an **auto** badge if it was created automatically.

## Save a restore point

Open the **Change History** section at the bottom of the APK Explorer and click **Save restore point**, or run the checkpoint command from the command palette (`Ctrl+Shift+P`). The current patch set is snapshotted immediately. Save one whenever you are about to try something risky: you can always come back to exactly this state.

## Automatic checkpoint before a build

When you run **Build - align - sign** while you have pending edits, the app first writes an automatic checkpoint labeled `Auto - before build - N edits`, unless the newest checkpoint already stores byte-identical content. So every build that could change your output is preceded by a restorable snapshot - no extra clicks needed.

## Restore rules and base-hash safety

**Restore** replaces your current pending edits with the patch set stored in that checkpoint. Important guarantees:

- The original APK is never rewritten. Restore only changes which patches are pending.
- Each checkpoint records the hash of the APK it was made for. If you have since imported a *different* APK, restore is refused with "checkpoint base hash mismatch" and the advice to import the matching APK again. You cannot accidentally apply edits to the wrong base file.
- Exported and signed files already on disk are not deleted by a restore; rebuild to regenerate them.

Restoring a checkpoint whose patches equal your current ones simply reports how many edits are active.

## Delete a checkpoint

Hover a checkpoint and click **Delete**. Only the snapshot file is removed - your pending edits and your outputs are unaffected. A confirmation notice shows how many checkpoints remain.

Related: [editing-entries](editing-entries.md), [build-and-sign](build-and-sign.md), [getting-started](getting-started.md)
