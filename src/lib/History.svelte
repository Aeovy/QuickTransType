<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  interface TranslationRecord {
    id: number;
    original_text: string;
    translated_text: string;
    source_lang: string | null;
    target_lang: string;
    mode: string;
    timestamp: number;
  }

  let records: TranslationRecord[] = [];
  let searchQuery = "";
  let modeFilter: "all" | "selected" | "full" = "all";
  let currentPage = 1;
  let totalPages = 1;
  let isLoading = false;
  const pageSize = 20;

  onMount(() => {
    loadHistory();
  });

  async function loadHistory() {
    isLoading = true;
    try {
      const result = await invoke<{ records: TranslationRecord[]; total: number }>("get_history", {
        page: currentPage,
        pageSize,
        search: searchQuery || null,
        mode: modeFilter === "all" ? null : modeFilter,
      });
      records = result.records;
      totalPages = Math.ceil(result.total / pageSize);
    } catch (e) {
      console.error("Failed to load history:", e);
      records = [];
    }
    isLoading = false;
  }

  function handleSearch() {
    currentPage = 1;
    loadHistory();
  }

  function handleModeChange() {
    currentPage = 1;
    loadHistory();
  }

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
      loadHistory();
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      console.error("Failed to copy");
    }
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString("zh-CN");
  }

  function truncateText(text: string, maxLength: number = 100): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  }

  let showClearConfirm = false;

  async function confirmClearHistory() {
    showClearConfirm = true;
  }

  async function clearHistory() {
    try {
      await invoke("clear_history");
      showClearConfirm = false;
      currentPage = 1;
      loadHistory();
      // 通知性能监控组件刷新
      await emit("history-cleared");
    } catch (e) {
      console.error("Failed to clear history:", e);
    }
  }
</script>

