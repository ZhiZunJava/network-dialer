import React, { useState, useEffect, useCallback, useRef } from "react";
import { Card, Tag, Button, Space, Empty, Segmented, Badge } from "antd";
import { invoke } from "@tauri-apps/api/core";
import type { LogEntry, LogLevel } from "../types";

const levelConfig: Record<
  LogLevel,
  { color: string; icon: string; bg: string }
> = {
  Info: { color: "#1677ff", icon: "ri-information-line", bg: "#e6f4ff" },
  Warning: { color: "#faad14", icon: "ri-alert-line", bg: "#fffbe6" },
  Error: { color: "#ff4d4f", icon: "ri-close-circle-line", bg: "#fff2f0" },
  Success: { color: "#52c41a", icon: "ri-checkbox-circle-line", bg: "#f6ffed" },
};

type FilterType = "all" | LogLevel;

const LogViewer: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState<FilterType>("all");
  const [autoScroll, setAutoScroll] = useState(true);
  const listRef = useRef<HTMLDivElement>(null);

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<LogEntry[]>("get_logs");
      setLogs([...result].reverse());
    } catch (e) {
      console.error("获取日志失败:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const clearLogs = async () => {
    try {
      await invoke("clear_logs");
      setLogs([]);
    } catch (e) {
      console.error("清除日志失败:", e);
    }
  };

  useEffect(() => {
    fetchLogs();
    const interval = setInterval(fetchLogs, 3000);
    return () => clearInterval(interval);
  }, [fetchLogs]);

  // 自动滚动到顶部（最新日志）
  useEffect(() => {
    if (autoScroll && listRef.current) {
      listRef.current.scrollTop = 0;
    }
  }, [logs, autoScroll]);

  const filteredLogs =
    filter === "all" ? logs : logs.filter((l) => l.level === filter);

  // 统计各级别数量
  const counts = {
    Error: logs.filter((l) => l.level === "Error").length,
    Warning: logs.filter((l) => l.level === "Warning").length,
  };

  return (
    <Card
      title={
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <span style={{ fontSize: 13 }}>
            <i className="ri-file-list-3-line" style={{ marginRight: 6 }} />
            连接日志
            <span style={{ color: "#999", fontWeight: 400, marginLeft: 6, fontSize: 11 }}>
              ({filteredLogs.length})
            </span>
          </span>
          <Space size={4}>
            <Button
              size="small"
              type="text"
              icon={<i className="ri-refresh-line" />}
              onClick={fetchLogs}
              loading={loading}
              title="刷新"
            />
            <Button
              size="small"
              type="text"
              icon={
                <i
                  className={autoScroll ? "ri-pushpin-fill" : "ri-pushpin-line"}
                  style={{ color: autoScroll ? "#1677ff" : undefined }}
                />
              }
              onClick={() => setAutoScroll(!autoScroll)}
              title={autoScroll ? "已锁定到最新" : "点击锁定到最新"}
            />
            <Button
              size="small"
              type="text"
              danger
              icon={<i className="ri-delete-bin-line" />}
              onClick={clearLogs}
              title="清空日志"
            />
          </Space>
        </div>
      }
      size="small"
      styles={{ body: { padding: "0" } }}
    >
      {/* 筛选栏 */}
      <div style={{ padding: "8px 12px 4px", borderBottom: "1px solid #f0f0f0" }}>
        <Segmented
          size="small"
          value={filter}
          onChange={(v) => setFilter(v as FilterType)}
          options={[
            { label: "全部", value: "all" },
            {
              label: (
                <span>
                  错误
                  {counts.Error > 0 && (
                    <Badge
                      count={counts.Error}
                      size="small"
                      style={{ marginLeft: 4 }}
                    />
                  )}
                </span>
              ),
              value: "Error",
            },
            {
              label: (
                <span>
                  警告
                  {counts.Warning > 0 && (
                    <Badge
                      count={counts.Warning}
                      color="#faad14"
                      size="small"
                      style={{ marginLeft: 4 }}
                    />
                  )}
                </span>
              ),
              value: "Warning",
            },
            { label: "信息", value: "Info" },
            { label: "成功", value: "Success" },
          ]}
          block
          style={{ fontSize: 12 }}
        />
      </div>

      {/* 日志列表 */}
      <div
        ref={listRef}
        style={{
          height: 340,
          overflowY: "auto",
          padding: "4px 0",
        }}
      >
        {filteredLogs.length === 0 ? (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="暂无日志"
            style={{ marginTop: 60 }}
          />
        ) : (
          filteredLogs.map((log, i) => {
            const cfg = levelConfig[log.level];
            return (
              <div
                key={i}
                style={{
                  padding: "6px 12px",
                  borderBottom: "1px solid #fafafa",
                  display: "flex",
                  alignItems: "flex-start",
                  gap: 8,
                  transition: "background 0.15s",
                  cursor: "default",
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLDivElement).style.background = "#fafafa";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLDivElement).style.background = "transparent";
                }}
              >
                <i
                  className={cfg.icon}
                  style={{
                    color: cfg.color,
                    fontSize: 14,
                    marginTop: 1,
                    flexShrink: 0,
                  }}
                />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: 12,
                      lineHeight: 1.5,
                      color: "#333",
                      wordBreak: "break-word",
                    }}
                  >
                    {log.message}
                  </div>
                  <div
                    style={{
                      fontSize: 10,
                      color: "#bbb",
                      marginTop: 1,
                    }}
                  >
                    {log.timestamp}
                  </div>
                </div>
                <Tag
                  color={cfg.color}
                  style={{
                    fontSize: 10,
                    lineHeight: "16px",
                    padding: "0 4px",
                    margin: 0,
                    flexShrink: 0,
                    borderRadius: 4,
                  }}
                >
                  {log.level}
                </Tag>
              </div>
            );
          })
        )}
      </div>
    </Card>
  );
};

export default LogViewer;
