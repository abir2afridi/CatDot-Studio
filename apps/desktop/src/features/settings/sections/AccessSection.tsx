// Access level: persisted on change (global scope) with honest status.
import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { settingsGet, settingsSet } from "../../../lib/ipc";
import { Field, Toggle } from "../SettingsControls";

export function AccessSection() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [accessLevel, setAccessLevel] = useState(1);
  const [error, setError] = useState<string | null>(null);

  const { data: savedAccess } = useQuery({
    queryKey: ["settings", "access-level"],
    queryFn: () => settingsGet("safety.access_level"),
  });

  useEffect(() => {
    if (typeof savedAccess === "number") setAccessLevel(savedAccess);
  }, [savedAccess]);

  const save = useMutation({
    mutationFn: (level: number) => settingsSet("safety.access_level", level, "global"),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["settings"] }),
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  });

  function choose(level: number) {
    setAccessLevel(level);
    setError(null);
    save.mutate(level);
  }

  return (
    <section className="mt-6 space-y-4">
      <Field label={t("settings.accessLevel")} hint={error ?? undefined}>
        <div className="flex gap-2">
          {[0, 1, 2].map((lvl) => (
            <Toggle key={lvl} active={accessLevel === lvl} onClick={() => choose(lvl)}>
              L{lvl}
            </Toggle>
          ))}
        </div>
      </Field>
    </section>
  );
}
