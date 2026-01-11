//! 文本处理模块
//! 处理剪贴板操作、键盘模拟和翻译流程
//! 
//! 支持平台:
//! - macOS: 使用 AppleScript (osascript) 模拟键盘操作
//! - Windows: 使用 enigo 库模拟键盘操作

use crate::error::{AppError, Result};
use arboard::Clipboard;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info};

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "windows")]
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

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
        let backup_clone = backup.clone();
        *self.clipboard_backup.write().await = backup;
        
        // 模拟 Cmd+C 复制选中文本
        self.copy().await?;
        
        // 等待剪贴板更新
        sleep(Duration::from_millis(150)).await;
        
        // 获取剪贴板内容
        let text = self.get_clipboard().await?;
        
        // 验证剪贴板内容是否已更新（非空且与备份不同）
        if text.is_empty() {
            return Err(AppError::Clipboard("没有选中文本或复制失败".to_string()));
        }
        
        // 如果剪贴板内容与备份相同，可能复制失败
        if let Some(ref bak) = backup_clone {
            if &text == bak {
                debug!("Clipboard unchanged, copy may have failed");
                // 仍然返回文本，让调用者决定
            }
        }
        
        debug!("Got selected text: {} chars", text.len());
        Ok(text)
    }

    /// 全文模式 - 获取输入框全部文本
    /// 模拟 Cmd+A 全选，然后 Cmd+C 复制
    pub async fn translate_full(&self) -> Result<String> {
        info!("Getting full text");
        
        // 备份当前剪贴板
        let backup = self.get_clipboard().await.ok();
        let backup_clone = backup.clone();
        *self.clipboard_backup.write().await = backup;
        
        // 清空剪贴板，用于检测复制是否成功
        self.set_clipboard("").await.ok();
        
        // 模拟 Cmd+A 全选
        self.select_all().await?;
        
        // 等待一下
        sleep(Duration::from_millis(100)).await;
        
        // 模拟 Cmd+C 复制
        self.copy().await?;
        
        // 等待剪贴板更新
        sleep(Duration::from_millis(200)).await;
        
        // 获取剪贴板内容
        let text = self.get_clipboard().await?;
        
        // 验证复制是否成功
        if text.is_empty() {
            // 恢复备份
            if let Some(ref bak) = backup_clone {
                self.set_clipboard(bak).await.ok();
            }
            return Err(AppError::Clipboard("全选或复制失败，没有获取到文本".to_string()));
        }
        
        debug!("Got full text: {} chars", text.len());
        Ok(text)
    }

    /// 删除当前选中的文本（模拟 Delete/Backspace）
    pub async fn delete_selection(&self) -> Result<()> {
        debug!("Deleting selected text");
        self.delete_key().await?;
        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 流式输入文本（逐字打出效果）
    pub async fn type_text(&self, text: &str) -> Result<()> {
        debug!("Typing text: {} chars", text.len());
        
        // 使用剪贴板方式输入（更可靠）
        // 将文本分块输入，避免一次性输入太多
        for chunk in text.chars().collect::<Vec<_>>().chunks(50) {
            let chunk_str: String = chunk.iter().collect();
            self.set_clipboard(&chunk_str).await?;
            sleep(Duration::from_millis(10)).await;
            self.paste_clipboard().await?;
            sleep(Duration::from_millis(20)).await;
        }
        
        Ok(())
    }

    /// 输入单个文本片段（用于流式输出）
    pub async fn type_chunk(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        
        self.set_clipboard(text).await?;
        self.paste_clipboard().await?;
        sleep(Duration::from_millis(10)).await;
        
        Ok(())
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

    /// 模拟全选操作 (Cmd+A / Ctrl+A)
    #[cfg(target_os = "macos")]
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

    /// 模拟全选操作 (Ctrl+A) - Windows
    #[cfg(target_os = "windows")]
    pub async fn select_all(&self) -> Result<()> {
        debug!("Simulating Ctrl+A via enigo");
        
        let result = std::thread::spawn(|| -> Result<()> {
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| AppError::Keyboard(format!("创建键盘模拟器失败: {}", e)))?;
            
            enigo.key(Key::Control, Direction::Press)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            enigo.key(Key::Unicode('a'), Direction::Click)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            enigo.key(Key::Control, Direction::Release)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            
            Ok(())
        }).join().map_err(|_| AppError::Keyboard("键盘模拟线程崩溃".to_string()))??;

        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 模拟复制操作 (Cmd+C) - macOS
    #[cfg(target_os = "macos")]
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

    /// 模拟复制操作 (Ctrl+C) - Windows
    #[cfg(target_os = "windows")]
    pub async fn copy(&self) -> Result<()> {
        debug!("Simulating Ctrl+C via enigo");
        
        std::thread::spawn(|| -> Result<()> {
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| AppError::Keyboard(format!("创建键盘模拟器失败: {}", e)))?;
            
            enigo.key(Key::Control, Direction::Press)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            enigo.key(Key::Unicode('c'), Direction::Click)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            enigo.key(Key::Control, Direction::Release)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            
            Ok(())
        }).join().map_err(|_| AppError::Keyboard("键盘模拟线程崩溃".to_string()))??;

        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 模拟粘贴操作 (Cmd+V) - macOS
    #[cfg(target_os = "macos")]
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

    /// 模拟粘贴操作 (Ctrl+V) - Windows
    #[cfg(target_os = "windows")]
    async fn paste_clipboard(&self) -> Result<()> {
        debug!("Simulating Ctrl+V via enigo");
        
        std::thread::spawn(|| -> Result<()> {
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| AppError::Keyboard(format!("创建键盘模拟器失败: {}", e)))?;
            
            enigo.key(Key::Control, Direction::Press)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            enigo.key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            enigo.key(Key::Control, Direction::Release)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            
            Ok(())
        }).join().map_err(|_| AppError::Keyboard("键盘模拟线程崩溃".to_string()))??;

        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// 模拟删除键 (Backspace) - macOS
    #[cfg(target_os = "macos")]
    async fn delete_key(&self) -> Result<()> {
        debug!("Simulating Delete via AppleScript");
        
        let script = r#"tell application "System Events" to key code 51"#; // 51 = Backspace
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AppError::Keyboard(format!("无法执行 osascript: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("AppleScript Delete failed: {}", stderr);
            return Err(AppError::Permission(
                "键盘模拟失败，请在系统设置 > 隐私与安全性 > 辅助功能中授权本应用".to_string()
            ));
        }

        Ok(())
    }

    /// 模拟删除键 (Backspace) - Windows
    #[cfg(target_os = "windows")]
    async fn delete_key(&self) -> Result<()> {
        debug!("Simulating Delete via enigo");
        
        std::thread::spawn(|| -> Result<()> {
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| AppError::Keyboard(format!("创建键盘模拟器失败: {}", e)))?;
            
            enigo.key(Key::Backspace, Direction::Click)
                .map_err(|e| AppError::Keyboard(format!("按键失败: {}", e)))?;
            
            Ok(())
        }).join().map_err(|_| AppError::Keyboard("键盘模拟线程崩溃".to_string()))??;

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
