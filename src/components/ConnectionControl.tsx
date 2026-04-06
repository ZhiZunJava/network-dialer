import React from "react";
import { Button, Space, Card } from "antd";
import type { ConnectionState } from "../types";

interface ConnectionControlProps {
  state: ConnectionState;
  loading: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onRefresh: () => void;
}

const ConnectionControl: React.FC<ConnectionControlProps> = ({
  state,
  loading,
  onConnect,
  onDisconnect,
  onRefresh,
}) => {
  const isConnected = state === "Connected";
  const isConnecting = state === "Connecting";
  const isDisconnecting = state === "Disconnecting";

  return (
    <Card styles={{ body: { padding: "12px 16px" } }}>
      <Space style={{ width: "100%", justifyContent: "center" }} size="middle">
        <Button
          type="primary"
          size="large"
          icon={<i className="ri-link" style={{ marginRight: 4 }} />}
          onClick={onConnect}
          loading={isConnecting}
          disabled={isConnected || isConnecting || isDisconnecting || loading}
          style={{ minWidth: 100 }}
        >
          连接
        </Button>
        <Button
          danger
          size="large"
          icon={<i className="ri-link-unlink" style={{ marginRight: 4 }} />}
          onClick={onDisconnect}
          loading={isDisconnecting}
          disabled={
            (!isConnected && state !== "Error") ||
            isDisconnecting ||
            loading
          }
          style={{ minWidth: 100 }}
        >
          断开
        </Button>
        <Button
          size="large"
          icon={<i className="ri-refresh-line" />}
          onClick={onRefresh}
          disabled={loading}
        />
      </Space>
    </Card>
  );
};

export default ConnectionControl;
