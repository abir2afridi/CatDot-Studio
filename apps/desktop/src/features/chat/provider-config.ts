// AI provider config helpers. The non-secret config lives in the settings DB
// under `ai.provider` (JSON); the API key lives only in the secret store.
import type { OpenAICompatConfig } from "@catdot/shared-types";
import { settingsGet, settingsSet, secretHas } from "./ipc";

export const AI_PROVIDER_KEY = "ai.provider";
export const AI_KEY_ACCOUNT = "catdot-ai";

export const defaultProviderConfig: OpenAICompatConfig = {
  base_url: "https://api.openai.com/v1",
  default_model: "gpt-4o-mini",
  key_account: AI_KEY_ACCOUNT,
};

export async function loadProviderConfig(): Promise<OpenAICompatConfig> {
  const raw = await settingsGet(AI_PROVIDER_KEY);
  if (raw && typeof raw === "object") {
    return { ...defaultProviderConfig, ...(raw as Partial<OpenAICompatConfig>) };
  }
  return defaultProviderConfig;
}

export async function saveProviderConfig(cfg: OpenAICompatConfig): Promise<void> {
  await settingsSet(AI_PROVIDER_KEY, cfg, "user");
}

export function hasProviderConfig(raw: unknown): boolean {
  return raw != null && typeof raw === "object";
}

export { secretHas };
