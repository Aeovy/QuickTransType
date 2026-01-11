//! 文本处理模块
//! 处理剪贴板操作、键盘模拟和翻译流程

use crate::error::{AppError, Result};
use arboard::Clipboard;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info};

/// 文本处理器
pub struct TextHandler {
    /// 剪贴板备份（用于错误恢复）
    clipboard_backup: Arc<RwLock<Option<String>>>,
}

impl TextHandler {
    /// 创建新的文本处理器
    pub fn new() -> Result<Self> {
        Ok(Self {
            clipboard_backup: Arc::new(RwLock::new(None)),
        })
    }

    /// 选中模式 - 获取选中的文本
    /// 模拟 Cmd+C 复制选中文本，然后返回剪贴板内容
    pub async fn translate_selected(&self) -> Result<String> {
        info!("Getting selected text");
        
        // 备份当前剪贴板
        let backup = self.get_clipboard().await.ok();
        *self.clipboard_backup.write().await = backup;
        
        // 模拟 Cmd+C 复制选中文本
        self.copy().await?;
        
        // 等待剪贴板更新
        sleep(Duration::from_millis(150)).await;
        
        // 获取剪贴板内容
        let text = self.get_clipboard().await?;
        debug!("Got selected text: {} chars", text.len());
        
        Ok(text)
    }

    /// 全文模式 - 获取输入框全部文本
    /// 模拟 Cmd+A 全选，然后 Cmd+C 复制
    pub async fn translate_full(&self) -> Result<String> {
        info!("Getting full text");
        
        // 备份当前剪贴板
        let backup = self.get_clipboard().await.ok();
        *self.clipboard_backup.write().await = backup;
        
        // 模拟 Cmd+A 全选
        self.select_all().await?;
        
        // 等待一下
        sleep(Duration::from_millis(50)).await;
        
        // 模拟 Cmd+C 复制
        self.copy().await?;
        
        // 等待剪贴板更新
        sleep(Duration::from_millis(150)).await;
        
        // 获取剪贴板内容
        let text = self.get_clipboard().await?;
        debug!("Got full text: {} chars", text.len());
        
        Ok(text)
    }

    /// 粘贴文本
    pub async fn paste(&self, text: &str) -> Result<()> {
        info!("Pasting translated text: {} chars", text.len());
        
        // 设置剪贴板内容
        self.set_clipboard(text).await?;        // 等待剪贴板设置完成
        sleep(Duration::from_millis(50)).await;
        
        // 模拟 Cmd+V 粘贴
        self.paste_clipboard().await?;
        
        Ok(())
    }

    /// 获取剪贴板内容
    async fn get_clipboard(&self) -> Result<String> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| AppError::Clipboard(format!("无法访问剪贴板: {}", e)))?;
        
        clipboard
            .get_text()
            .map_err(|e| AppError::Clipboard(format!("无法读取剪贴板: {}", e)))
    }

    /// 设置剪贴板内容
    async fn set_clipboard(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| AppError::Clipboard(format!("无法访问剪贴板: {}", e)))?;
        
        clipboard
            .set_text(text.to_string())
            .map_err(|e| AppError::Clipboard(format!("无法设置剪贴板: {}", e)))
    }

    /// 模拟全选操作 (Cmd+A) - 使用 AppleScript
    pub async fn select_all(&self) -> Result<()> {
        debug!("Simulating Cmd+A via AppleScript");
        
        let script = r#"tell application "System Events" to keystroke "a" using command down"#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AppError::Keyboard(format!("无法执行 osascript: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("AppleScript Cmd+A failed: {}", stderr);
            return Err(AppError::Permission(
                "键盘模拟失败，请在系统设置 > 隐私与安全性 > 辅助功能中授权本应用".to_string()
            ));
        }

        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 模拟复制操作 (Cmd+C) - 使用 AppleScript
    pub async fn copy(&self) -> Result<()> {
        debug!("Simulating Cmd+C via AppleScript");
        
        let script = r#"tell application "System Events" to keystroke "c" using command down"#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AppError::Keyboard(format!("无法执行 osascript: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("AppleScript Cmd+C failed: {}", stderr);
            return Err(AppError::Permission(
                "键盘模拟失败，请在系统设置 > 隐私与安全性 > 辅助功能中授权本应用".to_string()
            ));
        }

        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 模拟粘贴操作 (Cmd+V) - 使用 AppleScript
    async fn paste_clipboard(&self) -> Result<()> {
        debug!("Simulating Cmd+V via AppleScript");
        
        let script = r#"tell application "System Events" to keystroke "v" using command down"#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AppError::Keyboard(format!("无法执行 osascript: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("AppleScript Cmd+V failed: {}", stderr);
            return Err(AppError::Permission(
                "键盘模拟失败，请在系统设置 > 隐私与安全性 > 辅助功能中授权本应用".to_string()
            ));
        }

        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 获取剪贴板备份
    pub async fn get_backup(&self) -> Option<String> {
        self.clipboard_backup.read().await.clone()
    }

    /// 清除剪贴板备份
    pub async fn clear_backup(&self) {
        *self.clipboard_backup.write().await = None;
    }
}

impl Default for TextHandler {
    fn default() -> Self {
        Self::new().expect("Failed to create TextHandler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_handler_creation() {
        let handler = TextHandler::new();
        assert!(handler.is_ok());
    }
}
