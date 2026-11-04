import { useEffect, useState } from "react";

import { api } from "../lib/api";
import { colorIndex, initials } from "../lib/format";
import type { AppIdentity } from "../lib/types";
import styles from "./AppAvatar.module.css";

interface AppAvatarProps {
  app: AppIdentity;
  size?: number;
}

export function AppAvatar({ app, size = 32 }: AppAvatarProps) {
  const icon = useAppIcon(app);
  const [broken, setBroken] = useState(false);

  if (icon && !broken) {
    return (
      <img
        src={icon}
        alt=""
        aria-hidden="true"
        draggable={false}
        onError={() => setBroken(true)}
        className={styles.icon}
        style={{ width: size, height: size }}
      />
    );
  }

  const c = colorIndex(app.name);
  return (
    <span
      aria-hidden="true"
      className={styles.avatar}
      style={{
        width: size,
        height: size,
        borderRadius: size * 0.25,
        fontSize: Math.round(size * 0.36),
        background: `var(--avatar-${c}-bg)`,
        color: `var(--avatar-${c}-fg)`,
      }}
    >
      {initials(app.name)}
    </span>
  );
}

const cache = new Map<string, Promise<string | null>>();

function useAppIcon(app: AppIdentity): string | null {
  const key = app.exePath.toLowerCase();
  const [icon, setIcon] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    let request = cache.get(key);
    if (!request) {
      request = api.appIcon(app).catch(() => null);
      cache.set(key, request);
    }
    void request.then((url) => alive && setIcon(url));
    return () => {
      alive = false;
    };
  }, [key]);

  return icon;
}
