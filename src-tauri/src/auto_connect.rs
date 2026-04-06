use crate::commands::AppState;
use crate::ras::{dial, status, types::*};
use tauri::{AppHandle, Emitter, Manager};

/// 启动自动连接后台任务
pub fn start_auto_connect(app: AppHandle) {
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let mut retry_count: u32 = 0;

        loop {
            let state = app_handle.state::<AppState>();

            // 获取配置
            let config = {
                let c = state.config.lock().unwrap();
                c.clone()
            };

            // 检查自动连接是否启用
            if !config.auto_connect || config.entry_name.is_empty() {
                retry_count = 0;
                tokio::time::sleep(tokio::time::Duration::from_secs(config.check_interval_secs)).await;
                continue;
            }

            // 检查当前应用内状态
            let current_state = {
                let s = state.connection_state.lock().unwrap();
                *s
            };

            // 如果正在手动连接或断开，跳过
            if current_state == ConnectionState::Disconnecting {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }

            // 检查实际 RAS 连接状态（不信任内部状态，每次都查系统）
            let is_connected = match status::is_entry_connected(&config.entry_name) {
                Ok((connected, handle)) => {
                    if connected {
                        if let Some(h) = handle {
                            let mut hm = state.connection_handle.lock().unwrap();
                            *hm = Some(h.0 as usize);
                        }
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };

            if is_connected {
                // 已连接，重置计数器
                if current_state != ConnectionState::Connected {
                    {
                        let mut s = state.connection_state.lock().unwrap();
                        *s = ConnectionState::Connected;
                    }
                    let _ = app_handle.emit("connection-status-changed", StatusPayload {
                        state: ConnectionState::Connected,
                        message: format!("{} 已连接", config.entry_name),
                    });
                }
                retry_count = 0;
                // 已连接状态下更频繁地检查（快速发现外部断开/休眠唤醒后断线）
                let check_secs = config.check_interval_secs.min(3);
                tokio::time::sleep(tokio::time::Duration::from_secs(check_secs)).await;
            } else {
                // 未连接 — 可能是外部断开、休眠唤醒、或者一直没连上

                // 如果之前认为是已连接状态，先更新为断开（让 UI 立即反映）
                if current_state == ConnectionState::Connected {
                    {
                        let mut handle = state.connection_handle.lock().unwrap();
                        *handle = None;
                    }
                    {
                        let mut s = state.connection_state.lock().unwrap();
                        *s = ConnectionState::Disconnected;
                    }
                    let _ = app_handle.emit("connection-status-changed", StatusPayload {
                        state: ConnectionState::Disconnected,
                        message: "连接已断开，准备重连...".to_string(),
                    });
                    state.add_log(
                        LogLevel::Warning,
                        "检测到连接断开（外部断开/休眠唤醒）".to_string(),
                    );
                }

                // 如果正在 Connecting 状态（上次拨号还没完成），跳过避免重复拨号
                if current_state == ConnectionState::Connecting {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }

                // 检查是否超出最大重试次数（max_retries=0 表示无限重试）
                if config.max_retries > 0 && retry_count >= config.max_retries {
                    state.add_log(
                        LogLevel::Warning,
                        format!(
                            "已达最大重试次数 ({})，等待 30 秒后重新开始",
                            config.max_retries
                        ),
                    );
                    {
                        let mut s = state.connection_state.lock().unwrap();
                        *s = ConnectionState::Error;
                    }
                    let _ = app_handle.emit("connection-status-changed", StatusPayload {
                        state: ConnectionState::Error,
                        message: format!(
                            "已达最大重试次数 ({})，30 秒后重试",
                            config.max_retries
                        ),
                    });
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    retry_count = 0;
                    state.add_log(LogLevel::Info, "重试计数已重置，重新开始自动连接".to_string());
                    continue;
                }

                retry_count += 1;
                state.add_log(
                    LogLevel::Info,
                    format!(
                        "自动重连中... (第 {} 次{})",
                        retry_count,
                        if config.max_retries > 0 {
                            format!("/{}", config.max_retries)
                        } else {
                            String::new()
                        }
                    ),
                );

                {
                    let mut s = state.connection_state.lock().unwrap();
                    *s = ConnectionState::Connecting;
                }
                let _ = app_handle.emit("connection-status-changed", StatusPayload {
                    state: ConnectionState::Connecting,
                    message: format!("自动重连中 (第 {} 次)...", retry_count),
                });

                // 尝试拨号 — 立即提取 handle 值为 usize 以避免 HRASCONN 跨 await
                let dial_outcome: Result<usize, String> = {
                    match dial::ras_dial(&config.entry_name, &config.username, &config.password) {
                        Ok(h_conn) => Ok(h_conn.0 as usize),
                        Err(e) => Err(e.to_string()),
                    }
                };

                match dial_outcome {
                    Ok(handle_val) => {
                        {
                            let mut handle = state.connection_handle.lock().unwrap();
                            *handle = Some(handle_val);
                        }
                        {
                            let mut s = state.connection_state.lock().unwrap();
                            *s = ConnectionState::Connected;
                        }
                        let _ = app_handle.emit("connection-status-changed", StatusPayload {
                            state: ConnectionState::Connected,
                            message: format!("{} 已连接 (自动重连成功)", config.entry_name),
                        });
                        state.add_log(
                            LogLevel::Success,
                            format!("自动重连成功 (第 {} 次)", retry_count),
                        );
                        retry_count = 0;
                    }
                    Err(err_msg) => {
                        {
                            let mut s = state.connection_state.lock().unwrap();
                            *s = ConnectionState::Error;
                        }
                        let _ = app_handle.emit("connection-status-changed", StatusPayload {
                            state: ConnectionState::Error,
                            message: format!("重连失败: {}", err_msg),
                        });
                        state.add_log(
                            LogLevel::Error,
                            format!("自动重连失败: {}", err_msg),
                        );

                        // 所有错误都直接使用配置的重试间隔，不区分致命/非致命
                        // 用户要求：一直重连就能连上
                        let wait_secs = config.retry_interval_secs.max(1);
                        state.add_log(
                            LogLevel::Info,
                            format!("{} 秒后重试...", wait_secs),
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs)).await;
                        continue;
                    }
                }
            }
        }
    });
}
