import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AddSoundsResult,
  AppIdentity,
  Config,
  Meta,
  Rule,
  RulePatch,
  Settings,
  TriggerKind,
} from "./types";

const EVENT_CONFIG_CHANGED = "config-changed";

export const api = {
  getMeta: () => invoke<Meta>("get_meta"),
  getConfig: () => invoke<Config>("get_config"),

  listRunningApps: () => invoke<AppIdentity[]>("list_running_apps"),
  listInstalledApps: () => invoke<AppIdentity[]>("list_installed_apps"),
  identityFromPath: (path: string) => invoke<AppIdentity>("identity_from_path", { path }),

  updateSettings: (settings: Settings) => invoke<Config>("update_settings", { settings }),

  addRule: (app: AppIdentity) => invoke<Rule>("add_rule", { app }),
  updateRule: (id: string, patch: RulePatch) => invoke<Rule>("update_rule", { id, patch }),
  removeRule: (id: string) => invoke<void>("remove_rule", { id }),

  addSounds: (ruleId: string, kind: TriggerKind, paths: string[]) =>
    invoke<AddSoundsResult>("add_sounds", { ruleId, kind, paths }),
  removeSound: (ruleId: string, kind: TriggerKind, soundId: string) =>
    invoke<Rule>("remove_sound", { ruleId, kind, soundId }),
  reorderSound: (ruleId: string, kind: TriggerKind, soundId: string, toIndex: number) =>
    invoke<Rule>("reorder_sound", { ruleId, kind, soundId, toIndex }),

  previewSound: (ruleId: string, kind: TriggerKind, soundId: string) =>
    invoke<void>("preview_sound", { ruleId, kind, soundId }),
  stopPreview: () => invoke<void>("stop_preview"),

  accessibilityStatus: (prompt: boolean) => invoke<boolean>("accessibility_status", { prompt }),
  appIcon: (app: AppIdentity) => invoke<string | null>("app_icon", { app }),

  quitApp: () => invoke<void>("quit_app"),

  onConfigChanged: (handler: (config: Config) => void): Promise<UnlistenFn> =>
    listen<Config>(EVENT_CONFIG_CHANGED, (event) => handler(event.payload)),
};
