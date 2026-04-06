use crate::ras::{dial, entries, error::RasError, status, types::*};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_plugin_store::StoreExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use windows::Win32::NetworkManagement::Rras::HRASCONN;

const STORE_PATH: &str = "config.json";
const CONFIG_KEY: &str = "connection_config";

/// 应用共享状态
pub struct AppState {
    pub config: Mutex<ConnectionConfig>,
    pub connection_handle: Mutex<Option<usize>>,
    pub connection_state: Mutex<ConnectionState>,
    pub logs: Mutex<Vec<LogEntry>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ConnectionConfig::default()),
            connection_handle: Mutex::new(None),
            connection_state: Mutex::new(ConnectionState::Disconnected),
            logs: Mutex::new(Vec::new()),
        }
    }

    /// 从 tauri-plugin-store 加载持久化配置
    pub fn load_persisted_config(&self, app: &tauri::AppHandle) {
        if let Ok(store) = app.store(STORE_PATH) {
            if let Some(val) = store.get(CONFIG_KEY) {
                if let Ok(config) = serde_json::from_value::<ConnectionConfig>(val) {
                    let mut current = self.config.lock().unwrap();
                    *current = config;
                }
            }
        }
    }

    /// 将配置持久化到 tauri-plugin-store
    pub fn persist_config(&self, app: &tauri::AppHandle) {
        if let Ok(store) = app.store(STORE_PATH) {
            let config = self.config.lock().unwrap().clone();
            if let Ok(val) = serde_json::to_value(&config) {
                let _ = store.set(CONFIG_KEY, val);
                let _ = store.save();
            }
        }
    }

    pub fn add_log(&self, level: LogLevel, message: String) {
        let entry = LogEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level,
            message,
        };
        let mut logs = self.logs.lock().unwrap();
        logs.push(entry);
        // 最多保留 500 条
        if logs.len() > 500 {
            let excess = logs.len() - 500;
            logs.drain(0..excess);
        }
    }
}

/// 列出系统中所有 RAS 连接条目
#[tauri::command]
pub fn list_entries() -> Result<Vec<RasEntry>, RasError> {
    entries::list_ras_entries()
}

/// 手动发起拨号连接
#[tauri::command]
pub fn connect(
    state: State<'_, AppState>,
    entry_name: String,
    username: String,
    password: String,
    app: tauri::AppHandle,
) -> Result<(), RasError> {
    // 更新状态为连接中
    {
        let mut s = state.connection_state.lock().unwrap();
        *s = ConnectionState::Connecting;
    }
    let _ = app.emit("connection-status-changed", StatusPayload {
        state: ConnectionState::Connecting,
        message: format!("正在连接 {}...", entry_name),
    });
    state.add_log(LogLevel::Info, format!("开始拨号: {}", entry_name));

    match dial::ras_dial(&entry_name, &username, &password) {
        Ok(h_conn) => {
            {
                let mut handle = state.connection_handle.lock().unwrap();
                *handle = Some(h_conn.0 as usize);
            }
            {
                let mut s = state.connection_state.lock().unwrap();
                *s = ConnectionState::Connected;
            }
            let _ = app.emit("connection-status-changed", StatusPayload {
                state: ConnectionState::Connected,
                message: format!("{} 已连接", entry_name),
            });
            state.add_log(LogLevel::Success, format!("连接成功: {}", entry_name));
            Ok(())
        }
        Err(e) => {
            {
                let mut s = state.connection_state.lock().unwrap();
                *s = ConnectionState::Error;
            }
            let _ = app.emit("connection-status-changed", StatusPayload {
                state: ConnectionState::Error,
                message: format!("连接失败: {}", e),
            });
            state.add_log(LogLevel::Error, format!("连接失败: {}", e));
            Err(e)
        }
    }
}

