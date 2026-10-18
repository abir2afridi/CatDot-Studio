// Global IDE keyboard shortcuts (Part X1). Guarded: never swallow plain
// typing; only real Ctrl chords act. Shortcuts are discoverable from the
// palette (hay includes "ctrl+…").
import { useEffect } from "react";
import { useUi } from "../state/ui";

function isEditable(el: EventTarget | null): boolean {
  return el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement;
}

export function useShortcuts(): void {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const ui = useUi.getState();
      const ctrl = e.ctrlKey && !e.altKey;
      if (!ctrl) return;

      // Ctrl+Shift+P — command palette
      if (e.shiftKey && (e.key === "P" || e.key === "p")) {
        e.preventDefault();
        ui.setPaletteOpen(!ui.paletteOpen);
        return;
      }
      if (ui.paletteOpen) {
        // keep Escape for the palette even while typing its input
        if (e.key === "Escape") {
          e.preventDefault();
          ui.setPaletteOpen(false);
        }
        return;
      }
      switch (e.key.toLowerCase()) {
        case "b": // toggle sidebar
          e.preventDefault();
          ui.toggleExplorer();
          break;
        case "j": // toggle bottom panel
          e.preventDefault();
          ui.toggleBottom();
          break;
        case "s": // save active tab (only when a draft is dirty)
          if (isEditable(e.target)) {
            e.preventDefault();
            ui.requestSave();
          }
          break;
        case "w": // close active tab
          if (ui.activeFile != null) {
            e.preventDefault();
            ui.closeFile(ui.activeFile);
          }
          break;
        case "k": // alt navigation: Ctrl+K opens the palette too (below P)
          if (!e.shiftKey) {
            e.preventDefault();
            ui.setPaletteOpen(true);
          }
          break;
        default:
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
