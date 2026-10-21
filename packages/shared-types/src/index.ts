// Single source of IPC types (Part D2). These are re-exported from the
// ts-rs–generated bindings in `./generated` (generated from the Rust core by
// `cargo test` — see ADR for the export pipeline). Never hand-edit types here;
// add a `#[derive(TS)]` on the Rust side and re-export the generated file.

export type { AccessLevel } from "./generated/AccessLevel";
export type { AiStreamEvent } from "./generated/AiStreamEvent";
export type { ApkEntry } from "./generated/ApkEntry";
export type { ApkReport } from "./generated/ApkReport";
export type { ApkReplaceResult } from "./generated/ApkReplaceResult";
export type { ApkSearchHit } from "./generated/ApkSearchHit";
export type { ApkSearchResult } from "./generated/ApkSearchResult";
export type { ApkStatus } from "./generated/ApkStatus";
export type { AppError } from "./generated/AppError";
export type { Autonomy } from "./generated/Autonomy";
export type { BuildReport } from "./generated/BuildReport";
export type { ChatAttachment } from "./generated/ChatAttachment";
export type { ChatMessage } from "./generated/ChatMessage";
export type { ChatUsage } from "./generated/ChatUsage";
export type { CheckpointInfo } from "./generated/CheckpointInfo";
export type { DeviceInfo } from "./generated/DeviceInfo";
export type { ErrorCode } from "./generated/ErrorCode";
export type { ExportResult } from "./generated/ExportResult";
export type { JobEvent } from "./generated/JobEvent";
export type { JobState } from "./generated/JobState";
export type { LogEntry } from "./generated/LogEntry";
export type { ManifestComponent } from "./generated/ManifestComponent";
export type { ManifestInfo } from "./generated/ManifestInfo";
export type { OpenAICompatConfig } from "./generated/OpenAICompatConfig";
export type { OpenedFile } from "./generated/OpenedFile";
export type { Patch } from "./generated/Patch";
export type { PatchSummary } from "./generated/PatchSummary";
export type { ProviderCapabilities } from "./generated/ProviderCapabilities";
export type { RiskTier } from "./generated/RiskTier";
export type { Setting } from "./generated/Setting";
export type { SignStep } from "./generated/SignStep";
export type { TermInfo } from "./generated/TermInfo";
export type { ToolInfo } from "./generated/ToolInfo";
export type { ToolchainReport } from "./generated/ToolchainReport";
export type { JsonValue } from "./generated/serde_json/JsonValue";
