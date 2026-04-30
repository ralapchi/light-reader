# Reader Demo Product Spec

## 1. 文档信息

- 项目名称：`reader-demo`
- 当前技术栈：`Rust + eframe/egui`
- 当前产品形态：本地桌面小说阅读器
- 本期策略：继续使用 `egui`，将 UI 实现方式升级为“主题化 + 状态驱动 + 组件化”
- 文档状态：实施前规格书
- 文档目的：为后续开发提供统一的数据模型、UI 架构、状态流、持久化设计和迁移顺序

## 2. 项目背景

当前项目已经具备以下基础能力：

- 可打开本地 `EPUB` 文件并进行章节阅读
- 已具备基础 `TXT` 解析支持
- 当前 UI 已拆分为目录、工具栏、正文三个区域
- 解析层已有工厂模式基础，适合继续扩展

当前主要问题：

- 应用状态过于扁平，尚不足以支撑搜索、书签、设置、最近阅读、进度恢复等功能
- UI 虽已模块化，但仍偏向原型结构，缺少统一主题、组件状态边界和完整交互流
- 解析层输出仍较轻，后续高级功能缺少足够的数据支撑
- 设置、进度、书签、最近阅读尚未形成稳定的本地持久化方案

## 3. 产品目标

### 3.1 核心目标

1. 将项目从“可读取 EPUB/TXT 的原型”升级为“可持续使用的本地阅读器”
2. 保持 `Rust + egui` 技术路线，降低当前迭代风险
3. 建立统一领域模型和本地存储结构，支持后续功能持续叠加
4. 将 UI 升级为主题化、状态驱动、组件化架构
5. 为书签、搜索、最近阅读、进度恢复、设置面板等功能提供稳定底座

### 3.2 用户体验目标

1. 打开一本书后能快速进入阅读
2. 再次打开应用时能自动回到上次位置
3. 调整阅读样式时有即时反馈
4. 搜索、目录跳转、书签操作自然顺畅
5. 空状态、错误状态、正常阅读状态都清晰友好

### 3.3 成功标准

1. 能稳定打开常见 `EPUB/TXT`
2. 重启后自动恢复阅读位置
3. 能查看并跳转目录
4. 能进行当前章节和全书搜索
5. 能添加、删除、跳转书签
6. 能保存并恢复主题与阅读设置
7. UI 看起来是完整桌面应用，而不是仅供验证的 demo

## 4. 非目标

- 本期不做云同步
- 本期不做登录账号系统
- 本期不做多人协作或跨设备同步
- 本期不引入 WebView，不迁移到 Tauri
- 本期不改用 Slint 或其他 UI 框架
- 本期不实现复杂批注系统
- 本期不支持 PDF/MOBI 的正式实现，只做结构预留

## 5. 关键设计决策

### 5.1 UI 路线决策

继续使用 `egui`，但不再以“每个区域直接拼 UI”方式实现，而改为：

- 以 `AppState` 为单一状态源
- 以 `Action` 为统一交互入口
- 以 `ThemeTokens` 统一所有视觉参数
- 以 `Screen / Panel / Widget` 三级组件结构拆分界面

### 5.2 阅读模型决策

统一使用“书籍 -> 章节 -> 段落”的数据模型，不让 `EPUB` 和 `TXT` 在上层 UI 逻辑中分叉。

### 5.3 存储决策

采用本地文件持久化方案，使用 `JSON` 保存设置、进度、最近阅读和书签，便于调试和迁移。

### 5.4 状态管理决策

使用单向数据流：

`User Input -> Action -> Reducer / Handler -> AppState Update -> UI Render`

不在零散组件中持有关键业务状态，不让组件直接操作解析或存储逻辑。

## 6. 用户场景

1. 用户点击“打开书籍”，选择一个 `EPUB/TXT` 文件并开始阅读
2. 用户使用目录面板跳转到指定章节
3. 用户调整字体、行距、正文宽度、主题颜色获得更舒适的阅读体验
4. 用户关闭应用后再次打开，自动恢复到上次阅读位置
5. 用户按关键词搜索当前章节或全书内容
6. 用户对当前阅读位置添加书签并稍后返回
7. 用户从最近阅读列表重新打开书籍
8. 用户遇到损坏书籍或缺失文件时收到清晰提示

## 7. 总体架构

### 7.1 目标模块结构

建议逐步演进为以下结构：

```text
src/
  app/
    mod.rs
    state.rs
    actions.rs
    reducer.rs
    controller.rs
  domain/
    mod.rs
    book.rs
    chapter.rs
    toc.rs
    settings.rs
    progress.rs
    bookmark.rs
    search.rs
    error.rs
  parser/
    mod.rs
    parsers/
      base.rs
      epub.rs
      txt.rs
      factory.rs
    html_cleaner.rs
    toc_builder.rs
  storage/
    mod.rs
    settings_store.rs
    progress_store.rs
    bookmark_store.rs
    recent_store.rs
    paths.rs
  services/
    mod.rs
    book_loader.rs
    progress_service.rs
    bookmark_service.rs
    search_service.rs
    recent_service.rs
    theme_service.rs
  ui/
    mod.rs
    shell.rs
    theme.rs
    styles.rs
    widgets/
      mod.rs
      icon_button.rs
      segmented_tabs.rs
      empty_state.rs
      loading_state.rs
      error_state.rs
    panels/
      mod.rs
      top_bar.rs
      left_sidebar.rs
      reader_view.rs
      status_bar.rs
      settings_panel.rs
      search_panel.rs
      bookmarks_panel.rs
      recent_panel.rs
      toc_panel.rs
```

