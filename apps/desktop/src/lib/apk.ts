// Typed IPC wrappers for the APK pipeline (Part F1–F5): import → edit →
// export → sign. Errors reject as structured AppError (same `call` helper).
import type {
  ApkReport,
  ApkStatus,
  BuildReport,
  ExportResult,
  OpenedFile,
  PatchSummary,
} from "@apk-forge/shared-types";
import { call } from "./ipc";

export const apkImport = (sourcePath: string) =>
  call<ApkReport>("apk_import", { sourcePath });

export const apkReport = () => call<ApkReport>("apk_report");

export const apkOpenEntry = (entryPath: string) =>
  call<OpenedFile>("apk_open_entry", { entryPath });

export const apkSaveEntry = (entryPath: string, content: string) =>
  call<PatchSummary>("apk_save_entry", { entryPath, content });

export const apkRevertEntry = (entryPath: string) =>
  call<PatchSummary>("apk_revert_entry", { entryPath });

export const apkPatchSummary = () => call<PatchSummary>("apk_patch_summary");

export const apkExport = () => call<ExportResult>("apk_export");

export const apkBuildSign = () => call<BuildReport>("apk_build_sign");

export const apkStatus = () => call<ApkStatus>("apk_status");
