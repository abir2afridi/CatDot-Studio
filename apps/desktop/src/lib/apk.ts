// Typed IPC wrappers for the APK pipeline (Part F1–F5): import → edit →
// export → sign, checkpoints (F4) and device steps (F7).
import type {
  ApkReport,
  DecodedSources,
  DecodeStatus,
  ApkReplaceResult,
  ApkSearchResult,
  ApkStatus,
  BuildReport,
  CheckpointInfo,
  DeviceInfo,
  ExportResult,
  ManifestInfo,
  OpenedFile,
  PatchSummary,
} from "@catdot/shared-types";
import { call } from "./ipc";

export const apkImport = (sourcePath: string) =>
  call<ApkReport>("apk_import", { sourcePath });

/** Re-attach the on-disk workspace session after an app restart. */
export const apkReopen = () => call<boolean>("apk_reopen");

export const apkManifest = () => call<ManifestInfo>("apk_manifest");

// Structured edit ops (Guided recipes / decode pipeline).
export const apkDecode = () => call<string>("apk_decode");

/** Background decode with real progress polling + cancellation. */
export const apkDecodeStart = () => call<DecodeStatus>("apk_decode_start");
export const apkDecodeStatus = () => call<DecodeStatus>("apk_decode_status");
export const apkDecodeCancel = () => call<DecodeStatus>("apk_decode_cancel");

export const apkSetLabel = (label: string) =>
  call<PatchSummary>("apk_set_label", { label });

export const apkTogglePermission = (perm: string, add: boolean) =>
  call<PatchSummary>("apk_toggle_permission", { perm, add });

export const apkSetVersion = (code: number, name?: string | null) =>
  call<PatchSummary>("apk_set_version", { code, name: name ?? null });

export const apkReport = () => call<ApkReport>("apk_report");

/** Editable text sources of the decoded apktool project (empty before decode). */
export const apkDecodedEntries = () => call<DecodedSources>("apk_decoded_entries");

export const apkOpenEntry = (entryPath: string) =>
  call<OpenedFile>("apk_open_entry", { entryPath });

export const apkSaveEntry = (entryPath: string, content: string) =>
  call<PatchSummary>("apk_save_entry", { entryPath, content });

export const apkRevertEntry = (entryPath: string) =>
  call<PatchSummary>("apk_revert_entry", { entryPath });

export const apkRevertAll = () => call<PatchSummary>("apk_revert_all");

export const apkPatchSummary = () => call<PatchSummary>("apk_patch_summary");

export const apkPatchPaths = () => call<string[]>("apk_patch_paths");

/** Full-text search over readable entries (pending patches included). */
export const apkSearch = (query: string, caseSensitive: boolean) =>
  call<ApkSearchResult>("apk_search", { query, caseSensitive });

/** Replace an exact string across readable entries; every change is a patch. */
export const apkReplaceAll = (query: string, replacement: string) =>
  call<ApkReplaceResult>("apk_replace_all", { query, replacement });

export const apkExport = () => call<ExportResult>("apk_export");

export const apkBuildSign = () => call<BuildReport>("apk_build_sign");

export const apkStatus = () => call<ApkStatus>("apk_status");

// Part F4 change history (checkpoints — patch snapshots, not Git).
export const checkpointCreate = (label?: string) =>
  call<CheckpointInfo>("apk_checkpoint_create", { label: label ?? null });

export const checkpointList = () => call<CheckpointInfo[]>("apk_checkpoint_list");

export const checkpointRestore = (id: string) =>
  call<PatchSummary>("apk_checkpoint_restore", { id });

export const checkpointDelete = (id: string) =>
  call<number>("apk_checkpoint_delete", { id });

// Part F7 device steps (real adb; honest errors when no device/adb).
// A serial selects an explicit device; null → first ready device.
export const deviceList = () => call<DeviceInfo[]>("device_list");

export const deviceInstallSigned = (serial?: string | null) =>
  call<string>("device_install_signed", { serial: serial ?? null });

export const deviceLogcat = (lines?: number, serial?: string | null) =>
  call<string>("device_logcat", { lines: lines ?? null, serial: serial ?? null });
