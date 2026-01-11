//! 应用状态模块
//! 管理全局状态和共享资源

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::Result;
use crate::hotkey::HotkeyManager;
use crate::llm::LLMClient;
use crate::text_handler::TextHandler;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 应用程序全局状态
pub struct AppState {
    /// 配置
    pub config: Arc<RwLock<AppConfig>>,
    /// 数据库
    pub database: Arc<Database>,
    /// LLM 客户端
    pub llm_client: Arc<LLMClient>,
    /// 热键管理器
    pub hotkey_manager: Arc<HotkeyManager>,
    /// 文本处理器
    pub text_handler: Arc<TextHandler>,
    /// 是否启用翻译监听
    pub is_enabled: Arc<RwLock<bool>>,
    /// 配置文件路径
    config_path: PathBuf,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> Result<Self> {
        info!("Initializing application state...");

        // 获取配置路径
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::error::AppError::Config("无法获取配置目录".to_string()))?;
        let config_path = config_dir.join("AITyping").join("config.json");

        // 加载或创建配置
        let config = Self::load_config(&config_path).await;
        debug!("Config loaded: {:?}", config.llm.model);

        // 初始化数据库
        let database = Database::new().await?;
        info!("Database initialized");

        // 初始化 LLM 客户端
        let llm_client = LLMClient::new()?;
        debug!("LLM client created");

        // 初始化热键管理器
        let hotkey_manager = HotkeyManager::new();
        debug!("Hotkey manager created");

        // 初始化文本处理器
        let text_handler = TextHandler::new()?;
        debug!("Text handler created");

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            database: Arc::new(database),
            llm_client: Arc::new(llm_client),
            hotkey_manager: Arc::new(hotkey_manager),
            text_handler: Arc::new(text_handler),
            is_enabled: Arc::new(RwLock::new(true)),
            config_path,
        })
    }

    /// 加载配置文件
    async fn load_config(path: &PathBuf) -> AppConfig {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            tracing::warn!("Failed to parse config: {}, using defaults", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read config: {}, using defaults", e);
                }
            }
        }
        AppConfig::default()
    }

    /// 保存配置文件
    pub async fn save_config(&self, config: &AppConfig) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;

        // 更新内存中的配置
        *self.config.write().await = config.clone();

        info!("Config saved to {:?}", self.config_path);
        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// 设置启用状态
    pub async fn set_enabled(&self, enabled: bool) {
        *self.is_enabled.write().await = enabled;
        info!("Translation monitoring {}", if enabled { "enabled" } else { "disabled" });
    }

    /// 检查是否启用
    pub async fn is_enabled(&self) -> bool {
        *self.is_enabled.read().await
    }

    /// 获取 LLM 客户端
    pub async fn get_llm_client(&self) -> Arc<LLMClient> {
        self.llm_client.clone()
    }
}
