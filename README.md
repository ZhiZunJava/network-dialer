<div align="center">

# Network Line

**Windows 宽带拨号连接管理器**

一个轻量级的 Windows 桌面应用，用于管理 PPPoE / 宽带拨号连接，支持自动重连、系统托盘常驻和连接状态监控。

![Windows](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows&logoColor=white)
![Tauri 2](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri&logoColor=white)
![React 18](https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-1.75+-DEA584?logo=rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

</div>

---

## Features

| 功能 | 描述 |
|------|------|
| **一键拨号** | 选择系统已配置的宽带连接，一键连接/断开 |
| **自动重连** | 断线后自动重连，支持指数退避和智能错误分类 |
| **系统托盘** | 最小化到托盘后台运行，不占用任务栏 |
| **状态监控** | 实时显示连接状态，记录详细连接日志 |
| **配置持久化** | 所有设置永久保存，重启不丢失 |
| **凭据管理** | 支持手动输入或自动使用系统已保存的账号密码 |

## 界面预览

应用采用三页签布局：

- **主页** — 连接状态卡片 + 连接/断开按钮
- **设置** — 连接配置、自动重连参数、关闭行为
- **日志** — 连接日志记录与查看

## 快速开始

### 环境要求

- **Node.js** >= 18
- **Rust** >= 1.75
- **Windows 10/11**（使用 Windows RAS API）

### 安装依赖

````bash
npm install
````

### 开发模式

```bash
npm run tauri dev
```

### 构建安装包

```bash
npm run tauri build
```

构建产物在 `src-tauri/target/release/bundle/nsis/` 目录下。

## 技术架构

```
┌─────────────────────────────────────────┐
│           Frontend (React 18)           │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │StatusCard│ │ Settings │ │LogViewer│  │
│  └────┬─────┘ └────┬─────┘ └────┬────┘ │
│       │  Ant Design 5 + RemixIcon│      │
│       └─────────┬────────────────┘      │
├─────────────────┼───────────────────────┤
│    Tauri IPC    │   Events              │
├─────────────────┼───────────────────────┤
│           Backend (Rust)                │
│  ┌──────────┐ ┌───────────┐ ┌────────┐ │
│  │ commands │ │auto_connect│ │ ras/*  │ │
│  └──────────┘ └───────────┘ └────────┘ │
│            Windows RAS API              │
└─────────────────────────────────────────┘
```

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2 |
| 前端 | React 18 + TypeScript |
| UI 组件 | Ant Design 5 |
| 图标 | Remix Icon |
| 后端 | Rust |
| 网络接口 | Windows RAS API (rasapi32.dll) |
| 持久化 | tauri-plugin-store |
| 打包 | NSIS (Windows Installer) |

## 配置说明

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| 宽带连接 | — | 选择系统中已配置的拨号连接 |
| 用户名/密码 | 空 | 留空自动使用系统已保存的凭据 |
| 自动连接 | 开启 | 断线后自动重新拨号 |
| 重试间隔 | 2 秒 | 首次重试等待时间（指数退避） |
| 最大重试次数 | 0 | 0 = 无限重试 |
| 检查间隔 | 5 秒 | 连接状态轮询频率 |
| 关闭窗口时 | 最小化到托盘 | 可选择直接退出程序 |

## 项目结构

```
network_line/
├── src/                          # 前端源码
│   ├── components/               # React 组件
│   │   ├── StatusCard.tsx        # 连接状态卡片
│   │   ├── ConnectionControl.tsx # 连接控制按钮
│   │   ├── SettingsPanel.tsx     # 设置面板
│   │   └── LogViewer.tsx         # 日志查看器
│   ├── hooks/                    # React Hooks
│   │   ├── useConnection.ts      # 连接逻辑
│   │   └── useSettings.ts        # 配置管理
│   ├── types/index.ts            # TypeScript 类型定义
│   ├── App.tsx                   # 主组件
│   └── main.tsx                  # 入口
├── src-tauri/                    # 后端源码 (Rust)
│   ├── src/
│   │   ├── ras/                  # RAS API 封装
│   │   │   ├── dial.rs           # 拨号/挂断 + 连接验证
│   │   │   ├── status.rs         # 连接状态查询
│   │   │   ├── entries.rs        # 条目枚举
│   │   │   ├── error.rs          # 错误码 (150+ 条)
│   │   │   └── types.rs          # 数据类型定义
│   │   ├── auto_connect.rs       # 自动重连 (指数退避)
│   │   ├── commands.rs           # Tauri 命令 + 持久化
│   │   └── lib.rs                # 应用入口
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## RAS API 错误处理

应用内置了 **150+ 条** Windows RAS 错误码的中文翻译，并对错误进行智能分类：

- **致命错误**（如 691 密码错误、756 重复拨号）— 暂停 120 秒，提示检查配置
- **可恢复错误**（如 619 端口断开、678 无应答）— 指数退避自动重试
- **连接验证** — 拨号后轮询 `RasGetConnectStatus` 确认连接真正建立

## License

[MIT](LICENSE)
