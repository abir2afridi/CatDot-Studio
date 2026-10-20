// Reusable settings form atoms (label field, toggle chip, text input).
export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block">
      <span className="text-sm text-text-dim">{label}</span>
      <div className="mt-1">{children}</div>
      {hint && <span className="mt-1 block text-xs text-text-dim/70">{hint}</span>}
    </label>
  );
}

export function Toggle({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-md border px-3 py-1.5 text-sm ${
        active
          ? "border-accent bg-accent/15 text-accent"
          : "border-border text-text-dim hover:bg-surface-2"
      }`}
    >
      {children}
    </button>
  );
}

export function TextInput({
  value,
  onChange,
  type = "text",
  placeholder,
  mono,
}: {
  value: string;
  onChange: (v: string) => void;
  type?: string;
  placeholder?: string;
  mono?: boolean;
}) {
  return (
    <input
      type={type}
      value={value}
      placeholder={placeholder}
      onChange={(e) => onChange(e.currentTarget.value)}
      className={`w-full rounded-md border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent ${
        mono ? "mono" : ""
      }`}
    />
  );
}
