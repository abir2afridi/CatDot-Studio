// Shared empty state for panel tabs.
export function PanelEmpty({ text }: { text: string }) {
  return <div className="p-3 text-xs text-text-dim">{text}</div>;
}
