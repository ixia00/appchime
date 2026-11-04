export type Platform = "macos" | "windows" | "linux";
export type Language = "system" | "ko" | "en" | "ja";
export type Theme = "system" | "light" | "dark";
export type Accent = "blue" | "purple" | "pink" | "orange" | "green";
export type PlayMode = "fixed" | "random" | "sequential";
export type TriggerKind = "launch" | "quit";
export type LaunchTiming = "mainWindow" | "process";

export interface AppIdentity {
  name: string;
  exePath: string;
  bundlePath?: string;
  bundleId?: string;
}

export interface Sound {
  id: string;
  name: string;
  file: string;
  durationMs?: number;
}

export interface Trigger {
  enabled: boolean;
  mode: PlayMode;
  sounds: Sound[];
  timing: LaunchTiming;
  delayMs: number;
}

export interface Rule {
  id: string;
  app: AppIdentity;
  enabled: boolean;
  volume: number;
  onLaunch: Trigger;
  onQuit: Trigger;
}

export interface Settings {
  soundEnabled: boolean;
  launchAtLogin: boolean;
  toggleShortcut: string | null;
  dedupeSeconds: number;
  language: Language;
  theme: Theme;
  accent: Accent;
}

export interface Config {
  version: number;
  settings: Settings;
  rules: Rule[];
}

export interface Meta {
  platform: Platform;
  version: string;
  supportedFormats: string[];
}

export interface TriggerPatch {
  enabled?: boolean;
  mode?: PlayMode;
  timing?: LaunchTiming;
  delayMs?: number;
}

export interface RulePatch {
  enabled?: boolean;
  volume?: number;
  onLaunch?: TriggerPatch;
  onQuit?: TriggerPatch;
}

export type ErrorCode =
  | "duplicateApp"
  | "notFound"
  | "unsupportedAudio"
  | "fileTooLarge"
  | "invalidApp"
  | "invalidShortcut"
  | "invalidInput"
  | "io"
  | "other";

export interface AppError {
  code: ErrorCode;
  message: string;
}

export interface FailedFile {
  name: string;
  error: AppError;
}

export interface AddSoundsResult {
  rule: Rule;
  failed: FailedFile[];
}

export function triggerOf(rule: Rule, kind: TriggerKind): Trigger {
  return kind === "launch" ? rule.onLaunch : rule.onQuit;
}
