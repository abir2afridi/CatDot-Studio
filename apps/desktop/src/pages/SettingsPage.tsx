import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import {
  aiTestConnection,
  secretHas,
  secretSet,
  settingsGet,
  settingsSet,
  type IpcError,
} from "../lib/ipc";
import {
  AI_KEY_ACCOUNT,
  defaultProviderConfig,
  loadProviderConfig,
  saveProviderConfig,
} from "../lib/ai";
import { useUi } from "../state/ui";
import type { OpenAICompatConfig } from "@apk-forge/shared-types";

export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const qc = useQueryClient();
  const theme = useUi((s) => s.theme);
  const setTheme = useUi((s) => s.setTheme);
  const locale = useUi((s) => s.locale);
  const setLocale = useUi((s) => s.setLocale);

  const [cfg, setCfg] = useState<OpenAICompatConfig>(defaultProviderConfig);
  const [apiKey, setApiKey] = useState("");
  const [accessLevel, setAccessLevel] = useState(1);
  const [status, setStatus] = useState<{ kind: "ok" | "err"; msg: string } | null>(null);

  // Load persisted provider config + access level + secret presence.
  const { data: loadedCfg } = useQuery({
    queryKey: ["settings", "ai-provider"],
    queryFn: loadProviderConfig,
  });
  const { data: savedAccess } = useQuery({
    queryKey: ["settings", "access-level"],
    queryFn: () => settingsGet("safety.access_level"),
  });
  const { data: hasKey } = useQuery({
    queryKey: ["settings", "has-key", cfg.key_account],
    queryFn: () => secretHas(cfg.key_account),
  });

  useEffect(() => {
    if (loadedCfg) setCfg(loadedCfg);
  }, [loadedCfg]);
  useEffect(() => {
    if (typeof savedAccess === "number") setAccessLevel(savedAccess);
  }, [savedAccess]);

  const saveAll = useMutation({
    mutationFn: async () => {
      await saveProviderConfig(cfg);
      await settingsSet("safety.access_level", accessLevel, "user");
      if (apiKey.trim()) {
        await secretSet(cfg.key_account || AI_KEY_ACCOUNT, apiKey.trim());
        setApiKey("");
      }
    },
    onSuccess: () => {
      setStatus({ kind: "ok", msg: t("settings.saved") });
      void qc.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (e) => {
      setStatus({ kind: "err", msg: (e as IpcError).message });
    },
  });

  const test = useMutation({
    mutationFn: async () => {
      // Persist first so the backend reads the latest config/key.
      await saveProviderConfig(cfg);
      if (apiKey.trim()) await secretSet(cfg.key_account || AI_KEY_ACCOUNT, apiKey.trim());
      return aiTestConnection();
    },
    onSuccess: (caps) => {
      setStatus({
        kind: "ok",
        msg: `${t("ai.connectionOk")} · streaming=${caps.streaming} tools=${caps.tools}`,
      });
    },
    onError: (e) => {
      setStatus({ kind: "err", msg: (e as IpcError).message });
    },
  });

  function applyTheme(next: "dark" | "light") {
    setTheme(next);
    document.documentElement.dataset.theme = next;
    void settingsSet("ui.theme", next, "user");
  }

  function applyLocale(next: "en" | "bn") {
    setLocale(next);
    void i18n.changeLanguage(next);
    void settingsSet("ui.locale", next, "user");
  }

  return (
    <div className="max-w-2xl p-6">
      <h1 className="text-xl font-semibold">{t("settings.title")}</h1>
      <p className="mt-1 text-sm text-text-dim">{t("settings.subtitle")}</p>

      <section className="mt-6 space-y-4">
        <Field label={t("settings.theme")}>
          <div className="flex gap-2">
            <Toggle active={theme === "dark"} onClick={() => applyTheme("dark")}>
              {t("settings.themeDark")}
            </Toggle>
            <Toggle active={theme === "light"} onClick={() => applyTheme("light")}>
              {t("settings.themeLight")}
            </Toggle>
          </div>
        </Field>

        <Field label={t("settings.locale")}>
          <div className="flex gap-2">
            <Toggle active={locale === "en"} onClick={() => applyLocale("en")}>
              English
            </Toggle>
            <Toggle active={locale === "bn"} onClick={() => applyLocale("bn")}>
              বাংলা
            </Toggle>
          </div>
        </Field>

        <Field label={t("settings.accessLevel")}>
          <div className="flex gap-2">
            {[0, 1, 2].map((lvl) => (
              <Toggle
                key={lvl}
                active={accessLevel === lvl}
                onClick={() => setAccessLevel(lvl)}
              >
                L{lvl}
              </Toggle>
            ))}
          </div>
        </Field>
      </section>

      <section className="mt-8 border-t border-border pt-6">
        <h2 className="font-medium">{t("settings.aiProvider")}</h2>
        <div className="mt-3 space-y-3">
          <Field label={t("settings.aiBaseurl")}>
            <Input
              value={cfg.base_url}
              onChange={(v) => setCfg({ ...cfg, base_url: v })}
              mono
            />
          </Field>
          <Field label={t("settings.aiModel")}>
            <Input
              value={cfg.default_model}
              onChange={(v) => setCfg({ ...cfg, default_model: v })}
              mono
            />
          </Field>
          <Field label={t("settings.aiKeyAccount")}>
            <Input
              value={cfg.key_account}
              onChange={(v) => setCfg({ ...cfg, key_account: v })}
              mono
            />
          </Field>
          <Field
            label={t("settings.aiApiKey")}
            hint={hasKey ? t("settings.aiKeySaved") : t("settings.aiKeyNotSaved")}
          >
            <Input
              value={apiKey}
              type="password"
              placeholder={hasKey ? "••••••••" : ""}
              onChange={setApiKey}
              mono
            />
          </Field>
        </div>
      </section>

      {status && (
        <div
          className={`mt-4 rounded-md border px-3 py-2 text-sm ${
            status.kind === "ok"
              ? "border-success/30 bg-success/10 text-success"
              : "border-danger/30 bg-danger/10 text-danger"
          }`}
        >
          {status.msg}
        </div>
      )}

      <div className="mt-6 flex gap-3">
        <button
          type="button"
          onClick={() => saveAll.mutate()}
          disabled={saveAll.isPending}
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
        >
          {t("common.save")}
        </button>
        <button
          type="button"
          onClick={() => test.mutate()}
          disabled={test.isPending}
          className="rounded-md border border-border px-4 py-2 text-sm hover:bg-surface-2 disabled:opacity-50"
        >
          {test.isPending ? t("common.loading") : t("settings.testConnection")}
        </button>
      </div>
    </div>
  );
}

function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="text-sm text-text-dim">{label}</span>
      <div className="mt-1">{children}</div>
      {hint && <span className="mt-1 block text-xs text-text-dim/70">{hint}</span>}
    </label>
  );
}

function Toggle({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-md border px-3 py-1.5 text-sm ${
        active
          ? "border-accent bg-accent/15 text-accent"
          : "border-border text-text-dim hover:bg-surface-2"
      }`}
    >
      {children}
    </button>
  );
}

function Input({
  value,
  onChange,
  type = "text",
  placeholder,
  mono,
}: {
  value: string;
  onChange: (v: string) => void;
  type?: string;
  placeholder?: string;
  mono?: boolean;
}) {
  return (
    <input
      type={type}
      value={value}
      placeholder={placeholder}
      onChange={(e) => onChange(e.currentTarget.value)}
      className={`w-full rounded-md border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent ${
        mono ? "mono" : ""
      }`}
    />
  );
}
