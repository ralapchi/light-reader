# light-reader 性能/内存/简洁性优化方案

> 审查日期：2026-06-09
> 审查范围：Rust 后端 + React/TS 前端 + 架构层面
> 状态：待审核
> 共 34 项优化（高 9 / 中 15 / 低 16）

---

## 高优先级（直接影响用户体验）

### H1. Chapter 三重数据冗余 → 去掉 `content` 和 `paragraphs`

**现状：** 每个 Chapter 同时存储三个字段：
- `content: String` — 拼接全文，仅用于 word/char count 统计
- `paragraphs: Vec<Paragraph>` — 解析后段落列表
- `blocks: Vec<ChapterBlock>` — 渲染块（Paragraph/Heading/Quote 各持有完整 Paragraph 实例 + Image/Separator）

`blocks` 是 `paragraphs` 的严格超集：每个 Paragraph 完整地装在 ChapterBlock 的文本变体中，所有字段（links、indent_level、source_line_hint、index）原样保留。`paragraphs` 的 4 处使用场景（TTS 分段、全文搜索、搜索进度、书签摘要）都只读 `text` 和 `index`，可从 blocks 过滤得到。

**方案：**
- 从 `Chapter` 结构体中移除 `content` 和 `paragraphs` 两个字段
- 新增 `word_count: usize` 和 `char_count: usize`，在构建 Chapter 时从 blocks 计算
- 在 `ChapterBlock` 上提供辅助方法 `text_paragraphs() -> impl Iterator<Item = &Paragraph>` 过滤文本变体，方便 TTS/搜索等场景使用

**改动文件：**
- `src/domain/chapter.rs` — 移除 `content`/`paragraphs`，新增统计字段，添加辅助方法
- `src/domain/chapter_builder.rs` — 构建时计算统计值，不再设置 `paragraphs`
- `src/parser/epub.rs` — 适配
- `src/parser/txt.rs` — 适配
- `src/services/reader_service_impl.rs` — 适配
- `src/tauri_api/commands/tts.rs:224` — 改为从 blocks 提取文本块
- `src/tauri_api/commands/bookmark.rs:22-53,114` — 改为遍历 blocks 文本变体

**验证：** 编译通过 + 现有解析器测试通过 + 前端字数显示正确 + TTS/搜索/书签功能正常

---

### H2. TtsSession 持有整个 Book → 仅保留当前章节文本

**现状：** `TtsSession.book: Option<Book>` 存储了完整的 Book 对象（含所有章节的 blocks），直到打开新书或应用退出。TTS 播放实际只需要当前朗读章节的段落文本列表。

**方案：**
- 将 `TtsSession.book: Option<Book>` 替换为 `segments: Vec<String>`（当前章节的纯文本段落）
- 在 `tts_start` 时只提取当前章节的文本段落传入
- 打开新书时清理旧数据

**改动文件：**
- `src/tauri_api/commands/mod.rs` — 修改 `TtsSession` 结构体
- `src/tauri_api/commands/tts.rs` — 适配 `tts_start` 逻辑

**验证：** TTS 播放功能正常 + 内存占用下降

---

### H3. TTS/Reader 锁竞争 → 分离状态

**现状：** 同一个 `Mutex<TtsSession>` 同时持有 `book`（Reader 用）和 TTS 相关字段。TTS 播放时持锁会阻塞 Reader 章节获取，反之亦然。

**方案：**
- 将 `TtsSession` 拆分为两个独立状态：
  - `ReaderState { book: Option<Book> }` — 由 `tauri::State` 独立管理
  - `TtsSession { segments, player, ... }` — 仅 TTS 相关
- 两个 Mutex 独立，互不阻塞

**改动文件：**
- `src/tauri_api/commands/mod.rs` — 拆分状态结构体
- `src/tauri_api/commands/reader.rs` — 使用 `ReaderState`
- `src/tauri_api/commands/tts.rs` — 仅使用 `TtsSession`
- `src/main.rs` — 注册两个 State

**验证：** TTS 播放中翻页不卡顿 + TTS 功能正常

---

### H4. search 全量克隆章节 → 逐章持锁搜索

**现状：** `search_in_book` 为释放 Mutex 锁，克隆了整本书所有章节的 `paragraphs` 数据。大书（数百章）会产生巨大内存分配。

**方案：**
- 改为在持锁状态下逐章搜索，每次只处理一个章节的引用
- 搜索结果立即收集，不持有全书数据

