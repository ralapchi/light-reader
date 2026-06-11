# 轻看 (Light Reader)

本地优先的桌面 EPUB/TXT 阅读器，基于 Tauri 2 构建。

## 功能

- **多格式解析** — 支持 EPUB 和 TXT 格式，自动章节拆分与目录构建
- **单页 / 双页模式** — 支持传统滚动阅读和双页翻页两种阅读模式
- **行内图片** — EPUB 中的生僻字替代图片自动内联到段落文本中
- **脚注预览** — 鼠标悬停脚注链接时显示内容预览，点击跳转后自动隐藏
- **阅读进度** — 章节级与滚动位置级进度记录，跨会话自动恢复
- **全文搜索** — 书籍内关键词搜索，结果定位到段落
- **书签系统** — 添加/移除书签，独立书签管理页，快捷键 `Ctrl+B`
- **主题与排版** — 多种内置配色主题，可调字体、字号、行距、段间距
- **TTS 朗读** — 接入小米 TTS 服务，支持播放/暂停/继续/停止，段落级语音缓存
- **图书馆管理** — 导入、搜索、删除书籍，封面自动提取，删除时自动清理进度、书签和缓存
- **设置持久化** — 全局偏好与阅读设置在本地存储

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | [Tauri 2](https://tauri.app) |
| 后端 | Rust (edition 2024) |
| 前端 | React 19 + TypeScript |
| 状态管理 | [Zustand](https://zustand.docs.pmnd.rs) |
| 路由 | [React Router v7](https://reactrouter.com) |
| 构建工具 | [Vite 8](https://vite.dev) |
| XML 解析 | [quick-xml](https://crates.io/crates/quick-xml) |
| EPUB 解压 | [zip](https://crates.io/crates/zip) |
| 音频播放 | [rodio](https://crates.io/crates/rodio) |

## 项目结构

```
├── src/                    # Rust 后端源码
│   ├── domain/             # 领域模型
│   ├── parser/             # EPUB/TXT 解析器
│   ├── services/           # 业务服务层
│   ├── storage/            # 持久化存储 (JSON 文件)
│   ├── tauri_api/          # Tauri 命令与 DTO
│   └── tts/                # TTS 引擎 (合成、缓存、播放)
├── frontend/               # React 前端
│   └── src/
│       ├── pages/          # 页面组件
│       ├── hooks/          # 通用 hooks
│       ├── store/          # Zustand 状态
│       ├── services/       # 后端 API 调用封装
│       └── utils/          # 工具函数
├── src-tauri/              # Tauri 原生配置
└── icons/                  # 应用图标
```

## 开发

### 环境要求

- Rust 1.85+
- Node.js 20+
- macOS / Linux / Windows

### 启动开发服务器

```bash
# 安装前端依赖
cd frontend && npm install && cd ..

# 启动 Tauri 开发模式 (同时启动 Vite HMR)
cargo tauri dev
```

### 构建发布包

```bash
# 构建当前平台安装包
cargo tauri build

# 指定 Apple Silicon 目标
cargo tauri build --target aarch64-apple-darwin
```

产物在 `target/<target>/release/bundle/`，macOS 下可生成 `.dmg` 和 `.app`。如需 `.pkg` 安装包：

```bash
productbuild --component "target/aarch64-apple-darwin/release/bundle/macos/轻看.app" /Applications 轻看.pkg
```

### 日志

运行时日志写入应用数据目录的 `logs/reader.log`，可通过 `RUST_LOG=debug` 环境变量调整日志级别。

## 数据存储

书籍、进度、书签、设置等数据存储在系统应用数据目录：

- **macOS** — `~/Library/Application Support/light-reader/`
- **Linux** — `~/.local/share/light-reader/`
- **Windows** — `%APPDATA%/light-reader/`

## 下载

前往 [GitHub Releases](https://github.com/ralapchi/light-reader/releases) 下载最新版本。

## License

MIT
