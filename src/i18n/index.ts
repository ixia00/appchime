import { useCallback } from "react";

import { useAppStore } from "../store/appStore";
import type { Language } from "../lib/types";
import { en } from "./en";
import { ja } from "./ja";
import { ko, type MessageKey, type Messages } from "./ko";

type Lang = Exclude<Language, "system">;

const DICTIONARIES: Record<Lang, Messages> = { ko, en, ja };

export const LANGUAGE_NAMES: Record<Lang, string> = { ko: "한국어", en: "English", ja: "日本語" };

export function resolveLanguage(language: Language): Lang {
  if (language !== "system") return language;
  const nav = (navigator.language || "en").toLowerCase();
  if (nav.startsWith("ko")) return "ko";
  if (nav.startsWith("ja")) return "ja";
  return "en";
}

export type Translate = (key: MessageKey, vars?: Record<string, string | number>) => string;

export function translate(lang: Lang, key: MessageKey, vars?: Record<string, string | number>) {
  const template = DICTIONARIES[lang][key] ?? ko[key];
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, name: string) => String(vars[name] ?? `{${name}}`));
}

export function useT(): Translate {
  const language = useAppStore((s) => s.config?.settings.language ?? "system");
  const lang = resolveLanguage(language);
  return useCallback((key, vars) => translate(lang, key, vars), [lang]);
}

export function useLang(): Lang {
  return resolveLanguage(useAppStore((s) => s.config?.settings.language ?? "system"));
}

export type { MessageKey };
