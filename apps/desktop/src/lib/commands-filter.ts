// Command-palette filter: deterministic ranking used by the palette UI and
// covered by unit tests. Later commands >= earlier commands.

export function rank(query: string, haystack: string): number {
  const q = query.toLowerCase().trim();
  const h = haystack.toLowerCase();
  if (q === "") return 0;
  const idx = h.indexOf(q);
  if (idx === 0) return 3;
  if (idx > 0) {
    // word-boundary match beats a plain substring match
    return h[idx - 1] === " " ? 2 : 1;
  }
  // subsequence (e.g. "impk" -> "import apk"); low rank, still shown
  let from = 0;
  for (const c of q) {
    const at = h.indexOf(c, from);
    if (at < 0) return -1;
    from = at + 1;
  }
  return 0;
}

/** Stable ranked filter: matched-first, then input order. */
export function filterCommands<T extends { haystack: string }>(
  commands: readonly T[],
  query: string,
): T[] {
  return commands
    .map((c, i) => ({ c, i, r: rank(query, c.haystack) }))
    .filter((x) => x.r >= 0)
    .sort((a, b) => (a.r !== b.r ? b.r - a.r : a.i - b.i))
    .map((x) => x.c);
}
