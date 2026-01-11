//! LLM 客户端模块
//! 处理与 LLM API 的通信，支持流式传输

use crate::config::LLMConfig;
use crate::error::{AppError, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info};

/// LLM 客户端
pub struct LLMClient {
    client: Client,
}

/// 翻译结果，包含性能指标
#[derive(Debug, Clone)]
pub struct TranslationResult {
    /// 翻译后的文本
    pub translated_text: String,
    /// 完成 tokens 数量
    pub completion_tokens: Option<u32>,
    /// 请求耗时（毫秒）
    pub duration_ms: u64,
    /// 输出速率 (tokens/s)
    pub tokens_per_second: Option<f64>,
}

/// 流式传输的事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 增量文本
    Delta(String),
    /// 完成，包含统计信息
    Done {
        completion_tokens: Option<u32>,
        duration_ms: u64,
    },
    /// 错误
    Error(String),
}

/// OpenAI API 请求体
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    top_p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

/// 流式选项
#[derive(Debug, Serialize)]
struct StreamOptions {
    include_usage: bool,
}

/// 消息结构
#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI API 响应体 (非流式)
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

/// Usage 统计
#[derive(Debug, Deserialize, Default)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

/// 选择结构
#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

/// 响应消息
#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

/// 流式响应块
#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

/// 流式选择
#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

/// 流式增量内容
#[derive(Debug, Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

/// API 错误响应
#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiError,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

impl LLMClient {
    /// 创建新的 LLM 客户端
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(AppError::Network)?;

