//! AITyping - AI 驱动的翻译助手
//!
//! 一个基于 Tauri 的 macOS 翻译应用，支持全局热键触发翻译

pub mod config;
pub mod database;
pub mod error;
pub mod hotkey;
pub mod llm;
pub mod text_handler;

mod commands;
mod state;

use config::Hotkey;
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 检查 macOS 辅助功能权限
#[cfg(target_os = "macos")]
fn check_accessibility_permission() -> bool {
    use std::ffi::c_void;
    
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    }
    
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFRelease(cf: *const c_void);
        
        static kCFBooleanTrue: *const c_void;
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
    }
    
    // kAXTrustedCheckOptionPrompt key
    const K_AX_TRUSTED_CHECK_OPTION_PROMPT: &[u8] = b"AXTrustedCheckOptionPrompt\0";
    
    unsafe {
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFStringCreateWithCString(
                alloc: *const c_void,
                c_str: *const u8,
                encoding: u32,
            ) -> *const c_void;
        }
        
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            K_AX_TRUSTED_CHECK_OPTION_PROMPT.as_ptr(),
            0x08000100, // kCFStringEncodingUTF8
        );
        
        let keys = [key];
        let values = [kCFBooleanTrue];
        
        let options = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const _ as *const c_void,
            &kCFTypeDictionaryValueCallBacks as *const _ as *const c_void,
        );
        
        let trusted = AXIsProcessTrustedWithOptions(options);
        
        CFRelease(options);
        CFRelease(key);
        
        trusted
    }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility_permission() -> bool {
    true
}

/// 将配置中的热键转换为 Shortcut
fn hotkey_to_shortcut(hotkey: &Hotkey) -> Option<Shortcut> {
    match hotkey {
        Hotkey::Combination { modifiers, key } => {
            let mut mods = Modifiers::empty();
            for m in modifiers {
                match m.as_str() {
                    "Meta" => mods |= Modifiers::META,
                    "Control" => mods |= Modifiers::CONTROL,
                    "Alt" => mods |= Modifiers::ALT,
                    "Shift" => mods |= Modifiers::SHIFT,
                    _ => {}
                }
            }

            // 解析按键码
            let code = match key.to_lowercase().as_str() {
                "a" => tauri_plugin_global_shortcut::Code::KeyA,
                "b" => tauri_plugin_global_shortcut::Code::KeyB,
                "c" => tauri_plugin_global_shortcut::Code::KeyC,
                "d" => tauri_plugin_global_shortcut::Code::KeyD,
                "e" => tauri_plugin_global_shortcut::Code::KeyE,
                "f" => tauri_plugin_global_shortcut::Code::KeyF,
                "g" => tauri_plugin_global_shortcut::Code::KeyG,
                "h" => tauri_plugin_global_shortcut::Code::KeyH,
                "i" => tauri_plugin_global_shortcut::Code::KeyI,
                "j" => tauri_plugin_global_shortcut::Code::KeyJ,
                "k" => tauri_plugin_global_shortcut::Code::KeyK,
                "l" => tauri_plugin_global_shortcut::Code::KeyL,
                "m" => tauri_plugin_global_shortcut::Code::KeyM,
                "n" => tauri_plugin_global_shortcut::Code::KeyN,
                "o" => tauri_plugin_global_shortcut::Code::KeyO,
                "p" => tauri_plugin_global_shortcut::Code::KeyP,
                "q" => tauri_plugin_global_shortcut::Code::KeyQ,
                "r" => tauri_plugin_global_shortcut::Code::KeyR,
                "s" => tauri_plugin_global_shortcut::Code::KeyS,
                "t" => tauri_plugin_global_shortcut::Code::KeyT,
                "u" => tauri_plugin_global_shortcut::Code::KeyU,
                "v" => tauri_plugin_global_shortcut::Code::KeyV,
                "w" => tauri_plugin_global_shortcut::Code::KeyW,
                "x" => tauri_plugin_global_shortcut::Code::KeyX,
                "y" => tauri_plugin_global_shortcut::Code::KeyY,
                "z" => tauri_plugin_global_shortcut::Code::KeyZ,
                " " => tauri_plugin_global_shortcut::Code::Space,
                "space" => tauri_plugin_global_shortcut::Code::Space,
                _ => {
                    warn!("Unsupported key: {}", key);
                    return None;
                }
            };

            Some(Shortcut::new(Some(mods), code))
        }
        Hotkey::Consecutive { .. } => {
            // 连续按键不使用全局快捷键，需要单独处理
            None
        }
    }
}

