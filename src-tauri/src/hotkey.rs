//! 热键模块
//! 处理全局热键监听和冲突检测

use crate::config::Hotkey;
use crate::error::{AppError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// 热键管理器
pub struct HotkeyManager {
    /// 连续按键检测器状态
    consecutive_state: Arc<RwLock<ConsecutiveState>>,
}

/// 连续按键检测状态
struct ConsecutiveState {
    /// 上一个按键
    last_key: Option<String>,
    /// 连续次数
    count: u8,
    /// 上次按键时间
    last_time: std::time::Instant,
}

impl Default for ConsecutiveState {
    fn default() -> Self {
        Self {
            last_key: None,
            count: 0,
            last_time: std::time::Instant::now(),
        }
    }
}

impl HotkeyManager {
    /// 创建新的热键管理器
    pub fn new() -> Self {
        Self {
            consecutive_state: Arc::new(RwLock::new(ConsecutiveState::default())),
        }
    }

    /// 检查按键是否触发连续按键热键
    pub async fn check_consecutive(&self, key: &str, target: &Hotkey) -> bool {
        if let Hotkey::Consecutive { key: target_key, count: target_count } = target {
            let mut state = self.consecutive_state.write().await;
            let now = std::time::Instant::now();
            
            // 检查是否是同一个键且在 500ms 内
            if Some(key.to_string()) == state.last_key 
                && now.duration_since(state.last_time).as_millis() < 500 
            {
                state.count += 1;
                state.last_time = now;
                
                if state.count >= *target_count && key == target_key {
                    // 重置状态
                    state.count = 0;
                    state.last_key = None;
                    debug!("Consecutive hotkey triggered: {} x {}", key, target_count);
                    return true;
                }
            } else {
                // 重置状态
                state.last_key = Some(key.to_string());
                state.count = 1;
                state.last_time = now;
            }
        }
        false
    }

    /// 重置连续按键状态
    pub async fn reset_consecutive(&self) {
        let mut state = self.consecutive_state.write().await;
        *state = ConsecutiveState::default();
    }

    /// 检测系统热键冲突
    pub fn check_system_conflicts(hotkey: &Hotkey) -> Vec<String> {
        let mut conflicts = Vec::new();

        if let Hotkey::Combination { modifiers, key } = hotkey {
            // 读取 macOS 系统快捷键配置
            let system_hotkeys = Self::get_system_hotkeys();

            for (name, sys_mods, sys_key) in system_hotkeys {
                if Self::hotkeys_match(modifiers, key, &sys_mods, &sys_key) {
                    conflicts.push(name);
                }
            }
        }

        conflicts
    }

    /// 获取系统热键列表
    fn get_system_hotkeys() -> Vec<(String, Vec<String>, String)> {
        let mut hotkeys = Vec::new();

        // 尝试读取系统配置
        if let Some(home) = dirs::home_dir() {
            let plist_path = home.join("Library/Preferences/com.apple.symbolichotkeys.plist");
            if let Ok(parsed_hotkeys) = Self::parse_symbolic_hotkeys(&plist_path) {
                hotkeys.extend(parsed_hotkeys);
            }
        }

        // 添加常见的系统热键
        hotkeys.extend(vec![
            ("Spotlight".to_string(), vec!["Meta".to_string()], " ".to_string()),
            ("Spotlight".to_string(), vec!["Meta".to_string(), "Alt".to_string()], " ".to_string()),
            ("截图".to_string(), vec!["Meta".to_string(), "Shift".to_string()], "3".to_string()),
            ("截图".to_string(), vec!["Meta".to_string(), "Shift".to_string()], "4".to_string()),
            ("截图".to_string(), vec!["Meta".to_string(), "Shift".to_string()], "5".to_string()),
        ]);

        hotkeys
    }

    /// 解析系统热键配置文件
    fn parse_symbolic_hotkeys(path: &PathBuf) -> Result<Vec<(String, Vec<String>, String)>> {
        let file = std::fs::File::open(path)?;
        let plist: plist::Value = plist::from_reader(file)
            .map_err(|e| AppError::Config(format!("解析 plist 失败: {}", e)))?;

        let mut hotkeys = Vec::new();

        // 解析 plist 结构
        if let Some(dict) = plist.as_dictionary() {
            if let Some(apple_hotkeys) = dict.get("AppleSymbolicHotKeys") {
                if let Some(hotkey_dict) = apple_hotkeys.as_dictionary() {
                    for (id, value) in hotkey_dict {
                        if let Some(hotkey_info) = Self::parse_hotkey_entry(id, value) {
                            hotkeys.push(hotkey_info);
                        }
                    }
                }
            }
        }

        Ok(hotkeys)
    }

    /// 解析单个热键条目
    fn parse_hotkey_entry(id: &str, value: &plist::Value) -> Option<(String, Vec<String>, String)> {
        let dict = value.as_dictionary()?;
        
        // 检查是否启用
        let enabled = dict.get("enabled")?.as_boolean()?;
        if !enabled {
            return None;
        }

        let value_dict = dict.get("value")?.as_dictionary()?;
        let parameters = value_dict.get("parameters")?.as_array()?;

        if parameters.len() < 3 {
            return None;
        }

        // 解析修饰键
        let modifier_flags = parameters.get(2)?.as_unsigned_integer()? as u32;
        let mut modifiers = Vec::new();
        
        if modifier_flags & (1 << 17) != 0 { modifiers.push("Shift".to_string()); }
        if modifier_flags & (1 << 18) != 0 { modifiers.push("Control".to_string()); }
        if modifier_flags & (1 << 19) != 0 { modifiers.push("Alt".to_string()); }
        if modifier_flags & (1 << 20) != 0 { modifiers.push("Meta".to_string()); }

        // 解析按键
        let key_code = parameters.get(1)?.as_unsigned_integer()? as u32;
        let key = Self::keycode_to_string(key_code);

        let name = format!("系统快捷键 #{}", id);
        Some((name, modifiers, key))
    }

    /// 将 keycode 转换为字符串
    fn keycode_to_string(keycode: u32) -> String {
        match keycode {
            49 => " ".to_string(),    // Space
            36 => "Return".to_string(),
            48 => "Tab".to_string(),
            51 => "Delete".to_string(),
            53 => "Escape".to_string(),
            0 => "a".to_string(),
            1 => "s".to_string(),
            2 => "d".to_string(),
            3 => "f".to_string(),
            4 => "h".to_string(),
            5 => "g".to_string(),
            6 => "z".to_string(),
            7 => "x".to_string(),
            8 => "c".to_string(),
            9 => "v".to_string(),
            11 => "b".to_string(),
            12 => "q".to_string(),
            13 => "w".to_string(),
            14 => "e".to_string(),
            15 => "r".to_string(),
            16 => "y".to_string(),
            17 => "t".to_string(),
            _ => format!("key_{}", keycode),
        }
    }

    /// 比较两个热键是否匹配
    fn hotkeys_match(mods1: &[String], key1: &str, mods2: &[String], key2: &str) -> bool {
        if key1.to_lowercase() != key2.to_lowercase() {
            return false;
        }

        let mut sorted1: Vec<_> = mods1.iter().map(|s| s.to_lowercase()).collect();
        let mut sorted2: Vec<_> = mods2.iter().map(|s| s.to_lowercase()).collect();
        sorted1.sort();
        sorted2.sort();

        sorted1 == sorted2
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkeys_match() {
        assert!(HotkeyManager::hotkeys_match(
            &["Meta".to_string(), "Shift".to_string()],
            "t",
            &["Shift".to_string(), "Meta".to_string()],
            "T"
        ));

        assert!(!HotkeyManager::hotkeys_match(
            &["Meta".to_string()],
            "t",
            &["Control".to_string()],
            "t"
        ));
    }

    #[tokio::test]
    async fn test_consecutive_detection() {
        let manager = HotkeyManager::new();
        let target = Hotkey::Consecutive {
            key: " ".to_string(),
            count: 3,
        };

        // 第一次按键
        assert!(!manager.check_consecutive(" ", &target).await);
        // 第二次按键
        assert!(!manager.check_consecutive(" ", &target).await);
        // 第三次按键 - 应该触发
        assert!(manager.check_consecutive(" ", &target).await);
    }

    #[test]
    fn test_check_conflicts() {
        let hotkey = Hotkey::Combination {
            modifiers: vec!["Meta".to_string()],
            key: " ".to_string(),
        };
        let conflicts = HotkeyManager::check_system_conflicts(&hotkey);
        // Spotlight 使用 Cmd+Space，应该检测到冲突
        assert!(!conflicts.is_empty());
    }
}
