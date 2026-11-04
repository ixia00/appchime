import { useId } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { Switch } from "../../components/Switch";
import { LANGUAGE_NAMES, useT } from "../../i18n";
import { api } from "../../lib/api";
import type { Accent, Language, Theme } from "../../lib/types";
import { SegmentedControl } from "../../components/SegmentedControl";
import { useAppStore } from "../../store/appStore";
import { ShortcutRecorder } from "./ShortcutRecorder";
import styles from "./SettingsDialog.module.css";

const DEDUPE_OPTIONS = [0, 3, 5, 10, 30, 60];
const THEMES: Theme[] = ["system", "light", "dark"];
const ACCENTS: Accent[] = ["blue", "purple", "pink", "orange", "green"];
const SWATCH: Record<Accent, string> = {
  blue: "#2f6feb",
  purple: "#7c4dd6",
  pink: "#c23069",
  orange: "#b85410",
  green: "#1e8050",
};

export function SettingsDialog() {
  const t = useT();
  const settings = useAppStore((s) => s.config?.settings);
  const version = useAppStore((s) => s.meta?.version ?? "");
  const { updateSettings, openDialog } = useAppStore.getState();
  const dedupeId = useId();
  const langId = useId();
  const loginId = useId();

  if (!settings) return null;
  const close = () => openDialog(null);

  return (
    <Dialog
      title={t("settings.title")}
      width={440}
      onClose={close}
      footer={
        <>
          <span className={styles.version}>{t("settings.version", { v: version })}</span>
          <Button variant="link" onClick={() => api.quitApp()}>
            {t("settings.quit")}
          </Button>
          <span style={{ flex: 1 }} />
          <Button variant="primary" onClick={close} data-autofocus>
            {t("common.done")}
          </Button>
        </>
      }
    >
      <div className={styles.row}>
        <span id={loginId} className={styles.label}>
          {t("settings.launchAtLogin")}
        </span>
        <Switch
          checked={settings.launchAtLogin}
          label={t("settings.launchAtLogin")}
          onChange={(launchAtLogin) => updateSettings({ launchAtLogin })}
        />
      </div>

      <div className={styles.group}>
        <div className={styles.row}>
          <span className={styles.label}>{t("settings.shortcut")}</span>
          <ShortcutRecorder
            value={settings.toggleShortcut}
            onChange={(toggleShortcut) => updateSettings({ toggleShortcut })}
          />
        </div>
        <p className={styles.hint}>{t("settings.shortcut.hint")}</p>
      </div>

      <div className={styles.group}>
        <div className={styles.row}>
          <label htmlFor={dedupeId} className={styles.label}>
            {t("settings.dedupe")}
          </label>
          <select
            id={dedupeId}
            className={styles.select}
            value={settings.dedupeSeconds}
            onChange={(e) => updateSettings({ dedupeSeconds: Number(e.target.value) })}
          >
            {DEDUPE_OPTIONS.includes(settings.dedupeSeconds) ? null : (
              <option value={settings.dedupeSeconds}>{t("settings.dedupe.sec", { n: settings.dedupeSeconds })}</option>
            )}
            {DEDUPE_OPTIONS.map((n) => (
              <option key={n} value={n}>
                {n === 0 ? t("settings.dedupe.off") : t("settings.dedupe.sec", { n })}
              </option>
            ))}
          </select>
        </div>
        <p className={styles.hint}>{t("settings.dedupe.hint")}</p>
      </div>

      <div className={styles.row}>
        <span className={styles.label}>{t("settings.appearance")}</span>
        <SegmentedControl
          label={t("settings.appearance")}
          value={settings.theme}
          options={THEMES.map((v) => ({ value: v, label: t(`settings.appearance.${v}`) }))}
          onChange={(theme) => updateSettings({ theme })}
        />
      </div>

      <div className={styles.row}>
        <span className={styles.label}>{t("settings.accent")}</span>
        <div role="radiogroup" aria-label={t("settings.accent")} className={styles.swatches}>
          {ACCENTS.map((a) => (
            <button
              key={a}
              type="button"
              role="radio"
              aria-checked={settings.accent === a}
              aria-label={t(`accent.${a}`)}
              title={t(`accent.${a}`)}
              className={styles.swatch}
              style={{ background: SWATCH[a] }}
              onClick={() => updateSettings({ accent: a })}
            />
          ))}
        </div>
      </div>
      
      <div className={styles.row}>
        <label htmlFor={langId} className={styles.label}>
          {t("settings.language")}
        </label>
        <select
          id={langId}
          className={styles.select}
          value={settings.language}
          onChange={(e) => updateSettings({ language: e.target.value as Language })}
        >
          <option value="system">{t("lang.system")}</option>
          {Object.entries(LANGUAGE_NAMES).map(([code, name]) => (
            <option key={code} value={code}>
              {name}
            </option>
          ))}
        </select>
      </div>
    </Dialog>
  );
}
