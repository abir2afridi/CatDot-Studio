#!/usr/bin/env node
// pnpm audit:reality — self-audit gate (Part A3.8).
// Fails if product code matches banned patterns; checks registry consistency;
// regenerates docs/FEATURE_STATUS.md from feature-registry.json.
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { join, extname, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const REG_PATH = join(ROOT, "feature-registry.json");

// Banned patterns in product code (non-test, non-fixture).
const BANNED = [
  { re: /lorem ipsum/i, label: "lorem-ipsum" },
  { re: /setTimeout\s*\(/, label: "setTimeout-work-simulation" },
  { re: /unimplemented!/, label: "unimplemented!" },
  { re: /todo!\(/, label: "todo!()" },
  { re: /\bconsole\.log\s*\(/, label: "console.log-only-handler" },
];

const ALLOWLIST = new Set([
  // "path:line-label" entries with justification live here when needed.
]);

const SCAN_DIRS = ["apps", "packages", "crates"];
const SKIP = ["target", "node_modules", "dist", ".git"];

function* walk(dir) {
  let entries;
  try {
    entries = readdirSync(dir);
  } catch {
    return;
  }
  for (const e of entries) {
    if (SKIP.includes(e)) continue;
    const p = join(dir, e);
    let st;
    try {
      st = statSync(p);
    } catch {
      continue;
    }
    if (st.isDirectory()) yield* walk(p);
    else if ([".ts", ".tsx", ".rs"].includes(extname(p))) yield p;
  }
}

let failures = [];
for (const d of SCAN_DIRS) {
  for (const f of walk(join(ROOT, d))) {
    if (f.includes("__tests__") || f.includes(".test.") || f.includes("/tests/")) continue;
    let text;
    try {
      text = readFileSync(f, "utf8");
    } catch {
      continue;
    }
    const lines = text.split("\n");
    lines.forEach((line, i) => {
      // TODO comments reachable from UI are banned; allow ADR/docs refs.
      if (/\/\/\s*TODO|\/\*\s*TODO|#\s*TODO/i.test(line) && !line.includes("ALLOW:")) {
        failures.push(`${f}:${i + 1} TODO-reachable`);
      }
      for (const b of BANNED) {
        if (b.re.test(line)) {
          const key = `${f}:${i + 1}-${b.label}`;
          if (!ALLOWLIST.has(key)) failures.push(`${f}:${i + 1} banned:${b.label}`);
        }
      }
      // Hard-coded API keys (very conservative). Intentional test vectors
      // containing ForTestOnly/seeded-fake-key are waived (they prove
      // redaction works; see security::redaction tests).
      if (/sk-[A-Za-z0-9]{16,}|AIza[A-Za-z0-9_-]{20,}/.test(line)) {
        if (!/ForTestOnly|seeded-fake-key/i.test(line)) {
          failures.push(`${f}:${i + 1} possible-hardcoded-key`);
        }
      }
    });
  }
}

// Registry consistency + FEATURE_STATUS.md generation.
if (!existsSync(REG_PATH)) {
  failures.push("feature-registry.json missing");
} else {
  const reg = JSON.parse(readFileSync(REG_PATH, "utf8"));
  const valid = new Set(["real", "disabled-planned", "unverified-env"]);
  for (const f of reg.features ?? []) {
    if (!valid.has(f.status)) failures.push(`registry: bad status ${f.id}`);
    if (f.status === "real" && !(f.evidence && f.evidence.length > 0))
      failures.push(`registry: real without evidence ${f.id}`);
  }
  const lines = [
    "# FEATURE_STATUS",
    "",
    "Generated from `feature-registry.json` by `pnpm audit:reality`. Do not hand-edit.",
    "",
    "| id | title | phase | status | evidence |",
    "|---|---|---|---|---|",
  ];
  for (const f of reg.features ?? []) {
    lines.push(
      `| ${f.id} | ${f.title} | ${f.phase} | ${f.status} | ${(f.evidence ?? []).join("; ")} |`,
    );
  }
  writeFileSync(join(ROOT, "docs", "FEATURE_STATUS.md"), lines.join("\n") + "\n");
  console.log(`registry: ${(reg.features ?? []).length} features checked`);
}

if (failures.length > 0) {
  console.error("audit:reality FAILED:");
  for (const f of failures) console.error(" - " + f);
  process.exit(1);
}
console.log("audit:reality PASSED");
