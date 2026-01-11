<script lang="ts">
  import { appState, type AppConfig, type LLMConfig, type Hotkey } from "./stores/appState";
  import LLMSettings from "./settings/LLMSettings.svelte";
  import HotkeySettings from "./settings/HotkeySettings.svelte";
  import LanguageSettings from "./settings/LanguageSettings.svelte";
  import Toast from "./components/Toast.svelte";

  export let activeTab: "llm" | "hotkey" | "language";

  let toast: { message: string; type: "success" | "error" } | null = null;

  let config: AppConfig | null = null;
  
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

  async function handleSave() {
    if (!config) return;
    const result = await appState.saveConfig(config);
    if (result.success) {
      showToast("保存成功！", "success");
    } else {
      showToast(`保存失败: ${result.error}`, "error");
    }
  }

  async function handleTestConnection(llmConfig: LLMConfig) {
    const result = await appState.testLLMConnection(llmConfig);
    if (result.success) {
      showToast(result.message, "success");
    } else {
      showToast(`连接失败: ${result.message}`, "error");
    }
  }

  function updateLLMConfig(llmConfig: LLMConfig) {
    if (config) {
      appState.updateConfig({ llm: llmConfig });
    }
  }

  function updateHotkeyConfig(selected: Hotkey, full: Hotkey) {
    if (config) {
      appState.updateConfig({ 
        hotkey: { selected_mode: selected, full_mode: full } 
      });
    }
  }

  function updateLanguageConfig(currentTarget: string, favorites: { code: string; name: string }[], historyLimit: number) {
    if (config) {
      appState.updateConfig({
        language: { current_target: currentTarget, favorite_languages: favorites },
        history_limit: historyLimit,
      });
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
      onSave={handleSave}
    />
  {:else if activeTab === "hotkey"}
    <HotkeySettings 
      hotkeyConfig={config.hotkey}
      onUpdate={updateHotkeyConfig}
      onSave={handleSave}
    />
  {:else if activeTab === "language"}
    <LanguageSettings 
      languageConfig={config.language}
      historyLimit={config.history_limit}
      onUpdate={updateLanguageConfig}
      onSave={handleSave}
    />
  {/if}
{:else}
  <p>加载配置中...</p>
{/if}
