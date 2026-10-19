// Reveal a real file/folder in Windows Explorer (Part E4: minimal granted
// permissions — only `opener:allow-reveal-item-in-dir` is allowed).
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { IpcError } from "./ipc";

export async function revealPath(path: string): Promise<void> {
  try {
    await revealItemInDir(path);
  } catch (e) {
    // structured error → surfaced by the caller as a status-bar notice
    throw e instanceof IpcError ? e : new Error(String(e));
  }
}

export const describeError = (e: unknown): string =>
  e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
