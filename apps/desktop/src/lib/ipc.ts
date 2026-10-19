// Typed IPC wrappers (Part D3). All errors surface as the structured `AppError`
// shape; `invoke` rejects with that object, so we normalize it here.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AiStreamEvent,
  AppError,
  LogEntry,
  ProviderCapabilities,
  Setting,
  TermInfo,
  ToolchainReport,
} from "@catdot/shared-types";

export class IpcError extends Error {
  readonly appError: AppError;
  constructor(appError: AppError) {
    super(appError.user_message || appError.message);
    this.name = "IpcError";
    this.appError = appError;
  }
}

export function normalize(e: unknown): IpcError {
  if (e && typeof e === "object" && "code" in e && "user_message" in e) {
    return new IpcError(e as AppError);
  }
  return new IpcError({
    code: "INTERNAL",
    message: String(e),
    user_message: String(e),
    retryable: false,
    hint: null,
    help_article: null,
  });
}

export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw normalize(e);
  }
}

// ---------- Settings ----------
export const settingsGet = (key: string) =>
  call<unknown | null>("settings_get", { key });

export const settingsSet = (key: string, value: unknown, scope = "user") =>
  call<void>("settings_set", { key, value, scope });

export const settingsList = () => call<Setting[]>("settings_list");

// ---------- Secrets ----------
export const secretSet = (account: string, secret: string) =>
  call<void>("secret_set", { account, secret });

export const secretHas = (account: string) =>
  call<boolean>("secret_has", { account });

export const secretDelete = (account: string) =>
  call<void>("secret_delete", { account });

// ---------- Toolchain ----------
export const toolchainDetect = () => call<ToolchainReport>("toolchain_detect");

// ---------- AI ----------
export const aiTestConnection = () =>
  call<ProviderCapabilities>("ai_test_connection");

export const aiChatStart = (message: string) =>
  call<string>("ai_chat_start", { message });

export const aiChatCancel = (streamId: string) =>
  call<void>("ai_chat_cancel", { streamId });

export interface ChatStreamPayload {
  stream_id: string;
  event: AiStreamEvent;
}

export function onChatEvent(
  streamId: string,
  cb: (event: AiStreamEvent) => void,
): Promise<UnlistenFn> {
  return listen<ChatStreamPayload>("ai://chat", (payload) => {
    if (payload.payload.stream_id === streamId) {
      cb(payload.payload.event);
    }
  });
}

// ---------- System logs ----------
export const logsRecent = () => call<LogEntry[]>("logs_recent");

export const logsClear = () => call<void>("logs_clear");

export function onLogEntry(cb: (entry: LogEntry) => void): Promise<UnlistenFn> {
  return listen<LogEntry>("log://entry", (payload) => cb(payload.payload));
}

// ---------- Integrated terminals (real ConPTY; part X-Term) ----------
export const termSpawn = () => call<TermInfo>("term_spawn");

export const termWrite = (id: number, data: string) =>
  call<void>("term_write", { id, data });

export const termResize = (id: number, cols: number, rows: number) =>
  call<void>("term_resize", { id, cols, rows });

export const termClose = (id: number) => call<void>("term_close", { id });

export function onTermOut(cb: (id: number, data: string) => void): Promise<UnlistenFn> {
  return listen<{ id: number; data: string }>("term-out", (p) => cb(p.payload.id, p.payload.data));
}

export function onTermExit(cb: (id: number) => void): Promise<UnlistenFn> {
  return listen<number>("term-exit", (p) => cb(p.payload));
}
