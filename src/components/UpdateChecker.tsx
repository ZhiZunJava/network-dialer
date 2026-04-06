import React, { useState } from "react";
import { Button, Typography, Progress, Card, message, Space } from "antd";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";

const { Text } = Typography;

interface UpdateInfo {
  version: string;
  body: string;
}

const UpdateChecker: React.FC = () => {
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [currentVersion, setCurrentVersion] = useState("");
  const [noUpdate, setNoUpdate] = useState(false);

  const handleCheck = async () => {
    setChecking(true);
    setNoUpdate(false);
    setUpdateInfo(null);

    try {
      const ver = await getVersion();
      setCurrentVersion(ver);

      const update = await check();
      if (update) {
        setUpdateInfo({
          version: update.version,
          body: update.body ?? "",
        });
      } else {
        setNoUpdate(true);
      }
    } catch (e) {
      message.error("检查更新失败: " + String(e));
    } finally {
      setChecking(false);
    }
  };

  const handleDownload = async () => {
    setDownloading(true);
    setProgress(0);

    try {
      const update = await check();
      if (!update) {
        message.info("没有可用的更新");
        setDownloading(false);
        return;
      }

      let totalLen = 0;
      let downloaded = 0;

      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          totalLen = (event.data as any).contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += (event.data as any).chunkLength ?? 0;
          if (totalLen > 0) {
            setProgress(Math.round((downloaded / totalLen) * 100));
          }
        } else if (event.event === "Finished") {
          setProgress(100);
        }
      });

      message.success("更新下载完成，即将重启应用...");
      setTimeout(async () => {
        await relaunch();
      }, 1500);
    } catch (e) {
      message.error("下载更新失败: " + String(e));
      setDownloading(false);
    }
  };

  return (
    <Card size="small" styles={{ body: { padding: "12px 16px" } }}>
      <div style={{ marginBottom: 4 }}>
        <Text strong style={{ fontSize: 13 }}>
          <i
            className="ri-download-cloud-2-line"
            style={{ marginRight: 6, color: "#1677ff" }}
          />
          软件更新
        </Text>
        {currentVersion && (
          <Text type="secondary" style={{ fontSize: 11, marginLeft: 8 }}>
            当前版本: v{currentVersion}
          </Text>
        )}
      </div>

      {!updateInfo && !downloading && (
        <div style={{ marginTop: 8 }}>
          <Button
            size="small"
            icon={<i className="ri-refresh-line" style={{ marginRight: 4 }} />}
            onClick={handleCheck}
            loading={checking}
          >
            检查更新
          </Button>
          {noUpdate && (
            <Text
              type="secondary"
              style={{ fontSize: 12, marginLeft: 10 }}
            >
              <i
                className="ri-check-line"
                style={{ marginRight: 4, color: "#52c41a" }}
              />
              已是最新版本
            </Text>
          )}
        </div>
      )}

      {updateInfo && !downloading && (
        <div style={{ marginTop: 8 }}>
          <div style={{ marginBottom: 8 }}>
            <Text style={{ fontSize: 13 }}>
              <i
                className="ri-sparkling-2-line"
                style={{ marginRight: 4, color: "#faad14" }}
              />
              发现新版本:{" "}
              <Text strong style={{ color: "#1677ff" }}>
                v{updateInfo.version}
              </Text>
            </Text>
          </div>
          {updateInfo.body && (
            <div
              style={{
                background: "#f6f8fa",
                borderRadius: 6,
                padding: "8px 12px",
                marginBottom: 8,
                maxHeight: 100,
                overflow: "auto",
                fontSize: 12,
                lineHeight: 1.6,
              }}
            >
              <Text type="secondary" style={{ whiteSpace: "pre-wrap" }}>
                {updateInfo.body}
              </Text>
            </div>
          )}
          <Space size={8}>
            <Button
              type="primary"
              size="small"
              icon={
                <i
                  className="ri-download-2-line"
                  style={{ marginRight: 4 }}
                />
              }
              onClick={handleDownload}
            >
              立即更新
            </Button>
            <Button
              size="small"
              onClick={() => {
                setUpdateInfo(null);
              }}
            >
              稍后再说
            </Button>
          </Space>
        </div>
      )}

      {downloading && (
        <div style={{ marginTop: 8 }}>
          <Text style={{ fontSize: 12, marginBottom: 4, display: "block" }}>
            <i
              className="ri-download-line"
              style={{ marginRight: 4, color: "#1677ff" }}
            />
            {progress < 100 ? "正在下载更新..." : "下载完成，准备安装..."}
          </Text>
          <Progress
            percent={progress}
            size="small"
            status={progress < 100 ? "active" : "success"}
          />
        </div>
      )}
    </Card>
  );
};

export default UpdateChecker;
