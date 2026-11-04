import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useRef, useState } from "react";

import type { Rule, TriggerKind } from "../../lib/types";
import { useAppStore } from "../../store/appStore";

export function useSoundDrop(rule: Rule): TriggerKind | null {
  const [hover, setHover] = useState<TriggerKind | null>(null);
  const ruleId = useRef(rule.id);
  ruleId.current = rule.id;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "leave") {
          setHover(null);
          return;
        }
        const kind = kindAt(payload.position.x, payload.position.y);
        if (payload.type === "drop") {
          setHover(null);
          if (kind) void useAppStore.getState().addSounds(ruleId.current, kind, payload.paths);
        } else {
          setHover(kind);
        }
      })
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => {
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return hover;
}

function kindAt(x: number, y: number): TriggerKind | null {
  const scale = window.devicePixelRatio || 1;
  const el = document.elementFromPoint(x / scale, y / scale);
  const kind = el?.closest<HTMLElement>("[data-drop-kind]")?.dataset.dropKind;
  return kind === "launch" || kind === "quit" ? kind : null;
}
