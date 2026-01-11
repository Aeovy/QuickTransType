<script lang="ts">
  import Toast from "./components/Toast.svelte";
  import HotkeySettings from "./settings/HotkeySettings.svelte";
  import LanguageSettings from "./settings/LanguageSettings.svelte";
  import LLMSettings from "./settings/LLMSettings.svelte";
  import { appState, type AppConfig, type Hotkey, type LLMConfig } from "./stores/appState";

  let { activeTab }: { activeTab: "llm" | "hotkey" | "language" } = $props();

  let toast = $state<{ message: string; type: "success" | "error" } | null>(null);

  let config = $state<AppConfig | null>(null);
  
  // 订阅状态变化
  appState.subscribe((state) => {
    config = state.config;
  });

  function showToast(message: string, type: "success" | "error") {
    toast = { message, type };
    setTimeout(() => {
      toast = null;
    }, 3000);
  }

  async function handleTestConnection(llmConfig: LLMConfig) {
    const result = await appState.testLLMConnection(llmConfig);
    if (result.success) {
      showToast(result.message, "success");
    } else {
      showToast(`连接失败: ${result.message}`, "error");
    }
  }

  async function updateLLMConfig(llmConfig: LLMConfig) {
    if (config) {
      const updatedConfig = { ...config, llm: llmConfig };
      await appState.saveConfig(updatedConfig);
    }
  }

  async function updateHotkeyConfig(selected: Hotkey, full: Hotkey) {
    if (config) {
      const updatedConfig = { 
        ...config,
        hotkey: { selected_mode: selected, full_mode: full } 
      };
      await appState.saveConfig(updatedConfig);
    }
  }

  async function updateLanguageConfig(currentTarget: string, favorites: { code: string; name: string }[], historyLimit: number) {
    if (config) {
      const updatedConfig = {
        ...config,
        language: { current_target: currentTarget, favorite_languages: favorites },
        history_limit: historyLimit,
      };
      // 直接保存最新配置
      await appState.saveConfig(updatedConfig);
    }
  }
</script>

{#if toast}
  <Toast message={toast.message} type={toast.type} />
{/if}

{#if config}
  {#if activeTab === "llm"}
    <LLMSettings 
      llmConfig={config.llm} 
      onUpdate={updateLLMConfig}
      onTestConnection={handleTestConnection}
    />
  {:else if activeTab === "hotkey"}
    <HotkeySettings 
      hotkeyConfig={config.hotkey}
      onUpdate={updateHotkeyConfig}
    />
  {:else if activeTab === "language"}
    <LanguageSettings 
      languageConfig={config.language}
      historyLimit={config.history_limit}
      onUpdate={updateLanguageConfig}
    />
  {/if}
{:else}
  <p>加载配置中...</p>
{/if}
