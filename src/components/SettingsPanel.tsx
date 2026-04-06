import React, { useEffect } from "react";
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
} from "antd";
import type { ConnectionConfig, RasEntry } from "../types";

interface SettingsPanelProps {
  config: ConnectionConfig;
  entries: RasEntry[];
  saving: boolean;
  onSave: (config: ConnectionConfig) => Promise<void>;
  onRefreshEntries: () => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  config,
  entries,
  saving,
  onSave,
  onRefreshEntries,
}) => {
  const [form] = Form.useForm<ConnectionConfig>();

  useEffect(() => {
    form.setFieldsValue(config);
  }, [config, form]);

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
    <Card
      title={
        <span>
          <i className="ri-settings-3-line" style={{ marginRight: 6 }} />
          连接设置
        </span>
      }
      size="small"
      styles={{ body: { padding: "12px 16px" } }}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={config}
        size="small"
      >
        <Form.Item
          label={
            <span>
              <i className="ri-router-line" style={{ marginRight: 4 }} />
              宽带连接
            </span>
          }
          name="entry_name"
          rules={[{ required: true, message: "请选择连接" }]}
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

        <Form.Item
          label={
            <span>
              <i className="ri-user-line" style={{ marginRight: 4 }} />
              用户名
              <Tooltip title="留空则使用系统已保存的凭据">
                <i
                  className="ri-information-line"
                  style={{ marginLeft: 4, color: "#999", cursor: "pointer" }}
                />
              </Tooltip>
            </span>
          }
          name="username"
        >
          <Input placeholder="留空使用系统已保存的账号" />
        </Form.Item>

        <Form.Item
          label={
            <span>
              <i className="ri-lock-line" style={{ marginRight: 4 }} />
              密码
              <Tooltip title="留空则使用系统已保存的凭据">
                <i
                  className="ri-information-line"
                  style={{ marginLeft: 4, color: "#999", cursor: "pointer" }}
                />
              </Tooltip>
            </span>
          }
          name="password"
        >
          <Input.Password placeholder="留空使用系统已保存的密码" />
        </Form.Item>

        <Divider style={{ margin: "8px 0 12px" }} />

        <Form.Item
          label={
            <span>
              <i className="ri-loop-right-line" style={{ marginRight: 4 }} />
              自动连接
            </span>
          }
          name="auto_connect"
          valuePropName="checked"
        >
          <Switch checkedChildren="开" unCheckedChildren="关" />
        </Form.Item>

        <Form.Item label="重试间隔(秒)" name="retry_interval_secs">
          <InputNumber min={1} max={300} style={{ width: "100%" }} />
        </Form.Item>

        <Form.Item
          label="最大重试次数"
          name="max_retries"
          extra="0 = 无限重试"
        >
          <InputNumber min={0} max={9999} style={{ width: "100%" }} />
        </Form.Item>

        <Form.Item label="检查间隔(秒)" name="check_interval_secs">
          <InputNumber min={1} max={60} style={{ width: "100%" }} />
        </Form.Item>

        <Divider style={{ margin: "8px 0 12px" }} />

        <Form.Item
          label={
            <span>
              <i className="ri-window-line" style={{ marginRight: 4 }} />
              关闭窗口时
            </span>
          }
          name="close_to_tray"
          valuePropName="checked"
          extra="开启后点击关闭按钮将最小化到系统托盘，关闭则直接退出程序"
        >
          <Switch checkedChildren="最小化到托盘" unCheckedChildren="退出程序" />
        </Form.Item>

        <Form.Item style={{ marginBottom: 0 }}>
          <Button
            type="primary"
            icon={<i className="ri-save-line" style={{ marginRight: 4 }} />}
            onClick={handleSave}
            loading={saving}
            block
          >
            保存配置
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
};

export default SettingsPanel;