**改动文件：**
- `src/tauri_api/commands/bookmark.rs` — 重写搜索逻辑

**验证：** 搜索功能正常 + 大书搜索时内存不再飙升

---

### H5. 同步 command 阻塞 runtime → 改为 async + spawn_blocking

**现状：** `reader_chapter_image`、`reader_chapter_image_path`、`asset_read_file` 是同步 command，内部做 zip 解压 + base64 编码，可能阻塞 Tauri async runtime。

**方案：**
- 将这三个 command 改为 `async`
- 内部阻塞操作（zip 读取、文件读取、base64 编码）包装在 `spawn_blocking` 中
- 与 `reader_open_book` 保持一致的模式

**改动文件：**
- `src/tauri_api/commands/reader.rs` — `reader_chapter_image`、`reader_chapter_image_path`
- `src/tauri_api/commands/asset.rs` — `asset_read_file`

**验证：** 图片加载功能正常 + 大图片时 UI 不卡顿

---

### H6. 设置滑块每帧触发 IPC → debounce

**现状：** `useSettingsPersistence.ts` 中 `updateAndSave` 每次调用都执行 `settingsSave()` (Tauri IPC)。slider 使用 `onChange`，拖动过程中每帧都会触发。

**方案：**
- 对 `settingsSave` 添加 500ms debounce
- 或改用 slider 的 `onChangeCommitted` 事件（仅松手时触发）
- 推荐 debounce 方案：拖动过程中 UI 实时响应（local state），但 IPC 调用被合并

**改动文件：**
- `frontend/src/hooks/useSettingsPersistence.ts` — 添加 debounce

**验证：** 设置滑块拖动流畅 + 松手后设置正确持久化

---

### H7. handleLinkClick 未 memoize → 子组件链全量重渲染

**现状：** `ReaderPage.tsx:98-100` 中 `handleLinkClick` 是普通箭头函数，每次 ReaderPage 渲染都生成新引用，导致 `ReaderContent` 及其所有子组件即使包裹 `React.memo` 也会失效。

**方案：**
```tsx
const handleLinkClick = useCallback(
  (href: string) => navigation.navigateToHref(href),
  [navigation.navigateToHref]
)
```

**改动文件：**
- `frontend/src/pages/ReaderPage.tsx` — useCallback 包裹

**验证：** 阅读器中非相关状态变化不再触发内容重渲染

---

### H8. 全局零 React.memo → 阅读器组件全部 memo 化

**现状：** 整个代码库没有任何组件使用 `React.memo`。阅读器中每个 `ReaderBlock`（段落/标题/引用）都是独立组件但未 memo，当前章节几十到上百个 block 在任何状态变化时全部重渲染。

**方案：**
- `ReaderBlock` 用 `React.memo` 包裹（与 M5 合并，提升为高优先级）
- `SinglePageReaderContent` / `TwoPageReaderContent` 用 `React.memo` 包裹
- 配合 H7 的 `handleLinkClick` useCallback，确保 memo 生效

**改动文件：**
- `frontend/src/pages/reader/ReaderBlock.tsx` — React.memo
- `frontend/src/pages/reader/SinglePageReaderContent.tsx` — React.memo
- `frontend/src/pages/reader/TwoPageReaderContent.tsx` — React.memo

**验证：** React DevTools Profiler 确认 block 级别重渲染大幅减少

---

### H9. TeeWriter unwrap 可能导致日志 panic

**现状：** `main.rs:25-26` 中 `self.a.lock().unwrap().write_all(buf)` 和 `self.b.lock().unwrap()` 在 Mutex 中毒时会 panic，导致整个应用崩溃。TeeWriter 仅用于日志，不应因日志问题崩溃应用。

**方案：**
```rust
if let Ok(mut guard) = self.a.lock() { guard.write_all(buf).ok(); }
if let Ok(mut guard) = self.b.lock() { guard.write_all(buf).ok(); }
```

**改动文件：** `src/main.rs` — TeeWriter 的 write/flush 方法

**验证：** 日志功能正常 + 故意制造 Mutex 中毒时不崩溃

---

## 中优先级（代码质量 + 可维护性）

### M1. 多处不必要 clone → move 或传引用

**现状与方案：**

