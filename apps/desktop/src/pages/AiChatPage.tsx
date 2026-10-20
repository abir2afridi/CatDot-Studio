import { useTranslation } from "react-i18next";
import { ChatView } from "../components/ChatView";

export function AiChatPage() {
  const { t } = useTranslation();

  return (
    <div className="flex h-full flex-col p-6">
      <div>
        <h1 className="text-xl font-semibold">{t("ai.title")}</h1>
        <p className="mt-1 text-sm text-text-dim">{t("ai.subtitle")}</p>
      </div>
      <div className="mt-4 min-h-0 flex-1">
        <ChatView />
      </div>
    </div>
  );
}
