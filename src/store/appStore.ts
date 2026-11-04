import { create } from "zustand";

import type { MessageKey } from "../i18n/ko";
import { api } from "../lib/api";
import { toAppError } from "../lib/errors";
import type {
  AppIdentity,
  Config,
  Meta,
  Rule,
  RulePatch,
  Settings,
  TriggerKind,
} from "../lib/types";

export type DialogName = "addApp" | "settings" | null;

export interface Toast {
  id: number;
  key: MessageKey;
  vars?: Record<string, string | number>;
}

interface AppStore {
  meta: Meta | null;
  config: Config | null;
  selectedId: string | null;
  dialog: DialogName;
  toast: Toast | null;
  previewing: string | null; 

  init: () => Promise<() => void>;
  select: (id: string | null) => void;
  openDialog: (d: DialogName) => void;

  addRule: (app: AppIdentity) => Promise<boolean>;
  updateRule: (id: string, patch: RulePatch) => Promise<void>;
  removeRule: (id: string) => Promise<void>;

  addSounds: (ruleId: string, kind: TriggerKind, paths: string[]) => Promise<void>;
  removeSound: (ruleId: string, kind: TriggerKind, soundId: string) => Promise<void>;
  reorderSound: (ruleId: string, kind: TriggerKind, soundId: string, toIndex: number) => Promise<void>;
  togglePreview: (ruleId: string, kind: TriggerKind, soundId: string, durationMs?: number) => Promise<void>;

  updateSettings: (patch: Partial<Settings>) => Promise<boolean>;

  showToast: (key: MessageKey, vars?: Toast["vars"]) => void;
  showError: (e: unknown) => void;
  dismissToast: () => void;
}

let toastSeq = 0;
let previewTimer: number | undefined;

export const useAppStore = create<AppStore>((set, get) => {
  const putRule = (rule: Rule) =>
    set((s) =>
      s.config
        ? { config: { ...s.config, rules: s.config.rules.map((r) => (r.id === rule.id ? rule : r)) } }
        : {},
    );

  async function run<T>(task: () => Promise<T>): Promise<T | undefined> {
    try {
      return await task();
    } catch (e) {
      get().showError(e);
      return undefined;
    }
  }

  return {
    meta: null,
    config: null,
    selectedId: null,
    dialog: null,
    toast: null,
    previewing: null,

    init: async () => {
      const [meta, config] = await Promise.all([api.getMeta(), api.getConfig()]);
      set({ meta, config, selectedId: config.rules[0]?.id ?? null });
      const unlisten = await api.onConfigChanged((next) => {
        set((s) => ({
          config: next,
          selectedId: next.rules.some((r) => r.id === s.selectedId)
            ? s.selectedId
            : (next.rules[0]?.id ?? null),
        }));
      });
      return unlisten;
    },

    select: (id) => set({ selectedId: id }),
    openDialog: (dialog) => set({ dialog }),

    addRule: async (app) => {
      const rule = await run(() => api.addRule(app));
      if (!rule) return false;
      set((s) => ({
        config: s.config && { ...s.config, rules: [...s.config.rules.filter((r) => r.id !== rule.id), rule] },
        selectedId: rule.id,
      }));
      return true;
    },

    updateRule: async (id, patch) => {
      const rule = await run(() => api.updateRule(id, patch));
      if (rule) putRule(rule);
    },

    removeRule: async (id) => {
      const ok = await run(() => api.removeRule(id).then(() => true));
      if (!ok) return;
      set((s) => {
        if (!s.config) return {};
        const index = s.config.rules.findIndex((r) => r.id === id);
        const rules = s.config.rules.filter((r) => r.id !== id);
        const neighbor = rules[Math.min(index, rules.length - 1)];
        return { config: { ...s.config, rules }, selectedId: neighbor?.id ?? null };
      });
    },

    addSounds: async (ruleId, kind, paths) => {
      if (paths.length === 0) return;
      const result = await run(() => api.addSounds(ruleId, kind, paths));
      if (!result) return;
      putRule(result.rule);
      if (result.failed.length === 1 && paths.length === 1) {
        get().showError(result.failed[0].error);
      } else if (result.failed.length > 0) {
        get().showToast("sounds.partialFail", {
          n: result.failed.length,
          names: result.failed.map((f) => f.name).join(", "),
        });
      }
    },

    removeSound: async (ruleId, kind, soundId) => {
      if (get().previewing === soundId) await get().togglePreview(ruleId, kind, soundId);
      const rule = await run(() => api.removeSound(ruleId, kind, soundId));
      if (rule) putRule(rule);
    },

    reorderSound: async (ruleId, kind, soundId, toIndex) => {
      const rule = await run(() => api.reorderSound(ruleId, kind, soundId, toIndex));
      if (rule) putRule(rule);
    },

    togglePreview: async (ruleId, kind, soundId, durationMs) => {
      window.clearTimeout(previewTimer);
      if (get().previewing === soundId) {
        set({ previewing: null });
        await run(() => api.stopPreview());
        return;
      }
      const ok = await run(() => api.previewSound(ruleId, kind, soundId).then(() => true));
      if (!ok) return;
      set({ previewing: soundId });
      previewTimer = window.setTimeout(() => set({ previewing: null }), (durationMs ?? 10_000) + 200);
    },

    updateSettings: async (patch) => {
      const current = get().config;
      if (!current) return false;
      const next = await run(() => api.updateSettings({ ...current.settings, ...patch }));
      if (!next) return false;
      set({ config: next });
      return true;
    },

    showToast: (key, vars) => set({ toast: { id: ++toastSeq, key, vars } }),
    showError: (e) => {
      const err = toAppError(e);
      console.error(err);
      get().showToast(`error.${err.code}` as MessageKey);
    },
    dismissToast: () => set({ toast: null }),
  };
});

export function useSelectedRule(): Rule | null {
  return useAppStore((s) => s.config?.rules.find((r) => r.id === s.selectedId) ?? null);
}
