<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

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
</script>

<div class="history-section">
  <h2>翻译历史</h2>

  <div class="filters">
    <div class="search-box">
      <input
        type="text"
        bind:value={searchQuery}
        on:keyup={(e) => e.key === "Enter" && handleSearch()}
        placeholder="搜索翻译内容..."
      />
      <button on:click={handleSearch}>搜索</button>
    </div>

    <div class="mode-filter">
      <label>
        <input
          type="radio"
          bind:group={modeFilter}
          value="all"
          on:change={handleModeChange}
        />
        全部
      </label>
      <label>
        <input
          type="radio"
          bind:group={modeFilter}
          value="selected"
          on:change={handleModeChange}
        />
        选中翻译
      </label>
      <label>
        <input
          type="radio"
          bind:group={modeFilter}
          value="full"
          on:change={handleModeChange}
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
              <button class="copy-btn" on:click={() => copyToClipboard(record.original_text)}>
                复制
              </button>
            </div>
            <div class="arrow">→</div>
            <div class="text-block">
              <span class="label">译文</span>
              <p>{truncateText(record.translated_text)}</p>
              <button class="copy-btn" on:click={() => copyToClipboard(record.translated_text)}>
                复制
              </button>
            </div>
          </div>
        </li>
      {/each}
    </ul>

    {#if totalPages > 1}
      <div class="pagination">
        <button disabled={currentPage === 1} on:click={() => goToPage(currentPage - 1)}>
          上一页
        </button>
        <span>{currentPage} / {totalPages}</span>
        <button disabled={currentPage === totalPages} on:click={() => goToPage(currentPage + 1)}>
          下一页
        </button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .history-section h2 {
    margin: 0 0 20px 0;
    color: #00d4ff;
    font-size: 1.3rem;
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
    border: 1px solid #333;
    border-radius: 6px;
    background: #0f1729;
    color: #eaeaea;
    font-size: 0.95rem;
  }

  .search-box input:focus {
    outline: none;
    border-color: #00d4ff;
  }

  .search-box button {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    background: #00d4ff;
    color: #0f1729;
    cursor: pointer;
    font-weight: 500;
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
    color: #888;
    cursor: pointer;
  }

  .mode-filter input[type="radio"] {
    accent-color: #00d4ff;
  }

  .loading, .empty {
    text-align: center;
    padding: 40px;
    color: #666;
  }

  .record-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .record-item {
    background: #0f1729;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 15px;
    margin-bottom: 15px;
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
    background: rgba(139, 92, 246, 0.2);
    color: #a78bfa;
  }

  .mode-tag.selected {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .lang-tag {
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 0.75rem;
    background: rgba(0, 212, 255, 0.2);
    color: #00d4ff;
  }

  .time {
    margin-left: auto;
    color: #666;
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
    color: #666;
    font-size: 0.75rem;
    margin-bottom: 5px;
  }

  .text-block p {
    margin: 0;
    color: #ccc;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .copy-btn {
    position: absolute;
    top: 0;
    right: 0;
    padding: 4px 10px;
    border: none;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.1);
    color: #888;
    cursor: pointer;
    font-size: 0.75rem;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .text-block:hover .copy-btn {
    opacity: 1;
  }

  .copy-btn:hover {
    background: rgba(0, 212, 255, 0.2);
    color: #00d4ff;
  }

  .arrow {
    color: #444;
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
    border: 1px solid #333;
    border-radius: 6px;
    background: transparent;
    color: #888;
    cursor: pointer;
  }

  .pagination button:hover:not(:disabled) {
    border-color: #00d4ff;
    color: #00d4ff;
  }

  .pagination button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .pagination span {
    color: #666;
  }
</style>