### 7.2 分层职责

- `domain`：纯数据结构与领域规则
- `parser`：负责文件内容解析和领域模型构建
- `storage`：负责本地持久化读写
- `services`：负责搜索、进度恢复、最近阅读管理等业务流程
- `app`：负责动作处理、状态更新和模块协调
- `ui`：只负责展示和输入收集

## 8. 领域模型设计

### 8.1 Book

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `id` | `String` | 是 | 书籍唯一标识，建议由规范化路径或内容摘要生成 |
| `source_path` | `PathBuf` | 是 | 原始文件路径 |
| `format` | `BookFormat` | 是 | 书籍格式 |
| `metadata` | `BookMetadata` | 是 | 元信息 |
| `toc` | `Vec<TocItem>` | 是 | 目录结构 |
| `chapters` | `Vec<Chapter>` | 是 | 章节数据 |
| `assets` | `BookAssets` | 是 | 封面等资源 |
| `load_info` | `BookLoadInfo` | 是 | 本次加载信息 |

### 8.2 BookFormat

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `variant` | `enum` | 是 | `Epub` / `Txt` / `ReservedPdf` / `ReservedMobi` |

### 8.3 BookMetadata

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `title` | `String` | 是 | 书名，若无法提取则回退为文件名 |
| `author` | `Option<String>` | 否 | 作者 |
| `language` | `Option<String>` | 否 | 语言 |
| `publisher` | `Option<String>` | 否 | 出版社 |
| `description` | `Option<String>` | 否 | 简介 |
| `identifier` | `Option<String>` | 否 | 原书唯一标识 |
| `series` | `Option<String>` | 否 | 丛书信息，首期可空 |
| `cover_title` | `Option<String>` | 否 | 封面标题或展示兜底文案 |
| `created_at` | `Option<String>` | 否 | 原书元数据创建时间 |
| `modified_at` | `Option<String>` | 否 | 原书元数据更新时间 |

### 8.4 BookAssets

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `cover_image_bytes` | `Option<Vec<u8>>` | 否 | 封面原始字节 |
| `cover_media_type` | `Option<String>` | 否 | 如 `image/jpeg` |
| `has_images` | `bool` | 是 | 是否检测到图像资源 |
| `embedded_styles_detected` | `bool` | 是 | 是否检测到内嵌样式或 CSS |

### 8.5 BookLoadInfo

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `parser_name` | `String` | 是 | 实际使用的解析器名称 |
| `parse_warnings` | `Vec<String>` | 是 | 非致命解析告警 |
| `chapter_count` | `usize` | 是 | 章节数 |
| `loaded_at` | `String` | 是 | 加载时间，ISO 8601 |
| `source_file_size` | `u64` | 是 | 文件大小 |
| `load_duration_ms` | `u64` | 是 | 加载耗时 |

### 8.6 Chapter

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `id` | `String` | 是 | 章节唯一标识 |
| `index` | `usize` | 是 | 顺序索引 |
| `title` | `String` | 是 | 展示标题 |
| `raw_title` | `Option<String>` | 否 | 原始标题 |
| `content` | `String` | 是 | 清洗后的纯文本全文 |
| `paragraphs` | `Vec<Paragraph>` | 是 | 段落结构 |
| `word_count` | `usize` | 是 | 单词数 |
| `char_count` | `usize` | 是 | 字符数 |
| `source_href` | `Option<String>` | 否 | EPUB 内部路径 |
| `anchor` | `Option<String>` | 否 | 对应锚点 |
| `warnings` | `Vec<String>` | 是 | 章节告警 |

### 8.7 Paragraph

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `index` | `usize` | 是 | 段落索引 |
| `text` | `String` | 是 | 段落文本 |
| `kind` | `ParagraphKind` | 是 | `Title` / `Subtitle` / `Body` / `Quote` / `Separator` |
| `indent_level` | `u8` | 是 | 缩进等级 |
| `source_line_hint` | `Option<usize>` | 否 | 调试或映射使用 |

### 8.8 TocItem

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `id` | `String` | 是 | 目录唯一标识 |
| `title` | `String` | 是 | 目录标题 |
| `chapter_index` | `Option<usize>` | 否 | 对应章节索引 |
| `href` | `Option<String>` | 否 | 原始 href |
| `depth` | `u8` | 是 | 层级深度 |
| `children` | `Vec<TocItem>` | 是 | 子目录，首期可为空 |
| `is_generated` | `bool` | 是 | 是否为回退生成目录 |

### 8.9 ReaderSettings

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `theme` | `ThemeKind` | 是 | 当前主题 |
| `font_family` | `String` | 是 | 字体族名称 |
| `font_size` | `f32` | 是 | 正文字号 |
| `line_height` | `f32` | 是 | 行距倍率 |
| `paragraph_spacing` | `f32` | 是 | 段间距 |
| `content_width` | `f32` | 是 | 正文最大宽度 |
| `side_margin` | `f32` | 是 | 左右留白 |
| `show_toc` | `bool` | 是 | 是否显示侧栏 |
| `toc_width` | `f32` | 是 | 侧栏宽度 |
| `reading_mode` | `ReadingMode` | 是 | `ChapterScroll` / `ReservedPaged` |
| `auto_save_progress` | `bool` | 是 | 自动保存进度 |
| `show_status_bar` | `bool` | 是 | 显示底部状态栏 |
| `show_chapter_progress` | `bool` | 是 | 显示章节进度 |
| `smooth_scroll` | `bool` | 是 | 平滑滚动开关 |
| `open_last_book_on_startup` | `bool` | 是 | 启动时恢复最近阅读 |
| `restore_last_position` | `bool` | 是 | 打开书籍时恢复进度 |
| `window_padding` | `f32` | 是 | 主阅读区域内边距 |

