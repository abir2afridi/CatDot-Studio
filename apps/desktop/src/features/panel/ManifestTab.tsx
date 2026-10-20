// Manifest analyzer tab (Phase 2 slice): real `aapt2 dump xmltree` output,
// parsed by the Rust core — package, label, versions, permissions, components.
// Read-only: editing happens via the structured ops in Guided/Palette.
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkStatus, apkManifest } from "../lib/apk";import { IpcError, normalize } from "../lib/ipc";

export function ManifestTab() {
  const { t } = useTranslation();
  const status = useQuery({
    queryKey: ["apk", "status"],
    queryFn: apkStatus,
    staleTime: 3_000,
  });
  const hasApk = status.data?.imported === true;

  const q = useQuery({
    queryKey: ["apk-manifest"],
    queryFn: apkManifest,
    enabled: hasApk,
    refetchInterval: false,
    staleTime: 60_000,
    retry: false,
  });

  if (!hasApk) {
    return <Empty text={t("manifest.noApk")} />;
  }
  if (q.isPending) {
    return <Empty text={t("manifest.analyzing")} />;
  }
  if (q.isError) {
    return <ErrorBlock e={q.error} />;
  }
  const info = q.data;
  return (
    <div data-testid="manifest-view" className="h-full overflow-y-auto px-3 py-2 text-xs leading-5">
      <dl className="mb-2 grid grid-cols-[auto,1fr] gap-x-3">
        <Kv k={t("manifest.package")} v={info.package ?? "—"} />
        <Kv k={t("manifest.label")} v={info.label ?? "—"} />
        <Kv k={t("manifest.version")} v={`${info.version_code ?? "?"} (${info.version_name ?? "?"})`} />
        <Kv k={t("manifest.sdk")} v={`${t("manifest.minSdk")} ${info.min_sdk ?? "?"} · ${t("manifest.targetSdk")} ${info.target_sdk ?? "?"}`} />
      </dl>
      <section className="mb-2">
        {/* permissions sorted; count chip in the section head like the tree */}
        <h3 className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-text-dim">
          {t("manifest.permissions")} ({info.permissions.length})
        </h3>
        {info.permissions.length === 0 ? (
          <p className="text-text-dim">{t("manifest.noPermissions")}</p>
        ) : (
          <ul className="font-mono text-[11px]">
            {[...info.permissions].sort().map((p) => (
              <li key={p} className="px-1 py-0.5">{p}</li>
            ))}
          </ul>
        )}
      </section>
      <section>
        <h3 className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-text-dim">
          {t("manifest.components")} ({info.components.length})
        </h3>
        {info.components.length === 0 ? (
          <p className="text-text-dim">{t("manifest.noComponents")}</p>
        ) : (
          <ul className="font-mono text-[11px]">
            {info.components.map((c, i) => (
              <li key={`${c.kind}-${c.name ?? i}`} className="px-1 py-0.5">
                <span className="text-accent">{c.kind}</span>{" "}
                <span className="break-all">{c.name ?? "?"}</span>
                {c.exported === "true" && <span className="text-warning"> · {t("manifest.exported")}</span>}
                {(c.actions.length > 0 || c.categories.length > 0) && (
                  <span className="text-text-dim">
                    {" "}
                    · {[...c.actions, ...c.categories].join(", ")}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function Kv({ k, v }: { k: string; v: string }) {
  return (
    <>
      <dt className="text-text-dim">{k}</dt>
      <dd className="break-all font-mono">{v}</dd>
    </>
  );
}

function ErrorBlock({ e }: { e: unknown }) {
  if (e instanceof IpcError) {
    return (
      <div className="p-3 text-xs text-danger">
        <p className="font-semibold">{e.appError.message}</p>
        {e.appError.user_message && <p className="text-text-dim">{e.appError.user_message}</p>}
      </div>
    );
  }
  return <Empty text={normalize(e).message} />;
}

function Empty({ text }: { text: string }) {
  return <p className="p-3 text-xs text-text-dim">{text}</p>;
}

