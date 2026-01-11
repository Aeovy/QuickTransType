//! 配置模块
//! 定义应用程序的配置结构和默认值

use serde::{Deserialize, Serialize};

/// 应用程序全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// LLM 配置
    pub llm: LLMConfig,
    /// 热键配置
    pub hotkey: HotkeyConfig,
    /// 语言配置
    pub language: LanguageConfig,
    /// 历史记录保存条数限制
    pub history_limit: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            hotkey: HotkeyConfig::default(),
            language: LanguageConfig::default(),
            history_limit: 500,
        }
    }
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// API Base URL
    pub base_url: String,
    /// API Key
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// Temperature 参数 (0.0 - 2.0)
    pub temperature: f32,
    /// Top P 参数 (0.0 - 1.0)
    pub top_p: f32,
    /// System Prompt
    pub system_prompt: String,
    /// User Prompt 模板，支持 {target_language} 和 {text} 变量
    pub user_prompt_template: String,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.3,
            top_p: 1.0,
            system_prompt:
                "You are a professional translator. Maintain the original formatting of the text."
                    .to_string(),
            user_prompt_template: "将下列文本翻译为{target_language}，保持原有格式：{text}"
                .to_string(),
        }
    }
}

/// 热键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// 选中翻译模式的热键
    pub selected_mode: Hotkey,
    /// 全文翻译模式的热键
    pub full_mode: Hotkey,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            selected_mode: Hotkey::Combination {
                modifiers: vec!["Control".to_string()],
                key: "k".to_string(),
            },
            // 默认使用组合键，避免 rdev 输入监控权限问题
            // 可以改为 Consecutive { key: " ", count: 3 } 启用连续空格触发
            full_mode: Hotkey::Combination {
                modifiers: vec!["Control".to_string()],
                key: "j".to_string(),
            },
        }
    }
}

/// 热键类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Hotkey {
    /// 组合键 (如 Cmd+T)
    Combination {
        /// 修饰键列表 (Meta, Control, Alt, Shift)
        modifiers: Vec<String>,
        /// 主键
        key: String,
    },
    /// 连续按键 (如 连续 3 次空格)
    Consecutive {
        /// 按键
        key: String,
        /// 按键次数
        count: u8,
    },
}

impl Hotkey {
    /// 验证选中模式热键是否有效（必须包含修饰键）
    pub fn validate_for_selected_mode(&self) -> bool {
        match self {
            Hotkey::Combination { modifiers, .. } => !modifiers.is_empty(),
            Hotkey::Consecutive { .. } => false, // 选中模式不支持连续按键
        }
    }

    /// 格式化热键显示
    pub fn format(&self) -> String {
        match self {
            Hotkey::Combination { modifiers, key } => {
                let mod_str = modifiers
                    .iter()
                    .map(|m| match m.as_str() {
                        "Meta" => "Cmd",
                        "Control" => "Ctrl",
                        "Alt" => "Option",
                        other => other,
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                format!("{} + {}", mod_str, key.to_uppercase())
            }
            Hotkey::Consecutive { key, count } => {
                let key_name = if key == " " { "Space" } else { key };
                format!("{} × {}", key_name.to_uppercase(), count)
            }
        }
    }
}

/// 语言配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// 当前目标语言
    pub current_target: String,
    /// 常用语言列表
    pub favorite_languages: Vec<Language>,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            current_target: "en-US".to_string(),
            favorite_languages: vec![
                Language {
                    code: "en-US".to_string(),
                    name: "English".to_string(),
                },
                Language {
                    code: "zh-CN".to_string(),
                    name: "简体中文".to_string(),
                },
                Language {
                    code: "ja-JP".to_string(),
                    name: "日本語".to_string(),
                },
                Language {
                    code: "ko-KR".to_string(),
                    name: "한국어".to_string(),
                },
                Language {
                    code: "fr-FR".to_string(),
                    name: "Français".to_string(),
                },
                Language {
                    code: "es-ES".to_string(),
                    name: "Español".to_string(),
                },
            ],
        }
    }
}

/// 语言信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    /// 语言代码 (如 en-US)
    pub code: String,
    /// 语言名称 (如 English)
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_format() {
        let hotkey = Hotkey::Combination {
            modifiers: vec!["Meta".to_string(), "Shift".to_string()],
            key: "t".to_string(),
        };
        assert_eq!(hotkey.format(), "Cmd + Shift + T");

        let hotkey = Hotkey::Consecutive {
            key: " ".to_string(),
            count: 3,
        };
        assert_eq!(hotkey.format(), "SPACE × 3");
    }

    #[test]
    fn test_hotkey_validation() {
        let valid = Hotkey::Combination {
            modifiers: vec!["Meta".to_string()],
            key: "t".to_string(),
        };
        assert!(valid.validate_for_selected_mode());

        let invalid = Hotkey::Combination {
            modifiers: vec![],
            key: "t".to_string(),
        };
        assert!(!invalid.validate_for_selected_mode());

        let consecutive = Hotkey::Consecutive {
            key: " ".to_string(),
            count: 3,
        };
        assert!(!consecutive.validate_for_selected_mode());
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.llm.model, "gpt-4o-mini");
        assert_eq!(config.history_limit, 500);
        assert_eq!(config.language.current_target, "en-US");
    }
}
