import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionConfig } from "../types";

const defaultConfig: ConnectionConfig = {
  entry_name: "",
  username: "",
  password: "",
  auto_connect: true,
  retry_interval_secs: 2,
  max_retries: 0,
  check_interval_secs: 5,
  close_to_tray: true,
};

export function useSettings() {
  const [config, setConfig] = useState<ConnectionConfig>(defaultConfig);
  const [saving, setSaving] = useState(false);

  // 加载配置
  const loadConfig = useCallback(async () => {
    try {
      const result = await invoke<ConnectionConfig>("get_config");
      setConfig(result);
    } catch (e) {
      console.error("加载配置失败:", e);
    }
  }, []);

  // 保存配置
  const saveConfig = useCallback(async (newConfig: ConnectionConfig) => {
    setSaving(true);
    try {
      await invoke("update_config", { config: newConfig });
      setConfig(newConfig);
    } catch (e) {
      console.error("保存配置失败:", e);
      throw e;
    } finally {
      setSaving(false);
    }
  }, []);

  // 切换自动连接
  const toggleAutoConnect = useCallback(
    async (enabled: boolean) => {
      try {
        await invoke("set_auto_connect", { enabled });
        setConfig((prev) => ({ ...prev, auto_connect: enabled }));
      } catch (e) {
        console.error("切换自动连接失败:", e);
      }
    },
    []
  );

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  return {
    config,
    saving,
    loadConfig,
    saveConfig,
    toggleAutoConnect,
  };
}
