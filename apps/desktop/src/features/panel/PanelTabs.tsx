// Panel tab row: real tab switching + maximize/restore + collapse.
import { useTranslation } from "react-i18next";
import { useUi, type BottomTab } from "../../state/ui";
import { PlannedBadge } from "../../components/PlannedBadge";

export function PanelTabs() {
  const { t } = useTranslation();
  const activeTab = useUi((s) => s.activeBottomTab);
  const setActiveTab = useUi((s) => s.setActiveBottomTab);
  const toggleBottom = useUi((s) => s.toggleBottom);
  const maximized = useUi((s) => s.bottomMaximized);
  const toggleMaximize = useUi((s) => s.toggleBottomMaximize);
  const activeFile = useUi((s) => s.activeFile);
  const draft = useUi((s) => (activeFile == null ? null : s.drafts[activeFile]));

  const tabs: ReadonlyArray<{ id: BottomTab; label: string; enabled: boolean }> = [
    { id: "terminal", label: t("panel.terminal"), enabled: true },
    { id: "logs", label: t("panel.logs"), enabled: true },
    { id: "build", label: t("panel.build"), enabled: true },
    { id: "diff", label: t("panel.diff"), enabled: draft != null },
    { id: "device", label: t("panel.device"), enabled: true },
    { id: "manifest", label: t("panel.manifest"), enabled: true },
    { id: "problems", label: t("panel.problems"), enabled: false },
  ];

  return (
    <div className="flex items-center gap-1 border-b border-border px-2 py-1">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          disabled={!tab.enabled}
          onClick={() => setActiveTab(tab.id)}
          aria-selected={activeTab === tab.id}
          role="tab"
          className={`rounded px-2 py-0.5 text-xs ${
            tab.id === activeTab
              ? "bg-bg text-text"
              : tab.enabled
                ? "text-text-dim hover:bg-surface-2"
                : "cursor-not-allowed opacity-50"
          }`}
        >
          {tab.label}
          {tab.id === "problems" && <PlannedBadge phase={4} />}
        </button>
      ))}
      <button
        type="button"
        data-testid="panel-maximize"
        onClick={toggleMaximize}
        aria-label={maximized ? t("panel.restore") : t("panel.maximize")}
        title={maximized ? t("panel.restore") : t("panel.maximize")}
        className="ml-auto rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text"
      >
        {maximized ? "⤡" : "⤢"}
      </button>
      <button
        type="button"
        onClick={toggleBottom}
        aria-label={t("cmd.togglePanel")}
        title={t("cmd.togglePanel")}
        className="rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text"
      >
        ⌄
      </button>
    </div>
  );
}
