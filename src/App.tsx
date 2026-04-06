import React, { useState } from "react";
import { Layout, Typography, ConfigProvider, theme, Segmented } from "antd";
import zhCN from "antd/locale/zh_CN";
import StatusCard from "./components/StatusCard";
import ConnectionControl from "./components/ConnectionControl";
import SettingsPanel from "./components/SettingsPanel";
import LogViewer from "./components/LogViewer";
import { useConnection } from "./hooks/useConnection";
import { useSettings } from "./hooks/useSettings";
import "./styles/global.css";

const { Header, Content } = Layout;
const { Title } = Typography;

const App: React.FC = () => {
  const {
    state,
    message,
    entries,
    loading,
    fetchEntries,
    refreshStatus,
    handleConnect,
    handleDisconnect,
  } = useConnection();

  const { config, saving, saveConfig } = useSettings();
  const [tab, setTab] = useState<string>("main");

  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary: "#1677ff",
          borderRadius: 8,
          fontSize: 13,
        },
      }}
    >
      <Layout style={{ minHeight: "100vh", background: "#f7f8fa" }}>
        <Header
          style={{
            background: "#fff",
            padding: "0 16px",
            borderBottom: "1px solid #f0f0f0",
            display: "flex",
            alignItems: "center",
            height: 48,
            lineHeight: "48px",
          }}
        >
          <i
            className="ri-global-line"
            style={{ fontSize: 20, color: "#1677ff", marginRight: 8 }}
          />
          <Title level={5} style={{ margin: 0, fontSize: 15 }}>
            网络拨号管理器
          </Title>
        </Header>
        <Content style={{ padding: "12px 16px" }}>
          <div style={{ marginBottom: 12 }}>
            <Segmented
              value={tab}
              onChange={(v) => setTab(v as string)}
              options={[
                {
                  label: (
                    <span>
                      <i className="ri-dashboard-line" style={{ marginRight: 4 }} />
                      主页
                    </span>
                  ),
                  value: "main",
                },
                {
                  label: (
                    <span>
                      <i className="ri-settings-3-line" style={{ marginRight: 4 }} />
                      设置
                    </span>
                  ),
                  value: "settings",
                },
                {
                  label: (
                    <span>
                      <i className="ri-file-list-3-line" style={{ marginRight: 4 }} />
                      日志
                    </span>
                  ),
                  value: "logs",
                },
              ]}
              block
              style={{ marginBottom: 0 }}
            />
          </div>

          {tab === "main" && (
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <StatusCard state={state} message={message} />
              <ConnectionControl
                state={state}
                loading={loading}
                onConnect={() => handleConnect()}
                onDisconnect={handleDisconnect}
                onRefresh={refreshStatus}
              />
            </div>
          )}

          {tab === "settings" && (
            <SettingsPanel
              config={config}
              entries={entries}
              saving={saving}
              onSave={saveConfig}
              onRefreshEntries={fetchEntries}
            />
          )}

          {tab === "logs" && <LogViewer />}
        </Content>
      </Layout>
    </ConfigProvider>
  );
};

export default App;