### 8.10 ThemeKind

| 取值 | 说明 |
|---|---|
| `Light` | 浅色主题 |
| `Dark` | 深色主题 |
| `Sepia` | 护眼棕色系 |
| `Paper` | 纸张风格 |
| `Custom` | 自定义主题 |

### 8.11 ThemeConfig

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `name` | `String` | 是 | 主题名称 |
| `colors` | `ThemeColors` | 是 | 全部颜色令牌 |
| `spacing` | `ThemeSpacing` | 是 | 间距令牌 |
| `typography` | `ThemeTypography` | 是 | 字体令牌 |
| `radius` | `ThemeRadius` | 是 | 圆角令牌 |
| `shadow` | `ThemeShadow` | 是 | 阴影令牌 |
| `panel` | `ThemePanel` | 是 | 面板令牌 |

### 8.12 ThemeColors

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `window_bg` | `ColorValue` | 是 | 应用背景 |
| `panel_bg` | `ColorValue` | 是 | 面板背景 |
| `panel_bg_muted` | `ColorValue` | 是 | 次级面板背景 |
| `reader_bg` | `ColorValue` | 是 | 正文背景 |
| `text_primary` | `ColorValue` | 是 | 主文本色 |
| `text_secondary` | `ColorValue` | 是 | 次文本色 |
| `text_muted` | `ColorValue` | 是 | 弱化文本色 |
| `accent` | `ColorValue` | 是 | 强调色 |
| `accent_hover` | `ColorValue` | 是 | 强调 hover 色 |
| `accent_pressed` | `ColorValue` | 是 | 强调按下色 |
| `border_subtle` | `ColorValue` | 是 | 弱边框 |
| `border_strong` | `ColorValue` | 是 | 强边框 |
| `selection_bg` | `ColorValue` | 是 | 选中背景 |
| `selection_text` | `ColorValue` | 是 | 选中文本 |
| `success` | `ColorValue` | 是 | 成功提示 |
| `warning` | `ColorValue` | 是 | 警告提示 |
| `danger` | `ColorValue` | 是 | 错误提示 |
| `focus_ring` | `ColorValue` | 是 | 焦点边框 |

### 8.13 ThemeSpacing

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `xxs` | `f32` | 是 | 极小间距 |
| `xs` | `f32` | 是 | 很小间距 |
| `sm` | `f32` | 是 | 小间距 |
| `md` | `f32` | 是 | 中间距 |
| `lg` | `f32` | 是 | 大间距 |
| `xl` | `f32` | 是 | 超大间距 |
| `reader_top_padding` | `f32` | 是 | 阅读区顶部留白 |
| `paragraph_gap` | `f32` | 是 | 段落间距 |
| `panel_gap` | `f32` | 是 | 面板间距 |

### 8.14 ThemeTypography

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `font_family_ui` | `String` | 是 | UI 字体 |
| `font_family_reader` | `String` | 是 | 阅读字体 |
| `title_size` | `f32` | 是 | 页面标题字号 |
| `section_title_size` | `f32` | 是 | 分区标题字号 |
| `body_size` | `f32` | 是 | 正文字号 |
| `caption_size` | `f32` | 是 | 注释字号 |
| `toolbar_size` | `f32` | 是 | 工具栏字号 |
| `line_height` | `f32` | 是 | 正文行高 |

### 8.15 ThemeRadius

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `button` | `f32` | 是 | 按钮圆角 |
| `panel` | `f32` | 是 | 面板圆角 |
| `card` | `f32` | 是 | 卡片圆角 |
| `input` | `f32` | 是 | 输入框圆角 |

### 8.16 ThemeShadow

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `panel_blur` | `f32` | 是 | 面板阴影强度 |
| `panel_alpha` | `f32` | 是 | 面板阴影透明度 |
| `floating_blur` | `f32` | 是 | 浮层阴影强度 |

### 8.17 ThemePanel

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `top_bar_height` | `f32` | 是 | 顶部栏高度 |
| `status_bar_height` | `f32` | 是 | 状态栏高度 |
| `sidebar_min_width` | `f32` | 是 | 侧栏最小宽度 |
| `sidebar_default_width` | `f32` | 是 | 侧栏默认宽度 |
| `sidebar_max_width` | `f32` | 是 | 侧栏最大宽度 |
| `content_max_width` | `f32` | 是 | 正文最大宽度 |

### 8.18 ReadingProgress

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `book_id` | `String` | 是 | 关联书籍 |
| `chapter_index` | `usize` | 是 | 当前章节索引 |
| `paragraph_index` | `Option<usize>` | 否 | 当前段落位置 |
| `scroll_offset` | `f32` | 是 | 当前滚动偏移 |
| `progress_percent` | `f32` | 是 | 全书阅读进度 |
| `last_read_at` | `String` | 是 | 最后阅读时间 |
| `session_read_seconds` | `u64` | 是 | 当前会话阅读时长 |
| `total_read_seconds` | `u64` | 是 | 历史累计阅读时长 |

