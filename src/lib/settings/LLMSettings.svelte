<script lang="ts">
  import type { LLMConfig } from "../stores/appState";

  export let llmConfig: LLMConfig;
  export let onUpdate: (config: LLMConfig) => Promise<void>;
  export let onTestConnection: (config: LLMConfig) => void;

  let showApiKey = false;
  let isTesting = false;
  let isSaving = false;

  const predefinedModels = [
    "hy-mt1.5-1.8b"
  ];

  async function handleSave() {
    isSaving = true;
    try {
      await onUpdate(llmConfig);
    } finally {
      isSaving = false;
    }
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
          placeholder="sk-..."
        />
      {:else}
        <input
          type="password"
          id="api-key"
          bind:value={llmConfig.api_key}
          placeholder="sk-..."
        />
      {/if}
      <button
        class="icon-button"
        onclick={() => (showApiKey = !showApiKey)}
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
      rows="3"
      placeholder="将下列文本翻译为&#123;target_language&#125;，保持原有格式：&#123;text&#125;"
    ></textarea>
  </div>

  <div class="form-group">
    <label>输出模式</label>
    <div class="toggle-group">
      <label class="toggle-option">
        <input
          type="radio"
          bind:group={llmConfig.stream_mode}
          value={true}
        />
        <span class="toggle-label">
          <span class="toggle-title">Stream（流式）</span>
          <span class="toggle-desc">逐字输出翻译结果，体验更流畅</span>
        </span>
      </label>
      <label class="toggle-option">
        <input
          type="radio"
          bind:group={llmConfig.stream_mode}
          value={false}
        />
        <span class="toggle-label">
          <span class="toggle-title">Invoke（一次性）</span>
          <span class="toggle-desc">等待完成后一次性替换，更稳定</span>
        </span>
      </label>
    </div>
  </div>

  <div class="button-row">
    <button class="btn secondary" onclick={handleTestConnection} disabled={isTesting}>
      {isTesting ? "测试中..." : "测试连接"}
    </button>
    <button class="btn primary" onclick={handleSave} disabled={isSaving}>
      {isSaving ? "保存中..." : "保存配置"}
    </button>
  </div>
</div>

<style>
  .settings-section h2 {
    margin: 0 0 20px 0;
    color: #2563eb;
    font-size: 1.3rem;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    color: #4b5563;
    font-size: 0.9rem;
  }

  .form-group input[type="text"],
  .form-group input[type="password"],
  .form-group textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #ffffff;
    color: #1f2937;
    font-size: 0.95rem;
    box-sizing: border-box;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .form-group input:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.2);
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
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #f9fafb;
    cursor: pointer;
    font-size: 1rem;
    color: #4b5563;
  }

  .icon-button:hover {
    background: #f3f4f6;
    border-color: #9ca3af;
  }

  .hint {
    margin: 0 0 8px 0;
    color: #6b7280;
    font-size: 0.8rem;
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

  .btn.secondary {
    background: transparent;
    color: #2563eb;
    border: 1px solid #2563eb;
  }

  .btn.secondary:hover {
    background: rgba(37, 99, 235, 0.05);
  }

  .btn.primary {
    background: #2563eb;
    color: #ffffff;
    border: 1px solid #2563eb;
  }

  .btn.primary:hover {
    background: #1d4ed8;
    border-color: #1d4ed8;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .toggle-option {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 16px;
    background: #ffffff;
    border: 2px solid #f1f5f9;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .toggle-option:hover {
    border-color: #e2e8f0;
    background: #f8fafc;
  }

  .toggle-option:has(input:checked) {
    border-color: #3b82f6;
    background: #eff6ff;
  }

  .toggle-option input[type="radio"] {
    width: 20px;
    height: 20px;
    margin-top: 2px;
    accent-color: #3b82f6;
  }

  .toggle-label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .toggle-title {
    color: #1f2937;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .toggle-desc {
    color: #6b7280;
    font-size: 0.85rem;
  }

  .button-row {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 16px;
    margin-top: 32px;
    padding-top: 16px;
    border-top: 1px solid #f1f5f9;
  }
</style>
