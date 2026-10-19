// Integrated terminals (part X-Term): REAL ConPTY children streamed through
// Tauri events into xterm.js. VS-Code-like tab row: + new terminal, × close
// per tab, panel minimize handled by the bottom panel. Multiple terminals.
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { TermInfo } from "@catdot/shared-types";
import { onTermExit, onTermOut, termClose, termResize, termSpawn, termWrite, IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";

interface TermTab {
  id: number;
  shell: string;
  title: string;
}

interface Session {
  term: Terminal;
  fit: FitAddon;
}

export function TerminalPanel() {
  const { t } = useTranslation();
  const [tabs, setTabs] = useState<TermTab[]>([]);
  const [activeId, setActiveId] = useState<number | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const sessions = useRef<Map<number, Session>>(new Map());
  const hostRef = useRef<HTMLDivElement | null>(null);
  const activeKey = activeId == null ? "none" : String(activeId);
  const terminalRequest = useUi((s) => s.terminalRequest);
  const terminalRequestHandled = useUi((s) => s.terminalRequestHandled);
  const markTerminalRequestHandled = useUi((s) => s.markTerminalRequestHandled);

  const spawn = useCallback(async () => {
    setErr(null);
    try {
      const info: TermInfo = await termSpawn();
      setTabs((prev) => [...prev, { id: info.id, shell: info.shell, title: `${info.shell} ${prev.length + 1}` }]);
      setActiveId(info.id);
    } catch (e) {
      setErr(e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e));
    }
  }, []);

  const closeTab = useCallback(async (id: number) => {
    try {
      await termClose(id);
    } catch {
      // already gone (exited) — still remove the tab below
    }
    sessions.current.delete(id);
    setTabs((prev) => {
      const next = prev.filter((p) => p.id !== id);
      setActiveId((cur) => (cur === id ? (next[next.length - 1]?.id ?? null) : cur));
      return next;
    });
  }, []);

  // Menu bar "New Terminal" (Ctrl+`) → spawn once per request, even if the
  // panel was closed when the request was made.
  useEffect(() => {
    if (terminalRequest > terminalRequestHandled) {
      markTerminalRequestHandled();
      void spawn();
    }
  }, [terminalRequest, terminalRequestHandled, markTerminalRequestHandled, spawn]);

  // Global output/exit routing (single listener).
  useEffect(() => {
    let un1: (() => void) | null = null;
    let un2: (() => void) | null = null;
    void onTermOut((id, data) => {
      const s = sessions.current.get(id);
      s?.term.write(data);
    }).then((u) => { un1 = u; });
    void onTermExit((id) => {
      setTabs((prev) => prev.filter((p) => p.id !== id));
      sessions.current.delete(id);
      setActiveId((cur) => (cur === id ? null : cur));
    }).then((u) => { un2 = u; });
    return () => { un1?.(); un2?.(); };
  }, []);

  // Attach/detach the xterm DOM for the active tab.
  useEffect(() => {
    const host = hostRef.current;
    if (activeId == null || host == null) return;
    let s = sessions.current.get(activeId);
    if (s == null) {
      const term = new Terminal({
        fontFamily: "'Cascadia Mono', Consolas, monospace",
        fontSize: 12,
        cursorBlink: true,
        theme: { background: "#0c1116", foreground: "#d4d4d4", cursor: "#4f8ccd" },
        scrollback: 2000,
      });
      const fit = new FitAddon();
      term.loadAddon(fit);
      term.onData((data) => void termWrite(activeId, data));
      term.onResize(({ cols, rows }) => void termResize(activeId, cols, rows).catch(() => {}));
      s = { term, fit };
      sessions.current.set(activeId, s);
      term.open(host);
      term.writeln(t("terminal.welcome"));
      fit.fit();
    } else {
      // Recreate the DOM node in the host for the activated tab.
      const el = s.term.element;
      if (el != null && el.parentElement !== host) host.appendChild(el);
      s.term.focus();
      s.fit.fit();
    }
  }, [activeId, activeKey, t]);

  // Resize + refit on panel geometry changes.
  useEffect(() => {
    const host = hostRef.current;
    if (host == null || activeId == null) return;
    const ro = new ResizeObserver(() => {
      const s = sessions.current.get(activeId);
      s?.fit.fit();
    });
    ro.observe(host);
    return () => ro.disconnect();
  }, [activeId]);

  return (
    <div className="flex h-full flex-col bg-[#0c1116]" data-testid="terminal-panel">
      {/* tab strip: VS Code style */}
      <div className="flex items-center gap-1 border-b border-border px-1.5 py-1">
        <button
          type="button"
          onClick={() => void spawn()}
          title={t("terminal.new")}
          aria-label={t("terminal.new")}
          className="rounded px-1.5 py-0.5 text-xs text-text-dim hover:bg-surface-2 hover:text-text"
          data-testid="terminal-new"
        >
          ＋
        </button>
        <div className="flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              className={`group flex shrink-0 items-center gap-1 rounded px-2 py-0.5 text-[11px] ${
                tab.id === activeId ? "bg-surface-2 text-text" : "text-text-dim hover:bg-surface-2/60"
              }`}
            >
              <button
                type="button"
                onClick={() => setActiveId(tab.id)}
                data-testid={`terminal-tab-${tab.id}`}
              >
                {tab.title}
              </button>
              <button
                type="button"
                aria-label={t("terminal.close")}
                title={t("terminal.close")}
                onClick={() => void closeTab(tab.id)}
                className="rounded px-0.5 opacity-0 hover:text-danger group-hover:opacity-100"
              >
                ×
              </button>
            </div>
          ))}
        </div>
        {err != null && <span className="text-[10px] text-danger">{err}</span>}
      </div>
      <div className="min-h-0 flex-1 p-1">
        {activeId == null ? (
          <button
            type="button"
            onClick={() => void spawn()}
            className="m-2 block text-xs text-text-dim hover:text-text"
          >
            {t("terminal.empty")}
          </button>
        ) : (
          <div ref={hostRef} className="h-full w-full" data-terminal-id={activeKey} />
        )}
      </div>
    </div>
  );
}