/// 注册全局热键
fn register_global_shortcuts(app: &tauri::App, state: &Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let config = tauri::async_runtime::block_on(async { state.get_config().await });

    // 注册选中翻译热键
    if let Some(shortcut) = hotkey_to_shortcut(&config.hotkey.selected_mode) {
        let app_handle = app.handle().clone();
        
        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                debug!("Selected mode hotkey triggered");
                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = trigger_translation(&handle, "selected").await {
                        error!("Translation failed: {}", e);
                    }
                });
            }
        })?;
        
        info!("Registered selected mode hotkey: {:?}", config.hotkey.selected_mode);
    }

    // 注册全文翻译热键（如果是组合键）
    if let Some(shortcut) = hotkey_to_shortcut(&config.hotkey.full_mode) {
        let app_handle = app.handle().clone();
        
        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                debug!("Full mode hotkey triggered");
                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = trigger_translation(&handle, "full").await {
                        error!("Translation failed: {}", e);
                    }
                });
            }
        })?;
        
        info!("Registered full mode hotkey: {:?}", config.hotkey.full_mode);
    }

    Ok(())
}

/// 触发翻译
async fn trigger_translation(app: &tauri::AppHandle, mode: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Triggering {} translation", mode);
    
    let state = app.state::<Arc<AppState>>();
    let config = state.get_config().await;
    
    // 获取文本
    let text = if mode == "selected" {
        // 选中翻译：复制当前选中的文本
        state.text_handler.translate_selected().await
            .map_err(|e| format!("Failed to get selected text: {}", e))?
    } else {
        // 全文翻译：选中全部并复制
        state.text_handler.translate_full().await
            .map_err(|e| format!("Failed to get full text: {}", e))?
    };
    
    if text.is_empty() {
        warn!("No text to translate");
        return Ok(());
    }
    
    info!("Translating {} characters", text.len());
    
    // 调用 LLM 翻译
    let llm_client = state.get_llm_client().await;
    let target_lang = &config.language.current_target;
    
    let result = llm_client.translate(&config.llm, &text, target_lang).await
        .map_err(|e| format!("Translation API error: {}", e))?;
    
    // 粘贴翻译结果
    state.text_handler.paste(&result).await
        .map_err(|e| format!("Failed to paste result: {}", e))?;
    
    info!("Translation completed");
    Ok(())
}

/// 初始化日志系统
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aityping=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// 应用程序入口
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    info!("Starting AITyping...");

    // 检查辅助功能权限
    if !check_accessibility_permission() {
        warn!("辅助功能权限未授权，键盘模拟功能可能无法正常工作");
        warn!("请在 系统设置 > 隐私与安全性 > 辅助功能 中授权本应用");
    } else {
        info!("辅助功能权限已授权");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            info!("Initializing application...");

            // 同步初始化应用状态
            let state = tauri::async_runtime::block_on(async {
                AppState::new().await
            }).map_err(|e| {
                error!("Failed to initialize application state: {}", e);
                e
            })?;
            let state = Arc::new(state);
            app.manage(state.clone());
            info!("Application state initialized");

            // 注册全局热键
            if let Err(e) = register_global_shortcuts(app, &state) {
                error!("Failed to register global shortcuts: {}", e);
            }

            // 设置系统托盘
            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::TrayIconBuilder;

                let toggle = MenuItemBuilder::with_id("toggle", "启用/暂停").build(app)?;
                let settings = MenuItemBuilder::with_id("settings", "打开设置").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

                let menu = MenuBuilder::new(app)
                    .item(&toggle)
                    .separator()
                    .item(&settings)
                    .separator()
                    .item(&quit)
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().cloned().expect("no icon"))
                    .menu(&menu)
                    .menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "toggle" => {
                            info!("Toggle translation monitoring");
                            // TODO: 实现启用/暂停逻辑
                        }
                        "settings" => {
                            info!("Opening settings window");
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            info!("Quitting application");
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::test_llm_connection,
            commands::get_history,
            commands::get_performance_stats,
            commands::check_hotkey_conflicts,
            commands::switch_language,
            commands::translate_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