        Ok(Self { client })
    }

    /// 测试 LLM 连接
    pub async fn test_connection(&self, config: &LLMConfig) -> Result<String> {
        info!("Testing LLM connection...");
        
        if config.api_key.is_empty() {
            return Err(AppError::Config("API Key 不能为空".to_string()));
        }
        if config.base_url.is_empty() {
            return Err(AppError::Config("Base URL 不能为空".to_string()));
        }

        let test_text = "Hello";
        let result = self.translate(config, test_text, "中文").await?;

        info!("LLM connection test successful");
        Ok(format!(
            "连接成功！测试翻译: {} → {} ({}ms, {:.1} tokens/s)",
            test_text,
            result.translated_text.trim(),
            result.duration_ms,
            result.tokens_per_second.unwrap_or(0.0)
        ))
    }

    /// 翻译文本（非流式）
    pub async fn translate(
        &self,
        config: &LLMConfig,
        text: &str,
        target_language: &str,
    ) -> Result<TranslationResult> {
        debug!("Translating text ({} chars) to {}", text.len(), target_language);

        if config.api_key.is_empty() {
            return Err(AppError::Config("API Key 未配置".to_string()));
        }

        let user_prompt = build_user_prompt(&config.user_prompt_template, target_language, text);
        let start_time = Instant::now();

        let request_body = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: config.system_prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: config.temperature,
            top_p: config.top_p,
            stream: None,
            stream_options: None,
        };

        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            
            if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                return Err(AppError::LlmApi(api_error.error.message));
            }
            
            return Err(AppError::LlmApi(format!("翻译请求失败 ({})", status)));
        }

        // 解析完整响应以获取 usage
        let response_text = response.text().await?;
        let result: ChatCompletionResponse = serde_json::from_str(&response_text)
            .map_err(|e| AppError::LlmApi(format!("解析翻译响应失败: {}", e)))?;

        let translated = result
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| AppError::LlmApi("翻译 API 返回空响应".to_string()))?;

        // 获取 completion_tokens
        let completion_tokens = result.usage.as_ref().map(|u| u.completion_tokens);
        
        // 如果 usage 中没有，尝试从响应文本中搜索
        let completion_tokens = completion_tokens.or_else(|| {
            extract_completion_tokens(&response_text)
        });

        let tokens_per_second = completion_tokens.map(|t| {
            if duration_ms > 0 {
                (t as f64) / (duration_ms as f64 / 1000.0)
            } else {
                0.0
            }
        });

        debug!(
            "Translation completed: {} chars, {} tokens, {}ms, {:.1} tokens/s",
            translated.len(),
            completion_tokens.unwrap_or(0),
            duration_ms,
            tokens_per_second.unwrap_or(0.0)
        );

        Ok(TranslationResult {
            translated_text: translated,
            completion_tokens,
            duration_ms,
            tokens_per_second,
        })
    }

    /// 流式翻译文本
    pub async fn translate_stream(
        &self,
        config: &LLMConfig,
        text: &str,
        target_language: &str,
    ) -> Result<mpsc::Receiver<StreamEvent>> {
        debug!("Starting streaming translation ({} chars) to {}", text.len(), target_language);

        if config.api_key.is_empty() {
            return Err(AppError::Config("API Key 未配置".to_string()));
        }

        let (tx, rx) = mpsc::channel(100);

        let user_prompt = build_user_prompt(&config.user_prompt_template, target_language, text);

        let request_body = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: config.system_prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: config.temperature,
            top_p: config.top_p,
            stream: Some(true),
            stream_options: Some(StreamOptions {
                include_usage: true,
            }),
        };

        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let client = self.client.clone();
        let api_key = config.api_key.clone();

        // 在后台任务中处理流式响应
        tokio::spawn(async move {
            let start_time = Instant::now();
            let mut total_tokens = 0u32;

            let response = match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(StreamEvent::Error(format!("请求失败: {}", e))).await;
                    return;
                }
            };

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                let _ = tx.send(StreamEvent::Error(format!("API 错误: {}", error_text))).await;
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(format!("读取流失败: {}", e))).await;
                        break;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // 处理 SSE 格式的数据
                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }

                    if let Some(json_str) = line.strip_prefix("data: ") {
                        match serde_json::from_str::<StreamChunk>(json_str) {
                            Ok(chunk_data) => {
                                // 检查 usage (某些 API 在流式响应的最后一块包含 usage)
                                if let Some(usage) = &chunk_data.usage {
                                    total_tokens = usage.completion_tokens;
                                    debug!("Received usage info: {} completion_tokens", total_tokens);
                                }

                                for choice in chunk_data.choices {
                                    if let Some(content) = choice.delta.content {
                                        if !content.is_empty() {
                                            let _ = tx.send(StreamEvent::Delta(content)).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse chunk: {}, raw: {}", e, json_str);
                            }
                        }
                    }
                }
            }

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let _ = tx.send(StreamEvent::Done {
                completion_tokens: if total_tokens > 0 { Some(total_tokens) } else { None },
                duration_ms,
            }).await;
        });

        Ok(rx)
    }
}

impl Default for LLMClient {
    fn default() -> Self {
        Self::new().expect("Failed to create LLM client")
    }
}

/// 构建用户提示
fn build_user_prompt(template: &str, target_language: &str, text: &str) -> String {
    template
        .replace("{target_language}", target_language)
        .replace("{text}", text)
}

/// 从响应文本中提取 completion_tokens
fn extract_completion_tokens(response_text: &str) -> Option<u32> {
    // 尝试用正则或简单搜索找 completion_tokens
    if let Some(pos) = response_text.find("\"completion_tokens\"") {
        let after = &response_text[pos..];
        // 查找数字
        let mut num_str = String::new();
        let mut found_colon = false;
        for c in after.chars() {
            if c == ':' {
                found_colon = true;
            } else if found_colon && c.is_ascii_digit() {
                num_str.push(c);
            } else if found_colon && !c.is_whitespace() && !c.is_ascii_digit() {
                break;
            }
        }
        if !num_str.is_empty() {
            return num_str.parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_user_prompt() {
        let template = "将下列文本翻译为{target_language}：{text}";
        let result = build_user_prompt(template, "English", "你好");
        assert_eq!(result, "将下列文本翻译为English：你好");
    }

    #[test]
    fn test_extract_completion_tokens() {
        let response = r#"{"usage":{"completion_tokens":92,"prompt_tokens":10}}"#;
        assert_eq!(extract_completion_tokens(response), Some(92));
    }
}
