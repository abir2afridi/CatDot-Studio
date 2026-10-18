import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { aiChatCancel, aiChatStart, onChatEvent, type IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";
import type { AiStreamEvent } from "@apk-forge/shared-types";

interface ChatLine {
  role: "user" | "assistant";
  text: string;
  done: boolean;
}

export function AiChatPage() {
  const { t } = useTranslation();
  const setPage = useUi((s) => s.setPage);
  const [lines, setLines] = useState<ChatLine[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const activeStream = useRef<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [lines]);

  async function send() {
    const message = input.trim();
    if (!message || busy) return;
    setError(null);
    setInput("");
    setBusy(true);
    setLines((prev) => [...prev, { role: "user", text: message, done: true }]);

    try {
      const streamId = await aiChatStart(message);
      activeStream.current = streamId;
      setLines((prev) => [...prev, { role: "assistant", text: "", done: false }]);

      const unlisten = await onChatEvent(streamId, (ev: AiStreamEvent) => {
        if (ev.kind === "chunk") {
          setLines((prev) => {
            const next = [...prev];
            const last = next[next.length - 1];
            if (last && last.role === "assistant") {
              next[next.length - 1] = { ...last, text: last.text + ev.text };
            }
            return next;
          });
        } else if (ev.kind === "done") {
          setLines((prev) => {
            const next = [...prev];
            const last = next[next.length - 1];
            if (last && last.role === "assistant") {
              next[next.length - 1] = {
                ...last,
                text: ev.text || last.text,
                done: true,
              };
            }
            return next;
          });
          setBusy(false);
          activeStream.current = null;
          void unlisten();
        } else if (ev.kind === "error") {
          setError(ev.message);
          setBusy(false);
          activeStream.current = null;
          void unlisten();
        }
      });
    } catch (e) {
      setError((e as IpcError).message);
      setBusy(false);
    }
  }

  async function stop() {
    if (activeStream.current) {
      await aiChatCancel(activeStream.current);
    }
  }

  return (
    <div className="flex h-full flex-col p-6">
      <div className="flex items-start">
        <div>
          <h1 className="text-xl font-semibold">{t("ai.title")}</h1>
          <p className="mt-1 text-sm text-text-dim">{t("ai.subtitle")}</p>
        </div>
        {busy && (
          <button
            type="button"
            onClick={stop}
            className="ml-auto rounded-md border border-danger/40 px-3 py-1.5 text-sm text-danger hover:bg-danger/10"
          >
            {t("ai.stop")}
          </button>
        )}
      </div>

      <div
        ref={scrollRef}
        className="mt-4 flex-1 space-y-3 overflow-y-auto rounded-lg border border-border bg-surface p-4"
      >
        {lines.length === 0 && (
          <p className="text-sm text-text-dim/60">{t("ai.placeholder")}</p>
        )}
        {lines.map((line, i) => (
          <div
            key={i}
            className={`max-w-[85%] rounded-lg px-3 py-2 text-sm ${
              line.role === "user"
                ? "ml-auto bg-accent/15 text-text"
                : "bg-surface-2 text-text"
            }`}
          >
            <pre className="whitespace-pre-wrap font-sans">{line.text}</pre>
            {line.role === "assistant" && !line.done && (
              <span className="mt-1 inline-block h-3 w-1.5 animate-pulse bg-accent" />
            )}
          </div>
        ))}
      </div>

      {error && (
        <div className="mt-3 rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
          {error}{" "}
          <button
            type="button"
            onClick={() => setPage("settings")}
            className="underline"
          >
            {t("ai.goSettings")}
          </button>
        </div>
      )}

      <form
        className="mt-3 flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          void send();
        }}
      >
        <input
          value={input}
          onChange={(e) => setInput(e.currentTarget.value)}
          placeholder={t("ai.placeholder")}
          className="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
        />
        <button
          type="submit"
          disabled={busy || !input.trim()}
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
        >
          {busy ? t("ai.working") : t("common.send")}
        </button>
      </form>
    </div>
  );
}
