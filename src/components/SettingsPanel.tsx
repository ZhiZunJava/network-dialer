import React, { useEffect, useState } from "react";
import {
  Card,
  Form,
  Input,
  Select,
  InputNumber,
  Switch,
  Button,
  message,
  Divider,
  Tooltip,
  Row,
  Col,
  Typography,
} from "antd";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionConfig, RasEntry } from "../types";

const { Text } = Typography;

interface SettingsPanelProps {
  config: ConnectionConfig;
  entries: RasEntry[];
  saving: boolean;
  onSave: (config: ConnectionConfig) => Promise<void>;
  onRefreshEntries: () => void;
}

const SectionTitle: React.FC<{
  icon: string;
  title: string;
  subtitle?: string;
}> = ({ icon, title, subtitle }) => (
  <div style={{ marginBottom: 4 }}>
    <Text strong style={{ fontSize: 13 }}>
      <i className={icon} style={{ marginRight: 6, color: "#1677ff" }} />
      {title}
    </Text>
    {subtitle && (
      <Text type="secondary" style={{ fontSize: 11, marginLeft: 8 }}>
        {subtitle}
      </Text>
    )}
  </div>
);

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  config,
  entries,
  saving,
  onSave,
  onRefreshEntries,
}) => {
  const [form] = Form.useForm<ConnectionConfig>();
  const [autoStart, setAutoStart] = useState(false);

  useEffect(() => {
    form.setFieldsValue(config);
  }, [config, form]);

  useEffect(() => {
    invoke<boolean>("get_auto_start")
      .then(setAutoStart)
      .catch(() => {});
  }, []);

  const handleAutoStartChange = async (checked: boolean) => {
    try {
      await invoke("set_auto_start", { enabled: checked });
      setAutoStart(checked);
      message.success(checked ? "已启用开机自启动" : "已关闭开机自启动");
    } catch (e) {
      message.error("设置开机自启动失败: " + String(e));
    }
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      await onSave(values);
      message.success("配置已保存");
    } catch (e: any) {
      if (e.errorFields) return;
      message.error("保存失败: " + String(e));
    }
  };

  return (
    <Form
      form={form}
      layout="vertical"
      initialValues={config}
      size="small"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
        {/* 连接配置 */}
        <Card size="small" styles={{ body: { padding: "12px 16px" } }}>
          <SectionTitle icon="ri-router-line" title="连接配置" />

          <Form.Item
            label="宽带连接"
            name="entry_name"
            rules={[{ required: true, message: "请选择连接" }]}
            style={{ marginBottom: 10, marginTop: 8 }}
          >
            <Select
              placeholder="选择系统已配置的宽带连接"
              options={entries.map((e) => ({ label: e.name, value: e.name }))}
              dropdownRender={(menu) => (
                <>
                  {menu}
                  <Divider style={{ margin: "4px 0" }} />
                  <div style={{ padding: "4px 8px", textAlign: "center" }}>
                    <Button
                      type="link"
                      size="small"
                      icon={<i className="ri-refresh-line" />}
                      onClick={onRefreshEntries}
                    >
                      刷新列表
                    </Button>
                  </div>
                </>
              )}
            />
          </Form.Item>

          <Row gutter={10}>
            <Col span={12}>
              <Form.Item
                label={
                  <span>
                    用户名
                    <Tooltip title="留空则使用系统已保存的凭据">
                      <i
                        className="ri-information-line"
                        style={{ marginLeft: 4, color: "#bbb", cursor: "pointer", fontSize: 12 }}
                      />
                    </Tooltip>
                  </span>
                }
                name="username"
                style={{ marginBottom: 0 }}
              >
                <Input placeholder="留空用系统账号" />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item
                label={
                  <span>
                    密码
                    <Tooltip title="留空则使用系统已保存的凭据">
                      <i
                        className="ri-information-line"
                        style={{ marginLeft: 4, color: "#bbb", cursor: "pointer", fontSize: 12 }}
                      />
                    </Tooltip>
                  </span>
                }
                name="password"
                style={{ marginBottom: 0 }}
              >
                <Input.Password placeholder="留空用系统密码" />
              </Form.Item>
            </Col>
          </Row>
        </Card>

        {/* 自动重连 */}
        <Card size="small" styles={{ body: { padding: "12px 16px" } }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 10 }}>
            <SectionTitle icon="ri-loop-right-line" title="自动重连" />
            <Form.Item name="auto_connect" valuePropName="checked" noStyle>
              <Switch size="small" checkedChildren="开" unCheckedChildren="关" />
            </Form.Item>
          </div>

          <Row gutter={10}>
            <Col span={8}>
              <Form.Item
                label={<Text type="secondary" style={{ fontSize: 11 }}>重试间隔(秒)</Text>}
                name="retry_interval_secs"
                style={{ marginBottom: 0 }}
              >
                <InputNumber min={1} max={300} style={{ width: "100%" }} />
              </Form.Item>
            </Col>
            <Col span={8}>
              <Form.Item
                label={<Text type="secondary" style={{ fontSize: 11 }}>最大重试</Text>}
                name="max_retries"
                style={{ marginBottom: 0 }}
                tooltip="0 = 无限重试"
              >
                <InputNumber min={0} max={9999} style={{ width: "100%" }} />
              </Form.Item>
            </Col>
            <Col span={8}>
              <Form.Item
                label={<Text type="secondary" style={{ fontSize: 11 }}>检查间隔(秒)</Text>}
                name="check_interval_secs"
                style={{ marginBottom: 0 }}
              >
                <InputNumber min={1} max={60} style={{ width: "100%" }} />
              </Form.Item>
            </Col>
          </Row>
        </Card>

        {/* 应用设置 */}
        <Card size="small" styles={{ body: { padding: "12px 16px" } }}>
          <SectionTitle icon="ri-settings-3-line" title="应用设置" />
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginTop: 8,
            }}
          >
            <div>
              <Text style={{ fontSize: 13 }}>
                <i className="ri-power-line" style={{ marginRight: 6 }} />
                开机自启动
              </Text>
              <br />
              <Text type="secondary" style={{ fontSize: 11, paddingLeft: 20 }}>
                开机后自动启动程序
              </Text>
            </div>
            <Switch
              size="small"
              checked={autoStart}
              onChange={handleAutoStartChange}
            />
          </div>
          <Divider style={{ margin: "10px 0" }} />
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <div>
              <Text style={{ fontSize: 13 }}>
                <i className="ri-window-line" style={{ marginRight: 6 }} />
                关闭时最小化到托盘
              </Text>
              <br />
              <Text type="secondary" style={{ fontSize: 11, paddingLeft: 20 }}>
                关闭后程序将在系统托盘继续运行
              </Text>
            </div>
            <Form.Item name="close_to_tray" valuePropName="checked" noStyle>
              <Switch size="small" />
            </Form.Item>
          </div>
        </Card>

        {/* 保存按钮 */}
        <Button
          type="primary"
          icon={<i className="ri-save-line" style={{ marginRight: 4 }} />}
          onClick={handleSave}
          loading={saving}
          block
          size="middle"
          style={{ borderRadius: 8 }}
        >
          保存配置
        </Button>
      </div>
    </Form>
  );
};

export default SettingsPanel;
