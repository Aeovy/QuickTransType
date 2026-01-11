<script lang="ts">
  import { onMount } from "svelte";
  import Settings from "./lib/Settings.svelte";
  import History from "./lib/History.svelte";
  import Performance from "./lib/Performance.svelte";
  import { appState } from "./lib/stores/appState";

  let activeTab: "llm" | "hotkey" | "language" | "history" | "performance" =
    "llm";

  onMount(async () => {
    await appState.loadConfig();
  });
</script>

<main class="container">
  <header>
    <h1>AITyping</h1>
    <p class="subtitle">智能翻译助手</p>
  </header>

  <nav class="tabs">
    <button
      class:active={activeTab === "llm"}
      on:click={() => (activeTab = "llm")}
    >
      LLM 设置
    </button>
    <button
      class:active={activeTab === "hotkey"}
      on:click={() => (activeTab = "hotkey")}
    >
      热键设置
    </button>
    <button
      class:active={activeTab === "language"}
      on:click={() => (activeTab = "language")}
    >
      语言设置
    </button>
    <button
      class:active={activeTab === "history"}
      on:click={() => (activeTab = "history")}
    >
      翻译历史
    </button>
    <button
      class:active={activeTab === "performance"}
      on:click={() => (activeTab = "performance")}
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
    background-color: #1a1a2e;
    color: #eaeaea;
  }

  .container {
    max-width: 900px;
    margin: 0 auto;
    padding: 20px;
  }

  header {
    text-align: center;
    margin-bottom: 30px;
  }

  header h1 {
    margin: 0;
    font-size: 2rem;
    color: #00d4ff;
  }

  .subtitle {
    margin: 5px 0 0 0;
    color: #888;
    font-size: 0.9rem;
  }

  .tabs {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
    border-bottom: 1px solid #333;
    padding-bottom: 10px;
  }

  .tabs button {
    padding: 10px 20px;
    border: none;
    background: transparent;
    color: #888;
    cursor: pointer;
    font-size: 0.95rem;
    border-radius: 5px 5px 0 0;
    transition: all 0.2s;
  }

  .tabs button:hover {
    color: #eaeaea;
    background: rgba(255, 255, 255, 0.05);
  }

  .tabs button.active {
    color: #00d4ff;
    background: rgba(0, 212, 255, 0.1);
    border-bottom: 2px solid #00d4ff;
  }

  .content {
    background: #16213e;
    border-radius: 10px;
    padding: 25px;
    min-height: 400px;
  }
</style>
