mod auto_connect;
mod commands;
mod ras;

use commands::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .setup(|app| {
            // 从磁盘加载持久化配置
            {
                let state = app.state::<AppState>();
                state.load_persisted_config(app.handle());
                state.add_log(
                    ras::types::LogLevel::Info,
                    "应用启动，配置已加载".to_string(),
                );
            }

            // 创建系统托盘
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let connect_i = MenuItem::with_id(app, "connect", "连接", true, None::<&str>)?;
            let disconnect_i = MenuItem::with_id(app, "disconnect", "断开", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &connect_i, &disconnect_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "connect" => {
                        let _ = app.emit("tray-connect", ());
                    }
                    "disconnect" => {
                        let _ = app.emit("tray-disconnect", ());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 关闭窗口时根据配置决定最小化到托盘还是退出
            let app_handle = app.handle().clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let state = app_handle.state::<AppState>();
                        let close_to_tray = {
                            let config = state.config.lock().unwrap();
                            config.close_to_tray
                        };

                        if close_to_tray {
                            // 最小化到托盘
                            api.prevent_close();
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.hide();
                            }
                        }
                        // 如果 close_to_tray 为 false，不阻止关闭，应用正常退出
                    }
                });
            }

            // 启动自动连接后台任务
            auto_connect::start_auto_connect(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_entries,
            commands::connect,
            commands::disconnect,
            commands::get_status,
            commands::get_config,
            commands::update_config,
            commands::get_logs,
            commands::clear_logs,
            commands::set_auto_connect,
            commands::refresh_status,
            commands::get_close_to_tray,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