### 8.19 Bookmark

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `id` | `String` | 是 | 书签 ID |
| `book_id` | `String` | 是 | 关联书籍 ID |
| `chapter_index` | `usize` | 是 | 所在章节 |
| `paragraph_index` | `Option<usize>` | 否 | 所在段落 |
| `title` | `String` | 是 | 书签标题 |
| `snippet` | `String` | 是 | 摘要内容 |
| `created_at` | `String` | 是 | 创建时间 |
| `note` | `Option<String>` | 否 | 附加备注，首期保留 |

### 8.20 SearchQuery

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `keyword` | `String` | 是 | 搜索词 |
| `case_sensitive` | `bool` | 是 | 是否区分大小写 |
| `scope` | `SearchScope` | 是 | 当前章节或全书 |

### 8.21 SearchResult

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `book_id` | `String` | 是 | 关联书籍 |
| `chapter_index` | `usize` | 是 | 命中章节 |
| `paragraph_index` | `usize` | 是 | 命中段落 |
| `match_start` | `usize` | 是 | 起始字符位置 |
| `match_end` | `usize` | 是 | 结束字符位置 |
| `chapter_title` | `String` | 是 | 章节标题 |
| `snippet` | `String` | 是 | 命中片段 |
| `score` | `f32` | 是 | 排序评分 |

### 8.22 RecentBookItem

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `book_id` | `String` | 是 | 书籍 ID |
| `title` | `String` | 是 | 展示标题 |
| `author` | `Option<String>` | 否 | 作者 |
| `source_path` | `String` | 是 | 文件路径 |
| `format` | `String` | 是 | 展示格式 |
| `last_opened_at` | `String` | 是 | 上次打开时间 |
| `last_progress_percent` | `f32` | 是 | 上次阅读进度 |
| `cover_cached` | `bool` | 是 | 是否缓存封面 |
| `is_missing` | `bool` | 是 | 文件是否丢失 |

### 8.23 AppState

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `current_book` | `Option<Book>` | 否 | 当前书籍 |
| `reader_settings` | `ReaderSettings` | 是 | 阅读设置 |
| `reading_progress` | `Option<ReadingProgress>` | 否 | 当前进度 |
| `recent_books` | `Vec<RecentBookItem>` | 是 | 最近阅读 |
| `bookmarks` | `Vec<Bookmark>` | 是 | 当前书书签缓存 |
| `search_state` | `SearchState` | 是 | 搜索状态 |
| `ui_state` | `UiState` | 是 | UI 状态 |
| `status_message` | `String` | 是 | 状态提示 |
| `last_error` | `Option<AppError>` | 否 | 最近错误 |

### 8.24 UiState

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `screen` | `ScreenKind` | 是 | 当前主界面状态 |
| `left_panel_tab` | `LeftPanelTab` | 是 | 左侧标签页 |
| `show_settings_panel` | `bool` | 是 | 是否显示设置 |
| `show_search_panel` | `bool` | 是 | 是否显示搜索 |
| `show_status_bar` | `bool` | 是 | 状态栏显隐 |
| `is_loading` | `bool` | 是 | 是否正在加载书籍 |
| `pending_open_path` | `Option<PathBuf>` | 否 | 待打开文件 |
| `focused_search_input` | `bool` | 是 | 是否聚焦搜索框 |
| `hovered_toc_item` | `Option<String>` | 否 | 当前悬浮目录项 |
| `selected_search_result` | `Option<usize>` | 否 | 当前高亮搜索结果 |
| `show_command_hint` | `bool` | 是 | 是否显示快捷键提示 |
| `window_size` | `Option<(f32, f32)>` | 否 | 窗口尺寸缓存 |
| `sidebar_collapsed` | `bool` | 是 | 左侧栏折叠状态 |

### 8.25 SearchState

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `current_query` | `Option<SearchQuery>` | 否 | 当前查询 |
| `results` | `Vec<SearchResult>` | 是 | 查询结果 |
| `selected_result_index` | `Option<usize>` | 否 | 当前选中结果 |
| `is_searching` | `bool` | 是 | 搜索进行中 |
| `last_search_at` | `Option<String>` | 否 | 最近搜索时间 |

### 8.26 AppError

| 字段 | 类型 | 必填 | 说明 |
|---|---|---:|---|
| `code` | `String` | 是 | 错误码 |
| `message` | `String` | 是 | 用户可读错误信息 |
| `detail` | `Option<String>` | 否 | 调试细节 |
| `recoverable` | `bool` | 是 | 是否可恢复 |

## 9. 本地存储设计

### 9.1 存储目录

建议目录：`$APP_DATA/reader-demo/`

```text
reader-demo/
  settings.json
  recent_books.json
  progress/
    {book_id}.json
  bookmarks/
    {book_id}.json
  cache/
    covers/
      {book_id}.bin
  logs/
```

### 9.2 settings.json

用途：

- 全局主题与阅读设置
- 窗口尺寸与布局设置
- 启动行为设置

字段：

- `version`
- `reader_settings`
- `window_size`
- `window_pos`
- `last_opened_book_id`

### 9.3 recent_books.json

用途：

- 最近阅读列表
- 记录最近打开顺序和阅读进度摘要

字段：

- `version`
- `items: Vec<RecentBookItem>`

### 9.4 progress/{book_id}.json

用途：

- 记录单本书阅读定位

字段：

- `version`
- `progress: ReadingProgress`

### 9.5 bookmarks/{book_id}.json

用途：

