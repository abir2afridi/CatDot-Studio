// Monaco bootstrap (ADR-0014): self-hosted, offline, and deliberately slim —
// only the core editor feature registrations + XML tokenizer + editor worker
// are bundled (no json/css/html/ts language services → no dead worker
// bundles). `getMonaco()` is the single configure entry for the React
// wrapper (@monaco-editor/react loader.config).
import * as monaco from "monaco-editor/editor/editor.api";
import editorWorker from "monaco-editor/editor/editor.worker?worker";
import "monaco-editor/features/anchorSelect/register";
import "monaco-editor/features/bracketMatching/register";
import "monaco-editor/features/caretOperations/register";
import "monaco-editor/features/clipboard/register";
import "monaco-editor/features/codeEditor/register";
import "monaco-editor/features/comment/register";
import "monaco-editor/features/contextmenu/register";
import "monaco-editor/features/cursorUndo/register";
import "monaco-editor/features/diffEditor/register";
import "monaco-editor/features/find/register";
import "monaco-editor/features/folding/register";
import "monaco-editor/features/fontZoom/register";
import "monaco-editor/features/format/register";
import "monaco-editor/features/gotoError/register";
import "monaco-editor/features/gotoLine/register";
import "monaco-editor/features/hover/register";
import "monaco-editor/features/indentation/register";
import "monaco-editor/features/inPlaceReplace/register";
import "monaco-editor/features/insertFinalNewLine/register";
import "monaco-editor/features/inspectTokens/register";
import "monaco-editor/features/lineSelection/register";
import "monaco-editor/features/linesOperations/register";
import "monaco-editor/features/linkedEditing/register";
import "monaco-editor/features/links/register";
import "monaco-editor/features/multicursor/register";
import "monaco-editor/features/quickCommand/register";
import "monaco-editor/features/smartSelect/register";
import "monaco-editor/features/unicodeHighlighter/register";
import "monaco-editor/features/wordHighlighter/register";
import "monaco-editor/features/wordOperations/register";
import "monaco-editor/languages/definitions/xml/register";
import { loader } from "@monaco-editor/react";

const globalWindow = window as unknown as {
  MonacoEnvironment?: { getWorker?: () => Worker };
};
globalWindow.MonacoEnvironment = {
  getWorker: () => new editorWorker(),
};

let configured = false;

/** Configure the React wrapper to use the self-hosted monaco instance. */
export function getMonaco(): typeof monaco {
  if (!configured) {
    loader.config({ monaco });
    configured = true;
  }
  return monaco;
}

// CRITICAL: the wrapper resolves its loader on first mount — configure it NOW,
// at module import time, so it never tries the default CDN (blocked offline /
// by the app CSP, which left editors stuck on "Loading…" forever).
getMonaco();

/** Monaco theme name for the current app theme token. */
export function monacoTheme(appTheme: "dark" | "light"): string {
  return appTheme === "light" ? "vs" : "vs-dark";
}

/** Language id from an entry path (xml resources → xml; rest → plaintext). */
export function monacoLanguage(path: string): string {
  const leaf = path.split("/").pop() ?? path;
  const dot = leaf.lastIndexOf(".");
  const ext = dot >= 0 ? leaf.slice(dot + 1).toLowerCase() : "";
  if (["xml", "html", "svg"].includes(ext)) return "xml";
  return "plaintext";
}

export { monaco };
