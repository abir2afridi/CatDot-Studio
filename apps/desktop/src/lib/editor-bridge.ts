// Bridge to the live Monaco instance (ADR-0014): lets chrome outside the
// editor (menu bar) run REAL editor actions — undo/redo/find/select/copy —
// against whichever tab is mounted. No editor → actions are disabled.
import type { monaco } from "./monaco";

type CodeEditor = monaco.editor.IStandaloneCodeEditor;

let active: CodeEditor | null = null;

export function setActiveEditor(editor: CodeEditor | null): void {
  active = editor;
}

export function getActiveEditor(): CodeEditor | null {
  return active;
}

/** Run a built-in Monaco action id on the active editor (menu bridge). */
export function runEditorAction(actionId: string): void {
  active?.trigger("catdot-menu", actionId, null);
  active?.focus();
}