| 位置 | 问题 | 方案 |
|------|------|------|
| `reader_service_impl.rs:51` | chapters 整体 clone 后仅 move 进 Book | 去掉 clone |
| `tts.rs:224` | clone 整个 paragraphs 仅传给 `segment_chapter` | 改为传 `&paragraphs` |
| `tts.rs:241-242` | segments 被 clone 两次 | 先 move 进 guard，再从 guard clone 给轮询线程 |
| `bookmark.rs:131-137` | book_id/snippet/note 创建后不再使用却 clone | 直接 move |
| `bookmark.rs:141` | bm.push(bm.clone()) | 先 push 再从 items.last() 取值 |
| `progress_store.rs:54` | progress clone 用于序列化 | 改为 `ProgressFile<'a>` 持有引用 |
| `library_store.rs:38` | index clone 用于修改 version | 序列化时临时覆盖 version |

**改动文件：** 涉及上述 7 个文件

---

### M2. 设置保存双次 load → 合并操作

**现状：** `save_settings` 和 `save_tts_config` 各自调用 `settings_store::load()` 读取整个 settings.json，修改一个字段再全量写回。连续调用时文件被读写 4 次。

**方案：**
- 提供 `update_settings(f: impl FnOnce(&mut SettingsFile))` 函数，一次读取 + 修改 + 写入
- 或在 command 层合并：先在内存中修改所有字段，最后一次性写入

**改动文件：**
- `src/storage/settings_store.rs` — 新增 `update` 函数
- `src/services/settings_service_impl.rs` — 使用新接口
- `src/tauri_api/commands/settings.rs` — 合并调用

---

### M3. 频繁保存进度重复 load/save index → 内存缓存

**现状：** `reader_save_progress` 每次翻页都全量读写 `library_index.json`。

**方案：**
- 对 `library_index.json` 添加内存缓存（`OnceCell` 或 `tauri::State` 持有）
- 读取时优先从缓存取，写入时更新缓存并异步落盘
- 对 progress 文件改用紧凑 JSON 格式

**改动文件：**
- `src/storage/library_store.rs` — 添加缓存层
- `src/tauri_api/commands/reader.rs` — 适配

---

### M4. pretty-print JSON → 紧凑格式

**现状：** `write_json_atomic` 使用 `serde_json::to_string_pretty`，增加 30-50% 文件体积。

**方案：**
- 对频繁写入的文件（progress、library_index）改用 `serde_json::to_string`（紧凑格式）
- 对低频文件（settings、bookmarks）保留 pretty-print（便于手动调试）

**改动文件：**
- `src/storage/util.rs` — 新增 `write_json_atomic_compact` 函数
- `src/storage/progress_store.rs` — 使用紧凑格式
- `src/storage/library_store.rs` — 使用紧凑格式

---

### ~~M5. ReaderBlock 未 memo~~ → 已合并至 H8

此条目已提升并合并至高优先级 H8。

---

### M6. 封面并发加载无限制 → 限制并发

**现状：** 所有封面同时发起 IPC 请求。

**方案：**
- 复用现有的 `MAX_CONCURRENT = 4` 模式（参考 `useChapterImages.ts`）
- 或仅加载视口内可见封面（需配合虚拟列表）

**改动文件：** `frontend/src/pages/library/useCoverLoader.ts`

---

### M7. trait + impl 一对一 → 简化为直接 impl

**现状：** `LibraryService`、`SettingsService`、`AssetService` 等 trait 只有一个实现，增加间接性但无多态价值。

**方案：**
- 暂不改动（当前代码已稳定，改动风险大于收益）
- 记录为技术债务：后续新代码不再创建一对一 trait，需要测试替身时再抽取

**改动文件：** 无（记录决策）

---

### M8. Commands 层绕过 Service 直接访问 Storage → 统一

**现状：** `reader_save_progress` 直接调用 storage，`bookmark_*` 命令直接调用 `bookmark_store`，而 `library_*` 通过 Service。职责边界不一致。

**方案：**
- 将 `persist_progress`/`load_progress` 移入 `ReaderServiceImpl`
- 将 bookmark CRUD 移入一个 `BookmarkServiceImpl`（或并入 `LibraryServiceImpl`）
- Commands 层只做参数转换 + 调用 Service

**改动文件：**
- `src/services/reader_service_impl.rs` — 新增 progress 方法
- `src/tauri_api/commands/reader.rs` — 调用 Service
- `src/tauri_api/commands/bookmark.rs` — 调用 Service