- 记录单本书全部书签

字段：

- `version`
- `book_id`
- `items: Vec<Bookmark>`

### 9.6 持久化规则

- 启动时读取 `settings.json`
- 打开书籍成功后读取该书 `progress` 与 `bookmarks`
- 章节切换后保存进度
- 应用退出前保存当前进度和设置
- 书签增删后立即保存
- 最近阅读变更后立即保存
- 保存失败不阻塞主流程，但要更新状态栏并写日志

## 10. 解析层规格

### 10.1 统一输出要求

- 所有解析器最终都返回统一 `Book`
- 章节必须填充 `paragraphs`
- 目录必须映射到 `chapter_index` 或至少保留 `href`
- 元信息不足时允许回退，但必须给出合理兜底值

### 10.2 EPUB 解析要求

1. 支持 `META-INF/container.xml` 读取
2. 支持读取 `opf` 中的 `manifest` 和 `spine`
3. 优先解析 `nav.xhtml` 目录，其次回退 `ncx`
4. 解析 `dc:title`、`dc:creator`、`dc:language`、`dc:identifier`
5. 提取封面资源
6. HTML 文本清洗处理以下标签：
   - `p`
   - `div`
   - `br`
   - `h1-h6`
   - `blockquote`
   - `li`
   - `hr`
7. 对缺失章节、空章节、错误 href 记录非致命告警

### 10.3 TXT 解析要求

1. 默认支持全文单章节加载
2. 支持按常见中文章回规则自动切章
3. 支持按空行切分段落
4. 标题段落自动识别为 `ParagraphKind::Title`
5. 若编码失败，需要返回明确错误而不是空内容

## 11. UI 实现方案

### 11.1 UI 总体原则

1. 组件不直接读写磁盘
2. 组件不直接调用解析器
3. 组件不直接持有核心业务状态
4. 所有核心操作通过 `Action` 进入应用层
5. 所有视觉样式来自主题令牌，而不是散落在各文件中的魔法数

### 11.2 页面结构

应用实际只保留一个主窗口，但内部按页面状态管理：

- `EmptyLibraryScreen`
  - 启动后未打开任何书时展示
- `LoadingBookScreen`
  - 正在解析或恢复书籍时展示
- `ReaderScreen`
  - 正常阅读状态
- `ErrorScreen`
  - 关键错误状态

### 11.3 主窗口布局

```text
AppShell
  TopBar
  BodyArea
    LeftSidebar
    ReaderView
    OverlayPanels
      SearchPanel
      SettingsPanel
      BookmarksPanel
      RecentPanel
  StatusBar
```

### 11.4 组件树

```text
AppShell
  TopBar
    OpenBookButton
    RecentButton
    SearchButton
    PrevChapterButton
    ChapterProgressLabel
    NextChapterButton
    ThemeSwitcher
    BookmarkButton
    SettingsButton
    StatusInlineMessage
  LeftSidebar
    SidebarHeader
      BookIdentityBlock
      CollapseButton
    SidebarTabs
      TocTab
      BookmarksTab
      RecentTab
    SidebarContent
      TocPanel
        TocList
          TocRow
      BookmarksPanel
        BookmarkList
          BookmarkRow
      RecentPanel
        RecentList
          RecentBookRow
  ReaderView
    ReaderViewport
      ChapterHeader
      ParagraphList
        ParagraphBlock
      ChapterEndSpacer
    ReaderOverlay
      SearchHighlightLayer
      InlineToast
  SearchPanel
    SearchInput
    SearchScopeSegmentedControl
    SearchResultList
      SearchResultRow
  SettingsPanel
    ThemeSection
    TypographySection
    LayoutSection
    BehaviorSection
    RestoreDefaultsButton
  StatusBar
    ReadingPercent
    ChapterPosition
    WordCountHint
    LastActionMessage
```

### 11.5 组件职责

#### AppShell

- 负责整体布局
- 根据 `ScreenKind` 决定显示何种主内容
- 将当前主题应用到所有子组件

#### TopBar

- 承载高频操作
- 只发出 Action，不直接修改书籍或存储

#### LeftSidebar

- 负责目录、书签、最近阅读切换
- 由 `UiState.left_panel_tab` 驱动

#### ReaderView

- 负责正文显示与滚动行为
- 负责段落视觉排版
- 负责基于阅读位置更新进度

#### SearchPanel

- 负责搜索输入、范围切换、结果列表
- 通过 `SearchState` 驱动

#### SettingsPanel

- 负责修改 `ReaderSettings`
- 所有调整即时作用于 UI

#### StatusBar

- 展示低干扰状态信息
- 允许通过设置关闭

## 12. 主题系统设计

### 12.1 目标

- 将颜色、间距、字号、圆角、面板尺寸统一抽为令牌
- 保证主题切换仅影响配置，不影响业务层
- 让阅读区与工具区拥有相同的视觉语言

### 12.2 主题变量分层

1. 颜色令牌：定义所有颜色
2. 排版令牌：定义字体和字号体系
3. 间距令牌：定义内边距、段间距、组件间距
4. 面板令牌：定义侧栏宽度、工具栏高度、正文最大宽度
5. 状态令牌：定义 hover、selected、disabled、focus 的视觉反馈

### 12.3 首期建议主题

#### Light

- 主背景偏暖白
- 阅读区白底
- 字体深灰
- 强调色偏蓝灰或墨绿

#### Dark

- 主背景深灰蓝
- 阅读区深色但不纯黑
- 文本高对比但避免刺眼白

