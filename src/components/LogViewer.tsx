import React, { useState, useEffect, useCallback } from "react";
import { Card, Table, Tag, Button, Space } from "antd";
import { invoke } from "@tauri-apps/api/core";
import type { LogEntry, LogLevel } from "../types";

const levelColors: Record<LogLevel, string> = {
  Info: "blue",
  Warning: "orange",
  Error: "red",
  Success: "green",
};

const levelIcons: Record<LogLevel, string> = {
  Info: "ri-information-line",
  Warning: "ri-alert-line",
  Error: "ri-close-circle-line",
  Success: "ri-checkbox-circle-line",
};

const columns = [
  {
    title: "时间",
    dataIndex: "timestamp",
    key: "timestamp",
    width: 150,
    render: (v: string) => (
      <span style={{ fontSize: 11, color: "#999" }}>{v}</span>
    ),
  },
  {
    title: "级别",
    dataIndex: "level",
    key: "level",
    width: 70,
    render: (level: LogLevel) => (
      <Tag
        color={levelColors[level]}
        style={{ fontSize: 11, lineHeight: "18px" }}
      >
        <i className={levelIcons[level]} style={{ marginRight: 2 }} />
        {level}
      </Tag>
    ),
  },
  {
    title: "消息",
    dataIndex: "message",
    key: "message",
    render: (v: string) => <span style={{ fontSize: 12 }}>{v}</span>,
  },
];

const LogViewer: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);

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
    const interval = setInterval(fetchLogs, 5000);
    return () => clearInterval(interval);
  }, [fetchLogs]);

  return (
    <Card
      title={
        <span>
          <i className="ri-file-list-3-line" style={{ marginRight: 6 }} />
          连接日志
        </span>
      }
      size="small"
      styles={{ body: { padding: "0" } }}
      extra={
        <Space size="small">
          <Button
            size="small"
            type="text"
            icon={<i className="ri-refresh-line" />}
            onClick={fetchLogs}
          />
          <Button
            size="small"
            type="text"
            danger
            icon={<i className="ri-delete-bin-line" />}
            onClick={clearLogs}
          />
        </Space>
      }
    >
      <Table
        columns={columns}
        dataSource={logs.map((log, i) => ({ ...log, key: i }))}
        size="small"
        pagination={{ pageSize: 8, size: "small", simple: true }}
        loading={loading}
        scroll={{ y: 220 }}
        style={{ fontSize: 12 }}
      />
    </Card>
  );
};

export default LogViewer;
