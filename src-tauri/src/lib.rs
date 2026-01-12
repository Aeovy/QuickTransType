//! QuickTransType - AI 驱动的翻译助手
//!
//! 一个基于 Tauri 的 macOS 翻译应用，支持全局热键触发翻译

pub mod config;
pub mod database;
pub mod error;
pub mod hotkey;
pub mod key_listener;
pub mod llm;
pub mod text_handler;

mod commands;
mod state;

use config::Hotkey;
use key_listener::{ConsecutiveKeyConfig, KeyListener};
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 构建托盘菜单
pub(crate) async fn build_tray_menu(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
) -> Result<tauri::menu::Menu<tauri::Wry>, String> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};

    let config = state.config.read().await;
    let current_target = config.language.current_target.clone();
    let is_enabled = *state.is_enabled.read().await;

    info!("构建托盘菜单，当前目标语言: {}", current_target);
    info!("当前启用状态: {}", is_enabled);

    // 构建语言子菜单 - 使用普通MenuItem而非CheckMenuItem避免状态残留
    let mut lang_submenu = SubmenuBuilder::new(app, "切换目标语言");
    for lang in &config.language.favorite_languages {
        let is_current = lang.code == current_target;
        // 使用系统标准的勾选标记
        let label = if is_current {
            format!("✓ {}", lang.name)
        } else {
            format!("  {}", lang.name) // 添加空格保持对齐
        };
        info!(
            "  语言项: {} ({}), 是否当前: {}",
            lang.name, lang.code, is_current
        );
        let item = MenuItemBuilder::with_id(&format!("lang_{}", lang.code), label)
            .build(app)
            .map_err(|e| e.to_string())?;
        lang_submenu = lang_submenu.item(&item);
    }
    let lang_menu = lang_submenu.build().map_err(|e| e.to_string())?;

    let toggle_label = if is_enabled {
        "✓ 已启用"
    } else {
        "  已暂停"
    };
    let toggle = MenuItemBuilder::with_id("toggle", toggle_label)
        .build(app)
        .map_err(|e| e.to_string())?;
    let settings = MenuItemBuilder::with_id("settings", "打开设置")
        .build(app)
        .map_err(|e| e.to_string())?;
    let quit = MenuItemBuilder::with_id("quit", "退出")
        .build(app)
        .map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(app)
        .item(&lang_menu)
        .separator()
        .item(&toggle)
        .separator()
        .item(&settings)
        .separator()
        .item(&quit)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(menu)
}

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
fn register_global_shortcuts(
    app: &tauri::App,
    state: &Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = tauri::async_runtime::block_on(async { state.get_config().await });

    // 注册选中翻译热键
    if let Some(shortcut) = hotkey_to_shortcut(&config.hotkey.selected_mode) {
        let app_handle = app.handle().clone();

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
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

        info!(
            "Registered selected mode hotkey: {:?}",
            config.hotkey.selected_mode
        );
    }

    // 注册全文翻译热键
    match &config.hotkey.full_mode {
        Hotkey::Combination { .. } => {
            // 组合键模式
            if let Some(shortcut) = hotkey_to_shortcut(&config.hotkey.full_mode) {
                let app_handle = app.handle().clone();

                app.global_shortcut()
                    .on_shortcut(shortcut, move |_app, _shortcut, event| {
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
        }
        Hotkey::Consecutive { key, count } => {
            // 连续按键模式 - 使用 rdev 监听器
            let app_handle = app.handle().clone();
            let key_config = ConsecutiveKeyConfig {
                key: key.clone(),
                count: *count,
                interval_ms: 300,
            };

            start_consecutive_key_listener(app_handle, key_config);
            info!(
                "Registered full mode consecutive key: '{}' x {}",
                key, count
            );
        }
    }

    Ok(())
}

/// 启动连续按键监听器
fn start_consecutive_key_listener(app_handle: tauri::AppHandle, config: ConsecutiveKeyConfig) {
    std::thread::spawn(move || {
        let mut listener = KeyListener::new();
        let mut rx = listener.start(config);

        // 使用 tokio 运行时处理接收到的触发信号
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async {
            while let Some(()) = rx.recv().await {
                debug!("Consecutive key trigger received");
                let handle = app_handle.clone();

                if let Err(e) = trigger_translation(&handle, "full").await {
                    error!("Full translation failed: {}", e);
                }
            }
        });
    });
}

/// 触发翻译（流式传输版本）
async fn trigger_translation(
    app: &tauri::AppHandle,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Triggering {} translation", mode);

    let state = app.state::<Arc<AppState>>();

    // 检查是否启用
    let is_enabled = *state.is_enabled.read().await;
    if !is_enabled {
        debug!("Translation is disabled, skipping");
        return Ok(());
    }

    let config = state.get_config().await;

    // 获取文本
    let text = if mode == "selected" {
        // 选中翻译：复制当前选中的文本
        match state.text_handler.translate_selected().await {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to get selected text: {}", e);
                return Ok(()); // 静默失败，不做任何操作
            }
        }
    } else {
        // 全文翻译：选中全部并复制
        match state.text_handler.translate_full().await {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to get full text: {}", e);
                return Ok(()); // 静默失败，不做任何操作
            }
        }
    };

    if text.is_empty() {
        warn!("No text to translate");
        return Ok(());
    }

    let original_text = text.clone();
    let char_count = text.len();
    info!("Translating {} characters", char_count);

    let llm_client = state.get_llm_client().await;
    let target_lang = config.language.current_target.clone();
    let use_stream = config.llm.stream_mode;

    let translated_text: String;
    let mut completion_tokens: Option<u32> = None;
    let mut duration_ms: u64 = 0;
    let mut tokens_per_second: Option<f64> = None;

    if use_stream {
        // 流式模式：删除选中的文本，逐字输入
        state
            .text_handler
            .delete_selection()
            .await
            .map_err(|e| format!("Failed to delete selection: {}", e))?;

        let mut stream = llm_client
            .translate_stream(&config.llm, &text, &target_lang)
            .await
            .map_err(|e| format!("Translation API error: {}", e))?;

        let mut result_text = String::new();

        // 处理流式响应
        use crate::llm::StreamEvent;
        while let Some(event) = stream.recv().await {
            match event {
                StreamEvent::Delta(delta) => {
                    // 流式输入每个增量文本
                    if let Err(e) = state.text_handler.type_chunk(&delta).await {
                        error!("Failed to type chunk: {}", e);
                    }
                    result_text.push_str(&delta);
                }
                StreamEvent::Done {
                    completion_tokens: tokens,
                    duration_ms: dur,
                } => {
                    completion_tokens = tokens;
                    duration_ms = dur;
                    debug!(
                        "Stream completed: {} tokens, {}ms",
                        tokens.unwrap_or(0),
                        dur
                    );
                }
                StreamEvent::Error(err) => {
                    error!("Stream error: {}", err);
                    // 发生错误时，尝试恢复原文
                    if let Some(backup) = state.text_handler.get_backup().await {
                        state.text_handler.paste(&backup).await.ok();
                    }
                    return Err(err.into());
                }
            }
        }

        translated_text = result_text;
        tokens_per_second = completion_tokens.map(|t| {
            if duration_ms > 0 {
                (t as f64) / (duration_ms as f64 / 1000.0)
            } else {
                0.0
            }
        });
    } else {
        // 非流式模式：等待完成后一次性替换
        let result = llm_client
            .translate(&config.llm, &text, &target_lang)
            .await
            .map_err(|e| format!("Translation API error: {}", e))?;

        translated_text = result.translated_text;
        completion_tokens = result.completion_tokens;
        duration_ms = result.duration_ms;
        tokens_per_second = result.tokens_per_second;

        // 替换选中的文本
        state
            .text_handler
            .paste(&translated_text)
            .await
            .map_err(|e| format!("Failed to paste translation: {}", e))?;
    }

    info!(
        "Translation completed: {} chars -> {} chars, {} tokens, {}ms, {:.1} tokens/s",
        original_text.len(),
        translated_text.len(),
        completion_tokens.unwrap_or(0),
        duration_ms,
        tokens_per_second.unwrap_or(0.0)
    );

    // 保存翻译历史
    if let Err(e) = state
        .database
        .insert_translation(
            &original_text,
            &translated_text,
            None, // source_lang 自动检测
            &target_lang,
            mode,
        )
        .await
    {
        error!("Failed to save translation history: {}", e);
    }

    // 保存性能指标（使用实际的操作模式）
    if let Err(e) = state
        .database
        .insert_metric(
            mode, // "selected" 或 "full"
            duration_ms as i64,
            true,
            None,
            char_count as i64,
            completion_tokens,
            tokens_per_second,
        )
        .await
    {
        error!("Failed to save performance metric: {}", e);
    }

    Ok(())
}

