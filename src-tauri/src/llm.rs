//! LLM 客户端模块
//! 处理与 LLM API 的通信

use crate::config::LLMConfig;
use crate::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

/// LLM 客户端
pub struct LLMClient {
    client: Client,
}

/// OpenAI API 请求体
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    top_p: f32,
}

/// 消息结构
#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI API 响应体
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
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

/// API 错误响应
#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiError,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

impl LLMClient {
    /// 创建新的 LLM 客户端
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Network(e))?;

        Ok(Self { client })
    }

    /// 测试 LLM 连接
    pub async fn test_connection(&self, config: &LLMConfig) -> Result<String> {
        info!("Testing LLM connection...");
        
        // 验证配置
        if config.api_key.is_empty() {
            return Err(AppError::Config("API Key 不能为空".to_string()));
        }
        if config.base_url.is_empty() {
            return Err(AppError::Config("Base URL 不能为空".to_string()));
        }

        // 发送测试请求
        let test_text = "Hello";
        let user_prompt = config
            .user_prompt_template
            .replace("{target_language}", "中文")
            .replace("{text}", test_text);

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
        };

        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        debug!("Sending test request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!("Network error: {}", e);
                AppError::Network(e)
            })?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            
            // 尝试解析 API 错误
            if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                let error_msg = match status.as_u16() {
                    401 => format!("认证失败: {}", api_error.error.message),
                    429 => format!("请求频率限制: {}", api_error.error.message),
                    500..=599 => format!("服务器错误: {}", api_error.error.message),
                    _ => format!("API 错误 ({}): {}", status, api_error.error.message),
                };
                return Err(AppError::LlmApi(error_msg));
            }
            
            return Err(AppError::LlmApi(format!(
                "API 请求失败 ({}): {}",
                status, error_text
            )));
        }

        let result: ChatCompletionResponse = response.json().await.map_err(|e| {
            error!("Failed to parse response: {}", e);
            AppError::LlmApi(format!("解析响应失败: {}", e))
        })?;

        let translated = result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::LlmApi("API 返回空响应".to_string()))?;

        info!("LLM connection test successful");
        Ok(format!("连接成功！测试翻译: {} → {}", test_text, translated.trim()))
    }

    /// 翻译文本
    pub async fn translate(
        &self,
        config: &LLMConfig,
        text: &str,
        target_language: &str,
    ) -> Result<String> {
        debug!("Translating text ({} chars) to {}", text.len(), target_language);

        if config.api_key.is_empty() {
            return Err(AppError::Config("API Key 未配置".to_string()));
        }

        // 构建用户提示
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
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            
            if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                return Err(AppError::LlmApi(api_error.error.message));
            }
            
            return Err(AppError::LlmApi(format!(
                "翻译请求失败 ({})",
                status
            )));
        }

        let result: ChatCompletionResponse = response.json().await.map_err(|e| {
            AppError::LlmApi(format!("解析翻译响应失败: {}", e))
        })?;

        let translated = result
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| AppError::LlmApi("翻译 API 返回空响应".to_string()))?;

        debug!("Translation completed: {} chars", translated.len());
        Ok(translated)
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
    fn test_build_user_prompt_multiple_vars() {
        let template = "Translate to {target_language}: {text} (language: {target_language})";
        let result = build_user_prompt(template, "Chinese", "Hello");
        assert_eq!(result, "Translate to Chinese: Hello (language: Chinese)");
    }
}
