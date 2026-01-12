<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import History from "./lib/History.svelte";
  import Performance from "./lib/Performance.svelte";
  import Settings from "./lib/Settings.svelte";
  import { appState } from "./lib/stores/appState";

  let activeTab = $state<"llm" | "hotkey" | "language" | "history" | "performance">("llm");
  
  let config = $derived($appState.config);
  let favoriteLanguages = $derived(config?.language.favorite_languages ?? []);
  let currentTarget = $state("en-US");
  let isEnabled = $state(true);
  let unlistenConfigUpdate: UnlistenFn | null = null;
  let unlistenEnabledStatus: UnlistenFn | null = null;

  // 同步currentTarget与config的变化
  $effect(() => {
    if (config?.language.current_target) {
      currentTarget = config.language.current_target;
    }
  });

  async function handleTargetLanguageChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const newTarget = select.value;
    console.log("顶部栏切换语言:", newTarget);
    if (config && newTarget !== config.language.current_target) {
      const updatedConfig = { 
        ...config, 
        language: { ...config.language, current_target: newTarget } 
      };
      await appState.saveConfig(updatedConfig);
    }
  }

  async function toggleEnabled() {
    const newStatus = !isEnabled;
    try {
      await invoke("set_enabled_status", { enabled: newStatus });
      isEnabled = newStatus;
    } catch (error) {
      console.error("Failed to toggle enabled status:", error);
    }
  }

  onMount(async () => {
    await appState.loadConfig();
    
    // 获取初始启用状态
    try {
      isEnabled = await invoke<boolean>("get_enabled_status");
    } catch (error) {
      console.error("Failed to get enabled status:", error);
    }
    
    // 监听配置更新事件（从托盘菜单或其他地方触发）
    unlistenConfigUpdate = await listen("config-updated", async () => {
      console.log("Config updated, reloading...");
      await appState.loadConfig();
    });

    // 监听启用状态变化事件
    unlistenEnabledStatus = await listen<boolean>("enabled-status-changed", (event) => {
      console.log("Enabled status changed:", event.payload);
      isEnabled = event.payload;
    });
  });

  onDestroy(() => {
    if (unlistenConfigUpdate) {
      unlistenConfigUpdate();
    }
    if (unlistenEnabledStatus) {
      unlistenEnabledStatus();
    }
  });
</script>

<main class="container">
  <header>
    <div class="header-content">
      <div class="header-left">
        <h1>AITyping</h1>
        <p class="subtitle">智能翻译助手</p>
      </div>
      <div class="header-right">
        <button 
          class="toggle-button" 
          class:enabled={isEnabled}
          onclick={toggleEnabled}
          title={isEnabled ? "点击暂停" : "点击启用"}
        >
          {isEnabled ? "✓ 已启用" : "⏸ 已暂停"}
        </button>
        <label class="target-lang-selector">
          <span class="selector-label">目标语言:</span>
          <select value={currentTarget} onchange={handleTargetLanguageChange}>
            {#each favoriteLanguages as lang}
              <option value={lang.code} selected={lang.code === currentTarget}>{lang.name}</option>
            {/each}
          </select>
        </label>
      </div>
    </div>
  </header>

  <nav class="tabs">
    <button
      class:active={activeTab === "llm"}
      onclick={() => (activeTab = "llm")}
    >
      LLM 设置
    </button>
    <button
      class:active={activeTab === "hotkey"}
      onclick={() => (activeTab = "hotkey")}
    >
      热键设置
    </button>
    <button
      class:active={activeTab === "language"}
      onclick={() => (activeTab = "language")}
    >
      语言设置
    </button>
    <button
      class:active={activeTab === "history"}
      onclick={() => (activeTab = "history")}
    >
      翻译历史
    </button>
    <button
      class:active={activeTab === "performance"}
      onclick={() => (activeTab = "performance")}
    >
      性能监控
    </button>
  </nav>

  <div class="content">
    {#if activeTab === "llm" || activeTab === "hotkey" || activeTab === "language"}
      <Settings {activeTab} />
    {:else if activeTab === "history"}
      <History />
    {:else if activeTab === "performance"}
      <Performance />
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      Oxygen,
      Ubuntu,
      Cantarell,
      sans-serif;
    background-color: #f5f7fa;
    color: #1f2937;
  }

  .container {
    max-width: 900px;
    margin: 0 auto;
    padding: 20px;
  }

  header {
    margin-bottom: 30px;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header-left {
    text-align: left;
  }

  header h1 {
    margin: 0;
    font-size: 2rem;
    color: #2563eb;
  }

  .subtitle {
    margin: 5px 0 0 0;
    color: #6b7280;
    font-size: 0.9rem;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 15px;
  }

  .toggle-button {
    padding: 8px 16px;
    border: 2px solid #d1d5db;
    border-radius: 6px;
    background: #ffffff;
    color: #6b7280;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .toggle-button:hover {
    border-color: #9ca3af;
  }

  .toggle-button.enabled {
    border-color: #10b981;
    background: #ecfdf5;
    color: #059669;
  }

  .toggle-button.enabled:hover {
    background: #d1fae5;
    border-color: #059669;
  }

  .target-lang-selector {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .selector-label {
    color: #6b7280;
    font-size: 0.9rem;
  }

  .target-lang-selector select {
    padding: 8px 12px;
    background: #ffffff;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    color: #1f2937;
    font-size: 0.95rem;
    cursor: pointer;
    min-width: 120px;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .target-lang-selector select:hover {
    border-color: #2563eb;
  }

  .target-lang-selector select:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.2);
  }

  .tabs {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
    border-bottom: 1px solid #e5e7eb;
    padding-bottom: 10px;
  }

  .tabs button {
    padding: 10px 20px;
    border: none;
    background: transparent;
    color: #6b7280;
    cursor: pointer;
    font-size: 0.95rem;
    border-radius: 5px 5px 0 0;
    transition: all 0.2s;
  }

  .tabs button:hover {
    color: #1f2937;
    background: rgba(0, 0, 0, 0.05);
  }

  .tabs button.active {
    color: #2563eb;
    background: rgba(37, 99, 235, 0.1);
    border-bottom: 2px solid #2563eb;
  }

  .content {
    background: #ffffff;
    border-radius: 10px;
    padding: 25px;
    min-height: 400px;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    border: 1px solid #e5e7eb;
  }
</style>
