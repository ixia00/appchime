import { useEffect, useId, useRef, useState } from "react";

import { useT } from "../../i18n";
import type { Rule } from "../../lib/types";
import { useAppStore } from "../../store/appStore";
import styles from "./VolumeRow.module.css";

const COMMIT_DELAY_MS = 250;

export function VolumeRow({ rule }: { rule: Rule }) {
  const t = useT();
  const id = useId();
  const updateRule = useAppStore((s) => s.updateRule);
  const [value, setValue] = useState(Math.round(rule.volume * 100));
  const timer = useRef<number>();

  useEffect(() => {
    if (timer.current === undefined) setValue(Math.round(rule.volume * 100));
  }, [rule.volume]);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const onChange = (next: number) => {
    setValue(next);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      timer.current = undefined;
      void updateRule(rule.id, { volume: next / 100 });
    }, COMMIT_DELAY_MS);
  };

  return (
    <div className={styles.row}>
      <label htmlFor={id} className={styles.label}>
        {t("detail.volume")}
      </label>
      <input
        id={id}
        type="range"
        min={0}
        max={100}
        step={1}
        value={value}
        className={styles.slider}
        style={{ "--fill": `${value}%` } as React.CSSProperties}
        onChange={(e) => onChange(Number(e.target.value))}
      />
      <output htmlFor={id} className={styles.value}>
        {value}%
      </output>
    </div>
  );
}
