// Splitter resize handles (Part X1 layout engineering): drag or arrow-key
// resize for the sidebar (horizontal) and bottom panel (vertical). The final
// value on mouse release is persisted through the settings IPC.
import { useEffect } from "react";
import { BOTTOM_HEIGHT, EXPLORER_WIDTH, useUi } from "../state/ui";
import { settingsGet, settingsSet } from "../lib/ipc";

export function SidebarResizeHandle() {
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      tabIndex={0}
      title={useUi.getState().explorerWidth.toString()}
      className="w-1 cursor-col-resize bg-border hover:bg-accent/40 focus:bg-accent/60 focus:outline-none"
      onMouseDown={beginResize("x")}
      onKeyDown={(e) => {
        const w = useUi.getState().explorerWidth;
        if (e.key === "ArrowLeft") {
          useUi.getState().setExplorerWidth(w - 16);
        } else if (e.key === "ArrowRight") {
          useUi.getState().setExplorerWidth(w + 16);
        }
      }}
    />
  );
}

export function BottomResizeHandle() {
  return (
    <div
      role="separator"
      aria-orientation="horizontal"
      tabIndex={0}
      title="Drag to resize"
      className="h-1 cursor-row-resize bg-border hover:bg-accent/40 focus:bg-accent/60 focus:outline-none"
      onMouseDown={beginResize("y")}
      onKeyDown={(e) => {
        const h = useUi.getState().bottomHeight;
        if (e.key === "ArrowUp") {
          useUi.getState().setBottomHeight(h - 16);
        } else if (e.key === "ArrowDown") {
          useUi.getState().setBottomHeight(h + 16);
        }
      }}
    />
  );
}

type Axis = "x" | "y";

function beginResize(axis: Axis): (e: React.MouseEvent) => void {
  return (down) => {
    down.preventDefault();

    const apply = (e: MouseEvent) => {
      if (axis === "x") {
        // mouse X minus the activity-bar width (56px) = sidebar width
        useUi.getState().setExplorerWidth(e.clientX - 56);
      } else {
        // bottom panel: the mouse Y directly defines the panel height
        useUi.getState().setBottomHeight(window.innerHeight - e.clientY);
      }
    };

    const onUp = () => {
      void persist(axis);
    };
    window.addEventListener("mousemove", apply);
    window.addEventListener("mouseup", onUp, { once: true });
  };
}

/** Persist the current layout value (debounced by the mouse-release event). */
function persist(axis: Axis): Promise<unknown> {
  const { explorerWidth, bottomHeight } = useUi.getState();
  return axis === "x"
    ? settingsSet("ui.explorerWidth", Math.round(clampW(explorerWidth)))
    : settingsSet("ui.bottomHeight", Math.round(clampH(bottomHeight)));
}

const clampW = (n: number) =>
  Math.min(Math.max(n, EXPLORER_WIDTH.min), EXPLORER_WIDTH.max);
const clampH = (n: number) =>
  Math.min(Math.max(n, BOTTOM_HEIGHT.min), BOTTOM_HEIGHT.max);

/** Restore persisted layout once at startup (misses = defaults kept). */
export function useLayoutPersistence(): void {
  useEffect(() => {
    void (async () => {
      const ui = useUi.getState();
      const [width, height] = await Promise.all([
        settingsGet("ui.explorerWidth").catch(() => null),
        settingsGet("ui.bottomHeight").catch(() => null),
      ]);
      if (typeof width === "number" && Number.isFinite(width)) {
        ui.setExplorerWidth(width);
      }
      if (typeof height === "number" && Number.isFinite(height)) {
        ui.setBottomHeight(height);
      }
    })();
  }, []);
}
