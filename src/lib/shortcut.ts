import type { Platform } from "./types";

const MODIFIER_KEYS = new Set(["Meta", "Control", "Alt", "Shift", "OS"]);

export function acceleratorFromEvent(e: KeyboardEvent, platform: Platform): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null;

  const mods: string[] = [];
  if (platform === "macos") {
    if (e.metaKey) mods.push("Command");
    if (e.ctrlKey) mods.push("Control");
  } else {
    if (e.ctrlKey) mods.push("Control");
    if (e.metaKey) mods.push("Super");
  }
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");

  const code = e.code;
  const isFunctionKey = /^F([1-9]|1[0-9]|2[0-4])$/.test(code);
  const supported =
    isFunctionKey || /^(Key[A-Z]|Digit[0-9]|Space|Arrow(Up|Down|Left|Right))$/.test(code);
  if (!supported) return null;
  if (mods.length === 0 && !isFunctionKey) return null;

  return [...mods, code].join("+");
}

export function formatAccelerator(accelerator: string, platform: Platform): string {
  const parts = accelerator.split("+");
  const key = prettyKey(parts[parts.length - 1]);
  const mods = parts.slice(0, -1);

  if (platform === "macos") {
    const order = ["Control", "Alt", "Shift", "Command", "CommandOrControl"];
    const symbol: Record<string, string> = {
      Control: "⌃",
      Alt: "⌥",
      Shift: "⇧",
      Command: "⌘",
      CommandOrControl: "⌘",
    };
    const sorted = [...mods].sort((a, b) => order.indexOf(a) - order.indexOf(b));
    return sorted.map((m) => symbol[m] ?? m).join("") + key;
  }
  const label: Record<string, string> = {
    Control: "Ctrl",
    CommandOrControl: "Ctrl",
    Super: "Win",
    Alt: "Alt",
    Shift: "Shift",
  };
  return [...mods.map((m) => label[m] ?? m), key].join("+");
}

function prettyKey(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  const arrows: Record<string, string> = {
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
  };
  return arrows[code] ?? code;
}
