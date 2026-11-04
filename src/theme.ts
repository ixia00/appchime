import { useEffect } from "react";

import type { Accent, Theme } from "./lib/types";

const media = window.matchMedia("(prefers-color-scheme: dark)");

export function applyTheme(theme: Theme, accent: Accent = "blue") {
  const resolved = theme === "system" ? (media.matches ? "dark" : "light") : theme;
  document.documentElement.dataset.theme = resolved;
  document.documentElement.dataset.accent = accent;
}

export function useThemeSync(theme: Theme, accent: Accent) {
  useEffect(() => {
    applyTheme(theme, accent);
    if (theme !== "system") return;
    const onChange = () => applyTheme("system", accent);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [theme, accent]);
}