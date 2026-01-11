//! Tauri 命令模块
//! 定义前端可调用的所有 IPC 命令

use crate::config::{AppConfig, Hotkey, LLMConfig};
use crate::database::{HistoryResult, PerformanceStats};
use crate::hotkey::HotkeyManager;
use crate::llm::LLMClient;
use crate::state::AppState;
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tracing::{debug, error, info};

/// 获取应用配置
#[tauri::command]
pub async fn get_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    debug!("Getting config");
    Ok(state.get_config().await)
}

/// 保存应用配置
#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Saving config");
    state
        .save_config(&config)
        .await
        .map_err(|e| e.to_string())?;

    // 清理历史记录（如果超过限制）
    state
        .database
        .cleanup_history(config.history_limit)
        .await
        .map_err(|e| {
            error!("Failed to cleanup history: {}", e);
            e.to_string()
        })?;

    Ok(())
}

/// 测试 LLM 连接
#[tauri::command]
pub async fn test_llm_connection(config: LLMConfig) -> Result<String, String> {
    info!("Testing LLM connection");
    let client = LLMClient::new().map_err(|e| e.to_string())?;
    client
        .test_connection(&config)
        .await
        .map_err(|e| e.to_string())
}

/// 获取翻译历史
#[tauri::command]
pub async fn get_history(
    page: i64,
    page_size: i64,
    search: Option<String>,
    mode: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<HistoryResult, String> {
    debug!("Getting history: page={}, size={}", page, page_size);
    state
        .database
        .get_history(page, page_size, search.as_deref(), mode.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 获取性能统计
#[tauri::command]
pub async fn get_performance_stats(
    period: String,
    state: State<'_, Arc<AppState>>,
) -> Result<PerformanceStats, String> {
    debug!("Getting performance stats for period: {}", period);
    state
        .database
        .get_performance_stats(&period)
        .await
        .map_err(|e| e.to_string())
}

/// 检查热键冲突
#[tauri::command]
pub async fn check_hotkey_conflicts(hotkey: Hotkey) -> Result<Vec<String>, String> {
    debug!("Checking hotkey conflicts: {:?}", hotkey);
    Ok(HotkeyManager::check_system_conflicts(&hotkey))
}

/// 切换目标语言
#[tauri::command]
pub async fn switch_language(
    language_code: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Switching target language to: {}", language_code);
    
    let mut config = state.get_config().await;
    config.language.current_target = language_code;
    
    state
        .save_config(&config)
        .await
        .map_err(|e| e.to_string())
}

/// 翻译文本（供测试和手动调用）
#[tauri::command]
pub async fn translate_text(
    text: String,
    mode: String,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    info!("Translating text ({} chars) in {} mode", text.len(), mode);
    
    let start = Instant::now();
    let config = state.get_config().await;
    
    let result = state
        .llm_client
        .translate(&config.llm, &text, &config.language.current_target)
        .await;

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as i64;

    match &result {
        Ok(translated) => {
            // 记录成功的翻译
            if let Err(e) = state
                .database
                .insert_translation(
                    &text,
                    translated,
                    None,
                    &config.language.current_target,
                    &mode,
                )
                .await
            {
                error!("Failed to save translation: {}", e);
            }

            // 记录性能指标
            if let Err(e) = state
                .database
                .record_metric(&mode, duration_ms, true, None, text.len() as i64)
                .await
            {
                error!("Failed to record metric: {}", e);
            }

            // 清理旧的历史记录
            if let Err(e) = state.database.cleanup_history(config.history_limit).await {
                error!("Failed to cleanup history: {}", e);
            }

            info!("Translation completed in {}ms", duration_ms);
        }
        Err(e) => {
            // 记录失败的指标
            let error_type = match &e {
                crate::error::AppError::Network(_) => "network",
                crate::error::AppError::LlmApi(_) => "api",
                crate::error::AppError::Config(_) => "config",
                _ => "other",
            };

            if let Err(record_err) = state
                .database
                .record_metric(&mode, duration_ms, false, Some(error_type), 0)
                .await
            {
                error!("Failed to record metric: {}", record_err);
            }

            error!("Translation failed: {}", e);
        }
    }

    result.map_err(|e| e.to_string())
}
