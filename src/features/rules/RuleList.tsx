import { AppAvatar } from "../../components/AppAvatar";
import { Button } from "../../components/Button";
import { Icon } from "../../components/Icon";
import { Switch } from "../../components/Switch";
import { useT, type Translate } from "../../i18n";
import type { Rule } from "../../lib/types";
import { useAppStore } from "../../store/appStore";
import styles from "./RuleList.module.css";

export function RuleList() {
  const t = useT();
  const rules = useAppStore((s) => s.config?.rules ?? []);
  const selectedId = useAppStore((s) => s.selectedId);
  const select = useAppStore((s) => s.select);
  const updateRule = useAppStore((s) => s.updateRule);
  const openDialog = useAppStore((s) => s.openDialog);

  return (
    <nav className={styles.sidebar} aria-label={t("sidebar.heading")}>
      <h2 className={styles.heading}>{t("sidebar.heading")}</h2>

      <ul className={styles.list}>
        {rules.map((rule) => (
          <li key={rule.id} className={styles.row} data-selected={rule.id === selectedId || undefined}>
            <button
              type="button"
              className={styles.rowMain}
              aria-current={rule.id === selectedId ? "true" : undefined}
              onClick={() => select(rule.id)}
            >
              <AppAvatar app={rule.app} />
              <span className={styles.texts}>
                <span className={styles.name}>{rule.app.name}</span>
                <span className={styles.summary}>{summarize(rule, t)}</span>
              </span>
            </button>
            <Switch
              checked={rule.enabled}
              label={t("sidebar.toggleRule", { name: rule.app.name })}
              onChange={(enabled) => updateRule(rule.id, { enabled })}
            />
          </li>
        ))}
      </ul>

      <div className={styles.footer}>
        <Button variant="dashed" icon="plus" onClick={() => openDialog("addApp")}>
          {t("sidebar.add")}
        </Button>
        <button type="button" className={styles.settings} onClick={() => openDialog("settings")}>
          <Icon name="gear" />
          {t("sidebar.settings")}
        </button>
      </div>
    </nav>
  );
}

function summarize(rule: Rule, t: Translate): string {
  if (!rule.enabled) return t("summary.off");
  const parts: string[] = [];
  for (const [trigger, label] of [
    [rule.onLaunch, t("summary.launch")],
    [rule.onQuit, t("summary.quit")],
  ] as const) {
    if (!trigger.enabled || trigger.sounds.length === 0) continue;
    const count = trigger.sounds.length;
    parts.push(count > 1 ? `${label} ${t(`mode.${trigger.mode}`)} ${t("summary.count", { n: count })}` : label);
  }
  return parts.length ? parts.join(" · ") : t("summary.noSound");
}
