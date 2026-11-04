import { Button } from "../../components/Button";
import { Icon } from "../../components/Icon";
import { useT } from "../../i18n";
import { useAppStore } from "../../store/appStore";
import styles from "./EmptyState.module.css";

export function EmptyState() {
  const t = useT();
  const openDialog = useAppStore((s) => s.openDialog);
  return (
    <main className={styles.empty}>
      <span className={styles.icon}>
        <Icon name="music" size={28} />
      </span>
      <h1 className={styles.title}>{t("empty.title")}</h1>
      <p className={styles.body}>{t("empty.body")}</p>
      <Button variant="primary" icon="plus" onClick={() => openDialog("addApp")}>
        {t("empty.cta")}
      </Button>
    </main>
  );
}
