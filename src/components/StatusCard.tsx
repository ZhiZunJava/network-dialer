import React from "react";
import { Card, Typography } from "antd";
import type { ConnectionState } from "../types";

const { Text } = Typography;

interface StatusCardProps {
  state: ConnectionState;
  message: string;
}

const stateConfig: Record<
  ConnectionState,
  {
    color: string;
    bgColor: string;
    icon: string;
    label: string;
  }
> = {
  Connected: {
    color: "#52c41a",
    bgColor: "#f6ffed",
    icon: "ri-wifi-line",
    label: "已连接",
  },
  Disconnected: {
    color: "#ff4d4f",
    bgColor: "#fff2f0",
    icon: "ri-wifi-off-line",
    label: "未连接",
  },
  Connecting: {
    color: "#1677ff",
    bgColor: "#e6f4ff",
    icon: "ri-loader-4-line ri-spin",
    label: "连接中",
  },
  Disconnecting: {
    color: "#faad14",
    bgColor: "#fffbe6",
    icon: "ri-loader-4-line ri-spin",
    label: "断开中",
  },
  Error: {
    color: "#ff4d4f",
    bgColor: "#fff2f0",
    icon: "ri-error-warning-line",
    label: "错误",
  },
};

const StatusCard: React.FC<StatusCardProps> = ({ state, message }) => {
  const cfg = stateConfig[state];

  return (
    <Card
      style={{
        textAlign: "center",
        background: cfg.bgColor,
        borderColor: cfg.color,
        borderWidth: 1,
      }}
      styles={{ body: { padding: "20px 16px 14px" } }}
    >
      <div style={{ marginBottom: 8 }}>
        <i
          className={cfg.icon}
          style={{ fontSize: 52, color: cfg.color, lineHeight: 1 }}
        />
      </div>
      <div
        style={{
          display: "inline-block",
          padding: "2px 14px",
          borderRadius: 12,
          background: cfg.color,
          color: "#fff",
          fontSize: 14,
          fontWeight: 500,
          marginBottom: 6,
        }}
      >
        {cfg.label}
      </div>
      <div>
        <Text type="secondary" style={{ fontSize: 12 }}>
          {message}
        </Text>
      </div>
    </Card>
  );
};

export default StatusCard;