---

### M9. 图片逐个 IPC → 批量接口

**现状：** 每张图片一次 Tauri invoke，一章 10 张图 = 10 次 IPC 往返。

**方案：**
- 新增 `reader_chapter_images(chapter_index) -> Vec<ChapterImageDto>` 批量接口
- 一次性返回所有图片的路径或 data URI
- 前端 `useChapterImages` 改为调用批量接口

**改动文件：**
- `src/tauri_api/commands/reader.rs` — 新增 command
- `frontend/src/hooks/useChapterImages.ts` — 改用批量接口

---

### M10. renderLinkedText 每次排序 → 提前排序

**现状：** 每个段落渲染时都对文本做 `Array.from(text)` + 链接排序。

**方案：** 将排序后的 links 作为 prop 传入，或在组件内用 `useMemo` 缓存。

**改动文件：** `frontend/src/pages/reader/ReaderBlock.tsx`

---

### M11. 异步操作无 AbortController → 组件卸载时取消请求

**现状：** 整个前端代码库未使用 `AbortController`。`useCoverLoader`、`useLibraryPage`、`useReaderSearch`、`useAdjacentChapterPreload` 等 hook 中的异步调用在组件卸载后仍可能 resolve 并尝试更新状态，造成资源浪费和潜在 stale state。

**方案：**
- 在 `useCoverLoader` 和 `useReaderSearch` 中使用 `AbortController`，cleanup 时 abort
- 其他低频调用（libraryList、readerGetChapter）可暂不处理

**改动文件：**
- `frontend/src/pages/library/useCoverLoader.ts`
- `frontend/src/hooks/useReaderSearch.ts`

---

### M12. 书架/目录列表未虚拟化 → 大数据量时 DOM 性能

**现状：**
- `LibraryPage.tsx:128-169` — `book-grid` 直接 `.map()` 渲染所有书籍卡片，书库数百本时所有卡片同时挂载 DOM
- `ReaderTocPanel.tsx:38-55` — 目录项直接 `.map()` 渲染，epub 目录可能有数百项

**方案：**
- 使用 `@tanstack/react-virtual` 做虚拟滚动（轻量，无额外依赖）
- 书架和目录面板各实现一个虚拟列表

**改动文件：**
- `frontend/src/pages/LibraryPage.tsx`
- `frontend/src/pages/reader/ReaderTocPanel.tsx`

---

### M13. EPUB 解析中 clone+clear → std::mem::take

**现状：** `epub.rs:385-583` 中 `paragraphs.push(current_para.clone())` 在每个段落结束时 clone 整个字符串，然后 `current_para.clear()` 重新开始。

**方案：** 用 `std::mem::take(&mut current_para)` 替代 `.clone()` + `.clear()`，避免重新分配。

**改动文件：** `src/parser/parsers/epub.rs`

---

### M14. to_ascii_lowercase 每次创建 Vec → eq_ignore_ascii_case

**现状：** `epub.rs:409,478` 中 `name.to_ascii_lowercase()` 对每个 XML 标签都创建一个新的 `Vec<u8>`。

**方案：** 用 `name.eq_ignore_ascii_case(b"p")` 替代 `name.to_ascii_lowercase() == b"p"`。

**改动文件：** `src/parser/parsers/epub.rs`

---

### M15. cover_path 每次遍历扩展名 → 使用缓存字段

**现状：** `asset_service_impl.rs:267-276` 中 `cover_path` 每次调用都对 6 种扩展名逐一检查文件是否存在。`item_to_dto` 每本书都会触发。

**方案：** 使用 `LibraryItem` 中已有的 `cover_cache_key` 字段缓存路径，避免每次查找。

**改动文件：** `src/services/asset_service_impl.rs`

---

## 低优先级（锦上添花）

### L1. EPUB 解析中重复代码 → 提取辅助函数

`epub.rs` 中 `Event::Start` 和 `Event::Empty` 对 `<a>` 标签和 `<br>`/`<hr>` 的处理逻辑完全重复（各约 30 行）。提取为 `handle_a_tag()` 和 `handle_line_break()` 辅助函数。

### L2. block_to_dto 三个变体分支重复 → 提取公共逻辑

`dto_convert.rs:46-104` 中 Paragraph/Heading/Quote 三个分支只有 tag name 不同，可提取公共的 link 转换逻辑。

### L3. continueReadingBooks 未 memoize

