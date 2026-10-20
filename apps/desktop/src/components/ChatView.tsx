// Reusable chat surface: used full-width on the AI page and compact inside the
// secondary (right) sidebar like VS Code's chat view. State lives in
// lib/chat-store so switching surfaces never loses the conversation.
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { cancelChat, getChatSnapshot, sendChat, subscribeChat } from "../lib/chat-store";
import { useUi } from "../state/ui";

export function ChatView({ compact = false }: { compact?: boolean }) {
  const { t } = useTranslation();
  const chat = useSyncExternalStore(subscribeChat, getChatSnapshot);
  const [input, setInput] = useState("");
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const setPage = useUi((s) => s.setPage);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [chat.lines]);

  const submit = () => {
    const message = input.trim();
    if (message === "" || chat.busy) return;
    setInput("");
    void sendChat(message);
  };

  return (
    <div className="flex h-full min-h-0 flex-col" data-testid="chat-view">
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
            className={`max-w-[92%] rounded-lg whitespace-pre-wrap ${
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

      {chat.error != null && (
        <div
          className={`rounded-md border border-danger/30 bg-danger/10 text-danger ${
            compact ? "m-1 px-2 py-1 text-[11px]" : "mt-3 px-3 py-2 text-sm"
          }`}
        >
          {chat.error}{" "}
          <button type="button" onClick={() => setPage("settings")} className="underline">
            {t("ai.goSettings")}
          </button>
        </div>
      )}

      <form
        className={compact ? "flex gap-1 p-1.5" : "mt-3 flex gap-2"}
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <input
          value={input}
          onChange={(e) => setInput(e.currentTarget.value)}
          placeholder={t("ai.placeholder")}
          className={`min-w-0 flex-1 rounded-md border border-border bg-surface outline-none focus:border-accent ${
            compact ? "px-2 py-1 text-xs" : "px-3 py-2 text-sm"
          }`}
        />
        {chat.busy ? (
          <button
            type="button"
            onClick={() => void cancelChat()}
            aria-label={t("ai.stop")}
            title={t("ai.stop")}
            className={`rounded-md border border-danger/40 text-danger hover:bg-danger/10 ${
              compact ? "px-2 text-xs" : "px-3 py-2 text-sm"
            }`}
          >
            ■
          </button>
        ) : (
          <button
            type="submit"
            disabled={!input.trim()}
            aria-label={t("common.send")}
            title={t("common.send")}
            className={`rounded-md bg-accent font-medium text-white hover:opacity-90 disabled:opacity-40 ${
              compact ? "px-2 text-xs" : "px-4 py-2 text-sm"
            }`}
          >
            {t("common.send")}
          </button>
        )}
      </form>
    </div>
  );
}
