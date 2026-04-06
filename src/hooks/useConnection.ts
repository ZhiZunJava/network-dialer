import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ConnectionState, StatusPayload, RasEntry } from "../types";

export function useConnection() {
  const [state, setState] = useState<ConnectionState>("Disconnected");
  const [message, setMessage] = useState<string>("未连接");
  const [entries, setEntries] = useState<RasEntry[]>([]);
  const [loading, setLoading] = useState(false);

  // 监听后端状态推送
  useEffect(() => {
    const unlisten = listen<StatusPayload>(
      "connection-status-changed",
      (event) => {
        setState(event.payload.state);
        setMessage(event.payload.message);
        setLoading(false);
      }
    );

    // 监听托盘事件
    const unlistenConnect = listen("tray-connect", () => {
      handleConnect();
    });
    const unlistenDisconnect = listen("tray-disconnect", () => {
      handleDisconnect();
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenConnect.then((fn) => fn());
      unlistenDisconnect.then((fn) => fn());
    };
  }, []);

  // 获取 RAS 条目列表
  const fetchEntries = useCallback(async () => {
    try {
      const result = await invoke<RasEntry[]>("list_entries");
      setEntries(result);
    } catch (e) {
      console.error("获取条目失败:", e);
    }
  }, []);

  // 刷新状态
  const refreshStatus = useCallback(async () => {
    try {
      const result = await invoke<ConnectionState>("refresh_status");
      setState(result);
    } catch (e) {
      console.error("刷新状态失败:", e);
    }
  }, []);

  // 连接
  const handleConnect = useCallback(
    async (entryName?: string, username?: string, password?: string) => {
      setLoading(true);
      try {
        // 如果没有传参，从配置中获取
        if (!entryName) {
          const config = await invoke<any>("get_config");
          entryName = config.entry_name;
          username = config.username || "";
          password = config.password || "";
        }
        await invoke("connect", {
          entryName,
          username: username || "",
          password: password || "",
        });
      } catch (e) {
        console.error("连接失败:", e);
        setLoading(false);
      }
    },
    []
  );

  // 断开
  const handleDisconnect = useCallback(async () => {
    setLoading(true);
    try {
      await invoke("disconnect");
    } catch (e) {
      console.error("断开失败:", e);
      setLoading(false);
    }
  }, []);

  // 初始化
  useEffect(() => {
    fetchEntries();
    refreshStatus();
  }, [fetchEntries, refreshStatus]);

  return {
    state,
    message,
    entries,
    loading,
    fetchEntries,
    refreshStatus,
    handleConnect,
    handleDisconnect,
  };
}