#### Sepia

- 阅读区浅棕纸张色
- 目录区与工具栏采用更深一级棕灰
- 强调色偏墨绿或酒红

#### Paper

- 类纸张阅读主题
- 阅读区背景略带暖感
- 标题与正文层次更柔和

### 12.4 egui 落地方式

- 使用一个 `ThemeConfig` 结构体生成 `egui::Visuals`
- 在应用启动时和主题切换时统一设置 `ctx.set_visuals(...)`
- 正文排版层额外使用 `ThemeTypography` 与 `ThemeSpacing`
- 不允许在组件内部硬编码颜色和尺寸，统一从主题对象读取

## 13. 状态驱动设计

### 13.1 核心原则

- `AppState` 是唯一真实状态
- `UI = f(AppState)`
- 组件只表达输入意图，不直接改业务状态

### 13.2 Action 设计

建议使用枚举定义：

```text
Action
  OpenBookDialogRequested
  OpenBookSelected(path)
  OpenBookSucceeded(book)
  OpenBookFailed(error)
  CloseBook
  RestoreLastSessionRequested
  ToggleSidebar
  SwitchLeftPanelTab(tab)
  GoToChapter(index)
  NextChapter
  PrevChapter
  UpdateScrollOffset(offset)
  RestoreProgress(progress)
  SaveProgressRequested
  AddBookmarkRequested
  RemoveBookmark(bookmark_id)
  JumpToBookmark(bookmark_id)
  SearchQueryChanged(query)
  SearchSubmitted
  SearchResultSelected(index)
  ClearSearch
  ToggleSearchPanel
  ToggleSettingsPanel
  ThemeChanged(theme_kind)
  ReaderSettingChanged(setting_key, value)
  RestoreDefaultSettings
  RecentBookSelected(book_id)
  RemoveRecentBook(book_id)
  DismissError
  StatusMessageTimedOut
```

### 13.3 状态更新规则

- 任何书籍打开动作先设置 `ui_state.is_loading = true`
- 书籍加载成功后更新：
  - `current_book`
  - `reading_progress`
  - `bookmarks`
  - `status_message`
  - `ui_state.screen = ReaderScreen`
- 书籍加载失败后更新：
  - `last_error`
  - `status_message`
  - `ui_state.screen = ErrorScreen`
- 搜索输入变化不立即触发 IO，只更新 `SearchState.current_query`
- 搜索提交时调用搜索服务并写回 `results`

## 14. 交互流设计

### 14.1 打开书籍

1. 用户点击“打开书籍”
2. `TopBar` 发出 `OpenBookDialogRequested`
3. 应用层调起文件选择
4. 用户选中文件后发出 `OpenBookSelected(path)`
5. 应用层进入 `LoadingBookScreen`
6. `BookLoaderService` 调用 `ParserFactory`
7. 解析成功后更新 `current_book`
8. 若存在进度且设置允许恢复，则读取 `ReadingProgress`
9. 读取书签和最近阅读记录
10. 切换到 `ReaderScreen`

### 14.2 切换章节

1. 用户点击目录项或上一章/下一章按钮
2. 发出 `GoToChapter(index)` 或 `PrevChapter` / `NextChapter`
3. 保存旧章节滚动偏移
4. 更新 `reading_progress.chapter_index`
5. 重置或恢复章节滚动
6. 更新状态栏信息
7. 触发延迟保存进度

### 14.3 搜索

1. 用户打开搜索面板
2. 输入关键词
3. 选择范围：当前章节 / 全书
4. 点击搜索或回车提交
5. `SearchService` 生成 `SearchResult`
6. 渲染结果列表
7. 用户点击结果后跳转到对应章节和段落
8. 阅读区高亮命中段落

### 14.4 添加书签

1. 用户点击书签按钮
2. 发出 `AddBookmarkRequested`
3. 应用层从当前 `chapter_index + paragraph_index + snippet` 构建 `Bookmark`
4. 更新本地书签列表
5. 写入书签存储
6. 状态栏提示“已添加书签”

### 14.5 修改设置

1. 用户打开设置面板
2. 调整字号、行距、主题等
3. 每次变更发出 `ReaderSettingChanged`
4. `AppState.reader_settings` 更新
5. UI 即时重绘
6. 延迟持久化到 `settings.json`

### 14.6 最近阅读

1. 用户打开最近阅读面板
2. 点击某项
3. 发出 `RecentBookSelected(book_id)`
4. 查找路径并重新打开
5. 若文件丢失则提示并允许移除记录

## 15. 页面与面板规格

### 15.1 EmptyLibraryScreen

用途：

- 第一次启动时的默认页面
- 当前无书籍可显示时的主页面

内容：

- 应用名
- 简短提示
- “打开书籍”主按钮
- 最近阅读入口
- 支持格式提示

### 15.2 LoadingBookScreen

用途：

- 解析 EPUB/TXT 时防止界面静止无反馈

内容：

- 加载中标题
- 当前文件名
- 进度提示文案

### 15.3 ErrorScreen

用途：

- 文件损坏或关键错误时展示

内容：

- 错误标题
- 简短原因
- 重试按钮
- 重新打开按钮

### 15.4 ReaderScreen

用途：

- 主阅读界面

要求：

- 阅读区是视觉重心
- 侧栏可折叠
- 顶部工具栏不应抢占太多注意力
- 底部状态栏信息密度适中

## 16. 字体与排版策略

