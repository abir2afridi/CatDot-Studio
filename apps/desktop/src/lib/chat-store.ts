// Shared AI chat state (sidebar <-> full page). Lives outside React so a
// stream keeps running while the panel is closed; exactly one real provider
// stream at a time. No fake replies: errors surface verbatim.
import type { AiStreamEvent } from "@catdot/shared-types";
import { aiChatCancel, aiChatStart, onChatEvent } from "./ipc";

export interface ChatLine {
  role: "user" | "assistant";
  text: string;
  done: boolean;
}

export interface ChatSnapshot {
  lines: ChatLine[];
  busy: boolean;
  error: string | null;
}

let snapshot: ChatSnapshot = { lines: [], busy: false, error: null };
const listeners = new Set<() => void>();
let activeStream: string | null = null;
let unlisten: (() => void) | null = null;

function emit(next: ChatSnapshot): void {
  snapshot = next;
  for (const listener of listeners) listener();
}

export function subscribeChat(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getChatSnapshot(): ChatSnapshot {
  return snapshot;
}

function setLast(update: (line: ChatLine) => ChatLine): void {
  const lines = [...snapshot.lines];
  const last = lines[lines.length - 1];
  if (last == null || last.role !== "assistant") return;
  lines[lines.length - 1] = update(last);
  emit({ ...snapshot, lines });
}

function describeError(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function cleanup(): void {
  unlisten?.();
  unlisten = null;
  activeStream = null;
}

export async function sendChat(raw: string): Promise<void> {
  const message = raw.trim();
  if (message === "" || snapshot.busy) return;
  emit({
    ...snapshot,
    busy: true,
    error: null,
    lines: [...snapshot.lines, { role: "user", text: message, done: true }],
  });
  try {
    const streamId = await aiChatStart(message);
    activeStream = streamId;
    emit({
      ...snapshot,
      lines: [...snapshot.lines, { role: "assistant", text: "", done: false }],
    });
    unlisten = await onChatEvent(streamId, (ev: AiStreamEvent) => {
      if (ev.kind === "chunk") {
        setLast((line) => ({ ...line, text: line.text + ev.text }));
      } else if (ev.kind === "done") {
        setLast((line) => ({ ...line, text: ev.text || line.text, done: true }));
        emit({ ...snapshot, busy: false });
        cleanup();
      } else if (ev.kind === "error") {
        emit({ ...snapshot, busy: false, error: ev.message });
        cleanup();
      }
    });
  } catch (e) {
    emit({ ...snapshot, busy: false, error: describeError(e) });
    cleanup();
  }
}

export async function cancelChat(): Promise<void> {
  const id = activeStream;
  if (id == null) return;
  try {
    await aiChatCancel(id);
  } catch (e) {
    emit({ ...snapshot, busy: false, error: describeError(e) });
    cleanup();
  }
}
