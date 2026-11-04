import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import { Toast } from "./components/Toast";
import { AddAppDialog } from "./features/addApp/AddAppDialog";
import { EmptyState } from "./features/rules/EmptyState";
import { RuleDetail } from "./features/rules/RuleDetail";
import { RuleList } from "./features/rules/RuleList";
import { SettingsDialog } from "./features/settings/SettingsDialog";
import { useLang } from "./i18n";
import { useAppStore, useSelectedRule } from "./store/appStore";
import { useThemeSync } from "./theme";
import styles from "./App.module.css";

export function App() {
  const ready = useAppStore((s) => s.config !== null);
  const theme = useAppStore((s) => s.config?.settings.theme ?? "system");
  const accent = useAppStore((s) => s.config?.settings.accent ?? "blue");
  const dialog = useAppStore((s) => s.dialog);
  const hasRules = useAppStore((s) => (s.config?.rules.length ?? 0) > 0);
  const rule = useSelectedRule();
  const lang = useLang();

  useThemeSync(theme, accent);

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    useAppStore
      .getState()
      .init()
      .then((fn) => (unlisten = fn))
      .catch((e) => useAppStore.getState().showError(e));
    return () => unlisten?.();
  }, []);

  useAppShortcuts();

  if (!ready) return <div className={styles.app} />;

  return (
    <div className={styles.app}>
      <RuleList />
      {rule ? <RuleDetail rule={rule} /> : hasRules ? <main className={styles.blank} /> : <EmptyState />}
      {dialog === "addApp" && <AddAppDialog />}
      {dialog === "settings" && <SettingsDialog />}
      <Toast />
    </div>
  );
}

function useAppShortcuts() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      const { dialog, openDialog } = useAppStore.getState();
      if (mod && e.key === ",") {
        e.preventDefault();
        openDialog("settings");
      } else if ((mod && e.key.toLowerCase() === "w") || (e.key === "Escape" && !dialog)) {
        e.preventDefault();
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
