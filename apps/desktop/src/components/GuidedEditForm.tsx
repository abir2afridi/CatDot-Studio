// Guided Edit Form (Part F4 structured ops): the three "recipes" run REAL
// decode-pipeline ops from the Rust core â€” apk_set_label (string ref or
// literal), apk_toggle_permission, apk_set_version. Every success records a
// reversible patch (Change history), every failure shows the honest message.
import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkSetLabel, apkSetVersion, apkTogglePermission } from "../lib/apk";
import { describeError } from "../lib/opener";
import type { PatchSummary } from "@catdot/shared-types";

export function GuidedEditForm() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [msg, setMsg] = useState<{ text: string; ok: boolean } | null>(null);

  const ok = (s: PatchSummary) => {
    setMsg({ text: t("recipes.applied", { count: s.count }), ok: true });
    void queryClient.invalidateQueries({ queryKey: ["apk"] });
  };
  const fail = (e: unknown) =>
    setMsg({ text: describeError(e), ok: false });

  const labelMut = useMutation({ mutationFn: apkSetLabel, onSuccess: ok, onError: fail });
  const permMut = useMutation({
    mutationFn: (v: { perm: string; add: boolean }) =>
      apkTogglePermission(v.perm, v.add),
    onSuccess: ok,
    onError: fail,
  });
  const verMut = useMutation({
    mutationFn: (v: { code: number; name: string | null }) =>
      apkSetVersion(v.code, v.name),
    onSuccess: ok,
    onError: fail,
  });

  return (
    <div className="space-y-2" data-testid="guided-edit-form">
      <LabelForm busy={labelMut.isPending} onApply={(v) => labelMut.mutate(v)} />
      <PermForm busy={permMut.isPending} onApply={(perm, add) => permMut.mutate({ perm, add })} />
      <VersionForm busy={verMut.isPending} onApply={(c, n) => verMut.mutate({ code: c, name: n })} />
      {msg == null ? (
        <p className="text-[11px] text-text-dim">{t("recipes.hint")}</p>
      ) : msg.ok ? (
        <p className="text-[11px] text-success" data-testid="recipes-ok">{msg.text}</p>
      ) : (
        <p className="text-[11px] text-danger" data-testid="recipes-err">{msg.text}</p>
      )}
    </div>
  );
}

function Row({ children }: { children: React.ReactNode }) {
  return <div className="flex items-center gap-2">{children}</div>;
}

function Input({  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder: string;
  onChange: (v: string) => void;
}) {
  return (
    <input
      type="text"
      value={value}
      placeholder={placeholder}
      spellCheck={false}
      onChange={(e) => onChange(e.target.value)}
      className="min-w-0 flex-1 rounded-md border border-border bg-bg px-2 py-1 text-xs text-text placeholder:text-text-dim/50 focus:border-accent focus:outline-none"
    />
  );
}

function LabelForm({
  busy,
  onApply,
}: {
  busy: boolean;
  onApply: (label: string) => void;
}) {
  const { t } = useTranslation();
  const [label, setLabel] = useState("");
  return (
    <Row>
      <Input
        value={label}
        placeholder={t("recipes.labelPlaceholder")}
        onChange={setLabel}
      />
      <button
        type="button"
        disabled={busy || label.trim().length === 0}
        onClick={() => onApply(label.trim())}
        className="shrink-0 rounded-md bg-accent/20 px-2.5 py-1 text-xs text-accent hover:bg-accent/30 disabled:opacity-50"
      >
        {busy ? "â€¦" : t("recipes.renameLabel")}
      </button>
    </Row>
  );
}

function PermForm({
  busy,
  onApply,
}: {
  busy: boolean;
  onApply: (perm: string, add: boolean) => void;
}) {
  const { t } = useTranslation();
  const [perm, setPerm] = useState("");
  const [add, setAdd] = useState(true);
  return (
    <Row>
      <Input
        value={perm}
        placeholder={t("recipes.permPlaceholder")}
        onChange={setPerm}
      />
      <select
        value={add ? "add" : "remove"}
        onChange={(e) => setAdd(e.target.value === "add")}
        aria-label={t("recipes.permAction")}
        className="rounded-md border border-border bg-bg px-1 py-1 text-xs text-text"
      >
        <option value="add">{t("recipes.permAdd")}</option>
        <option value="remove">{t("recipes.permRemove")}</option>
      </select>
      <button
        type="button"
        disabled={busy || perm.trim().length === 0}
        onClick={() => {
          onApply(perm.trim(), add);
        }}
        className="shrink-0 rounded-md bg-accent/20 px-2.5 py-1 text-xs text-accent hover:bg-accent/30 disabled:opacity-50"
      >
        {busy ? "â€¦" : t("recipes.permApply")}
      </button>
    </Row>
  );
}

function VersionForm({
  busy,
  onApply,
}: {
  busy: boolean;
  onApply: (code: number, name: string | null) => void;
}) {
  const { t } = useTranslation();
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const codeNum = Number(code.trim());
  const valid =
    Number.isInteger(codeNum) && codeNum > 0 && code.trim().length > 0;
  return (
    <Row>
      <input
        type="number"
        min={1}
        value={code}
        onChange={(e) => setCode(e.target.value)}
        placeholder={t("recipes.codePlaceholder")}
        aria-label={t("recipes.code")}
        className="w-24 rounded-md border border-border bg-bg px-2 py-1 text-xs text-text placeholder:text-text-dim/50 focus:border-accent focus:outline-none"
      />
      <Input
        value={name}
        placeholder={t("recipes.namePlaceholder")}
        onChange={setName}
      />
      <button
        type="button"
        disabled={busy || !valid}
        onClick={() => {
          const n = name.trim();
          onApply(codeNum, n === "" ? null : n);
        }}
        className="shrink-0 rounded-md bg-accent/20 px-2.5 py-1 text-xs text-accent hover:bg-accent/30 disabled:opacity-50"
      >
        {busy ? "â€¦" : t("recipes.versionApply")}
      </button>
    </Row>
  );
}

