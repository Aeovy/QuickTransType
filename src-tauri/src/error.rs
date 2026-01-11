//! 错误处理模块
//! 定义应用程序的统一错误类型

use thiserror::Error;

/// 应用程序统一错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 配置相关错误
    #[error("配置错误: {0}")]
    Config(String),

    /// LLM API 相关错误
    #[error("LLM API 错误: {0}")]
    LlmApi(String),

    /// 网络请求错误
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    /// 剪贴板操作错误
    #[error("剪贴板操作失败: {0}")]
    Clipboard(String),

    /// 热键相关错误
    #[error("热键错误: {0}")]
    Hotkey(String),

    /// 权限不足
    #[error("权限不足: {0}")]
    Permission(String),

    /// 键盘模拟错误
    #[error("键盘模拟失败: {0}")]
    Keyboard(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 通用错误
    #[error("{0}")]
    Other(String),
}

/// 将 AppError 转换为可序列化的字符串，用于 Tauri IPC
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

/// 从 anyhow::Error 转换
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Other(error.to_string())
    }
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, AppError>;

/// 将 Option 转换为 Result
pub trait OptionExt<T> {
    fn ok_or_config(self, msg: &str) -> Result<T>;
    fn ok_or_other(self, msg: &str) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_config(self, msg: &str) -> Result<T> {
        self.ok_or_else(|| AppError::Config(msg.to_string()))
    }

    fn ok_or_other(self, msg: &str) -> Result<T> {
        self.ok_or_else(|| AppError::Other(msg.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Config("test error".to_string());
        assert_eq!(err.to_string(), "配置错误: test error");
    }

    #[test]
    fn test_error_to_string() {
        let err = AppError::LlmApi("API key invalid".to_string());
        let s: String = err.into();
        assert!(s.contains("API key invalid"));
    }
}
