import { useTranslation } from "react-i18next";

export function PlannedBadge({ phase }: { phase: number }) {
  const { t } = useTranslation();
  return (
    <span className="ml-2 rounded-full border border-warning/40 bg-warning/10 px-2 py-0.5 text-[10px] font-medium text-warning">
      {t("nav.planned", { phase })}
    </span>
  );
}
