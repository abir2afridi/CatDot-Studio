// Composer row: attach menu, input, send/stop.
import { useTranslation } from "react-i18next";
import { MenuItem, usePopover } from "./ChatMenu";

export function ChatComposer({
  value,
  onChange,
  onSubmit,
  busy,
  onStop,
  onPickFile,
  onPickFolder,
}: {
  value: string;
  onChange: (next: string) => void;
  onSubmit: () => void;
  busy: boolean;
  onStop: () => void;
  onPickFile: () => void;
  onPickFolder: () => void;
}) {
  const { t } = useTranslation();
  const attach = usePopover();

  return (
    <form
      className="relative flex items-end gap-1 border-t border-border p-1.5"
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
    >
      <div ref={attach.ref} className="relative">
        <button
          type="button"
          data-testid="chat-attach"
          onClick={() => attach.setOpen(!attach.open)}
          aria-label={t("chat.attach")}
          title={t("chat.attach")}
          className="flex h-[26px] w-6 items-center justify-center rounded-md border border-border text-text-dim hover:bg-surface-2 hover:text-text"
        >
          +
        </button>
        {attach.open && (
          <div
            role="menu"
            className="absolute bottom-8 left-0 z-30 min-w-[180px] rounded-md border border-border bg-surface py-1 shadow-2xl"
            data-testid="chat-attach-popover"
          >
            <MenuItem
              label={t("chat.attachFile")}
              onClick={() => {
                attach.setOpen(false);
                onPickFile();
              }}
            />
            <MenuItem
              label={t("chat.attachFolder")}
              onClick={() => {
                attach.setOpen(false);
                onPickFolder();
              }}
            />
          </div>
        )}
      </div>
      <input
        value={value}
        onChange={(e) => onChange(e.currentTarget.value)}
        placeholder={t("ai.placeholder")}
        className="min-w-0 flex-1 rounded-md border border-border bg-surface px-2 py-1 text-xs outline-none focus:border-accent"
      />
      {busy ? (
        <button
          type="button"
          onClick={onStop}
          aria-label={t("ai.stop")}
          title={t("ai.stop")}
          className="shrink-0 rounded-md border border-danger/40 px-2 text-xs text-danger hover:bg-danger/10"
        >
          ■
        </button>
      ) : (
        <button
          type="submit"
          disabled={!value.trim()}
          aria-label={t("common.send")}
          title={t("common.send")}
          className="shrink-0 rounded-md bg-accent px-2 text-xs font-medium text-white hover:opacity-90 disabled:opacity-40"
        >
          {t("common.send")}
        </button>
      )}
    </form>
  );
}
