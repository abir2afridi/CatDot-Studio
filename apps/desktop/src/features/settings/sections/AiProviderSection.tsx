// AI provider: base URL/model/key account + API key save + real test call.
import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import type { OpenAICompatConfig } from "@catdot/shared-types";
import { aiTestConnection, secretHas, secretSet, IpcError } from "../../../lib/ipc";
import {
  AI_KEY_ACCOUNT,
  defaultProviderConfig,
  loadProviderConfig,
  saveProviderConfig,
} from "../../chat/provider-config";
import { Field, TextInput } from "../SettingsControls";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

export function AiProviderSection() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [cfg, setCfg] = useState<OpenAICompatConfig>(defaultProviderConfig);
  const [apiKey, setApiKey] = useState("");
  const [status, setStatus] = useState<{ kind: "ok" | "err"; msg: string } | null>(null);

  const { data: loadedCfg } = useQuery({
    queryKey: ["settings", "ai-provider"],
    queryFn: loadProviderConfig,
  });
  const { data: hasKey } = useQuery({
    queryKey: ["settings", "has-key", cfg.key_account],
    queryFn: () => secretHas(cfg.key_account),
  });

  useEffect(() => {
    if (loadedCfg) setCfg(loadedCfg);
  }, [loadedCfg]);

  const save = useMutation({
    mutationFn: async () => {
      await saveProviderConfig(cfg);
      if (apiKey.trim()) {
        await secretSet(cfg.key_account || AI_KEY_ACCOUNT, apiKey.trim());
        setApiKey("");
      }
    },
    onSuccess: () => {
      setStatus({ kind: "ok", msg: t("settings.saved") });
      void qc.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (e) => setStatus({ kind: "err", msg: describe(e) }),
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
    onError: (e) => setStatus({ kind: "err", msg: describe(e) }),
  });

  return (
    <section className="mt-8 border-t border-border pt-6">
      <h2 className="font-medium">{t("settings.aiProvider")}</h2>
      <div className="mt-3 space-y-3">
        <Field label={t("settings.aiBaseurl")}>
          <TextInput
            value={cfg.base_url}
            onChange={(v) => setCfg({ ...cfg, base_url: v })}
            mono
          />
        </Field>
        <Field label={t("settings.aiModel")}>
          <TextInput
            value={cfg.default_model}
            onChange={(v) => setCfg({ ...cfg, default_model: v })}
            mono
          />
        </Field>
        <Field label={t("settings.aiKeyAccount")}>
          <TextInput
            value={cfg.key_account}
            onChange={(v) => setCfg({ ...cfg, key_account: v })}
            mono
          />
        </Field>
        <Field
          label={t("settings.aiApiKey")}
          hint={hasKey ? t("settings.aiKeySaved") : t("settings.aiKeyNotSaved")}
        >
          <TextInput
            value={apiKey}
            type="password"
            placeholder={hasKey ? "••••••••" : ""}
            onChange={setApiKey}
            mono
          />
        </Field>
      </div>

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
          onClick={() => save.mutate()}
          disabled={save.isPending}
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
    </section>
  );
}