### 16.1 字体策略

- 启动时初始化字体，仅执行一次
- 区分 UI 字体与阅读字体
- 为不同平台准备回退字体列表
- 若指定字体不存在，自动回退系统可用字体

### 16.2 排版策略

- 正文使用固定最大宽度，避免超宽行
- 标题、正文、注释三层字号体系
- 标题居中或强化显示，但不喧宾夺主
- 正文段落保留明显呼吸感
- 段首缩进由段落类型与设置共同控制

## 17. 快捷键规格

| 快捷键 | 动作 |
|---|---|
| `Ctrl/Cmd + O` | 打开书籍 |
| `Ctrl/Cmd + F` | 打开搜索 |
| `Ctrl/Cmd + ,` | 打开设置 |
| `Ctrl/Cmd + B` | 添加书签 |
| `Left` / `PageUp` | 上一章 |
| `Right` / `PageDown` | 下一章 |
| `Esc` | 关闭搜索或设置面板 |
| `Ctrl/Cmd + 1` | 浅色主题 |
| `Ctrl/Cmd + 2` | 深色主题 |
| `Ctrl/Cmd + 3` | 护眼主题 |

## 18. 错误处理规格

### 18.1 错误码

- `FILE_NOT_FOUND`
- `FILE_OPEN_FAILED`
- `UNSUPPORTED_FORMAT`
- `EPUB_CONTAINER_MISSING`
- `EPUB_OPF_MISSING`
- `EPUB_TOC_PARSE_FAILED`
- `EPUB_METADATA_PARSE_FAILED`
- `TXT_DECODE_FAILED`
- `CONTENT_EMPTY`
- `SETTINGS_LOAD_FAILED`
- `SETTINGS_SAVE_FAILED`
- `PROGRESS_LOAD_FAILED`
- `PROGRESS_SAVE_FAILED`
- `BOOKMARK_SAVE_FAILED`
- `RECENT_SAVE_FAILED`

### 18.2 错误展示规则

- 可恢复错误：状态栏提示 + 允许继续操作
- 致命错误：切换到 `ErrorScreen`
- 调试细节不直接展示给用户，进入日志

## 19. 性能要求

- 常见 EPUB 打开目标：`< 1.5s`
- 章节切换目标：`< 100ms`
- 设置调整响应目标：肉眼无卡顿
- 全书搜索目标：中等体量小说 `< 500ms`
- 字体和主题应用不能在每帧重新初始化

## 20. 测试规格

### 20.1 单元测试

- EPUB 元信息解析
- EPUB 目录解析
- HTML 清洗为段落
- TXT 自动切章
- 设置读写
- 进度读写
- 书签读写
- 搜索结果定位

### 20.2 集成测试

- 打开书籍并恢复进度
- 切换主题后重启保持一致
- 添加书签后重启仍可读取
- 最近阅读去重与排序
- 错误书籍文件给出正确提示

### 20.3 回归样本

- 标准 EPUB
- 无封面 EPUB
- 无 nav 的 EPUB
- 目录层级复杂 EPUB
- 常规中文 TXT
- 超长 TXT
- 损坏 EPUB

## 21. 开发迁移顺序

### 阶段 1：状态与领域模型重构

目标：

- 从当前 `content + chapter_titles + current_page` 过渡到统一 `AppState`

任务：

1. 新建 `domain` 层模型
2. 定义 `AppState`、`UiState`、`SearchState`
3. 定义 `Action` 枚举
4. 统一错误模型
5. 为当前 UI 提供兼容适配层

完成标志：

- 旧逻辑仍可跑
- 新状态结构已可承载书籍和设置

### 阶段 2：主题系统与样式去硬编码

目标：

- 去掉散落在 UI 各处的颜色、间距和尺寸魔法数

任务：

1. 定义 `ThemeConfig` 及其子结构
2. 增加 `Light` / `Dark` / `Sepia` / `Paper`
3. 把字体初始化移出每帧 `update`
4. 将 `styles.rs` 升级为主题服务入口
5. 替换各组件中的硬编码样式

完成标志：

- 主题切换能全局生效
- 所有主要组件不再依赖硬编码颜色

### 阶段 3：UI 组件化重构

目标：

- 将现有 UI 升级为 `AppShell + Panels + Widgets`

任务：

1. 引入 `AppShell`
2. 重写 `TopBar`
3. 重写 `LeftSidebar`
4. 将目录、书签、最近阅读做成标签页结构
5. 将 `ReaderView` 从简单文本展示升级为段落组件列表
6. 增加 `StatusBar`

完成标志：

- 主页面结构清晰
- 各组件职责稳定

### 阶段 4：持久化与阅读行为

目标：

- 建立设置、进度、书签、最近阅读完整链路

任务：

1. 增加 `storage` 层
2. 实现设置读写
3. 实现进度恢复
4. 实现书签读写
5. 实现最近阅读读写
6. 增加章节切换时的进度保存

完成标志：

- 关闭重开后能恢复主要状态

### 阶段 5：搜索与高级交互

目标：

- 完成阅读器常用增强功能

任务：

1. 实现搜索输入与搜索面板
2. 实现当前章节与全书搜索
3. 实现搜索结果跳转
4. 实现快捷键系统
5. 增加状态消息与错误提示

完成标志：

- 搜索、快捷键、跳转流程完整

### 阶段 6：解析增强与回归测试

目标：

- 提高数据质量和稳定性

任务：

