import { useEffect, useState } from "react";

import { SegmentedControl } from "../../components/SegmentedControl";
import { Icon } from "../../components/Icon";
import { Switch } from "../../components/Switch";
import { useT } from "../../i18n";
import { api } from "../../lib/api";
import { triggerOf, type LaunchTiming, type PlayMode, type Rule, type TriggerKind } from "../../lib/types";
import { useAppStore } from "../../store/appStore";
import { SoundList } from "./SoundList";
import styles from "./TriggerCard.module.css";

const MODES: PlayMode[] = ["fixed", "random", "sequential"];
const TIMINGS: LaunchTiming[] = ["mainWindow", "process"];
const DELAYS = [0, 1000, 2000, 3000, 5000];

interface TriggerCardProps {
  rule: Rule;
  kind: TriggerKind;
  dropActive: boolean;
}

export function TriggerCard({ rule, kind, dropActive }: TriggerCardProps) {
  const t = useT();
  const updateRule = useAppStore((s) => s.updateRule);
  const platform = useAppStore((s) => s.meta?.platform);
  const trigger = triggerOf(rule, kind);
  const title = t(kind === "launch" ? "trigger.launch" : "trigger.quit");
  const patchKey = kind === "launch" ? "onLaunch" : "onQuit";
  const showTiming = kind === "launch";

  const [trusted, setTrusted] = useState(true);
  useEffect(() => {
    if (!showTiming || platform !== "macos") return;
    const check = () => void api.accessibilityStatus(false).then(setTrusted);
    check();
    window.addEventListener("focus", check);
    return () => window.removeEventListener("focus", check);
  }, [showTiming, platform]);

  return (
    <section
      className={styles.card}
      aria-label={title}
      data-drop-kind={trigger.enabled ? kind : undefined}
      data-drop-active={dropActive || undefined}
    >
      <div className={styles.head}>
        <Icon name={kind === "launch" ? "speaker" : "speakerOff"} size={18} />
        <h2 className={styles.title}>{title}</h2>
        <Switch
          checked={trigger.enabled}
          label={title}
          onChange={(enabled) => updateRule(rule.id, { [patchKey]: { enabled } })}
        />
      </div>

      {trigger.enabled ? (
        <>
          <SoundList rule={rule} kind={kind} />

          <div className={styles.modeRow}>
            <span className={styles.modeLabel}>{t("mode.label")}</span>
            <SegmentedControl
              label={t("mode.label")}
              value={trigger.mode}
              options={MODES.map((m) => ({ value: m, label: t(`mode.${m}`) }))}
              onChange={(mode) => updateRule(rule.id, { [patchKey]: { mode } })}
            />
            <span className={styles.modeHint}>{t(`mode.${trigger.mode}.hint`)}</span>
          </div>

          {showTiming && (
            <div className={styles.modeRow}>
              <span className={styles.modeLabel}>{t("timing.label")}</span>
              <SegmentedControl
                label={t("timing.label")}
                value={trigger.timing}
                options={TIMINGS.map((v) => ({ value: v, label: t(`timing.${v}`) }))}
                onChange={(timing) => updateRule(rule.id, { onLaunch: { timing } })}
              />
              <select
                aria-label={t("timing.delay")}
                className={styles.delay}
                value={trigger.delayMs}
                onChange={(e) => updateRule(rule.id, { onLaunch: { delayMs: Number(e.target.value) } })}
              >
                {DELAYS.map((ms) => (
                  <option key={ms} value={ms}>
                    {t("timing.delay")}: {ms === 0 ? t("timing.delayNone") : t("timing.delaySec", { n: ms / 1000 })}
                  </option>
                ))}
              </select>
            </div>
          )}

          {showTiming && trigger.timing === "mainWindow" && !trusted && (
            <div className={styles.permission}>
              <span>{t("timing.needPermission")}</span>
              <button type="button" onClick={() => void api.accessibilityStatus(true)}>
                {t("timing.grant")}
              </button>
            </div>
          )}
        </>
      ) : (
        <p className={styles.offHint}>{t("trigger.offHint")}</p>
      )}

      {dropActive && (
        <div className={styles.dropOverlay} aria-hidden="true">
          {t("sounds.drop")}
        </div>
      )}
    </section>
  );
}