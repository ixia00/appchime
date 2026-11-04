import { useEffect } from "react";

import { useT } from "../i18n";
import { useAppStore } from "../store/appStore";
import { IconButton } from "./Button";
import styles from "./Toast.module.css";

const VISIBLE_MS = 5000;

export function Toast() {
  const toast = useAppStore((s) => s.toast);
  const dismiss = useAppStore((s) => s.dismissToast);
  const t = useT();

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(dismiss, VISIBLE_MS);
    return () => window.clearTimeout(timer);
  }, [toast, dismiss]);

  return (
    <div className={styles.region} role="status" aria-live="polite">
      {toast && (
        <div key={toast.id} className={styles.toast}>
          <span className={styles.text}>{t(toast.key, toast.vars)}</span>
          <IconButton icon="close" label={t("common.close")} onClick={dismiss} />
        </div>
      )}
    </div>
  );
}
