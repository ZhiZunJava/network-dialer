export type ConnectionState =
  | "Disconnected"
  | "Connecting"
  | "Connected"
  | "Disconnecting"
  | "Error";

export type LogLevel = "Info" | "Warning" | "Error" | "Success";

export interface RasEntry {
  name: string;
}

export interface RasConnection {
  name: string;
  handle: number;
}

export interface ConnectionConfig {
  entry_name: string;
  username: string;
  password: string;
  auto_connect: boolean;
  retry_interval_secs: number;
  max_retries: number;
  check_interval_secs: number;
  close_to_tray: boolean;
}

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
}

export interface StatusPayload {
  state: ConnectionState;
  message: string;
}