1. EPUB 元信息提取
2. EPUB 目录增强
3. TXT 自动切章
4. 解析告警结构化
5. 增加样例与测试

完成标志：

- 常见输入都能稳定处理

## 22. 详细开发清单

### Epic A：架构重构

#### A1. 建立领域模型

- 新建 `Book`、`Chapter`、`Paragraph`、`TocItem`
- 新建 `ReaderSettings`、`ThemeConfig`
- 新建 `ReadingProgress`、`Bookmark`、`RecentBookItem`
- 新建 `AppError`

依赖：

- 无

输出：

- `domain` 层完整类型定义

#### A2. 建立 AppState 和 Action

- 定义 `AppState`
- 定义 `UiState`
- 定义 `SearchState`
- 定义 `Action` 枚举
- 设计 reducer / controller 边界

依赖：

- A1

输出：

- 状态驱动主框架

### Epic B：主题化

#### B1. 建立主题令牌系统

- 定义颜色、间距、字号、圆角、面板尺寸结构
- 定义默认主题值

依赖：

- A1

输出：

- 主题结构与默认令牌

#### B2. 主题应用到 egui

- 从 `ThemeConfig` 生成 `egui::Visuals`
- 更新面板、按钮、文本、边框样式

依赖：

- B1

输出：

- 主题可切换的 UI 基础

### Epic C：组件化 UI

#### C1. 建立 AppShell

- 负责窗口主布局
- 接入 `ScreenKind`

依赖：

- A2
- B2

输出：

- 主壳组件

#### C2. 重写 TopBar

- 拆分按钮和状态信息
- 统一事件分发

依赖：

- C1

输出：

- 高可维护顶栏

#### C3. 重写 LeftSidebar

- 标签页切换
- 目录面板
- 书签面板
- 最近阅读面板

依赖：

- C1

输出：

- 统一左侧栏

#### C4. 重写 ReaderView

- 段落级渲染
- 章节头部
- 空状态与错误态

依赖：

- C1
- A1

输出：

- 阅读视图主组件

### Epic D：持久化

#### D1. settings store

- 读取与写入全局设置

#### D2. progress store

- 单本书进度存取

#### D3. bookmark store

- 单本书书签存取

#### D4. recent store

- 最近阅读存取、去重、排序

### Epic E：行为功能

#### E1. 打开书籍流程

- 文件选择
- 解析
- 书籍写入状态
- 恢复进度

#### E2. 章节导航

- 上一章
- 下一章
- 目录跳转
- 滚动偏移恢复

#### E3. 搜索系统

- 当前章节搜索
- 全书搜索
- 结果跳转

#### E4. 书签系统

- 添加
- 删除
- 跳转

#### E5. 最近阅读

- 展示
- 重新打开
- 移除失效记录

### Epic F：解析增强

#### F1. EPUB 元信息

- title
- creator
- language
- identifier
- cover

#### F2. EPUB 目录

- nav
- ncx
- 标题回退

#### F3. TXT 自动切章

- 中文章回规则
- 全文回退

### Epic G：测试与稳定性

#### G1. 解析测试

- EPUB 样本
- TXT 样本

#### G2. 状态恢复测试

- 设置
- 进度
- 书签
- 最近阅读

#### G3. 搜索与交互测试

- 查询正确性
- 结果跳转正确性

## 23. Issue 列表建议

1. `SPEC-01` 建立领域模型与统一错误类型
2. `SPEC-02` 建立 `AppState` 和 `Action` 框架
3. `SPEC-03` 引入主题系统与默认主题
4. `SPEC-04` 将字体初始化改为启动时配置
5. `SPEC-05` 重构 `TopBar`
6. `SPEC-06` 重构 `LeftSidebar`
7. `SPEC-07` 重构 `ReaderView`
8. `SPEC-08` 增加 `StatusBar`
9. `SPEC-09` 实现设置持久化
10. `SPEC-10` 实现阅读进度持久化
11. `SPEC-11` 实现书签持久化
12. `SPEC-12` 实现最近阅读
13. `SPEC-13` 实现搜索功能
14. `SPEC-14` 实现快捷键系统
15. `SPEC-15` 增强 EPUB 元信息与目录
16. `SPEC-16` 实现 TXT 自动切章
17. `SPEC-17` 建立回归测试样本
18. `SPEC-18` 完成错误状态与空状态体系

## 24. 验收标准

- 项目仍基于 `egui`，但 UI 已具备主题切换能力
- 应用状态不再依赖分散的局部变量拼接
- 页面结构已升级为 `AppShell + Panels + Widgets`
- 所有核心交互都经过 `Action` 流转
- 设置、进度、书签、最近阅读均可持久化
- EPUB/TXT 最终走统一 `Book` 模型
- 搜索、书签、章节跳转、主题切换全部可用
- 主要状态包含空状态、加载状态、错误状态、正常状态

## 25. 实施建议

建议先完成“架构重构 + 主题系统 + UI 壳层重构”，再进入书签、搜索、最近阅读这些功能开发。原因是这三个基础项一旦稳住，后续功能可以直接落在新的状态和组件体系上，避免写一轮功能后再返工 UI 和状态结构。

建议具体顺序：

1. 先做领域模型和 `AppState`
2. 再做主题系统
3. 再做 `AppShell`、`TopBar`、`LeftSidebar`、`ReaderView`
4. 接着补持久化
5. 最后做搜索、书签、最近阅读和快捷键

这一顺序可以把返工成本压到最低，也最适合当前仓库的体量。
