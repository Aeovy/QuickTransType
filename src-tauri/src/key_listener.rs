//! 键盘监听模块
//! 使用 rdev 监听原始键盘输入，用于检测连续按键触发全文翻译

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// 连续按键配置
#[derive(Debug, Clone)]
pub struct ConsecutiveKeyConfig {
    /// 目标按键
    pub key: String,
    /// 需要的按键次数
    pub count: u8,
    /// 按键间隔阈值（毫秒）
    pub interval_ms: u64,
}

impl Default for ConsecutiveKeyConfig {
    fn default() -> Self {
        Self {
            key: " ".to_string(), // 空格
            count: 3,
            interval_ms: 300,
        }
    }
}

/// 键盘监听器
pub struct KeyListener {
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 触发事件发送器
    trigger_tx: Option<mpsc::Sender<()>>,
}

impl KeyListener {
    /// 创建新的键盘监听器
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            trigger_tx: None,
        }
    }

    /// 启动监听器
    /// 返回一个接收器，当检测到连续按键时会收到通知
    pub fn start(&mut self, config: ConsecutiveKeyConfig) -> mpsc::Receiver<()> {
        let (tx, rx) = mpsc::channel(10);
        self.trigger_tx = Some(tx.clone());
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let target_key = config.key.clone();
        let target_count = config.count;
        let interval = Duration::from_millis(config.interval_ms);

        info!(
            "Starting key listener for consecutive key: '{}' x {}",
            target_key, target_count
        );

        // 在单独的线程中运行 rdev 监听器
        std::thread::spawn(move || {
            let mut last_press_time: Option<Instant> = None;
            let mut press_count: u8 = 0;
            let tx = tx;

            let callback = move |event: rdev::Event| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }

                if let rdev::EventType::KeyPress(key) = event.event_type {
                    let key_str = key_to_string(key);

                    if key_str == target_key {
                        let now = Instant::now();

                        // 检查是否在时间间隔内
                        if let Some(last) = last_press_time {
                            if now.duration_since(last) <= interval {
                                press_count += 1;
                                debug!(
                                    "Consecutive key press detected: {} (count: {})",
                                    key_str, press_count
                                );
                            } else {
                                // 超时，重新计数
                                press_count = 1;
                                debug!("Key press timeout, resetting count");
                            }
                        } else {
                            press_count = 1;
                        }

                        last_press_time = Some(now);

                        // 检查是否达到目标次数
                        if press_count >= target_count {
                            info!("Consecutive key trigger activated!");
                            press_count = 0;
                            last_press_time = None;

                            // 发送触发信号
                            if let Err(e) = tx.blocking_send(()) {
                                error!("Failed to send trigger signal: {}", e);
                            }
                        }
                    } else {
                        // 按了其他键，重置计数
                        if press_count > 0 {
                            debug!("Different key pressed, resetting count");
                        }
                        press_count = 0;
                        last_press_time = None;
                    }
                }
            };

            // 启动 rdev 监听
            // 注意：macOS 需要"输入监控"权限，否则会失败
            info!("Starting rdev listener (requires Input Monitoring permission on macOS)");
            match rdev::listen(callback) {
                Ok(_) => info!("rdev listener stopped normally"),
                Err(e) => {
                    error!("Failed to start key listener: {:?}", e);
                    error!("On macOS, please grant Input Monitoring permission in:");
                    error!("System Settings > Privacy & Security > Input Monitoring");
                }
            }
        });

        rx
    }

    /// 停止监听器
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Key listener stopped");
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for KeyListener {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 rdev::Key 转换为字符串
fn key_to_string(key: rdev::Key) -> String {
    match key {
        rdev::Key::Space => " ".to_string(),
        rdev::Key::Return => "Enter".to_string(),
        rdev::Key::Tab => "Tab".to_string(),
        rdev::Key::Backspace => "Backspace".to_string(),
        rdev::Key::Escape => "Escape".to_string(),
        rdev::Key::KeyA => "a".to_string(),
        rdev::Key::KeyB => "b".to_string(),
        rdev::Key::KeyC => "c".to_string(),
        rdev::Key::KeyD => "d".to_string(),
        rdev::Key::KeyE => "e".to_string(),
        rdev::Key::KeyF => "f".to_string(),
        rdev::Key::KeyG => "g".to_string(),
        rdev::Key::KeyH => "h".to_string(),
        rdev::Key::KeyI => "i".to_string(),
        rdev::Key::KeyJ => "j".to_string(),
        rdev::Key::KeyK => "k".to_string(),
        rdev::Key::KeyL => "l".to_string(),
        rdev::Key::KeyM => "m".to_string(),
        rdev::Key::KeyN => "n".to_string(),
        rdev::Key::KeyO => "o".to_string(),
        rdev::Key::KeyP => "p".to_string(),
        rdev::Key::KeyQ => "q".to_string(),
        rdev::Key::KeyR => "r".to_string(),
        rdev::Key::KeyS => "s".to_string(),
        rdev::Key::KeyT => "t".to_string(),
        rdev::Key::KeyU => "u".to_string(),
        rdev::Key::KeyV => "v".to_string(),
        rdev::Key::KeyW => "w".to_string(),
        rdev::Key::KeyX => "x".to_string(),
        rdev::Key::KeyY => "y".to_string(),
        rdev::Key::KeyZ => "z".to_string(),
        rdev::Key::Num0 => "0".to_string(),
        rdev::Key::Num1 => "1".to_string(),
        rdev::Key::Num2 => "2".to_string(),
        rdev::Key::Num3 => "3".to_string(),
        rdev::Key::Num4 => "4".to_string(),
        rdev::Key::Num5 => "5".to_string(),
        rdev::Key::Num6 => "6".to_string(),
        rdev::Key::Num7 => "7".to_string(),
        rdev::Key::Num8 => "8".to_string(),
        rdev::Key::Num9 => "9".to_string(),
        _ => format!("{:?}", key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_string() {
        assert_eq!(key_to_string(rdev::Key::Space), " ");
        assert_eq!(key_to_string(rdev::Key::KeyA), "a");
        assert_eq!(key_to_string(rdev::Key::Return), "Enter");
    }

    #[test]
    fn test_consecutive_key_config_default() {
        let config = ConsecutiveKeyConfig::default();
        assert_eq!(config.key, " ");
        assert_eq!(config.count, 3);
        assert_eq!(config.interval_ms, 300);
    }
}
