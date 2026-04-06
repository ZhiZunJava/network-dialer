use serde::{Deserialize, Serialize};

/// RAS 连接条目（系统中配置的宽带连接）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasEntry {
    pub name: String,
}

/// 活动的 RAS 连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasConnection {
    pub name: String,
    pub handle: usize,
}

/// 连接状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::Disconnected
    }
}

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// 选择的 RAS 连接条目名称
    pub entry_name: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否启用自动连接
    pub auto_connect: bool,
    /// 重试间隔（秒）
    pub retry_interval_secs: u64,
    /// 最大重试次数（0 表示无限）
    pub max_retries: u32,
    /// 状态检查间隔（秒）
    pub check_interval_secs: u64,
    /// 关闭窗口时最小化到托盘（true）还是直接退出（false）
    #[serde(default = "default_close_to_tray")]
    pub close_to_tray: bool,
}

fn default_close_to_tray() -> bool {
    true
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            entry_name: String::new(),
            username: String::new(),
            password: String::new(),
            auto_connect: true,
            retry_interval_secs: 2,
            max_retries: 0,
            check_interval_secs: 5,
            close_to_tray: true,
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// 前端状态推送
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusPayload {
    pub state: ConnectionState,
    pub message: String,
}
