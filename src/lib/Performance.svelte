<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  interface PerformanceStats {
    total_translations: number;
    successful_translations: number;
    failed_translations: number;
    avg_duration_ms: number;
    min_duration_ms: number;
    max_duration_ms: number;
    total_chars_translated: number;
    selected_mode_count: number;
    full_mode_count: number;
    total_completion_tokens: number;
    avg_tokens_per_second: number;
    error_distribution: Array<{ error_type: string; count: number }>;
    hourly_data: Array<{ hour: number; avg_duration: number; count: number }>;
  }

  let stats: PerformanceStats | null = null;
  let period: "hour" | "day" | "week" = "day";
  let isLoading = false;
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let unlistenHistoryCleared: UnlistenFn | null = null;

  onMount(async () => {
    loadStats();
    refreshInterval = setInterval(loadStats, 30000); // 每30秒刷新
    
    // 监听历史清空事件
    unlistenHistoryCleared = await listen("history-cleared", () => {
      console.log("History cleared, refreshing performance stats");
      loadStats();
    });
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
    if (unlistenHistoryCleared) {
      unlistenHistoryCleared();
    }
  });

  async function loadStats() {
    isLoading = true;
    try {
      stats = await invoke<PerformanceStats>("get_performance_stats", { period });
    } catch (e) {
      console.error("Failed to load stats:", e);
      stats = null;
    }
    isLoading = false;
  }

  function handlePeriodChange() {
    loadStats();
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms.toFixed(0)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function getSuccessRate(): number {
    if (!stats || stats.total_translations === 0) return 0;
    return (stats.successful_translations / stats.total_translations) * 100;
  }

  function getSuccessRateColor(): string {
    const rate = getSuccessRate();
    if (rate >= 95) return "#22c55e";
    if (rate >= 80) return "#fbbf24";
    return "#ef4444";
  }

  function isPerformanceWarning(): boolean {
    return stats !== null && stats.avg_duration_ms > 5000;
  }

  function isErrorRateWarning(): boolean {
    const rate = getSuccessRate();
    return stats !== null && stats.total_translations > 0 && rate < 90;
  }
</script>

<div class="performance-section">
  <h2>性能监控</h2>

  <div class="period-selector">
    <label>
      <input
        type="radio"
        bind:group={period}
        value="hour"
        onchange={handlePeriodChange}
      />
      最近 1 小时
    </label>
    <label>
      <input
        type="radio"
        bind:group={period}
        value="day"
        onchange={handlePeriodChange}
      />
      最近 24 小时
    </label>
    <label>
      <input
        type="radio"
        bind:group={period}
        value="week"
        onchange={handlePeriodChange}
      />
      最近 7 天
    </label>
    <button class="refresh-btn" onclick={loadStats} disabled={isLoading}>
      {isLoading ? "刷新中..." : "🔄 刷新"}
    </button>
  </div>

  {#if isPerformanceWarning()}
    <div class="warning-banner">
      ⚠️ 性能异常：API 响应时间过长（平均 {formatDuration(stats?.avg_duration_ms ?? 0)}）
    </div>
  {/if}

  {#if isErrorRateWarning()}
    <div class="warning-banner error">
      ⚠️ 翻译失败率过高（{(100 - getSuccessRate()).toFixed(1)}%），请检查网络或 API 配置
    </div>
  {/if}

  {#if isLoading && !stats}
    <div class="loading">加载中...</div>
  {:else if stats}
    <div class="stats-grid">
      <div class="stat-card">
        <h3>总翻译次数</h3>
        <p class="stat-value">{stats.total_translations.toLocaleString()}</p>
      </div>
      <div class="stat-card">
        <h3>成功率</h3>
        <p class="stat-value" style="color: {getSuccessRateColor()}">
          {getSuccessRate().toFixed(1)}%
        </p>
      </div>
      <div class="stat-card">
        <h3>平均耗时</h3>
        <p class="stat-value">{formatDuration(stats.avg_duration_ms)}</p>
      </div>
      <div class="stat-card">
        <h3>平均输出速度</h3>
        <p class="stat-value">{stats.avg_tokens_per_second.toFixed(1)} <span class="unit">tokens/s</span></p>
      </div>
      <div class="stat-card">
        <h3>总字符数</h3>
        <p class="stat-value">{stats.total_chars_translated.toLocaleString()}</p>
      </div>
      <div class="stat-card">
        <h3>总 Token 数</h3>
        <p class="stat-value">{stats.total_completion_tokens.toLocaleString()}</p>
      </div>
    </div>

    <div class="charts-grid">
      <div class="chart-card">
        <h3>操作模式分布</h3>
        <div class="bar-chart">
          <div class="bar-item">
            <span class="bar-label">选中翻译</span>
            <div class="bar-container">
              <div 
                class="bar selected" 
                style="width: {stats.total_translations > 0 ? (stats.selected_mode_count / stats.total_translations) * 100 : 0}%"
              ></div>
            </div>
            <span class="bar-value">{stats.selected_mode_count}</span>
          </div>
          <div class="bar-item">
            <span class="bar-label">全文翻译</span>
            <div class="bar-container">
              <div 
                class="bar full" 
                style="width: {stats.total_translations > 0 ? (stats.full_mode_count / stats.total_translations) * 100 : 0}%"
              ></div>
            </div>
            <span class="bar-value">{stats.full_mode_count}</span>
          </div>
        </div>
      </div>

      <div class="chart-card">
        <h3>成功/失败统计</h3>
        <div class="pie-chart">
          <div class="pie-legend">
            <div class="legend-item">
              <span class="dot success"></span>
              成功: {stats.successful_translations}
            </div>
            <div class="legend-item">
              <span class="dot error"></span>
              失败: {stats.failed_translations}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="chart-card full-width">
      <h3>耗时统计</h3>
      <div class="duration-stats">
        <div class="duration-item">
          <span class="label">最小耗时</span>
          <span class="value">{formatDuration(stats.min_duration_ms)}</span>
        </div>
        <div class="duration-item">
          <span class="label">平均耗时</span>
          <span class="value highlight">{formatDuration(stats.avg_duration_ms)}</span>
        </div>
        <div class="duration-item">
          <span class="label">最大耗时</span>
          <span class="value">{formatDuration(stats.max_duration_ms)}</span>
        </div>
      </div>
    </div>

    {#if stats.error_distribution.length > 0}
      <div class="chart-card full-width">
        <h3>错误类型统计</h3>
        <table class="error-table">
          <thead>
            <tr>
              <th>错误类型</th>
              <th>次数</th>
              <th>占比</th>
            </tr>
          </thead>
          <tbody>
            {#each stats.error_distribution as error}
              <tr>
                <td>{error.error_type}</td>
                <td>{error.count}</td>
                <td>{((error.count / stats.failed_translations) * 100).toFixed(1)}%</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else}
    <div class="empty">暂无性能数据</div>
  {/if}
</div>

<style>
  .performance-section h2 {
    margin: 0 0 20px 0;
    color: #2563eb;
    font-size: 1.3rem;
  }

  .period-selector {
    display: flex;
    gap: 20px;
    align-items: center;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

  .period-selector label {
    display: flex;
    align-items: center;
    gap: 5px;
    color: #6b7280;
    cursor: pointer;
  }

  .period-selector input[type="radio"] {
    accent-color: #2563eb;
  }

  .refresh-btn {
    margin-left: auto;
    padding: 8px 16px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: transparent;
    color: #6b7280;
    cursor: pointer;
  }

  .refresh-btn:hover:not(:disabled) {
    border-color: #2563eb;
    color: #2563eb;
    background: #eff6ff;
  }

  .warning-banner {
    padding: 12px 20px;
    background: #fffbeb;
    border: 1px solid #f59e0b;
    border-radius: 8px;
    color: #b45309;
    margin-bottom: 20px;
  }

  .warning-banner.error {
    background: #fef2f2;
    border-color: #ef4444;
    color: #b91c1c;
  }

  .loading, .empty {
    text-align: center;
    padding: 40px;
    color: #6b7280;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 15px;
    margin-bottom: 25px;
  }

  .stat-card {
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 20px;
    text-align: center;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .stat-card h3 {
    margin: 0 0 10px 0;
    color: #6b7280;
    font-size: 0.85rem;
    font-weight: normal;
  }

  .stat-value {
    margin: 0;
    font-size: 1.8rem;
    font-weight: 600;
    color: #2563eb;
  }

  .stat-value .unit {
    font-size: 0.9rem;
    font-weight: normal;
    color: #6b7280;
  }

  .charts-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 20px;
    margin-bottom: 20px;
  }

  .chart-card {
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 20px;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  .chart-card.full-width {
    grid-column: 1 / -1;
  }

  .chart-card h3 {
    margin: 0 0 15px 0;
    color: #374151;
    font-size: 0.95rem;
  }

  .bar-chart {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .bar-item {
    display: flex;
    align-items: center;
    gap: 15px;
  }

  .bar-label {
    width: 80px;
    color: #6b7280;
    font-size: 0.85rem;
  }

  .bar-container {
    flex: 1;
    height: 24px;
    background: #f3f4f6;
    border-radius: 4px;
    overflow: hidden;
  }

  .bar {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .bar.selected {
    background: #22c55e;
  }

  .bar.full {
    background: #a78bfa;
  }

  .bar-value {
    width: 50px;
    text-align: right;
    color: #4b5563;
    font-size: 0.9rem;
  }

  .pie-chart {
    display: flex;
    justify-content: center;
    padding: 20px;
  }

  .pie-legend {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 10px;
    color: #4b5563;
    font-size: 0.95rem;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .dot.success {
    background: #22c55e;
  }

  .dot.error {
    background: #ef4444;
  }

  .duration-stats {
    display: flex;
    justify-content: space-around;
    padding: 20px 0;
  }

  .duration-item {
    text-align: center;
  }

  .duration-item .label {
    display: block;
    color: #6b7280;
    font-size: 0.8rem;
    margin-bottom: 8px;
  }

  .duration-item .value {
    font-size: 1.2rem;
    color: #374151;
  }

  .duration-item .value.highlight {
    color: #2563eb;
    font-weight: 600;
  }

  .error-table {
    width: 100%;
    border-collapse: collapse;
  }

  .error-table th,
  .error-table td {
    padding: 12px 15px;
    text-align: left;
    border-bottom: 1px solid #e5e7eb;
  }

  .error-table th {
    color: #6b7280;
    font-weight: normal;
    font-size: 0.85rem;
  }

  .error-table td {
    color: #374151;
  }

  @media (max-width: 768px) {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .charts-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
