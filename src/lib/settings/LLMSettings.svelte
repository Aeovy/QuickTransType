<script lang="ts">
  import type { LLMConfig } from "../stores/appState";

  export let llmConfig: LLMConfig;
  export let onUpdate: (config: LLMConfig) => void;
  export let onTestConnection: (config: LLMConfig) => void;
  export let onSave: () => void;

  let showApiKey = false;
  let isTesting = false;

  const predefinedModels = [
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4-turbo",
    "gpt-4",
    "gpt-3.5-turbo",
  ];

  function handleChange() {
    onUpdate(llmConfig);
  }

  async function handleTestConnection() {
    isTesting = true;
    await onTestConnection(llmConfig);
    isTesting = false;
  }
</script>

<div class="settings-section">
  <h2>LLM 配置</h2>

  <div class="form-group">
    <label for="base-url">Base URL</label>
    <input
      type="text"
      id="base-url"
      bind:value={llmConfig.base_url}
      on:input={handleChange}
      placeholder="https://api.openai.com/v1"
    />
  </div>

  <div class="form-group">
    <label for="api-key">API Key</label>
    <div class="input-with-button">
      {#if showApiKey}
        <input
          type="text"
          id="api-key"
          bind:value={llmConfig.api_key}
          on:input={handleChange}
          placeholder="sk-..."
        />
      {:else}
        <input
          type="password"
          id="api-key"
          bind:value={llmConfig.api_key}
          on:input={handleChange}
          placeholder="sk-..."
        />
      {/if}
      <button
        class="icon-button"
        on:click={() => (showApiKey = !showApiKey)}
        title={showApiKey ? "隐藏" : "显示"}
      >
        {showApiKey ? "👁️" : "👁️‍🗨️"}
      </button>
    </div>
  </div>

  <div class="form-group">
    <label for="model">模型</label>
    <div class="input-with-button">
      <input
        type="text"
        id="model"
        bind:value={llmConfig.model}
        on:input={handleChange}
        list="model-list"
        placeholder="gpt-4o-mini"
      />
      <datalist id="model-list">
        {#each predefinedModels as model}
          <option value={model}>{model}</option>
        {/each}
      </datalist>
    </div>
  </div>

  <div class="form-row">
    <div class="form-group half">
      <label for="temperature">Temperature: {llmConfig.temperature.toFixed(2)}</label>
      <input
        type="range"
        id="temperature"
        bind:value={llmConfig.temperature}
        on:input={handleChange}
        min="0"
        max="2"
        step="0.1"
      />
    </div>

    <div class="form-group half">
      <label for="top-p">Top P: {llmConfig.top_p.toFixed(2)}</label>
      <input
        type="range"
        id="top-p"
        bind:value={llmConfig.top_p}
        on:input={handleChange}
        min="0"
        max="1"
        step="0.01"
      />
    </div>
  </div>

  <div class="form-group">
    <label for="system-prompt">System Prompt</label>
    <textarea
      id="system-prompt"
      bind:value={llmConfig.system_prompt}
      on:input={handleChange}
      rows="3"
      placeholder="You are a professional translator..."
    ></textarea>
  </div>

  <div class="form-group">
    <label for="user-prompt">User Prompt 模板</label>
    <p class="hint">可用变量: &#123;target_language&#125;, &#123;text&#125;</p>
    <textarea
      id="user-prompt"
      bind:value={llmConfig.user_prompt_template}
      on:input={handleChange}
      rows="3"
      placeholder="将下列文本翻译为&#123;target_language&#125;，保持原有格式：&#123;text&#125;"
    ></textarea>
  </div>

  <div class="button-row">
    <button class="btn secondary" on:click={handleTestConnection} disabled={isTesting}>
      {isTesting ? "测试中..." : "测试连接"}
    </button>
    <button class="btn primary" on:click={onSave}>
      保存设置
    </button>
  </div>
</div>

<style>
  .settings-section h2 {
    margin: 0 0 20px 0;
    color: #00d4ff;
    font-size: 1.3rem;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    color: #ccc;
    font-size: 0.9rem;
  }

  .form-group input[type="text"],
  .form-group input[type="password"],
  .form-group textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #333;
    border-radius: 6px;
    background: #0f1729;
    color: #eaeaea;
    font-size: 0.95rem;
    box-sizing: border-box;
  }

  .form-group input:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: #00d4ff;
  }

  .form-group input[type="range"] {
    width: 100%;
    cursor: pointer;
  }

  .form-row {
    display: flex;
    gap: 20px;
  }

  .form-group.half {
    flex: 1;
  }

  .input-with-button {
    display: flex;
    gap: 10px;
  }

  .input-with-button input {
    flex: 1;
  }

  .icon-button {
    padding: 10px 15px;
    border: 1px solid #333;
    border-radius: 6px;
    background: #0f1729;
    cursor: pointer;
    font-size: 1rem;
  }

  .icon-button:hover {
    background: #1a2744;
  }

  .hint {
    margin: 0 0 8px 0;
    color: #666;
    font-size: 0.8rem;
  }

  .button-row {
    display: flex;
    gap: 15px;
    justify-content: flex-end;
    margin-top: 25px;
  }

  .btn {
    padding: 12px 25px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn.primary {
    background: #00d4ff;
    color: #0f1729;
  }

  .btn.primary:hover {
    background: #00b8e6;
  }

  .btn.secondary {
    background: transparent;
    color: #00d4ff;
    border: 1px solid #00d4ff;
  }

  .btn.secondary:hover {
    background: rgba(0, 212, 255, 0.1);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
