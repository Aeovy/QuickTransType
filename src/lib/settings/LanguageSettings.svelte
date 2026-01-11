<script lang="ts">
    import type { Language, LanguageConfig } from "../stores/appState";

    export let languageConfig: LanguageConfig;
    export let historyLimit: number;
    export let onUpdate: (currentTarget: string, favorites: Language[], historyLimit: number) => Promise<void>;

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

    // 自动保存的辅助函数
    async function updateAndSave() {
        await onUpdate(languageConfig.current_target, languageConfig.favorite_languages, historyLimit);
    }

    function handleTargetChange() {
        updateAndSave();
    }

    function handleHistoryLimitChange() {
        updateAndSave();
    }

    function addLanguage(lang: Language) {
        const exists = languageConfig.favorite_languages.some(l => l.code === lang.code);
        if (!exists) {
            languageConfig.favorite_languages = [...languageConfig.favorite_languages, lang];
            updateAndSave();
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
        updateAndSave();
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
        <select id="current-target" bind:value={languageConfig.current_target} onchange={handleTargetChange}>
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
                <button class="remove-btn" onclick={()=> removeLanguage(lang.code)}
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
                <button class="preset-btn" onclick={()=> addLanguage(preset)}>
                    + {preset.name}
                </button>
                {/each}
            </div>

            <button class="add-custom-btn" onclick={()=> showAddDialog = true}>
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
                <input type="text" id="new-lang-name" bind:value={newLangName} placeholder="例如: 粤语" />
            </div>
            <div class="form-group">
                <label for="new-lang-code">语言代码</label>
                <input type="text" id="new-lang-code" bind:value={newLangCode} placeholder="例如: zh-YUE" />
            </div>
            <div class="dialog-buttons">
                <button class="btn secondary" onclick={()=> showAddDialog = false}>取消</button>
                <button class="btn primary" onclick={addCustomLanguage}>添加</button>
            </div>
        </div>
    </div>
    {/if}

    <div class="form-group">
        <label for="history-limit">历史记录保存条数</label>
        <select id="history-limit" bind:value={historyLimit} onchange={handleHistoryLimitChange}>
            {#each historyLimits as limit}
            <option value={limit}>{limit} 条</option>
            {/each}
        </select>
    </div>

    <div class="button-row">
        <p class="auto-save-hint">✨ 设置已自动保存</p>
    </div>
</div>

<style>
    .settings-section h2 {
        margin: 0 0 20px 0;
        color: #2563eb;
        font-size: 1.3rem;
    }

    .form-group {
        margin-bottom: 25px;
    }

    .form-group label {
        display: block;
        margin-bottom: 8px;
        color: #4b5563;
        font-size: 0.9rem;
    }

    .hint {
        margin: 0 0 10px 0;
        color: #6b7280;
        font-size: 0.8rem;
    }

    .button-row {
        display: flex;
        justify-content: flex-end;
        margin-top: 32px;
        padding-top: 16px;
        border-top: 1px solid #f1f5f9;
    }

    .auto-save-hint {
        color: #64748b;
        font-size: 0.75rem;
        font-weight: 500;
        margin: 0;
        display: flex;
        align-items: center;
        gap: 6px;
        background: #f8fafc;
        padding: 6px 14px;
        border-radius: 20px;
        border: 1px solid #e2e8f0;
    }

    select,
    input[type="text"] {
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

    select:focus,
    input:focus {
        outline: none;
        border-color: #2563eb;
        box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.2);
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
        background: #ffffff;
        border: 1px solid #d1d5db;
        border-radius: 6px;
        margin-bottom: 8px;
        box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    }

    .lang-name {
        flex: 1;
        color: #1f2937;
    }

    .lang-code {
        color: #6b7280;
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
        background: #fecaca;
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
        border: 1px dashed #d1d5db;
        border-radius: 20px;
        background: #f9fafb;
        color: #6b7280;
        cursor: pointer;
        font-size: 0.85rem;
        transition: all 0.2s;
    }

    .preset-btn:hover {
        border-color: #2563eb;
        color: #2563eb;
        background: #eff6ff;
    }

    .add-custom-btn {
        padding: 10px 20px;
        border: 1px solid #2563eb;
        border-radius: 6px;
        background: transparent;
        color: #2563eb;
        cursor: pointer;
        font-size: 0.9rem;
    }

    .add-custom-btn:hover {
        background: rgba(37, 99, 235, 0.05);
    }

    .dialog-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
        backdrop-filter: blur(2px);
    }

    .dialog {
        background: #ffffff;
        padding: 25px;
        border-radius: 12px;
        width: 400px;
        max-width: 90%;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
    }

    .dialog h3 {
        margin: 0 0 20px 0;
        color: #2563eb;
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
        background: #2563eb;
        color: #ffffff;
    }

    .btn.primary:hover {
        background: #1d4ed8;
    }

    .btn.secondary {
        background: transparent;
        color: #6b7280;
        border: 1px solid #d1d5db;
    }

    .btn.secondary:hover {
        color: #1f2937;
        border-color: #9ca3af;
        background: #f3f4f6;
    }
</style>