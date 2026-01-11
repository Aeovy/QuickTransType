<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { Hotkey, HotkeyConfig } from "../stores/appState";

  export let hotkeyConfig: HotkeyConfig;
  export let onUpdate: (selected: Hotkey, full: Hotkey) => Promise<void>;

  let selectedRecording = false;
  let fullRecording = false;
  let conflictWarning: string | null = null;

  // 临时编辑状态
  let selectedMode: Hotkey = { ...hotkeyConfig.selected_mode };
  let fullMode: Hotkey = { ...hotkeyConfig.full_mode };

  // 连续按键次数
  let consecutiveCount = fullMode.type === "Consecutive" ? (fullMode.count ?? 3) : 3;

  const modifierMap: Record<string, string> = {
    Meta: "Cmd",
    Control: "Ctrl",
    Alt: "Option",
    Shift: "Shift",
  };

  function formatHotkey(hotkey: Hotkey): string {
    if (hotkey.type === "Combination") {
      const mods = (hotkey.modifiers ?? []).map((m) => modifierMap[m] ?? m);
      return [...mods, hotkey.key.toUpperCase()].join(" + ");
    } else {
      const keyName = hotkey.key === " " ? "Space" : hotkey.key.toUpperCase();
      return `${keyName} × ${hotkey.count ?? 3}`;
    }
  }

  function handleSelectedKeydown(event: KeyboardEvent) {
    if (!selectedRecording) return;
    event.preventDefault();

    const modifiers: string[] = [];
    if (event.metaKey) modifiers.push("Meta");
    if (event.ctrlKey) modifiers.push("Control");
    if (event.altKey) modifiers.push("Alt");
    if (event.shiftKey) modifiers.push("Shift");

    const key = event.key.toLowerCase();

    // 忽略纯修饰键
    if (["meta", "control", "alt", "shift"].includes(key)) return;

    // 验证：选中模式必须有修饰键
    if (modifiers.length === 0) {
      conflictWarning = "选中翻译热键必须包含修饰键（Cmd/Ctrl/Option）";
      return;
    }

    selectedMode = {
      type: "Combination",
      modifiers,
      key,
    };

    conflictWarning = null;
    selectedRecording = false;
    onUpdate(selectedMode, fullMode);
    checkConflict();
  }

  function handleFullKeydown(event: KeyboardEvent) {
    if (!fullRecording) return;
    event.preventDefault();

    const modifiers: string[] = [];
    if (event.metaKey) modifiers.push("Meta");
    if (event.ctrlKey) modifiers.push("Control");
    if (event.altKey) modifiers.push("Alt");
    if (event.shiftKey) modifiers.push("Shift");

    const key = event.key.toLowerCase();

    // 忽略纯修饰键
    if (["meta", "control", "alt", "shift"].includes(key)) return;

    if (modifiers.length > 0) {
      // 组合键模式
      fullMode = {
        type: "Combination",
        modifiers,
        key,
      };
    } else {
      // 连续按键模式
      fullMode = {
        type: "Consecutive",
        key: key === " " ? " " : key,
        count: consecutiveCount,
      };
    }

    fullRecording = false;
    onUpdate(selectedMode, fullMode);
    checkConflict();
  }

  function updateConsecutiveCount() {
    if (fullMode.type === "Consecutive") {
      fullMode = { ...fullMode, count: consecutiveCount };
      onUpdate(selectedMode, fullMode);
    }
  }

  async function checkConflict() {
    try {
      const conflicts = await invoke<string[]>("check_hotkey_conflicts", {
        hotkey: selectedMode,
      });
      if (conflicts.length > 0) {
        conflictWarning = `⚠️ 与系统快捷键冲突: ${conflicts.join(", ")}`;
      }
    } catch {
      // 冲突检测失败，忽略
    }
  }
</script>

<div class="settings-section">
  <h2>热键设置</h2>

  <div class="form-group">
    <label>选中翻译热键</label>
    <p class="hint">翻译当前选中的文本，必须包含修饰键</p>
    <button
      class="hotkey-input"
      class:recording={selectedRecording}
      tabindex="0"
      onclick={(e) => { selectedRecording = true; e.currentTarget.focus(); }}
      onkeydown={handleSelectedKeydown}
      onblur={() => (selectedRecording = false)}
    >
      {#if selectedRecording}
        <span class="recording-text">按下热键组合...</span>
      {:else}
        {formatHotkey(selectedMode)}
      {/if}
    </button>
  </div>

  <div class="form-group">
    <label>全文翻译热键</label>
    <p class="hint">翻译整个输入框，支持组合键或连续按键</p>
    <button
      class="hotkey-input"
      class:recording={fullRecording}
      tabindex="0"
      onclick={(e) => { fullRecording = true; e.currentTarget.focus(); }}
      onkeydown={handleFullKeydown}
      onblur={() => (fullRecording = false)}
    >
      {#if fullRecording}
        <span class="recording-text">按下热键...</span>
      {:else}
        {formatHotkey(fullMode)}
      {/if}
    </button>

    {#if fullMode.type === "Consecutive"}
      <div class="consecutive-count">
        <label for="count">连续按键次数:</label>
        <input
          type="number"
          id="count"
          bind:value={consecutiveCount}
          onchange={updateConsecutiveCount}
          min="2"
          max="10"
        />
      </div>
    {/if}
  </div>

  {#if conflictWarning}
    <div class="warning">
      {conflictWarning}
    </div>
  {/if}

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

  .hotkey-input {
    width: 100%;
    padding: 15px;
    border: 2px solid #d1d5db;
    border-radius: 8px;
    background: #ffffff;
    color: #2563eb;
    font-size: 1.1rem;
    font-weight: 600;
    text-align: center;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .hotkey-input:hover {
    border-color: #2563eb;
  }

  .hotkey-input:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.2);
  }

  .hotkey-input.recording {
    border-color: #d97706;
    animation: pulse 1s infinite;
  }

  .recording-text {
    color: #d97706;
  }

  @keyframes pulse {
    0%, 100% {
      box-shadow: 0 0 0 0 rgba(217, 119, 6, 0.4);
    }
    50% {
      box-shadow: 0 0 0 10px rgba(217, 119, 6, 0);
    }
  }

  .consecutive-count {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 15px;
  }

  .consecutive-count label {
    margin-bottom: 0;
    color: #6b7280;
  }

  .consecutive-count input {
    width: 60px;
    padding: 8px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #ffffff;
    color: #1f2937;
    font-size: 0.95rem;
    text-align: center;
  }

  .warning {
    padding: 12px 15px;
    background: #fffbeb;
    border: 1px solid #d97706;
    border-radius: 6px;
    color: #d97706;
    font-size: 0.9rem;
    margin-bottom: 20px;
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
</style>
