// Editor area: welcome surface or tab strip + the active entry editor.
import { useUi } from "../../state/ui";
import { EntryEditor } from "./EntryEditor";
import { TabStrip } from "./TabStrip";
import { Welcome } from "./Welcome";

export function EditorArea() {
  const openFiles = useUi((s) => s.openFiles);
  const activeFile = useUi((s) => s.activeFile);

  if (activeFile == null || !openFiles.includes(activeFile)) {
    return <Welcome />;
  }
  return (
    <div className="flex h-full min-w-0 flex-col">
      <TabStrip />
      <EntryEditor key={activeFile} entryPath={activeFile} />
    </div>
  );
}
