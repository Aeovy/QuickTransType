import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";

// 类型定义
export interface Language {
  code: string;
  name: string;
}

export interface Hotkey {
  type: "Combination" | "Consecutive";
  modifiers?: string[];
  key: string;
  count?: number;
}

export interface LLMConfig {
  base_url: string;
  api_key: string;
  model: string;
  temperature: number;
  top_p: number;
  system_prompt: string;
  user_prompt_template: string;
  stream_mode: boolean;
}

export interface HotkeyConfig {
  selected_mode: Hotkey;
  full_mode: Hotkey;
}

export interface LanguageConfig {
  current_target: string;
  favorite_languages: Language[];
}

export interface AppConfig {
  llm: LLMConfig;
  hotkey: HotkeyConfig;
  language: LanguageConfig;
  history_limit: number;
}

interface AppStateData {
  config: AppConfig | null;
  isLoading: boolean;
  error: string | null;
  isEnabled: boolean;
}

const defaultConfig: AppConfig = {
  llm: {
    base_url: "https://api.openai.com/v1",
    api_key: "",
    model: "gpt-4o-mini",
    temperature: 0.3,
    top_p: 1.0,
    system_prompt:
      "You are a professional translator. Maintain the original formatting of the text.",
    user_prompt_template:
      "将下列文本翻译为{target_language}，保持原有格式：{text}",
    stream_mode: true,
  },
  hotkey: {
    selected_mode: {
      type: "Combination",
      modifiers: ["Meta"],
      key: "t",
    },
    full_mode: {
      type: "Consecutive",
      key: " ",
      count: 3,
    },
  },
  language: {
    current_target: "en-US",
    favorite_languages: [
      { code: "en-US", name: "English" },
      { code: "zh-CN", name: "简体中文" },
      { code: "ja-JP", name: "日本語" },
      { code: "ko-KR", name: "한국어" },
      { code: "fr-FR", name: "Français" },
      { code: "es-ES", name: "Español" },
    ],
  },
  history_limit: 500,
};

function createAppState() {
  const { subscribe, update } = writable<AppStateData>({
    config: null,
    isLoading: false,
    error: null,
    isEnabled: true,
  });

  return {
    subscribe,

    async loadConfig() {
      update((state) => ({ ...state, isLoading: true, error: null }));
      try {
        const config = await invoke<AppConfig>("get_config");
        update((state) => ({ ...state, config, isLoading: false }));
      } catch (e) {
        console.warn("Failed to load config, using defaults:", e);
        update((state) => ({
          ...state,
          config: defaultConfig,
          isLoading: false,
        }));
      }
    },

    async saveConfig(config: AppConfig) {
      update((state) => ({ ...state, isLoading: true, error: null }));
      try {
        await invoke("save_config", { config });
        update((state) => ({ ...state, config, isLoading: false }));
        return { success: true };
      } catch (e) {
        const error = e as string;
        update((state) => ({ ...state, error, isLoading: false }));
        return { success: false, error };
      }
    },

    async testLLMConnection(config: LLMConfig): Promise<{ success: boolean; message: string }> {
      try {
        const message = await invoke<string>("test_llm_connection", { config });
        return { success: true, message };
      } catch (e) {
        return { success: false, message: e as string };
      }
    },

    updateConfig(partialConfig: Partial<AppConfig>) {
      update((state) => ({
        ...state,
        config: state.config ? { ...state.config, ...partialConfig } : null,
      }));
    },

    setEnabled(enabled: boolean) {
      update((state) => ({ ...state, isEnabled: enabled }));
    },

    setError(error: string | null) {
      update((state) => ({ ...state, error }));
    },
  };
}

export const appState = createAppState();

// 派生 store
export const config = derived(appState, ($state) => $state.config);
export const isLoading = derived(appState, ($state) => $state.isLoading);
export const error = derived(appState, ($state) => $state.error);
export const isEnabled = derived(appState, ($state) => $state.isEnabled);
