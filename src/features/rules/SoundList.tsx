import { open } from "@tauri-apps/plugin-dialog";
import { useRef, useState } from "react";

import { IconButton } from "../../components/Button";
import { Icon } from "../../components/Icon";
import { useT } from "../../i18n";
import { formatDuration } from "../../lib/format";
import { triggerOf, type Rule, type TriggerKind } from "../../lib/types";
import { useAppStore } from "../../store/appStore";
import styles from "./SoundList.module.css";

interface Drag {
  id: string;
  from: number;
  to: number;
  startY: number;
  dy: number;
  rowHeight: number;
}

export function SoundList({ rule, kind }: { rule: Rule; kind: TriggerKind }) {
  const t = useT();
  const formats = useAppStore((s) => s.meta?.supportedFormats ?? []);
  const previewing = useAppStore((s) => s.previewing);
  const { addSounds, removeSound, reorderSound, togglePreview } = useAppStore.getState();
  const sounds = triggerOf(rule, kind).sounds;
  const [drag, setDrag] = useState<Drag | null>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const canReorder = sounds.length > 1;

  const pickFiles = async () => {
    const picked = await open({
      multiple: true,
      title: t("sounds.pickerTitle"),
      filters: [{ name: t("sounds.pickerFilter"), extensions: formats }],
    });
    if (!picked) return;
    await addSounds(rule.id, kind, Array.isArray(picked) ? picked : [picked]);
  };

  const onGripDown = (e: React.PointerEvent, id: string, index: number) => {
    if (!canReorder) return;
    const row = (e.currentTarget as HTMLElement).closest("li");
    e.currentTarget.setPointerCapture(e.pointerId);
    setDrag({ id, from: index, to: index, startY: e.clientY, dy: 0, rowHeight: row?.offsetHeight ?? 44 });
  };

  const onGripMove = (e: React.PointerEvent) => {
    if (!drag) return;
    const dy = e.clientY - drag.startY;
    const to = clamp(drag.from + Math.round(dy / drag.rowHeight), 0, sounds.length - 1);
    setDrag({ ...drag, dy, to });
  };

  const onGripUp = () => {
    if (!drag) return;
    if (drag.to !== drag.from) void reorderSound(rule.id, kind, drag.id, drag.to);
    setDrag(null);
  };

  const onGripKey = (e: React.KeyboardEvent, id: string, index: number) => {
    const dir = e.key === "ArrowUp" ? -1 : e.key === "ArrowDown" ? 1 : 0;
    if (!dir) return;
    e.preventDefault();
    const to = index + dir;
    if (to < 0 || to >= sounds.length) return;
    void reorderSound(rule.id, kind, id, to).then(() => {
      listRef.current?.querySelectorAll<HTMLElement>("[data-grip]")[to]?.focus();
    });
  };

  const shiftFor = (index: number): number => {
    if (!drag) return 0;
    if (index === drag.from) return drag.dy;
    if (drag.from < index && index <= drag.to) return -drag.rowHeight;
    if (drag.to <= index && index < drag.from) return drag.rowHeight;
    return 0;
  };

  return (
    <div className={styles.box}>
      {sounds.length === 0 ? (
        <p className={styles.empty}>{t("sounds.empty")}</p>
      ) : (
        <ul ref={listRef} className={styles.list}>
          {sounds.map((sound, index) => {
            const playing = previewing === sound.id;
            const dragging = drag?.id === sound.id;
            return (
              <li
                key={sound.id}
                className={styles.row}
                data-dragging={dragging || undefined}
                style={{
                  transform: `translateY(${shiftFor(index)}px)`,
                  transition: dragging || !drag ? "none" : "transform 150ms var(--ease)",
                }}
              >
                {canReorder && (
                  <button
                    type="button"
                    data-grip
                    className={styles.grip}
                    aria-label={t("sounds.reorder", { name: sound.name })}
                    onPointerDown={(e) => onGripDown(e, sound.id, index)}
                    onPointerMove={onGripMove}
                    onPointerUp={onGripUp}
                    onPointerCancel={() => setDrag(null)}
                    onKeyDown={(e) => onGripKey(e, sound.id, index)}
                  >
                    <Icon name="grip" size={14} />
                  </button>
                )}
                <IconButton
                  round
                  icon={playing ? "stop" : "play"}
                  label={playing ? t("sounds.stop") : t("sounds.preview", { name: sound.name })}
                  onClick={() => togglePreview(rule.id, kind, sound.id, sound.durationMs)}
                />
                <span className={styles.name} title={sound.name}>
                  {sound.name}
                </span>
                <span className={styles.duration}>{formatDuration(sound.durationMs)}</span>
                <IconButton
                  icon="close"
                  label={t("sounds.remove", { name: sound.name })}
                  onClick={() => removeSound(rule.id, kind, sound.id)}
                />
              </li>
            );
          })}
        </ul>
      )}
      <button type="button" className={styles.add} onClick={pickFiles}>
        <Icon name="plus" size={12} strokeWidth={2} />
        {t("sounds.add")}
      </button>
    </div>
  );
}

function clamp(n: number, min: number, max: number) {
  return Math.max(min, Math.min(max, n));
}