<div class="history-section">
  <div class="header-row">
    <h2>翻译历史</h2>
    <button class="clear-btn" onclick={confirmClearHistory}>清空历史</button>
  </div>

  <div class="filters">
    <div class="search-box">
      <input
        type="text"
        bind:value={searchQuery}
        onkeyup={(e) => e.key === "Enter" && handleSearch()}
        placeholder="搜索翻译内容..."
      />
      <button onclick={handleSearch}>搜索</button>
    </div>

    <div class="mode-filter">
      <label>
        <input
          type="radio"
          bind:group={modeFilter}
          value="all"
          onchange={handleModeChange}
        />
        全部
      </label>
      <label>
        <input
          type="radio"
          bind:group={modeFilter}
          value="selected"
          onchange={handleModeChange}
        />
        选中翻译
      </label>
      <label>
        <input
          type="radio"
          bind:group={modeFilter}
          value="full"
          onchange={handleModeChange}
        />
        全文翻译
      </label>
    </div>
  </div>

  {#if isLoading}
    <div class="loading">加载中...</div>
  {:else if records.length === 0}
    <div class="empty">暂无翻译记录</div>
  {:else}
    <ul class="record-list">
      {#each records as record}
        <li class="record-item">
          <div class="record-header">
            <span class="mode-tag" class:selected={record.mode === "selected"}>
              {record.mode === "selected" ? "选中" : "全文"}
            </span>
            <span class="lang-tag">{record.target_lang}</span>
            <span class="time">{formatDate(record.timestamp)}</span>
          </div>
          <div class="record-content">
            <div class="text-block">
              <span class="label">原文</span>
              <p>{truncateText(record.original_text)}</p>
              <button class="copy-btn" onclick={() => copyToClipboard(record.original_text)}>
                复制
              </button>
            </div>
            <div class="arrow">→</div>
            <div class="text-block">
              <span class="label">译文</span>
              <p>{truncateText(record.translated_text)}</p>
              <button class="copy-btn" onclick={() => copyToClipboard(record.translated_text)}>
                复制
              </button>
            </div>
          </div>
        </li>
      {/each}
    </ul>

    {#if totalPages > 1}
      <div class="pagination">
        <button disabled={currentPage === 1} onclick={() => goToPage(currentPage - 1)}>
          上一页
        </button>
        <span>{currentPage} / {totalPages}</span>
        <button disabled={currentPage === totalPages} onclick={() => goToPage(currentPage + 1)}>
          下一页
        </button>
      </div>
    {/if}
  {/if}

  {#if showClearConfirm}
    <div class="dialog-overlay">
      <div class="dialog">
        <h3>确认清空历史</h3>
        <p>确定要清空所有翻译历史记录吗？此操作无法撤销。</p>
        <div class="dialog-buttons">
          <button class="btn secondary" onclick={() => showClearConfirm = false}>取消</button>
          <button class="btn danger" onclick={clearHistory}>确认清空</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .history-section h2 {
    margin: 0;
    color: #2563eb;
    font-size: 1.3rem;
  }

  .clear-btn {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid #ef4444;
    border-radius: 6px;
    color: #ef4444;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .clear-btn:hover {
    background: #fecaca;
  }

  .filters {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

  .search-box {
    display: flex;
    gap: 10px;
    flex: 1;
    min-width: 250px;
  }

  .search-box input {
    flex: 1;
    padding: 10px 15px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #ffffff;
    color: #1f2937;
    font-size: 0.95rem;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .search-box input:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.2);
  }

  .search-box button {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    background: #2563eb;
    color: #ffffff;
    cursor: pointer;
    font-weight: 500;
  }

  .search-box button:hover {
    background: #1d4ed8;
  }

  .mode-filter {
    display: flex;
    gap: 15px;
    align-items: center;
  }

  .mode-filter label {
    display: flex;
    align-items: center;
    gap: 5px;
    color: #6b7280;
    cursor: pointer;
  }

  .mode-filter input[type="radio"] {
    accent-color: #2563eb;
  }

  .loading, .empty {
    text-align: center;
    padding: 40px;
    color: #6b7280;
  }

  .record-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .record-item {
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 15px;
    margin-bottom: 15px;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .record-header {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 12px;
  }

  .mode-tag {
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 0.75rem;
    background: #ede9fe;
    color: #7c3aed;
  }

  .mode-tag.selected {
    background: #dcfce7;
    color: #16a34a;
  }

  .lang-tag {
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 0.75rem;
    background: #dbeafe;
    color: #1e40af;
  }

  .time {
    margin-left: auto;
    color: #6b7280;
    font-size: 0.8rem;
  }

  .record-content {
    display: flex;
    gap: 15px;
    align-items: flex-start;
  }

  .text-block {
    flex: 1;
    position: relative;
  }

  .text-block .label {
    display: block;
    color: #6b7280;
    font-size: 0.75rem;
    margin-bottom: 5px;
  }

  .text-block p {
    margin: 0;
    color: #374151;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .copy-btn {
    position: absolute;
    top: 0;
    right: 0;
    padding: 4px 10px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    background: #ffffff;
    color: #6b7280;
    cursor: pointer;
    font-size: 0.75rem;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .text-block:hover .copy-btn {
    opacity: 1;
  }

  .copy-btn:hover {
    background: #f3f4f6;
    color: #1f2937;
    border-color: #9ca3af;
  }

  .arrow {
    color: #9ca3af;
    font-size: 1.2rem;
    padding-top: 20px;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 20px;
    margin-top: 20px;
  }

  .pagination button {
    padding: 8px 16px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #ffffff;
    color: #6b7280;
    cursor: pointer;
  }

  .pagination button:hover:not(:disabled) {
    border-color: #2563eb;
    color: #2563eb;
    background: #eff6ff;
  }

  .pagination button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .pagination span {
    color: #6b7280;
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
    z-index: 1000;
    backdrop-filter: blur(2px);
  }

  .dialog {
    background: #ffffff;
    border: none;
    border-radius: 10px;
    padding: 25px;
    min-width: 400px;
    max-width: 500px;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  }

  .dialog h3 {
    margin: 0 0 15px 0;
    color: #1f2937;
    font-size: 1.1rem;
  }

  .dialog p {
    margin: 0 0 20px 0;
    color: #4b5563;
    line-height: 1.5;
  }

  .dialog-buttons {
    display: flex;
    gap: 15px;
    justify-content: flex-end;
  }

  .btn {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  .btn.secondary {
    background: transparent;
    color: #6b7280;
    border: 1px solid #d1d5db;
  }

  .btn.secondary:hover {
    border-color: #9ca3af;
    color: #1f2937;
    background: #f3f4f6;
  }

  .btn.danger {
    background: #ef4444;
    color: white;
  }

  .btn.danger:hover {
    background: #dc2626;
  }
</style>
