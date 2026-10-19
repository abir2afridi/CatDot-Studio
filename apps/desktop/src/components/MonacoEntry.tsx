// Monaco-based entry editor (ADR-0014): the VS Code engine on the row where
// it matters (find/replace, syntax tokens for XML, real caret tracking).
// Self-hosted + offline (see lib/monaco.ts).
import { Editor, type OnMount, type Monaco } from "@monaco-editor/react";
import { useTranslation } from "react-i18next";
import { getMonaco, monacoLanguage, monacoTheme, monaco } from "../lib/monaco";
import { getActiveEditor, setActiveEditor } from "../lib/editor-bridge";
import { useUi } from "../state/ui";

interface Props {
  entryPath: string;
  /** Committed content from the workspace (server value). */
  content: string;
}

/** Tracks cursor line/col for the status bar; returns an unlisten fn. */
function trackCaret(editor: monaco.editor.IStandaloneCodeEditor): void {
  editor.onDidChangeCursorPosition((e) => {
    useUi.getState().setCursor(e.position.lineNumber, e.position.column);
  });
}

export function MonacoEntry({ entryPath, content }: Props) {
  const { t } = useTranslation();
  const theme = useUi((s) => s.theme);
  const draft = useUi((s) => s.drafts[entryPath]);
  const setDraft = useUi((s) => s.setDraft);

  const onMount: OnMount = (editor) => {
    setActiveEditor(editor);
    editor.onDidDispose(() => {
      if (getActiveEditor() === editor) setActiveEditor(null);
    });
    trackCaret(editor);
  };

  const beforeMount = (m: Monaco) => {
    getMonaco();
    void m;
  };

  return (
    <Editor
      height="100%"
      language={monacoLanguage(entryPath)}
      theme={monacoTheme(theme)}
      value={draft ?? content}
      beforeMount={beforeMount}
      onMount={onMount}
      onChange={(v) => {
        if (v != null && v !== content) setDraft(entryPath, v);
      }}
      loading={<div className="p-3 text-sm text-text-dim">{t("common.loading")}</div>}
      options={{
        minimap: { enabled: false },
        fontSize: 13,
        fontFamily: "Cascadia Code, Consolas, Menlo, monospace",
        wordWrap: "on",
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 4,
        renderWhitespace: "boundary",
      }}
      aria-label={`${entryPath} (${t("editor.unsaved")}: ${draft != null ? "yes" : "no"})`}
    />
  );
}
