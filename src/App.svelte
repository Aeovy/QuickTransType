<script lang="ts">
  import { onMount } from "svelte";
  import History from "./lib/History.svelte";
  import Performance from "./lib/Performance.svelte";
  import Settings from "./lib/Settings.svelte";
  import { appState } from "./lib/stores/appState";

  let activeTab = $state<"llm" | "hotkey" | "language" | "history" | "performance">("llm");
  
  let config = $derived($appState.config);
  let favoriteLanguages = $derived(config?.language.favorite_languages ?? []);
  let currentTarget = $derived(config?.language.current_target ?? "en-US");

  async function handleTargetLanguageChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const newTarget = select.value;
    if (config) {
      appState.updateConfig({
        language: { ...config.language, current_target: newTarget }
      });
      // 自动保存
      await appState.saveConfig({ ...config, language: { ...config.language, current_target: newTarget } });
    }
  }

  onMount(async () => {
    await appState.loadConfig();
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
        <label class="target-lang-selector">
          <span class="selector-label">目标语言:</span>
          <select value={currentTarget} onchange={handleTargetLanguageChange}>
            {#each favoriteLanguages as lang}
              <option value={lang.code}>{lang.name}</option>
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
