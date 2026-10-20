// Bottom panel shell: tab row + the active tab's real content.
import { useTranslation } from "react-i18next";
import { useUi } from "../../state/ui";
import { TerminalPanel } from "../terminal/TerminalPanel";
import { BuildFeed } from "./BuildFeed";
import { DeviceLogsTab } from "./DeviceLogsTab";
import { DiffTab } from "./DiffTab";
import { LogsFeed } from "./LogsFeed";
import { ManifestTab } from "./ManifestTab";
import { PanelEmpty } from "./PanelEmpty";
import { PanelTabs } from "./PanelTabs";

export function BottomPanel() {
  const { t } = useTranslation();
  const height = useUi((s) => s.bottomHeight);
  const maximized = useUi((s) => s.bottomMaximized);
  const activeTab = useUi((s) => s.activeBottomTab);
  const activeFile = useUi((s) => s.activeFile);
  const draft = useUi((s) => (activeFile == null ? null : s.drafts[activeFile]));
  const lastBuild = useUi((s) => s.lastBuild);

  return (
    <div
      className={`flex flex-col border-t border-border bg-surface ${
        maximized ? "min-h-0 flex-1" : ""
      }`}
      style={maximized ? undefined : { height }}
      data-testid="bottom-panel"
    >
      <PanelTabs />
      <div className="min-h-0 flex-1 overflow-hidden">
        {activeTab === "terminal" ? (
          <TerminalPanel />
        ) : activeTab === "logs" ? (
          <LogsFeed />
        ) : activeTab === "build" ? (
          lastBuild == null ? (
            <PanelEmpty text={t("panel.noBuildYet")} />
          ) : (
            <BuildFeed />
          )
        ) : activeTab === "diff" ? (
          draft == null ? (
            <PanelEmpty text={t("diff.noDraft")} />
          ) : (
            <DiffTab />
          )
        ) : activeTab === "device" ? (
          <DeviceLogsTab />
        ) : activeTab === "manifest" ? (
          <ManifestTab />
        ) : (
          <PanelEmpty text={t("panel.problemsPlanned")} />
        )}
      </div>
    </div>
  );
}
