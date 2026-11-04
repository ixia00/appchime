import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";

import { AppAvatar } from "../../components/AppAvatar";
import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { Icon } from "../../components/Icon";
import { SegmentedControl } from "../../components/SegmentedControl";
import { useT } from "../../i18n";
import { api } from "../../lib/api";
import type { AppIdentity } from "../../lib/types";
import { useAppStore } from "../../store/appStore";
import styles from "./AddAppDialog.module.css";

type Source = "running" | "installed";

const keyOf = (app: AppIdentity) => app.exePath.toLowerCase();

export function AddAppDialog() {
  const t = useT();
  const platform = useAppStore((s) => s.meta?.platform ?? "macos");
  const rules = useAppStore((s) => s.config?.rules ?? []);
  const { addRule, openDialog, showError } = useAppStore.getState();

  const [source, setSource] = useState<Source>("running");
  const [apps, setApps] = useState<AppIdentity[] | null>(null);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<AppIdentity | null>(null);

  const registered = useMemo(() => new Set(rules.map((r) => keyOf(r.app))), [rules]);
  const close = () => openDialog(null);

  useEffect(() => {
    let alive = true;
    setApps(null);
    setSelected(null);
    (source === "running" ? api.listRunningApps() : api.listInstalledApps())
      .then((list) => alive && setApps(list))
      .catch((e) => {
        if (!alive) return;
        setApps([]);
        showError(e);
      });
    return () => {
      alive = false;
    };
  }, [source, showError]);

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (apps ?? []).filter((a) => !q || a.name.toLowerCase().includes(q));
  }, [apps, query]);

  const add = async (app: AppIdentity | null) => {
    if (!app || registered.has(keyOf(app))) return;
    if (await addRule(app)) close();
  };

  const browse = async () => {
    const picked = await open({
      title: t("addApp.browseTitle"),
      multiple: false,
      directory: false,
      defaultPath: platform === "macos" ? "/Applications" : undefined,
      filters:
        platform === "macos"
          ? [{ name: "App", extensions: ["app"] }]
          : platform === "windows"
            ? [{ name: "App", extensions: ["exe"] }]
            : undefined,
    });
    if (typeof picked !== "string") return;
    try {
      await add(await api.identityFromPath(picked));
    } catch (e) {
      showError(e);
    }
  };

  return (
    <Dialog
      title={t("addApp.title")}
      width={520}
      height={520}
      onClose={close}
      footer={
        <>
          <Button variant="ghost" icon="folder" onClick={browse}>
            {t("addApp.browse")}
          </Button>
          <span style={{ flex: 1 }} />
          <Button onClick={close}>{t("common.cancel")}</Button>
          <Button variant="primary" disabled={!selected} onClick={() => add(selected)}>
            {t("addApp.add")}
          </Button>
        </>
      }
    >
      <SegmentedControl
        label={t("addApp.title")}
        value={source}
        onChange={setSource}
        options={[
          { value: "running", label: t("addApp.running") },
          { value: "installed", label: t("addApp.installed") },
        ]}
      />

      <label className={styles.search}>
        <Icon name="search" size={14} />
        <span className="visually-hidden">{t("addApp.search")}</span>
        <input
          data-autofocus
          type="search"
          value={query}
          placeholder={t("addApp.search")}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && add(selected ?? visible[0] ?? null)}
        />
      </label>

      <div className={styles.list} role="listbox" aria-label={t("addApp.title")}>
        {apps === null ? (
          <p className={styles.message}>{t("addApp.loading")}</p>
        ) : visible.length === 0 ? (
          <p className={styles.message}>{query ? t("addApp.noMatch", { q: query }) : t("addApp.empty")}</p>
        ) : (
          visible.map((app) => {
            const already = registered.has(keyOf(app));
            const isSelected = selected !== null && keyOf(selected) === keyOf(app);
            return (
              <button
                key={keyOf(app)}
                type="button"
                role="option"
                aria-selected={isSelected}
                aria-disabled={already}
                className={styles.item}
                data-selected={isSelected || undefined}
                onClick={() => !already && setSelected(app)}
                onDoubleClick={() => add(app)}
                title={app.bundlePath ?? app.exePath}
              >
                <AppAvatar app={app} size={28} />
                <span className={styles.itemName}>{app.name}</span>
                {already ? (
                  <span className={styles.badge}>{t("addApp.already")}</span>
                ) : (
                  isSelected && <Icon name="check" className={styles.check} />
                )}
              </button>
            );
          })
        )}
      </div>
    </Dialog>
  );
}
