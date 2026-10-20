// Attachment chips: name, size, skip/cut reason, remove.
import { useTranslation } from "react-i18next";
import type { ChatAttachment } from "@catdot/shared-types";

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

export function AttachmentChips({
  attachments,
  onRemove,
}: {
  attachments: ChatAttachment[];
  onRemove: (path: string) => void;
}) {
  const { t } = useTranslation();
  if (attachments.length === 0) return null;
  return (
    <div className="flex flex-wrap gap-1 px-1.5 pt-1" data-testid="chat-attachments">
      {attachments.map((a) => (
        <span
          key={a.path}
          title={a.note ?? a.path}
          className={`flex max-w-full items-center gap-1 rounded border px-1.5 py-0.5 text-[10px] ${
            a.text == null
              ? "border-danger/40 bg-danger/10 text-danger"
              : "border-border bg-surface-2 text-text-dim"
          }`}
        >
          <span className="truncate">{a.name}</span>
          <span className="shrink-0 text-text-dim/60">
            {formatBytes(Number(a.bytes))}
            {a.truncated ? " · cut" : ""}
          </span>
          <button
            type="button"
            aria-label={t("chat.removeAttachment")}
            title={t("chat.removeAttachment")}
            onClick={() => onRemove(a.path)}
            className="shrink-0 text-text-dim hover:text-danger"
          >
            ✕
          </button>
        </span>
      ))}
    </div>
  );
}
