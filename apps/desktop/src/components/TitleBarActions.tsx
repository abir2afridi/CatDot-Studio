// VS Code-style title-bar actions (right cluster): command search pill, the
// three layout toggles (primary sidebar / panel / secondary sidebar), AI page
// chip and Settings gear. Every action is real; nothing is decorative.
import { useTranslation } from "react-i18next";
import { useUi } from "../state/ui";

function IconButton({
  label,
  testId,
  pressed,
  disabled,
  onClick,
  children,
}: {
  label: string;
  testId: string;
  pressed?: boolean;
  disabled?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      data-testid={testId}
      title={label}
      aria-label={label}
      aria-pressed={pressed}
      disabled={disabled}
      onClick={onClick}
      className={`flex h-6 w-6 items-center justify-center rounded-sm hover:bg-surface-2 disabled:opacity-30 disabled:hover:bg-transparent ${
        pressed === true ? "text-text" : "text-text-dim hover:text-text"
      }`}
    >
      {children}
    </button>
  );
}

function BackIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M9.5 3.5 5 8l4.5 4.5" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function ForwardIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M6.5 3.5 11 8l-4.5 4.5" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function PanelLeftIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" stroke="currentColor" />
      <rect x="2.8" y="3.8" width="3.6" height="8.4" fill="currentColor" opacity="0.6" />
    </svg>
  );
}

function PanelBottomIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" stroke="currentColor" />
      <rect x="2.8" y="8.6" width="10.4" height="3.6" fill="currentColor" opacity="0.6" />
    </svg>
  );
}

function PanelRightIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" stroke="currentColor" />
      <rect x="9.6" y="3.8" width="3.6" height="8.4" fill="currentColor" opacity="0.6" />
    </svg>
  );
}

function AiIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M8 1.8l1.3 3.9 3.9 1.3-3.9 1.3L8 12.2 6.7 8.3 2.8 7l3.9-1.3L8 1.8z"
        fill="currentColor"
        opacity="0.9"
      />
      <circle cx="13" cy="13" r="1.2" fill="currentColor" opacity="0.6" />
    </svg>
  );
}

function GearIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <circle cx="8" cy="8" r="2.3" stroke="currentColor" />
      <path
        d="M8 1.7v1.9M8 12.4v1.9M1.7 8h1.9M12.4 8h1.9M3.55 3.55l1.35 1.35M11.1 11.1l1.35 1.35M12.45 3.55 11.1 4.9M4.9 11.1l-1.35 1.35"
        stroke="currentColor"
        strokeLinecap="round"
      />
    </svg>
  );
}

/** Toggle that also reveals the Explorer page when invoked from another view. */
function showSidebar(): void {
  const ui = useUi.getState();
  if (ui.page === "explorer") {
    ui.toggleExplorer();
  } else {
    ui.setPage("explorer");
    if (!ui.explorerOpen) ui.toggleExplorer();
  }
}

function showSecondary(): void {
  const ui = useUi.getState();
  if (ui.page === "explorer") {
    ui.toggleSecondary();
  } else {
    ui.setPage("explorer");
    if (!ui.secondaryOpen) ui.toggleSecondary();
  }
}

export { showSidebar, showSecondary };

export function TitleBarActions() {
  const { t } = useTranslation();
  const page = useUi((s) => s.page);
  const explorerOpen = useUi((s) => s.explorerOpen);
  const bottomOpen = useUi((s) => s.bottomOpen);
  const secondaryOpen = useUi((s) => s.secondaryOpen);
  const toggleBottom = useUi((s) => s.toggleBottom);
  const setPage = useUi((s) => s.setPage);
  const setPaletteOpen = useUi((s) => s.setPaletteOpen);
  const canBack = useUi((s) => s.historyIndex > 0);
  const canForward = useUi((s) => s.historyIndex < s.fileHistory.length - 1);
  const goBack = useUi((s) => s.goBack);
  const goForward = useUi((s) => s.goForward);

  const sidebarsVisible = page === "explorer";

  return (
    <div className="flex items-center gap-1.5">
      <div className="flex items-center gap-0.5">
        <IconButton label={t("titlebar.back")} testId="tba-back" disabled={!canBack} onClick={goBack}>
          <BackIcon />
        </IconButton>
        <IconButton
          label={t("titlebar.forward")}
          testId="tba-forward"
          disabled={!canForward}
          onClick={goForward}
        >
          <ForwardIcon />
        </IconButton>
      </div>
      <button
        type="button"
        data-testid="tba-search"
        onClick={() => setPaletteOpen(true)}
        title={t("menu.commandPalette")}
        className="hidden h-6 w-56 items-center gap-1.5 rounded-md border border-border bg-bg px-2 text-[11px] text-text-dim hover:border-accent/40 hover:text-text md:flex"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <circle cx="7" cy="7" r="4.2" stroke="currentColor" />
          <path d="M10.2 10.2 14 14" stroke="currentColor" strokeLinecap="round" />
        </svg>
        <span className="truncate">{t("titlebar.search")}</span>
        <span className="ml-auto shrink-0 text-[9px] text-text-dim/70">Ctrl+Shift+P</span>
      </button>
      <div className="flex items-center gap-0.5 border-l border-border pl-1.5">
        <IconButton
          label={t("cmd.toggleSidebar")}
          testId="tba-sidebar"
          pressed={sidebarsVisible && explorerOpen}
          onClick={showSidebar}
        >
          <PanelLeftIcon />
        </IconButton>
        <IconButton
          label={t("cmd.togglePanel")}
          testId="tba-panel"
          pressed={bottomOpen}
          onClick={toggleBottom}
        >
          <PanelBottomIcon />
        </IconButton>
        <IconButton
          label={t("titlebar.toggleSecondary")}
          testId="tba-secondary"
          pressed={sidebarsVisible && secondaryOpen}
          onClick={showSecondary}
        >
          <PanelRightIcon />
        </IconButton>
      </div>
      <IconButton
        label={t("nav.ai")}
        testId="tba-ai"
        pressed={page === "ai"}
        onClick={() => setPage("ai")}
      >
        <AiIcon />
      </IconButton>
      <IconButton
        label={t("nav.settings")}
        testId="tba-settings"
        pressed={page === "settings"}
        onClick={() => setPage("settings")}
      >
        <GearIcon />
      </IconButton>
    </div>
  );
}