`useLibraryPage.ts:96` 中 `continueReadingBooks(books)` 每次渲染重新排序。改用 `useMemo`。

### L4. 死代码清理

`readerOptions.ts:41-57` 中 `LINE_HEIGHT_PRESETS`、`PARAGRAPH_SPACING_PRESETS`、`CONTENT_WIDTH_PRESETS` 未被引用。

### L5. TTS 缓存无容量上限

添加 LRU 淘汰机制或最大缓存大小限制（如 500MB），超过时自动清理最旧缓存。

### L6. item_to_dto 每次创建 AssetServiceImpl

`dto_convert.rs:8-10` 对 100 本书 = 100 次文件系统探测。批量处理或缓存 cover 路径。

### L7. renderLinkedText 中不必要的 Array.from

`ReaderBlock.tsx:17` 中 `Array.from(text)` 将字符串转为字符数组再 `chars.slice()`，但 `String.prototype.slice()` 直接可用。移除 `Array.from`，直接用字符串操作。

### L8. 4 个 ref 同步 effect 可合并

`useTwoPageNavigation.ts:41-49` 中 4 个独立 `useEffect` 仅将 state 同步到 ref（`spreadIndexRef`、`chapterSpreadStartsRef`、`flowChaptersRef`、`chapterRef`）。合并为一个 effect 减少不必要的中间渲染。

### L9. ReaderPage resize 监听无 debounce

`ReaderPage.tsx:47-49` 中 `window.addEventListener('resize', onResize)` 直接 `setWinW(window.innerWidth)`，无 debounce。添加 100ms debounce。

### L10. EPUB 解析中重复代码合并

| 位置 | 问题 |
|------|------|
| `epub.rs` Event::Start/Empty | `<a>` 标签 href/title 提取逻辑重复 ~30 行 |
| `epub.rs` Event::Start/Empty | `<br>`/`<hr>` 段落分割逻辑重复 |
| `epub.rs` Event::Start/Empty | `record_anchor` 调用重复 |

提取为 `handle_link_attrs()`、`handle_line_break()` 等辅助函数。

### L11. DTO 转换重复代码合并

| 位置 | 问题 |
|------|------|
| `dto_convert.rs:46-104` | Paragraph/Heading/Quote 三个分支几乎相同 |
| `bookmark.rs:59-92,144-153` | Bookmark → BookmarkDto 转换重复 3 处 |
| `dto_convert.rs:17-19,112-114` | format 字符串转换重复 |

提取公共转换函数；在 `BookFormat` 上实现 `Display`。

### L12. 领域模型不必要的 Serialize/Deserialize

Book、Chapter、Paragraph 等仅用于运行时的模型 derive 了 Serialize/Deserialize，但不会被持久化到文件。去掉可减少编译时间和二进制大小。（serde derive 开销很小，极低优先级）

### L13. 日志文件创建失败时 panic → fallback

`main.rs:49` 中 `.expect("无法创建日志文件")` 在日志目录不可写时 panic。改为 fallback 到 stderr-only 模式。

### L14. 安全的 unwrap 改为 if let

`txt.rs:116` 和 `epub.rs:1040` 中的 unwrap 逻辑上安全但不优雅，改为 `if let Some(...)` 更清晰。

### L15. content 字符串拼接优化

`chapter_builder.rs:76-80` 和 `epub.rs:1291` 中先 `collect::<Vec<_>>().join("\n\n")` 创建了中间 Vec。可用 `itertools::join` 或手动拼接避免。

### L16. 日志截断创建临时 String

`xiaomi_provider.rs:116` 中 `body_text.chars().take(300).collect::<String>()` 仅为日志截断。改用字节截断或 `char_indices` 避免分配。

---

## 执行计划

| 阶段 | 任务 | 预计改动量 |
|------|------|-----------|
| 阶段一 | H1-H3（后端内存优化） | 中等，涉及 domain 层重构 |
| 阶段二 | H4-H9（响应性 + 渲染 + 稳定性） | 中等，前后端联动 |
| 阶段三 | M1-M6 + M13-M15（代码质量 + 解析优化） | 较小，分散修改 |
| 阶段四 | M8-M12（架构一致性 + 虚拟化） | 中等，涉及 Service 层重组 |
| 阶段五 | L1-L16（锦上添花） | 较小 |

建议从阶段一和阶段二开始，这两部分对用户体验改善最明显。
