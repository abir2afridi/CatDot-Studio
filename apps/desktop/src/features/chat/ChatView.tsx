// Chat view container: shared conversation + attachments + toolbar/composer.
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import type { ChatAttachment } from "@catdot/shared-types";
import { aiReadAttachments, IpcError } from "../../lib/ipc";
import { useUi } from "../../state/ui";
import { AttachmentChips } from "./AttachmentChips";
import { ChatComposer } from "./ChatComposer";
import { ChatToolbar } from "./ChatToolbar";
import { cancelChat, getChatSnapshot, newChat, sendChat, subscribeChat } from "./store";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

function buildProviderText(message: string, attachments: ChatAttachment[]): string {
  const usable = attachments.filter((a) => a.text != null && a.text !== "");
  if (usable.length === 0) return message;
  const blocks = usable.map(
    (a) => `### ${a.path}${a.truncated ? " (truncated)" : ""}\n\`\`\`\n${a.text}\n\`\`\``,
  );
  return `${message}\n\nAttached files for context:\n\n${blocks.join("\n\n")}`;
}

export function ChatView({ compact = false }: { compact?: boolean }) {
  const { t } = useTranslation();
  const chat = useSyncExternalStore(subscribeChat, getChatSnapshot);
  const [input, setInput] = useState("");
  const [attachments, setAttachments] = useState<ChatAttachment[]>([]);
  const [attachError, setAttachError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const setPage = useUi((s) => s.setPage);
  const setNotice = useUi((s) => s.setNotice);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [chat.lines]);

  async function pick(kind: "file" | "folder") {
    setAttachError(null);
    try {
      const selection = await open(
        kind === "file"
          ? { multiple: true, title: t("chat.attachFile") }
          : { directory: true, multiple: false, title: t("chat.attachFolder") },
      );
      if (selection == null) return;
      const paths = Array.isArray(selection) ? selection : [selection];
      if (paths.length === 0) return;
      const picked = await aiReadAttachments(paths);
      setAttachments((prev) => [...prev, ...picked].slice(0, 40));
    } catch (e) {
      setAttachError(describe(e));
    }
  }

  function startNewChat() {
    void newChat();
    setAttachments([]);
    setAttachError(null);
  }

  function submit() {
    const message = input.trim();
    if (message === "" || chat.busy) return;
    const providerText = buildProviderText(message, attachments);
    setInput("");
    setAttachments([]);
    void sendChat(message, providerText);
  }

  async function copyConversation() {
    const text = chat.lines
      .map((l) => `${l.role === "user" ? t("chat.you") : t("chat.assistant")}: ${l.text}`)
      .join("\n\n");
    try {
      await navigator.clipboard.writeText(text);
      setNotice({ text: t("chat.copied"), ok: true });
    } catch (e) {
      setAttachError(describe(e));
    }
  }

  const usableCount = attachments.filter((a) => a.text != null && a.text !== "").length;

  return (
    <div className="flex h-full min-h-0 flex-col" data-testid="chat-view">
      <ChatToolbar
        onNewChat={startNewChat}
        onCopy={() => void copyConversation()}
        canCopy={chat.lines.length > 0}
        onPickFile={() => void pick("file")}
        onPickFolder={() => void pick("folder")}
        onOpenSettings={() => setPage("settings")}
      />

      <div
        ref={scrollRef}
        className={
          compact
            ? "min-h-0 flex-1 space-y-2 overflow-y-auto p-1.5"
            : "min-h-0 flex-1 space-y-3 overflow-y-auto rounded-lg border border-border bg-surface p-4"
        }
      >
        {chat.lines.length === 0 && (
          <p className={`text-text-dim/60 ${compact ? "text-[11px]" : "text-sm"}`}>
            {t("ai.placeholder")}
          </p>
        )}
        {chat.lines.map((line, i) => (
          <div
            key={i}
            className={`max-w-[92%] whitespace-pre-wrap rounded-lg ${
              compact ? "px-2 py-1 text-xs" : "px-3 py-2 text-sm"
            } ${line.role === "user" ? "ml-auto bg-accent/15 text-text" : "bg-surface-2 text-text"}`}
          >
            {line.text}
            {line.role === "assistant" && !line.done && (
              <span className="ml-1 inline-block h-3 w-1.5 animate-pulse bg-accent align-middle" />
            )}
          </div>
        ))}
      </div>

      {(chat.error != null || attachError != null) && (
        <div
          className={`rounded-md border border-danger/30 bg-danger/10 text-danger ${
            compact ? "m-1 px-2 py-1 text-[11px]" : "mt-3 px-3 py-2 text-sm"
          }`}
        >
          {chat.error ?? attachError}{" "}
          <button type="button" onClick={() => setPage("settings")} className="underline">
            {t("ai.goSettings")}
          </button>
        </div>
      )}

      <AttachmentChips
        attachments={attachments}
        onRemove={(path) => setAttachments((prev) => prev.filter((a) => a.path !== path))}
      />

      <ChatComposer
        value={input}
        onChange={setInput}
        onSubmit={submit}
        busy={chat.busy}
        onStop={() => void cancelChat()}
        onPickFile={() => void pick("file")}
        onPickFolder={() => void pick("folder")}
      />
      {usableCount > 0 && (
        <p className="px-1.5 pb-1 text-[10px] text-text-dim/60">
          {t("chat.contextNote", { count: usableCount })}
        </p>
      )}
    </div>
  );
}
