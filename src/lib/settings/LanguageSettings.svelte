<script lang="ts">
  import type { LanguageConfig, Language } from "../stores/appState";

  export let languageConfig: LanguageConfig;
  export let historyLimit: number;
  export let onUpdate: (currentTarget: string, favorites: Language[], historyLimit: number) => void;
  export let onSave: () => void;

  let showAddDialog = false;
  let newLangCode = "";
  let newLangName = "";

  const presetLanguages: Language[] = [
    { code: "en-US", name: "English" },
    { code: "zh-CN", name: "简体中文" },
    { code: "zh-TW", name: "繁體中文" },
    { code: "ja-JP", name: "日本語" },
    { code: "ko-KR", name: "한국어" },
    { code: "fr-FR", name: "Français" },
    { code: "de-DE", name: "Deutsch" },
    { code: "es-ES", name: "Español" },
    { code: "it-IT", name: "Italiano" },
    { code: "pt-BR", name: "Português" },
    { code: "ru-RU", name: "Русский" },
    { code: "ar-SA", name: "العربية" },
    { code: "th-TH", name: "ไทย" },
    { code: "vi-VN", name: "Tiếng Việt" },
  ];

  const historyLimits = [100, 200, 500, 1000, 2000, 5000];

  function handleTargetChange() {
    onUpdate(languageConfig.current_target, languageConfig.favorite_languages, historyLimit);
  }

  function handleHistoryLimitChange() {
    onUpdate(languageConfig.current_target, languageConfig.favorite_languages, historyLimit);
  }

  function addLanguage(lang: Language) {
    const exists = languageConfig.favorite_languages.some(l => l.code === lang.code);
    if (!exists) {
      languageConfig.favorite_languages = [...languageConfig.favorite_languages, lang];
      onUpdate(languageConfig.current_target, languageConfig.favorite_languages, historyLimit);
    }
  }

  function addCustomLanguage() {
    if (newLangCode && newLangName) {
      addLanguage({ code: newLangCode, name: newLangName });
      newLangCode = "";
      newLangName = "";
      showAddDialog = false;
    }
  }

  function removeLanguage(code: string) {
    languageConfig.favorite_languages = languageConfig.favorite_languages.filter(l => l.code !== code);
    // 如果删除的是当前目标语言，切换到第一个
    if (languageConfig.current_target === code && languageConfig.favorite_languages.length > 0) {
      languageConfig.current_target = languageConfig.favorite_languages[0].code;
    }
    onUpdate(languageConfig.current_target, languageConfig.favorite_languages, historyLimit);
  }

  function getAvailablePresets(): Language[] {
    return presetLanguages.filter(
      p => !languageConfig.favorite_languages.some(f => f.code === p.code)
    );
  }
</script>

<div class="settings-section">
  <h2>语言设置</h2>

  <div class="form-group">
    <label for="current-target">当前目标语言</label>
    <select
      id="current-target"
      bind:value={languageConfig.current_target}
      on:change={handleTargetChange}
    >
      {#each languageConfig.favorite_languages as lang}
        <option value={lang.code}>{lang.name}</option>
      {/each}
    </select>
  </div>

  <div class="form-group">
    <label>常用语言列表</label>
    <p class="hint">这些语言将显示在系统托盘的快速切换菜单中</p>
    
    <ul class="language-list">
      {#each languageConfig.favorite_languages as lang}
        <li>
          <span class="lang-name">{lang.name}</span>
          <span class="lang-code">{lang.code}</span>
          <button 
            class="remove-btn"
            on:click={() => removeLanguage(lang.code)}
            title="删除"
          >
            ✕
          </button>
        </li>
      {/each}
    </ul>

    <div class="add-language">
      <div class="preset-list">
        {#each getAvailablePresets() as preset}
          <button class="preset-btn" on:click={() => addLanguage(preset)}>
            + {preset.name}
          </button>
        {/each}
      </div>
      
      <button class="add-custom-btn" on:click={() => showAddDialog = true}>
        + 自定义语言
      </button>
    </div>
  </div>

  {#if showAddDialog}
    <div class="dialog-overlay">
      <div class="dialog">
        <h3>添加自定义语言</h3>
        <div class="form-group">
          <label for="new-lang-name">语言名称</label>
          <input
            type="text"
            id="new-lang-name"
            bind:value={newLangName}
            placeholder="例如: 粤语"
          />
        </div>
        <div class="form-group">
          <label for="new-lang-code">语言代码</label>
          <input
            type="text"
            id="new-lang-code"
            bind:value={newLangCode}
            placeholder="例如: zh-YUE"
          />
        </div>
        <div class="dialog-buttons">
          <button class="btn secondary" on:click={() => showAddDialog = false}>取消</button>
          <button class="btn primary" on:click={addCustomLanguage}>添加</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="form-group">
    <label for="history-limit">历史记录保存条数</label>
    <select
      id="history-limit"
      bind:value={historyLimit}
      on:change={handleHistoryLimitChange}
    >
      {#each historyLimits as limit}
        <option value={limit}>{limit} 条</option>
      {/each}
    </select>
  </div>

  <div class="button-row">
    <button class="btn primary" on:click={onSave}>保存设置</button>
  </div>
</div>

<style>
  .settings-section h2 {
    margin: 0 0 20px 0;
    color: #00d4ff;
    font-size: 1.3rem;
  }

  .form-group {
    margin-bottom: 25px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    color: #ccc;
    font-size: 0.9rem;
  }

  .hint {
    margin: 0 0 10px 0;
    color: #666;
    font-size: 0.8rem;
  }

  select, input[type="text"] {
    width: 100%;
    padding: 12px;
    border: 1px solid #333;
    border-radius: 6px;
    background: #0f1729;
    color: #eaeaea;
    font-size: 0.95rem;
    box-sizing: border-box;
  }

  select:focus, input:focus {
    outline: none;
    border-color: #00d4ff;
  }

  .language-list {
    list-style: none;
    padding: 0;
    margin: 0 0 15px 0;
  }

  .language-list li {
    display: flex;
    align-items: center;
    padding: 10px 15px;
    background: #0f1729;
    border: 1px solid #333;
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .lang-name {
    flex: 1;
    color: #eaeaea;
  }

  .lang-code {
    color: #666;
    font-size: 0.85rem;
    margin-right: 15px;
  }

  .remove-btn {
    background: transparent;
    border: none;
    color: #ef4444;
    cursor: pointer;
    font-size: 1rem;
    padding: 5px 10px;
    border-radius: 4px;
  }

  .remove-btn:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  .add-language {
    margin-top: 15px;
  }

  .preset-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 15px;
  }

  .preset-btn {
    padding: 6px 12px;
    border: 1px dashed #444;
    border-radius: 20px;
    background: transparent;
    color: #888;
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.2s;
  }

  .preset-btn:hover {
    border-color: #00d4ff;
    color: #00d4ff;
  }

  .add-custom-btn {
    padding: 10px 20px;
    border: 1px solid #00d4ff;
    border-radius: 6px;
    background: transparent;
    color: #00d4ff;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .add-custom-btn:hover {
    background: rgba(0, 212, 255, 0.1);
  }

  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: #16213e;
    padding: 25px;
    border-radius: 12px;
    width: 400px;
    max-width: 90%;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
    color: #00d4ff;
  }

  .dialog-buttons {
    display: flex;
    gap: 15px;
    justify-content: flex-end;
    margin-top: 20px;
  }

  .button-row {
    display: flex;
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
    color: #888;
    border: 1px solid #444;
  }

  .btn.secondary:hover {
    color: #eaeaea;
    border-color: #666;
  }
</style>
