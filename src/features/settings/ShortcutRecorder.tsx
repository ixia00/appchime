import { useEffect, useState } from "react";

import { IconButton } from "../../components/Button";
import { useT } from "../../i18n";
import { acceleratorFromEvent, formatAccelerator } from "../../lib/shortcut";
import { useAppStore } from "../../store/appStore";
import styles from "./ShortcutRecorder.module.css";

interface ShortcutRecorderProps {
  value: string | null;
  onChange: (value: string | null) => Promise<boolean>;
}

export function ShortcutRecorder({ value, onChange }: ShortcutRecorderProps) {
  const t = useT();
  const platform = useAppStore((s) => s.meta?.platform ?? "macos");
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        setRecording(false);
        return;
      }
      if ((e.key === "Backspace" || e.key === "Delete") && !e.metaKey && !e.ctrlKey && !e.altKey) {
        setRecording(false);
        void onChange(null);
        return;
      }
      const accel = acceleratorFromEvent(e, platform);
      if (!accel) return;
      setRecording(false);
      void onChange(accel);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, onChange, platform]);

  return (
    <div className={styles.wrap}>
      <button
        type="button"
        className={styles.field}
        data-recording={recording || undefined}
        data-empty={!value || undefined}
        aria-label={`${t("settings.shortcut")}: ${value ? formatAccelerator(value, platform) : t("settings.shortcut.record")}`}
        onClick={() => setRecording((r) => !r)}
        onBlur={() => setRecording(false)}
      >
        {recording
          ? t("settings.shortcut.recording")
          : value
            ? formatAccelerator(value, platform)
            : t("settings.shortcut.record")}
      </button>
      {value && !recording && (
        <IconButton icon="close" label={t("settings.shortcut.clear")} onClick={() => onChange(null)} />
      )}
    </div>
  );
}