/// 初始化日志系统
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "QuickTransType=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// 应用程序入口
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    info!("Starting QuickTransType...");

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
            let state =
                tauri::async_runtime::block_on(async { AppState::new().await }).map_err(|e| {
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
                use tauri::tray::TrayIconBuilder;

                // 构建菜单
                let menu = tauri::async_runtime::block_on(async {
                    build_tray_menu(&app.handle(), &state).await
                })?;

                let app_state = state.clone();
                let app_handle = app.handle().clone();
                let _tray = TrayIconBuilder::with_id("main")
                    .icon(app.default_window_icon().cloned().expect("no icon"))
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(move |app, event| {
                        let event_id = event.id().as_ref();

                        // 处理语言切换
                        if let Some(lang_code) = event_id.strip_prefix("lang_") {
                            info!("Switching language to: {}", lang_code);
                            let state = app_state.clone();
                            let lang = lang_code.to_string();
                            let app_handle_clone = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let mut config = state.get_config().await;
                                info!(
                                    "托盘点击前，当前目标语言: {}",
                                    config.language.current_target
                                );
                                config.language.current_target = lang.clone();
                                info!("准备保存新的目标语言: {}", lang);
                                if let Err(e) = state.save_config(&config).await {
                                    error!("Failed to save language config: {}", e);
                                    return;
                                }
                                info!("配置已保存");

                                // 等待一小段时间确保配置完全保存
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                                // 重新构建托盘菜单
                                if let Ok(new_menu) =
                                    build_tray_menu(&app_handle_clone, &state).await
                                {
                                    if let Some(tray) = app_handle_clone.tray_by_id("main") {
                                        // 先移除旧菜单
                                        if let Err(e) =
                                            tray.set_menu(None::<tauri::menu::Menu<tauri::Wry>>)
                                        {
                                            error!("Failed to remove old tray menu: {}", e);
                                        }
                                        // 等待 macOS 刷新
                                        tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                            .await;
                                        // 设置新菜单
                                        if let Err(e) = tray.set_menu(Some(new_menu)) {
                                            error!("Failed to update tray menu: {}", e);
                                        } else {
                                            info!("Tray menu updated for language: {}", lang);
                                        }
                                    }
                                }

                                // 发送配置更新事件通知前端
                                if let Err(e) = app_handle_clone.emit("config-updated", ()) {
                                    error!("Failed to emit config-updated event: {}", e);
                                }
                            });
                            return;
                        }

                        match event_id {
                            "toggle" => {
                                info!("Toggle translation monitoring");
                                let state = app_state.clone();
                                let app_clone = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    let mut is_enabled = state.is_enabled.write().await;
                                    *is_enabled = !*is_enabled;
                                    let new_status = *is_enabled;
                                    drop(is_enabled);

                                    info!("Translation monitoring toggled to: {}", new_status);

                                    // 更新托盘菜单
                                    tokio::time::sleep(tokio::time::Duration::from_millis(50))
                                        .await;
                                    if let Ok(new_menu) = build_tray_menu(&app_clone, &state).await
                                    {
                                        if let Some(tray) = app_clone.tray_by_id("main") {
                                            let _ = tray
                                                .set_menu(None::<tauri::menu::Menu<tauri::Wry>>);
                                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                                100,
                                            ))
                                            .await;
                                            if let Err(e) = tray.set_menu(Some(new_menu)) {
                                                error!("Failed to update tray menu: {}", e);
                                            }
                                        }
                                    }

                                    // 发送事件通知前端
                                    if let Err(e) =
                                        app_clone.emit("enabled-status-changed", new_status)
                                    {
                                        error!(
                                            "Failed to emit enabled-status-changed event: {}",
                                            e
                                        );
                                    }
                                });
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
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_enabled_status,
            commands::set_enabled_status,
            commands::test_llm_connection,
            commands::get_history,
            commands::clear_history,
            commands::get_performance_stats,
            commands::check_hotkey_conflicts,
            commands::switch_language,
            commands::translate_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