/// 手动断开连接
/// 策略：优先通过条目名查找活动连接，备选使用存储的句柄
#[tauri::command]
pub fn disconnect(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), RasError> {
    // 发送断开中状态
    {
        let mut s = state.connection_state.lock().unwrap();
        *s = ConnectionState::Disconnecting;
    }
    let _ = app.emit("connection-status-changed", StatusPayload {
        state: ConnectionState::Disconnecting,
        message: "正在断开连接...".to_string(),
    });

    let config = state.config.lock().unwrap();
    let entry_name = config.entry_name.clone();
    drop(config);

    let stored_handle = {
        let h = state.connection_handle.lock().unwrap();
        *h
    };

    // 策略 1：通过条目名查找当前活动的连接句柄（最可靠）
    let mut hangup_result: Option<Result<(), RasError>> = None;

    if !entry_name.is_empty() {
        if let Ok((true, Some(h))) = status::is_entry_connected(&entry_name) {
            hangup_result = Some(dial::ras_hangup(h));
        }
    }

    // 策略 2：如果策略 1 未找到连接，使用存储的句柄
    if hangup_result.is_none() {
        if let Some(h) = stored_handle {
            let h_conn = HRASCONN(h as *mut _);
            hangup_result = Some(dial::ras_hangup(h_conn));
        }
    }

    // 无论结果如何，都清理本地状态
    {
        let mut handle = state.connection_handle.lock().unwrap();
        *handle = None;
    }

    match hangup_result {
        Some(Ok(())) => {
            {
                let mut s = state.connection_state.lock().unwrap();
                *s = ConnectionState::Disconnected;
            }
            let _ = app.emit("connection-status-changed", StatusPayload {
                state: ConnectionState::Disconnected,
                message: "已断开连接".to_string(),
            });
            state.add_log(LogLevel::Info, "手动断开连接".to_string());
            Ok(())
        }
        Some(Err(e)) => {
            // 挂断 API 返回错误，但仍将状态置为 Disconnected
            // 因为连接可能已经断开了（错误 619 = 端口未连接，668 = 链路已终止）
            let err_msg = format!("{}", e);
            let is_already_disconnected = matches!(&e,
                RasError::ApiError(_, code) if *code == 619 || *code == 668 || *code == 606
            );

            if is_already_disconnected {
                {
                    let mut s = state.connection_state.lock().unwrap();
                    *s = ConnectionState::Disconnected;
                }
                let _ = app.emit("connection-status-changed", StatusPayload {
                    state: ConnectionState::Disconnected,
                    message: "已断开连接".to_string(),
                });
                state.add_log(LogLevel::Info, "连接已断开（确认）".to_string());
                Ok(())
            } else {
                {
                    let mut s = state.connection_state.lock().unwrap();
                    *s = ConnectionState::Disconnected;
                }
                let _ = app.emit("connection-status-changed", StatusPayload {
                    state: ConnectionState::Disconnected,
                    message: format!("断开时出现警告: {}", err_msg),
                });
                state.add_log(LogLevel::Warning, format!("断开时出现警告: {}", err_msg));
                // 仍然返回 Ok，因为状态已清理
                Ok(())
            }
        }
        None => {
            // 没有找到任何连接可断开
            {
                let mut s = state.connection_state.lock().unwrap();
                *s = ConnectionState::Disconnected;
            }
            let _ = app.emit("connection-status-changed", StatusPayload {
                state: ConnectionState::Disconnected,
                message: "已断开连接".to_string(),
            });
            state.add_log(LogLevel::Info, "断开连接（无活动连接）".to_string());
            Ok(())
        }
    }
}

/// 获取当前连接状态
#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> ConnectionState {
    let s = state.connection_state.lock().unwrap();
    *s
}

/// 获取配置
#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> ConnectionConfig {
    let config = state.config.lock().unwrap();
    config.clone()
}

/// 更新配置（同时持久化到磁盘）
#[tauri::command]
pub fn update_config(
    state: State<'_, AppState>,
    config: ConnectionConfig,
    app: tauri::AppHandle,
) -> Result<(), RasError> {
    {
        let mut current = state.config.lock().unwrap();
        *current = config;
    }
    state.persist_config(&app);
    state.add_log(LogLevel::Info, "配置已保存".to_string());
    Ok(())
}

/// 获取日志
#[tauri::command]
pub fn get_logs(state: State<'_, AppState>) -> Vec<LogEntry> {
    let logs = state.logs.lock().unwrap();
    logs.clone()
}

/// 清除日志
#[tauri::command]
pub fn clear_logs(state: State<'_, AppState>) {
    let mut logs = state.logs.lock().unwrap();
    logs.clear();
}

/// 设置自动连接开关
#[tauri::command]
pub fn set_auto_connect(
    state: State<'_, AppState>,
    enabled: bool,
    app: tauri::AppHandle,
) {
    {
        let mut config = state.config.lock().unwrap();
        config.auto_connect = enabled;
    }
    state.persist_config(&app);
    state.add_log(
        LogLevel::Info,
        format!("自动连接已{}", if enabled { "启用" } else { "禁用" }),
    );
}

