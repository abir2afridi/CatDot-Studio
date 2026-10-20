// Model picker + chat actions menu (new chat, copy, attach, settings).
import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { IpcError } from "../../lib/ipc";
import { useUi } from "../../state/ui";
import { MenuItem, usePopover } from "./ChatMenu";
import { loadProviderConfig, saveProviderConfig } from "./provider-config";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

export function ChatToolbar({
  onNewChat,
  onCopy,
  canCopy,
  onPickFile,
  onPickFolder,
  onOpenSettings,
}: {
  onNewChat: () => void;
  onCopy: () => void;
  canCopy: boolean;
  onPickFile: () => void;
  onPickFolder: () => void;
  onOpenSettings: () => void;
}) {
  const { t } = useTranslation();
  const setNotice = useUi((s) => s.setNotice);
  const qc = useQueryClient();
  const model = usePopover();
  const menu = usePopover();
  const [draft, setDraft] = useState("");

  const provider = useQuery({
    queryKey: ["settings", "ai-provider"],
    queryFn: loadProviderConfig,
    retry: false,
  });

  useEffect(() => {
    if (provider.data != null) setDraft(provider.data.default_model);
  }, [provider.data]);

  const saveModel = useMutation({
    mutationFn: async (next: string) => {
      const cfg = await loadProviderConfig();
      await saveProviderConfig({ ...cfg, default_model: next.trim() });
    },
    onSuccess: () => {
      model.setOpen(false);
      void qc.invalidateQueries({ queryKey: ["settings", "ai-provider"] });
      setNotice({ text: t("chat.modelSaved"), ok: true });
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const current = provider.data?.default_model ?? "";

  return (
    <div className="relative flex h-7 shrink-0 items-center gap-1 border-b border-border px-1.5">
      <div ref={model.ref} className="relative">
        <button
          type="button"
          data-testid="chat-model"
          onClick={() => {
            model.setOpen(!model.open);
            menu.setOpen(false);
          }}
          title={t("chat.model")}
          className="flex min-w-0 max-w-[150px] items-center gap-1 rounded-sm px-1.5 py-0.5 text-[11px] text-text-dim hover:bg-surface-2 hover:text-text"
        >
          <span className="truncate">{current === "" ? t("chat.model") : current}</span>
          <span aria-hidden="true" className="text-[8px]">▾</span>
        </button>
        {model.open && (
          <div
            className="absolute left-0 top-6 z-30 w-64 rounded-md border border-border bg-surface p-2 shadow-2xl"
            data-testid="chat-model-popover"
          >
            <p className="px-1 text-[10px] font-semibold uppercase tracking-wide text-text-dim">
              {t("chat.model")}
            </p>
            <input
              value={draft}
              onChange={(e) => setDraft(e.currentTarget.value)}
              placeholder="gpt-4o-mini"
              className="mt-1 w-full rounded border border-border bg-bg px-2 py-1 text-xs outline-none focus:border-accent"
            />
            <div className="mt-2 flex items-center justify-end gap-1">
              <button
                type="button"
                onClick={() => model.setOpen(false)}
                className="rounded px-2 py-0.5 text-[11px] text-text-dim hover:bg-surface-2"
              >
                {t("common.cancel")}
              </button>
              <button
                type="button"
                disabled={saveModel.isPending || draft.trim() === "" || draft.trim() === current}
                onClick={() => saveModel.mutate(draft)}
                className="rounded bg-accent px-2 py-0.5 text-[11px] font-medium text-white hover:opacity-90 disabled:opacity-40"
              >
                {t("chat.modelApply")}
              </button>
            </div>
            {provider.data == null && (
              <p className="mt-1 px-1 text-[10px] text-text-dim/70">{t("chat.noProvider")}</p>
            )}
          </div>
        )}
      </div>

      <div className="ml-auto flex items-center">
        <button
          type="button"
          data-testid="chat-new"
          onClick={onNewChat}
          aria-label={t("chat.newChat")}
          title={t("chat.newChat")}
          className="flex h-6 w-6 items-center justify-center rounded-sm text-text-dim hover:bg-surface-2 hover:text-text"
        >
          +
        </button>
        <div ref={menu.ref} className="relative">
          <button
            type="button"
            data-testid="chat-menu"
            onClick={() => {
              menu.setOpen(!menu.open);
              model.setOpen(false);
            }}
            aria-label={t("chat.menu")}
            title={t("chat.menu")}
            className="flex h-6 w-6 items-center justify-center rounded-sm text-text-dim hover:bg-surface-2 hover:text-text"
          >
            ⋯
          </button>
          {menu.open && (
            <div
              role="menu"
              className="absolute right-0 top-6 z-30 min-w-[190px] rounded-md border border-border bg-surface py-1 shadow-2xl"
              data-testid="chat-menu-popover"
            >
              <MenuItem
                label={t("chat.newChat")}
                onClick={() => {
                  menu.setOpen(false);
                  onNewChat();
                }}
              />
              <MenuItem
                label={t("chat.copy")}
                disabled={!canCopy}
                onClick={() => {
                  menu.setOpen(false);
                  onCopy();
                }}
              />
              <div className="my-1 border-t border-border" />
              <MenuItem
                label={t("chat.attachFile")}
                onClick={() => {
                  menu.setOpen(false);
                  onPickFile();
                }}
              />
              <MenuItem
                label={t("chat.attachFolder")}
                onClick={() => {
                  menu.setOpen(false);
                  onPickFolder();
                }}
              />
              <div className="my-1 border-t border-border" />
              <MenuItem
                label={t("chat.openSettings")}
                onClick={() => {
                  menu.setOpen(false);
                  onOpenSettings();
                }}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
