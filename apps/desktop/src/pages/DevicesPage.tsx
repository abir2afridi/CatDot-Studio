// Device Explorer (Phase 4): the real `adb devices -l` list with an explicit
// target selection that drives install + device logs. Honest empty/error
// states — no fabricated devices.
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import type { DeviceInfo } from "@catdot/shared-types";
import { apkStatus, deviceInstallSigned, deviceList } from "../lib/apk";
import { describeError } from "../lib/opener";
import { useUi } from "../state/ui";

export function DevicesPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const selected = useUi((s) => s.deviceSerial);
  const setSelected = useUi((s) => s.setDeviceSerial);
  const setActiveBottomTab = useUi((s) => s.setActiveBottomTab);
  const bottomOpen = useUi((s) => s.bottomOpen);
  const toggleBottom = useUi((s) => s.toggleBottom);
  const setNotice = useUi((s) => s.setNotice);

  const devices = useQuery({
    queryKey: ["devices"],
    queryFn: deviceList,
    retry: false,
    refetchInterval: 10_000,
  });
  const status = useQuery({ queryKey: ["apk", "status"], queryFn: apkStatus, retry: false });

  const installMut = useMutation({
    mutationFn: (serial: string | null) => deviceInstallSigned(serial),
    onSuccess: (serial) => {
      setNotice({ text: t("device.installed", { serial }), ok: true });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setNotice({ text: describeError(e), ok: false }),
  });

  const list = devices.data ?? [];
  const ready = list.find((d) => d.state === "device");
  const target = selected != null ? list.find((d) => d.serial === selected) : undefined;
  const effective = target ?? ready;
  const hasSigned = status.data?.signed_apk != null;

  const openLogs = () => {
    setActiveBottomTab("device");
    if (!bottomOpen) toggleBottom();
  };

  return (
    <div className="mx-auto h-full w-full max-w-3xl overflow-auto p-6">
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold">{t("nav.devices")}</h1>
        <button
          type="button"
          disabled={devices.isFetching}
          onClick={() => void devices.refetch()}
          className="rounded-md border border-border px-2 py-0.5 text-xs text-text-dim hover:bg-surface-2 hover:text-text disabled:opacity-50"
        >
          {devices.isFetching ? t("device.reading") : t("devices.refresh")}
        </button>
        {effective != null && (
          <span className="ml-auto text-xs text-text-dim">
            {t("devices.target", { serial: effective.serial })}
          </span>
        )}
      </div>

      {devices.isError ? (
        <p className="mt-4 text-sm text-warning">{describeError(devices.error)}</p>
      ) : list.length === 0 ? (
        <div className="mt-4 rounded-lg border border-dashed border-border p-4">
          <p className="text-sm text-text-dim">{t("devices.none")}</p>
          <p className="mt-1 text-xs text-text-dim/70">{t("devices.hint")}</p>
        </div>
      ) : (
        <ul className="mt-4 space-y-2">
          {list.map((d) => (
            <DeviceRow
              key={d.serial}
              device={d}
              selected={effective?.serial === d.serial}
              hasSigned={hasSigned}
              installing={installMut.isPending}
              onSelect={() => setSelected(d.serial)}
              onInstall={() => installMut.mutate(d.serial)}
              onLogs={() => {
                setSelected(d.serial);
                openLogs();
              }}
            />
          ))}
        </ul>
      )}
      <p className="mt-6 text-xs text-text-dim/70">{t("devices.footnote")}</p>
    </div>
  );
}

function DeviceRow({
  device,
  selected,
  hasSigned,
  installing,
  onSelect,
  onInstall,
  onLogs,
}: {
  device: DeviceInfo;
  selected: boolean;
  hasSigned: boolean;
  installing: boolean;
  onSelect: () => void;
  onInstall: () => void;
  onLogs: () => void;
}) {
  const { t } = useTranslation();
  const stateCls =
    device.state === "device"
      ? "bg-success/15 text-success"
      : device.state === "unauthorized"
        ? "bg-warning/15 text-warning"
        : "bg-surface-2 text-text-dim";
  const stateLabel =
    device.state === "device"
      ? t("devices.stateDevice")
      : device.state === "unauthorized"
        ? t("devices.stateUnauthorized")
        : device.state === "offline"
          ? t("devices.stateOffline")
          : device.state;
  return (
    <li
      className={`rounded-lg border p-3 ${selected ? "border-accent/60 bg-accent/5" : "border-border bg-surface"}`}
    >
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onSelect}
          aria-pressed={selected}
          className="font-mono text-sm text-text hover:underline"
          title={t("devices.use")}
        >
          {device.serial}
        </button>
        <span className={`rounded px-1.5 py-0.5 text-[11px] ${stateCls}`}>{stateLabel}</span>
        {device.model != null && <span className="text-xs text-text-dim">{device.model}</span>}
        {selected && <span className="ml-auto text-[11px] text-accent">{t("devices.inUse")}</span>}
      </div>
      <div className="mt-2 flex items-center gap-2">
        <button
          type="button"
          disabled={!hasSigned || device.state !== "device" || installing}
          onClick={onInstall}
          title={hasSigned ? undefined : t("cmd.noSigned")}
          className="rounded-md bg-success/15 px-2 py-0.5 text-xs text-success hover:bg-success/25 disabled:opacity-50"
        >
          {installing ? t("device.installing") : t("devices.install")}
        </button>
        <button
          type="button"
          disabled={device.state !== "device"}
          onClick={onLogs}
          className="rounded-md border border-border px-2 py-0.5 text-xs text-text-dim hover:bg-surface-2 hover:text-text disabled:opacity-50"
        >
          {t("devices.openLogs")}
        </button>
      </div>
    </li>
  );
}
