use crate::commands::AppState;
use crate::ras::{dial, error, status, types::*};
use tauri::{AppHandle, Emitter, Manager};

/// 启动自动连接后台任务
pub fn start_auto_connect(app: AppHandle) {
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let mut retry_count: u32 = 0;
        let mut backoff_secs: u64 = 0; // 当前退避时间，0 表示使用默认间隔

        loop {
            let state = app_handle.state::<AppState>();

            // 获取配置
            let config = {
                let c = state.config.lock().unwrap();
                c.clone()
            };

            // 检查自动连接是否启用
            if !config.auto_connect || config.entry_name.is_empty() {
                // 自动连接未启用时重置状态
                retry_count = 0;
                backoff_secs = 0;
                tokio::time::sleep(tokio::time::Duration::from_secs(config.check_interval_secs)).await;
                continue;
            }

            // 检查当前连接状态
            let current_state = {
                let s = state.connection_state.lock().unwrap();
                *s
            };

            // 如果正在连接中或正在断开，跳过
            if current_state == ConnectionState::Connecting
                || current_state == ConnectionState::Disconnecting
            {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }

            // 检查实际网络状态
            let is_connected = match status::is_entry_connected(&config.entry_name) {
                Ok((connected, handle)) => {
                    if connected {
                        // 更新句柄
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
                // 已连接，重置重试计数和退避
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
                backoff_secs = 0;
            } else {
                // 未连接，需要重连

                // 检查是否超出最大重试次数（max_retries=0 表示无限重试）
                if config.max_retries > 0 && retry_count >= config.max_retries {
                    state.add_log(
                        LogLevel::Warning,
                        format!(
                            "已达最大重试次数 ({})，等待 60 秒后重新开始",
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
                            "已达最大重试次数 ({})，60 秒后重试",
                            config.max_retries
                        ),
                    });
                    // 等待一段较长的冷却时间，然后重置计数器重新开始
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    retry_count = 0;
                    backoff_secs = 0;
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

                // 尝试拨号 — 立即提取 handle 值为 usize 以避免 HRASCONN(*mut c_void) 跨 await
                let dial_outcome: Result<usize, (String, u32)> = {
                    match dial::ras_dial(&config.entry_name, &config.username, &config.password) {
                        Ok(h_conn) => Ok(h_conn.0 as usize),
                        Err(e) => {
                            let code = match &e {
                                crate::ras::error::RasError::ApiError(_, c) => *c,
                                _ => 0,
                            };
                            Err((e.to_string(), code))
                        }
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
                        backoff_secs = 0;
                    }
                    Err((err_msg, err_code)) => {
                        // 根据错误类型决定重试策略
                        let is_fatal = err_code != 0 && error::is_fatal_error(err_code);

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

                        if is_fatal {
                            // 致命错误（如认证失败、条目不存在、已在拨号中）
                            // 不应继续频繁重试
                            state.add_log(
                                LogLevel::Warning,
                                format!(
                                    "检测到致命错误 (代码: {})，暂停自动重连 120 秒。请检查配置后重试。",
                                    err_code
                                ),
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                            retry_count = 0;
                            backoff_secs = 0;
                            continue;
                        }

                        // 可恢复错误：使用指数退避
                        // 首次使用配置的 retry_interval_secs，之后每次翻倍，最大 120 秒
                        if backoff_secs == 0 {
                            backoff_secs = config.retry_interval_secs.max(5);
                        } else {
                            backoff_secs = (backoff_secs * 2).min(120);
                        }

                        state.add_log(
                            LogLevel::Info,
                            format!("{} 秒后重试...", backoff_secs),
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                        continue;
                    }
                }
            }

            // 检查间隔
            tokio::time::sleep(tokio::time::Duration::from_secs(config.check_interval_secs)).await;
        }
    });
}