/// 获取 close_to_tray 配置（供窗口关闭事件使用）
#[tauri::command]
pub fn get_close_to_tray(state: State<'_, AppState>) -> bool {
    let config = state.config.lock().unwrap();
    config.close_to_tray
}

/// 刷新连接状态（检测实际网络状态）
/// 先检查配置的连接，再检查是否有任何活动的 RAS 连接
#[tauri::command]
pub fn refresh_status(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> ConnectionState {
    let config = state.config.lock().unwrap();
    let entry_name = config.entry_name.clone();
    drop(config);

    // 1. 如果配置了连接名，检查该连接
    if !entry_name.is_empty() {
        if let Ok((true, Some(h))) = status::is_entry_connected(&entry_name) {
            let handle_val = h.0 as usize;
            {
                let mut handle = state.connection_handle.lock().unwrap();
                *handle = Some(handle_val);
            }
            {
                let mut s = state.connection_state.lock().unwrap();
                *s = ConnectionState::Connected;
            }
            let _ = app.emit("connection-status-changed", StatusPayload {
                state: ConnectionState::Connected,
                message: format!("{} 已连接", entry_name),
            });
            return ConnectionState::Connected;
        }
    }

    // 2. 检查是否有任何活动的 RAS 连接
    if let Ok(Some((name, h))) = status::any_active_connection() {
        let handle_val = h.0 as usize;
        {
            let mut handle = state.connection_handle.lock().unwrap();
            *handle = Some(handle_val);
        }
        {
            let mut s = state.connection_state.lock().unwrap();
            *s = ConnectionState::Connected;
        }
        let _ = app.emit("connection-status-changed", StatusPayload {
            state: ConnectionState::Connected,
            message: format!("{} 已连接", name),
        });
        return ConnectionState::Connected;
    }

    // 3. 无连接
    {
        let mut handle = state.connection_handle.lock().unwrap();
        *handle = None;
    }
    {
        let mut s = state.connection_state.lock().unwrap();
        *s = ConnectionState::Disconnected;
    }
    let _ = app.emit("connection-status-changed", StatusPayload {
        state: ConnectionState::Disconnected,
        message: "未连接".to_string(),
    });
    ConnectionState::Disconnected
}

const TASK_NAME: &str = "NetworkLineDialer";

/// 设置开机自启动（通过 Windows 任务计划程序，支持管理员权限启动）
#[tauri::command]
pub fn set_auto_start(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<(), String> {
    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("获取程序路径失败: {}", e))?;
        let exe_str = exe_path.display().to_string();

        // 使用 schtasks 创建开机自启动任务，HIGHEST 表示以最高权限运行
        let output = std::process::Command::new("schtasks")
            .args([
                "/Create",
                "/TN", TASK_NAME,
                "/TR", &format!("\"{}\"", exe_str),
                "/SC", "ONLOGON",
                "/RL", "HIGHEST",
                "/F",       // 强制覆盖已有任务
                "/DELAY", "0000:06",  // 延迟6秒启动，避免托盘未就绪
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .map_err(|e| format!("执行 schtasks 失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("创建自启动任务失败: {} {}", stdout, stderr));
        }
    } else {
        let output = std::process::Command::new("schtasks")
            .args(["/Delete", "/TN", TASK_NAME, "/F"])
            .creation_flags(0x08000000)
            .output()
            .map_err(|e| format!("执行 schtasks 失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 如果任务不存在，也认为禁用成功
            if !stderr.contains("ERROR: The system cannot find")
                && !stderr.contains("错误: 系统找不到") {
                return Err(format!("删除自启动任务失败: {}", stderr));
            }
        }
    }

    let state = app.state::<AppState>();
    state.add_log(
        LogLevel::Info,
        format!("开机自启动已{}", if enabled { "启用" } else { "禁用" }),
    );
    Ok(())
}

/// 获取开机自启动状态（查询任务计划程序中是否存在对应任务）
#[tauri::command]
pub fn get_auto_start(
    _app: tauri::AppHandle,
) -> Result<bool, String> {
    let output = std::process::Command::new("schtasks")
        .args(["/Query", "/TN", TASK_NAME])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("查询自启动状态失败: {}", e))?;

    Ok(output.status.success())
}
