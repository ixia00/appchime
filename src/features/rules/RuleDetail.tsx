import { useState } from "react";

import { AppAvatar } from "../../components/AppAvatar";
import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { Icon } from "../../components/Icon";
import { useT } from "../../i18n";
import type { Rule } from "../../lib/types";
import { useAppStore } from "../../store/appStore";
import { TriggerCard } from "./TriggerCard";
import { useSoundDrop } from "./useSoundDrop";
import { VolumeRow } from "./VolumeRow";
import styles from "./RuleDetail.module.css";

export function RuleDetail({ rule }: { rule: Rule }) {
  const t = useT();
  const soundEnabled = useAppStore((s) => s.config?.settings.soundEnabled ?? true);
  const updateSettings = useAppStore((s) => s.updateSettings);
  const removeRule = useAppStore((s) => s.removeRule);
  const [confirming, setConfirming] = useState(false);
  const dropKind = useSoundDrop(rule);

  return (
    <main className={styles.detail}>
      {!soundEnabled && (
        <div className={styles.banner} role="status">
          <Icon name="speakerOff" />
          <span>{t("muted.banner")}</span>
          <Button variant="link" onClick={() => updateSettings({ soundEnabled: true })}>
            {t("muted.resume")}
          </Button>
        </div>
      )}

      <header className={styles.header}>
        <AppAvatar app={rule.app} size={48} />
        <div className={styles.titles}>
          <h1 className={styles.name}>{rule.app.name}</h1>
          <span className={styles.path} title={rule.app.bundlePath ?? rule.app.exePath}>
            {rule.app.bundlePath ?? rule.app.exePath}
          </span>
        </div>
        <Button variant="danger" onClick={() => setConfirming(true)}>
          {t("detail.remove")}
        </Button>
      </header>

      <div className={styles.cards} data-disabled={!rule.enabled || undefined}>
        <TriggerCard rule={rule} kind="launch" dropActive={dropKind === "launch"} />
        <TriggerCard rule={rule} kind="quit" dropActive={dropKind === "quit"} />
        <VolumeRow key={rule.id} rule={rule} />
      </div>

      {confirming && (
        <Dialog
          title={t("detail.remove")}
          width={400}
          onClose={() => setConfirming(false)}
          footer={
            <>
              <span style={{ flex: 1 }} />
              <Button onClick={() => setConfirming(false)} data-autofocus>
                {t("common.cancel")}
              </Button>
              <Button
                variant="primary"
                onClick={() => {
                  setConfirming(false);
                  void removeRule(rule.id);
                }}
              >
                {t("detail.removeConfirmAction")}
              </Button>
            </>
          }
        >
          <p className={styles.confirmText}>{t("detail.removeConfirm", { name: rule.app.name })}</p>
        </Dialog>
      )}
    </main>
  );
}
