
  ---
短期（小改动，高价值）

1. 阅读统计

- 记录每日/累计阅读时长、页数、书籍数
- 首页或独立页面展示统计卡片
- 数据模型简单，只需新增一个 reading_stats.json

2. 高亮批注

- 在阅读器中选中文本可添加高亮 + 文字批注
- 复用现有书签系统的存储模式，扩展 bookmark.rs 增加 highlight_range 和 note 字段
- 前端增加选中文本浮动工具栏

3. TTS 多引擎（阿里云）

- Cargo.toml 已有 tts-aliyun feature flag
- 实现 AliyunProvider trait，与小米 provider 并列
- TTS 配置页增加引擎选择下拉

4. 阅读进度导出/导入

- JSON 格式导出书签、进度、高亮
- 重装系统后可恢复，也方便跨设备迁移

  ---
中期（中等改动，拓展场景）

5. 更多格式支持

- PDF：引入 pdf-extract 或 lopdf crate，提取文本后复用现有渲染管线
- FB2：XML 格式，复用 quick-xml，实现成本较低
- MOBI/KF8：较复杂，可考虑 mobi crate 或自研解析

6. 书架分类与标签

- 支持自定义分组（"文学"、"技术"、"待读"）
- library_item.rs 增加 tags: Vec<String> 字段
- 书架页增加侧边栏标签过滤

7. OPDS 在线书库

- 支持 OPDS 1.x/2.0 协议浏览远程书库
- 可直接从 Calibre Web、Project Gutenberg 等源导入
- 新增独立页面 + 网络层

8. 多窗口 / 分屏阅读

- 同时打开两本书对比阅读
- Tauri 2 支持多窗口，需调整 CoreState 管理

  ---
长期（架构级改动，产品跃升）

9. 数据同步

- 基于文件的同步（iCloud/Dropix 目录同步 JSON 文件）
- 或自建轻量同步服务（WebSocket + 差量同步 CRDT）
- 同步进度、书签、高亮、设置

10. 插件系统

- 定义插件接口（自定义解析器、渲染器、TTS 引擎、主题）
- 前端用 iframe 沙箱加载，后端用动态库加载
- 社区可贡献格式支持和功能扩展

11. 移动端适配

- Tauri 2 支持 iOS/Android，但阅读器 UI 需大幅调整
- 触摸手势（滑动翻页、捏合缩放）
- 响应式布局重构
