# 代码优化整理方案

> 审查日期：2026-07-02
> 审查范围：Rust 后端全量代码 + React/TS 前端全量代码
> 前置文档：`OPTIMIZATION_PLAN.md`（2026-06-09）、`docs/refactor-plan.md`（2026-06-26）
> 状态：待审核

---

## 审查背景

前两轮方案中的结构性整改（epub.rs 拆分、reader.rs 拆分、OPF 解析去重、JSON 存储统一走数据库、HTML 设计稿移出 src）已落地。本轮在现有结构基础上，聚焦正确性、性能、可维护性和安全性维度的新发现问题，共 42 项（高 10 / 中 18 / 低 14）。

---

## 一、高优先级（正确性与性能风险）

### H1. `stable_book_id` 使用不保证稳定性的 `DefaultHasher`

**位置：** `src/domain/book.rs:32`、`src/parser/parsers/epub/epub_parser.rs:302-306,370-376`

**问题：** `DefaultHasher` 的文档明确说明其哈希值不保证跨 Rust 版本稳定。一旦 Rust 标准库内部更换 SipHash 密钥，所有已入库书籍的 ID 将改变，导致数据库中的进度、书签、标签等全部引用失效。

**方案：** 替换为固定算法。两种选择：
- 轻量方案：用固定密钥的 `SipHasher`（`std::hash::DefaultHasher` 的底层实现），通过 `BuildHasherDefault<SipHasher>` 创建，但 `SipHasher` 目前不是公开 API。
- 推荐方案：引入 `sha1` 或 `blake3` crate，对规范化路径做哈希。SHA-1 对此场景（非加密用途）足够且依赖轻。

**验证：** 现有书籍 ID 不变（需做迁移映射）或接受重新导入；`cargo test` 通过。

---

### H2. TXT 分段使用字节长度而非字符数

**位置：** `src/tts/segmenter.rs:34`

**问题：** `current_text.len() + para_text.len() + 1 > max_chars` 使用的是 UTF-8 字节长度，但 `max_chars` 的语义是字符数（小米 TTS Provider 限制 500 字符）。中文字符占 3 字节，实际分段仅约 166 字符，比 Provider 允许的短 3 倍，导致同等文本的 API 调用次数增加约 3 倍。

**方案：**
```rust
if !current_text.is_empty()
    && current_text.chars().count() + para_text.chars().count() + 1 > max_chars
{
```

注意：`chars().count()` 遍历字符串，但对 TTS 分段（非热路径）可接受。如需优化可维护累计 char_count 而非每次重算。

**验证：** `segmenter.rs` 的 6 个单元测试通过；中文文本分段长度接近 max_chars。

---

### H3. `tts_start` 在命令线程执行阻塞 HTTP 调用

**位置：** `src/tauri_api/commands/tts.rs:382`

**问题：** `tts_start` 是同步 `#[tauri::command]`，内部直接调用 `synthesize_and_play`（行 382），该方法执行阻塞 HTTP 请求（超时最长 60 秒）。这会阻塞 Tauri 异步 runtime，期间所有其他 command（翻页、设置保存等）无法响应。

**方案：** 将 `tts_start` 改为 `async`，首段合成放在 `spawn_blocking` 中：
```rust
#[tauri::command]
pub async fn tts_start(...) -> Result<(), String> {
    // ... 前置逻辑不变 ...
    let segment = segment.clone();
    let config = config.clone();
    let cache = cache.clone();
    let playback_tx = playback_tx.clone();
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        synthesize_and_play(&segment, &book_id, chapter_index, &config, &cache, &playback_tx, &app)
    }).await.map_err(|e| e.to_string())??;
    // ... 后续逻辑不变 ...
}
```

**验证：** TTS 启动时 UI 仍可响应翻页和设置操作。

---

### H4. `tts_max_text_length` 硬编码 200，忽略 Provider 声明的 500

**位置：** `src/tauri_api/commands/tts.rs:92`、`src/tts/xiaomi_provider.rs:163`

**问题：** `tts_max_text_length` 硬编码返回 200，而 `XiaomiTtsProvider::max_text_length()` 返回 500。`TtsProvider` trait 的 `max_text_length` 方法标注了 `#[allow(dead_code)]` 从未被调用。分段过短直接导致 API 调用次数增加 2.5 倍。

**方案：** 让 `tts_max_text_length` 根据当前 provider 动态查询。由于当前只有一个 provider，可简化为：
```rust
fn tts_max_text_length(config: &TtsConfig) -> usize {
    match config.provider {
        TtsProviderKind::Xiaomi => 500,
    }
}
```
或注册 provider 实例并调用 `provider.max_text_length()`。

**验证：** 分段长度接近 500 字符；TTS 合成功能正常。

---

### H5. `synthesize_blocking` 不检查缓存，预取浪费 API 调用

**位置：** `src/tts/synthesis_service.rs:70-89`、`src/tauri_api/commands/tts.rs:194-218`

**问题：** `synthesize_blocking` 静态方法每次创建新 Provider 实例并调用 `synthesize()`，完全不检查缓存。`prefetch_next_segment` 调用它预取下一段——如果该段已缓存（例如之前播放过），仍然发起新的 API 请求。

**方案：** 在 `synthesize_blocking` 中先查缓存，命中则直接返回：
```rust
pub fn synthesize_blocking(config, cache, request) -> Result<TtsResponse, TtsError> {
    let path = cache_path_for(config, request);
    if let Some(cached) = cache.read(&path) {
        return Ok(TtsResponse { audio_bytes: cached, ... });
    }
    // ... 创建 provider 并合成 ...
}
```

**验证：** 重复播放同一段章节时，不再产生额外 API 请求。

---

### H6. 大量 TXT 文件整体读入内存，存在 OOM 风险

**位置：** `src/parser/parsers/txt.rs:192`

**问题：** `file.read_to_string(&mut content_str)` 将整个文件读入内存。网络小说 TXT 文件常达数十甚至数百 MB，直接读入会导致内存飙升甚至 OOM 崩溃。

**方案：** 使用 `BufReader` 逐行读取，流式检测章节分隔符，避免全文驻留内存：
```rust
use std::io::{BufReader, BufRead};
let reader = BufReader::new(file);
let mut current_chapter = String::new();
for line in reader.lines() {
    let line = line?;
    if is_chapter_title(&line) {
        // flush current_chapter
        chapters.push(...);
        current_chapter.clear();
    } else {
        current_chapter.push_str(&line);
        current_chapter.push('\n');
    }
}
```

**验证：** 解析 100MB+ TXT 文件时内存占用稳定在 MB 级别；章节检测正确。

---

### H7. TXT 解析无编码检测，GBK/Big5 文件产生乱码

**位置：** `src/parser/parsers/txt.rs`

**问题：** 解析器假定 UTF-8 编码。大量中文 TXT 文件（尤其是旧资源）使用 GBK/GB18030/Big5 编码，读入后产生乱码或 `read_to_string` 失败。

**方案：**
- 检测 BOM 头（UTF-8 BOM / UTF-16 BOM）
- 无 BOM 时尝试 UTF-8 解码，失败则按 GBK 解码（引入 `encoding_rs` crate）
- 解码失败时返回带 `recoverable` 标记的 `AppError`

**验证：** GBK 编码的 TXT 文件正确解析为可读中文。

---

### H8. CSS 类名冲突：LibraryPage 与 LoadingPage 定义同名不同值样式

**位置：** `frontend/src/pages/LibraryPage.css` 与 `frontend/src/pages/LoadingPage.css`

**问题：** 以下类名在两个文件中重复定义且值不同，CSS 加载顺序决定最终生效的样式，属于潜在 bug：

| 类名 | LibraryPage.css | LoadingPage.css |
|------|-----------------|-----------------|
| `.book-title` | font-size: 12px | font-size: 22px |
| `.book-author` | font-size: 11px | font-size: 16px |
| `.cover-1` ~ `.cover-6` | 背景色定义 | 相同定义（冗余重复） |
| `.btn-secondary` | padding: 6px 16px | padding: 10px 24px |
| `@keyframes fadeIn` | opacity 变化 | translateY 12px + opacity |

**方案：**
- 将公共样式（`.book-title`、`.book-author`、`.cover-*`、`.btn-secondary`）提取到 `global.css` 或新建 `components/common-book.css`
- LibraryPage 和 LoadingPage 中的同名类改为更具体的选择器（如 `.library-book-title`、`.loading-book-title`）或通过父级 class 限定作用域
- `@keyframes fadeIn` 重命名为 `fadeInLibrary` / `fadeInLoading`

**验证：** 两个页面的样式互不影响；浏览器 DevTools 中无样式覆盖警告。

---

### H9. 7 个 Hook 使用 `useAppStore()` 无 selector，导致全量重渲染

**位置：**
- `frontend/src/pages/library/useLibraryPage.ts:23`
- `frontend/src/pages/loading/useLoadingPage.ts:17`
- `frontend/src/pages/reader/useChapterNavigation.ts:25-29`
- `frontend/src/pages/reader/useReadingProgress.ts:21`
- `frontend/src/pages/reader/useTtsReader.ts:18`
- `frontend/src/pages/reader/useReaderSearch.ts:11`
- `frontend/src/pages/reader/useBookmarks.ts:11`

**问题：** 这些 Hook 使用 `const { xxx, yyy } = useAppStore()` 解构模式，订阅整个 store。任何 store 变化（包括 TTS 状态、阅读器状态）都会触发这些组件重渲染。`useLibraryPage` 是最严重的情况：Library 页面在 TTS 播放进度更新时会反复重渲染。

**方案：** 每个需要的值单独使用 selector：
```ts
// 之前
const { books, setBooks, startOpening, setSidebarFooter } = useAppStore()

// 之后
const books = useAppStore(s => s.books)
const setBooks = useAppStore(s => s.setBooks)
const startOpening = useAppStore(s => s.startOpening)
const setSidebarFooter = useAppStore(s => s.setSidebarFooter)
```

Zustand 的 `set` 函数引用稳定，不会因 store 变化而产生新引用。

**验证：** React DevTools Profiler 确认 Library 页面在 TTS 状态变化时不再重渲染。

---

### H10. 可点击 `<div>` 缺少键盘可访问性

**位置：**
- `frontend/src/components/Sidebar.tsx` — 导航项为 `<div onClick>`
- `frontend/src/pages/LibraryPage.tsx` — 书籍卡片为 `<div onClick>`
- `frontend/src/pages/BookmarkPage.tsx` — 书签条目为 `<div onClick>`
- `frontend/src/pages/reader/ReaderTocPanel.tsx` — 目录项为 `<div onClick>`
- `frontend/src/pages/LoadingPage.tsx` — 返回按钮为 `<div onClick>`

**问题：** 所有可点击的 `<div>` 都没有 `role="button"`、`tabIndex`、`onKeyDown` 处理。键盘用户无法导航到这些元素，屏幕阅读器不识别为可操作控件。这违反 WCAG 2.1 Level A 的键盘可访问性要求。

**方案：** 优先改为语义化 `<button>` 或 `<a>` 元素。如需保持 `<div>`（例如卡片需要复杂布局），则添加：
```tsx
<div
  role="button"
  tabIndex={0}
  onClick={handler}
  onKeyDown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      handler(e)
    }
  }}
>
```

**验证：** 仅用键盘 Tab + Enter/Space 可操作所有导航和选择功能。

---

## 二、中优先级（代码质量与可维护性）

### M1. `epub_parser.rs::parse()` 仍达 380 行，职责混杂

**位置：** `src/parser/parsers/epub/epub_parser.rs:123-503`

**问题：** 虽然已拆分为 4 个子文件，但 `parse()` 方法本身仍是一个 380 行的巨型函数，同时处理：container.xml 解析、OPF manifest/spine/metadata 解析、TOC 解析（nav + ncx）、spine 遍历与 HTML 提取、图片资源处理、封面提取。

**方案：** 将 `parse()` 中的各阶段提取为独立方法（保持在同一文件/impl 内）：
- `parse_container_and_opf(&mut self) -> Result<(), AppError>`
- `parse_toc(&mut self) -> Result<(), AppError>`
- `build_chapters_from_spine(&mut self) -> Result<(), AppError>`
- `extract_cover(&mut self) -> Result<(), AppError>`
- `parse()` 编排这些方法

**验证：** `cargo test` 通过；解析结果不变。

---

### M2. EPUB 图片资源创建逻辑重复

**位置：** `src/parser/parsers/epub/epub_parser.rs:292-346`（内联图片）与 `354-410`（图片块）

**问题：** 两段代码几乎相同：解析图片路径、生成 asset_id（`DefaultHasher`）、查重 `!image_assets.iter().any(...)`、创建 `BookImageAsset`。任何修改需要同步两处。

**方案：** 提取为辅助方法：
```rust
fn get_or_create_image_asset(
    &mut self,
    img_src: &str,
    chapter_dir: &str,
    is_inline: bool,
) -> String // 返回 asset_id
```

**验证：** 图片提取功能正常；`parser/parsers/tests.rs` 通过。

---

### M3. 格式检测逻辑在 3 处重复

**位置：**
- `src/parser/parsers/factory.rs:26-29`
- `src/services/library_service_impl.rs:13-17`
- `src/services/reader_service_impl.rs:33-37`

**问题：** 三处独立使用 `path.ends_with(".epub")` / `path.ends_with(".txt")` 检测格式，且大小写敏感（`Book.EPUB` 不被识别）。

**方案：** 在 `BookFormat` 上实现 `from_path`：
```rust
impl BookFormat {
    pub fn from_path(path: &str) -> Option<Self> {
        let ext = std::path::Path::new(path).extension()?;
        match ext.to_ascii_lowercase().to_str()? {
            "epub" => Some(Self::Epub),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }
}
```
三处调用改为 `BookFormat::from_path(path)`。

**验证：** `.EPUB`、`.Epub` 等大小写变体正确识别。

---

### M4. HTML 标签匹配使用 `starts_with` 导致误匹配

**位置：** `src/parser/parsers/epub/epub_content.rs:306-308,317,149`

**问题：**
- `name.starts_with(&b"h1"[..])` 匹配 `h1` 但也匹配 `h1foo`
- `name.starts_with(&b"title"[..])` 匹配 `title` 但也匹配 `titlebar`
- `attr_name.starts_with(&b"style"[..])` 匹配 `style` 但也匹配 `styles`

这些误匹配可能对非标准 HTML 产生错误行为。

**方案：** 改用 `eq_ignore_ascii_case`：
```rust
name.eq_ignore_ascii_case(b"h1")
name.eq_ignore_ascii_case(b"title")
attr_name.eq_ignore_ascii_case(b"style")
```

**验证：** EPUB 解析测试通过；含非标准标签的 EPUB 不被误解析。

---

### M5. `epub_content.rs::extract_html_with_positions()` 达 258 行

**位置：** `src/parser/parsers/epub/epub_content.rs:25-283`

**问题：** 单个函数有 20+ 个可变局部变量，实现了一个复杂的状态机来解析 HTML。其中 `flush_para` 闭包接收 6 个可变引用参数，`handle_a_attrs` 接收 7 个。极难维护和测试。

**方案：** 将解析状态提取为一个 struct：
```rust
struct HtmlExtractorState {
    current_para: String,
    para_links: Vec<TextLink>,
    // ... 其他 20 个变量
}
impl HtmlExtractorState {
    fn flush_para(&mut self, ...) { ... }
    fn handle_a_attrs(&mut self, ...) { ... }
}
```
`extract_html_with_positions` 变为创建 `HtmlExtractorState` 并驱动事件循环。

**验证：** EPUB 内容提取结果不变；测试通过。

---

### M6. 多个 command 持锁期间执行 DB I/O

**位置：**
- `src/tauri_api/commands/reader_progress.rs:40-46` — `progress_state` 锁内调用 `db.progress().load()`
- `src/tauri_api/commands/reader_book.rs:111-118` — `progress_state` 锁内调用 `db.progress().load()`
- `src/tauri_api/commands/reader_book.rs:122-128` — `index_state` 锁内调用 `db.books().upsert()`

**问题：** 持锁期间执行 SQLite 查询，阻塞所有其他需要同一锁的 command。

**方案：** 先释放锁，执行 DB 操作，再短暂获取锁写入结果。或使用 `RwLock` 代替 `Mutex`，读操作不需要独占锁。

**验证：** 翻页和打开书籍时无卡顿。

---

### M7. `library_*` command 同步阻塞 Tauri runtime

**位置：** `src/tauri_api/commands/library.rs` — `library_list`、`library_import`、`library_remove`、`library_remove_batch`、`library_search`、`library_repair_path`、`library_cover`、`library_flush_index`

**问题：** 这些 command 都是同步函数，内部执行文件 I/O（`std::fs::read_dir`、`std::fs::remove_file` 等）和 DB 查询，阻塞 Tauri async runtime。

**方案：** 改为 `async fn`，I/O 操作放入 `spawn_blocking`。参考已有的 `reader_open_book` 模式。

**验证：** 书库操作期间 UI 响应正常。

---

### M8. TTS 自动推进使用 200ms 轮询，脆弱且延迟高

**位置：** `src/tauri_api/commands/tts.rs:240-293`

**问题：** `spawn_auto_advance_thread` 每 200ms 轮询 `is_playing_flag` 检测播放完成。问题：段落切换有 200ms 延迟；如果播放线程崩溃，轮询器永久空转；无法检测异常状态。

**方案：** 改为事件驱动——播放完成后通过 channel 通知：
```rust
// 在播放线程中，sink 结束后发送通知
let _ = completion_tx.send(());
```
自动推进线程等待 `completion_rx.recv()` 而非轮询。

**验证：** 段落切换无感知延迟；播放线程崩溃时能检测并通知前端。

---

### M9. 大量死代码：未使用的 Repo trait 和方法

**位置：**
- `src/storage/traits.rs` — `TagsRepo`、`SessionsRepo`、`AggregatesRepo` 整体标注 `#[allow(dead_code)]`（约 254 行）
- `src/storage/traits.rs` — `BooksRepo::get`、`search`、`update_progress`、`update_stats`（约 52 行）
- `src/storage/traits.rs` — `ProgressRepo::save`、`mark_dirty`、`flush_dirty`、`load_all`（约 45 行）
- `src/tts/synthesis_service.rs:92` — 实例方法 `synthesize()` 从未被调用（约 55 行）
- `src/tts/tts_provider.rs:47` — `TtsProvider` trait 的 `max_text_length`、`validate_config` 从未被调用
- `src/tts/player.rs:67` — `AudioPlayer::len` 从未被调用
- `src/tts/config.rs:52` — 空的 `impl TtsConfig {}` 块

**方案：**
- 对于统计功能相关的（TagsRepo、SessionsRepo、AggregatesRepo）：如果计划开发统计页，保留但在注释中标注 `// WIP: 统计功能待接入`；如果不计划开发，删除。
- 对于其他死代码：直接删除。
- `TtsProvider` trait 的 `max_text_length` 接入 H4 的修复后不再是死代码。

**验证：** `cargo check` 通过；无 warning。

---

### M10. 错误类型全量为 `String`，无结构化错误处理

**位置：** 全局 — 所有 `#[tauri::command]` 返回 `Result<T, String>`

**问题：** 所有 command 用 `.map_err(|e| e.to_string())` 将错误转为 String。前端无法区分错误类型（可恢复 vs 不可恢复、文件不存在 vs 权限不足），只能显示字符串。`AppError` 中的 `error_code` 和 `recoverable` 字段信息丢失。

**方案：**
- 定义 command 层错误 DTO：
  ```rust
  #[derive(Serialize)]
  struct CommandError {
      code: String,
      message: String,
      recoverable: bool,
  }
  ```
- command 返回 `Result<T, CommandError>`，Tauri 自动序列化为前端可解析的结构
- 前端按 `code` 分类处理（如 `FILE_NOT_FOUND` 显示重导入按钮）

**验证：** 前端能按 error code 分类处理错误。

---

### M11. SQLite schema 无 FOREIGN KEY 约束

**位置：** `src/storage/sqlite/schema.rs`

**问题：** 表之间通过 `book_id` 关联，但未定义 FOREIGN KEY 约束。删除书籍时靠代码级 cascade（`delete_book_cascade`），如果 cascade 失败或绕过，会产生孤儿记录。`connection.rs` 注释说"foreign keys enabled"但实际未执行 `PRAGMA foreign_keys=ON`。

**方案：**
- 在 schema 中添加 FOREIGN KEY 约束：
  ```sql
  -- bookmarks 表
  FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE,
  -- reading_progress 表
  FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE,
  ```
- 在 `connection.rs` 中添加 `PRAGMA foreign_keys=ON;`
- 保留代码级 cascade 作为双重保障

**验证：** 删除书籍后，数据库中无孤儿记录。

---

### M12. `search_in_book` 在循环内重复 `to_lowercase()`

**位置：** `src/tauri_api/commands/bookmark.rs:18-54`

**问题：** `query.to_lowercase()` 在内层循环中被调用（每个段落的每次比较都重新转换查询字符串），应提到循环外。同时 `BookSession` 锁在整个搜索过程中被持有。

**方案：**
```rust
let query_lower = query.to_lowercase();
// 循环内使用 query_lower
```
锁的持有：考虑将搜索结果收集后尽快释放锁，或克隆必要的文本数据后释放锁再搜索。

**验证：** 搜索功能正常；大书搜索性能改善。

---

### M13. `format!("{:?}", enum).to_lowercase()` 模式脆弱且重复

**位置：**
- `src/tauri_api/commands/dto_convert.rs:36,67,73,100,109` — `format!("{:?}", p.kind).to_lowercase()`
- `src/tauri_api/commands/dto_convert.rs:168` — `format!("{:?}", config.provider).to_lowercase()`
- `src/tts/synthesis_service.rs:16` — 同上

**问题：** 用 Debug 格式化获取枚举变体名称，再 lowercase。如果枚举变体被重命名，输出的字符串会静默改变，导致缓存路径失效或前端解析失败。

**方案：** 在枚举上实现 `Display` 或提供 `as_str()` 方法：
```rust
impl ParagraphKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading => "heading",
            Self::Quote => "quote",
        }
    }
}
```

**验证：** DTO 序列化结果不变；缓存路径不变。

---

### M14. EPUB 每次提取单张图片都重新打开 zip

**位置：** `src/services/asset_service_impl.rs:58-61`

**问题：** `extract_epub_image` 每次调用都打开 EPUB zip 文件并解析 zip 索引。一章 10 张内联图片 = 10 次文件打开和索引解析。

**方案：**
- 在 `BookSession` 或 `AssetService` 中缓存 `ZipArchive` 句柄
- 或在首次打开书籍时预提取所有图片到磁盘缓存

**验证：** 图片加载速度提升；多次请求同一图片不重复打开 zip。

---

### M15. Google Fonts `@import` 在 4 个 CSS 文件中重复

**位置：** `frontend/src/pages/ReaderPage.css:1`、`LibraryPage.css:1`、`BookmarkPage.css:1`、`LoadingPage.css:1`

**问题：** 相同的 `@import url('https://fonts.googleapis.com/...')` 在 4 个文件中重复。每次导航到新页面时浏览器可能重新请求（取决于缓存策略）。

**方案：** 删除 4 处 `@import`，在 `frontend/src/global.css` 中保留唯一一处。或更好：在 `index.html` 中用 `<link rel="preconnect">` + `<link rel="stylesheet">` 加载。

**验证：** 字体渲染正常；Network 面板中字体只加载一次。

---

### M16. `:root` CSS 变量定义在 `LibraryPage.css` 而非 `global.css`

**位置：** `frontend/src/pages/LibraryPage.css:3-18`

**问题：** 全局的亮色主题变量（`--bg`、`--surface`、`--text-primary` 等）定义在页面级 CSS 中。如果 LibraryPage 未来做代码分割，这些变量会消失。暗色变量在 `global.css` 中，亮色在 `LibraryPage.css` 中——主题变量分散两处。

**方案：** 将 `:root` 变量移到 `global.css`，与暗色变量放在一起。

**验证：** 所有页面主题变量一致。

---

### M17. 全局共享正则表达式 `INLINE_IMAGE_RE` 存在并发安全问题

**位置：** `frontend/src/pages/reader/ReaderBlock.tsx:27`、`frontend/src/pages/reader/readerUtils.ts:4`、`frontend/src/pages/reader/useChapterImages.ts:35`

**问题：** `INLINE_IMAGE_RE` 是模块级 `RegExp`，带 `g` flag。多处手动 `INLINE_IMAGE_RE.lastIndex = 0` 后使用。如果 React 并发模式下两个渲染同时调用，`lastIndex` 会互相覆盖。

**方案：** 改为每次使用时创建局部实例：
```ts
const re = /\u{E000}(.+?)\u{E001}/gu
```
或使用 `String.prototype.matchAll()` 传入正则字面量。

**验证：** 内联图片渲染正常。

---

### M18. `useTwoPageNavigation.ts`（399 行）过于复杂

**位置：** `frontend/src/pages/reader/useTwoPageNavigation.ts`

**问题：** 8 个 `useEffect`、4 个 `useRef`、3 个 `useState`，效果之间有隐式依赖关系（如 `needsNextSpreadRef` 在 `turnSpread` 中设置，在另一个 effect 中消费）。唯一的 `eslint-disable` 也在这个文件中。

**方案：** 拆分为更小的 hook：
- `useSpreadNavigation` — spread 索引管理和翻页逻辑
- `useSpreadKeyboard` — 键盘/滚轮事件处理
- `useSpreadPreload` — 章节预加载
- `useVisibleChapterSync` — 可见章节同步

每个 hook 200 行以内，效果间依赖通过明确的参数传递。

**验证：** 双页模式翻页、键盘导航、章节预加载功能正常。

---

## 三、低优先级（锦上添花）

### L1. 依赖版本过旧

**位置：** `Cargo.toml`

| 依赖 | 当前 | 建议 |
|------|------|------|
| `zip` | 0.6.6 | 0.7+（API 变更，需适配） |
| `env_logger` | 0.10 | 0.11+ |
| `dirs` | 5.0 | 6.0 |

`zip` 0.7 的 API 变更较大，建议先验证兼容性再升级。

---

### L2. 未使用的 Cargo feature flags

**位置：** `Cargo.toml:36-38`

`tts-aliyun`、`db-postgres`、`db-mysql` 三个 feature 声明但无对应代码。`storage/factory.rs` 有 `TODO: implement PostgresBackend` / `TODO: implement MysqlBackend`。如不计划实现，删除声明；如计划实现，标注 `# WIP`。

---

### L3. 前端未使用的 npm 依赖

**位置：** `frontend/package.json`

`@tauri-apps/plugin-fs` 和 `@tauri-apps/plugin-shell` 在 `src/` 中无任何 import。如确实未使用，移除以减小 `node_modules` 和锁文件体积。

---

### L4. `ReadingAggregates` 使用 `String` 作为 HashMap 键

**位置：** `src/domain/reading_aggregates.rs:9-13`

`daily_seconds: HashMap<String, u64>` 和 `hourly_seconds: HashMap<String, u64>` 用 String 存日期和小时。可分别用 `chrono::NaiveDate` 和 `u8`（0-23）代替，提高查找效率和类型安全。

---

### L5. `reader_get_progress` 返回 `Option` 而非 `Result`

**位置：** `src/tauri_api/commands/reader_progress.rs:111`

与其他 command 返回 `Result<T, String>` 不一致。锁中毒和 DB 错误被静默吞掉，前端收到 `None` 无法区分"无进度"和"读取失败"。改为 `Result<Option<SaveProgressDto>, String>`。

---

### L6. `settings_store::load()` 静默回退默认值

**位置：** `src/storage/settings_store.rs:76-85`

如果 settings.json 损坏，`load()` 静默返回 `SettingsFile::default()`，用户所有设置丢失且无任何提示。建议：将损坏文件重命名为 `.bak`，记录日志，然后回退默认值。

---

### L7. 前端缺少 `.option-select` CSS 规则

**位置：** `frontend/src/pages/reader/ReaderSettingsControls.tsx:51`

使用 `className="option-select"` 但任何 CSS 文件中无此规则，select 使用浏览器默认样式。应添加样式或改用已有的 `.settings-select`。

---

### L8. `.demo-controls` / `.demo-btn` 命名误导

**位置：** `frontend/src/pages/ReaderPage.css`

这些 class 是生产环境的设置 UI（`ReaderSettingsControls.tsx` 渲染），但命名暗示是演示/调试代码。建议重命名为 `.settings-floating-panel` / `.settings-floating-btn`。

---

### L9. 魔法数字散布各处

**位置：** 多处

| 位置 | 值 | 含义 |
|------|----|------|
| `useFootnotePreview.ts` | 200 | tooltip 位置阈值 |
| `useLoadingPage.ts:55` | 600 | 最小加载时间 |
| `useTwoPageNavigation.ts:343` | 50 | 滚轮翻页阈值 |
| `useTwoPageLayout.ts:59` | 4 | 每页最小行数 |
| `ReaderPage.tsx` | 960 | 双页模式最小宽度 |
| `tts.rs:240` | 200 | 轮询间隔 |

建议提取为命名常量。

---

### L10. 前端仅 1 个测试文件，后端无集成测试

**位置：** `frontend/src/pages/reader/twoPageCalcUtils.test.ts`（唯一测试）、`tests/`（空目录）

**现状：** 前端仅测试 3 个纯数学函数；后端 `tests/` 为空。所有 Tauri command、SQLite repository、TTS SSE 解析、DTO 转换均无测试。

**方案：**
- 后端：为 `xiaomi_provider::parse_sse_response` 添加单元测试（关键且无依赖）
- 后端：为 `resolve_chapter_index` 添加测试（复杂且无外部依赖）
- 前端：为 `readerUtils.ts` 的 DOM 操作函数添加测试（需配置 `jsdom`）
- 前端：在 `package.json` 添加 `jsdom` 和 `@testing-library/react` 依赖

优先级最低，但建议逐步补充。

---

### L11. `useRef` 类型不一致

**位置：** `frontend/src/pages/reader/useReaderSearch.ts:9` vs `useReadingProgress.ts:22`

前者 `useRef<ReturnType<typeof setTimeout>>(null)` 类型不完整，后者 `useRef<ReturnType<typeof setTimeout> | null>(null)` 正确。统一为后者写法。

---

### L12. `MutableRefObject` vs `RefObject` 不一致

**位置：** `frontend/src/pages/reader/TwoPageReaderContent.tsx`

使用 `React.MutableRefObject<TwoPageNav | null>`（旧式），其他文件使用 `RefObject<T | null>`（React 19 风格）。统一为 React 19 风格。

---

### L13. `opf_utils.rs` 与 `epub_metadata.rs` 存在重复 OPF 解析

**位置：** `src/parser/opf_utils.rs`（字符串匹配方式）与 `src/parser/parsers/epub/epub_metadata.rs`（quick_xml 方式）

两个实现各有用途（快速预览 vs 完整解析），但 OPF 解析逻辑变更需同步两处。`opf_utils.rs` 的行匹配方式对跨行标签不健壮。建议长期统一为 quick_xml 实现，或至少共享常量。

---

### L14. 隐式哨兵值耦合

**位置：**
- `INDENT_MARKER`（`\x01INDENT\x01`）：`epub_content.rs:257` 生成，`chapter_builder.rs:32` 消费。常量定义在 `chapter_builder.rs:10` 但生产方硬编码字符串。
- PUA 占位符（`\u{E000}...\u{E001}`）：`epub_content.rs:184` 生成，`epub_parser.rs:308-309` 消费。无共享常量。

**方案：** 在公共模块（如 `parser/mod.rs`）定义常量并双方引用：
```rust
pub const INDENT_MARKER: &str = "\x01INDENT\x01";
pub const INLINE_IMG_PREFIX: char = '\u{E000}';
pub const INLINE_IMG_SUFFIX: char = '\u{E001}';
```

**验证：** 解析结果不变。

---

## 四、执行计划

| 阶段 | 任务 | 预计改动量 | 风险 |
|------|------|-----------|------|
| 阶段一 | H1-H2, H4-H5（TTS 分段与缓存修正） | 小，聚焦 TTS 模块 | 低 |
| 阶段二 | H3, H6-H7（阻塞调用与 TXT 解析） | 中，涉及 command 改 async 和解析器重构 | 中 |
| 阶段三 | H8-H10（前端 CSS 与可访问性） | 中，CSS 整理 + div 改 button | 低 |
| 阶段四 | M1-M5（EPUB 解析器可读性） | 中，函数拆分和去重 | 低 |
| 阶段五 | M6-M8, M11-M12（锁优化、DB 约束、搜索） | 中，涉及并发和数据一致性 | 中 |
| 阶段六 | M9-M10, M13-M18（死代码清理、错误类型、前端重构） | 较大但分散 | 低 |
| 阶段七 | L1-L14（锦上添花） | 小 | 低 |

建议从阶段一开始——TTS 分段和缓存的修正对用户体验改善最直接，改动范围小且可独立验证。阶段二的 TXT 解析和阻塞调用修复影响面较大，建议在有充分测试覆盖后再执行。

---

## 五、与前两轮方案的关系

| 前轮方案项 | 状态 | 本轮关系 |
|-----------|------|---------|
| OPTIMIZATION_PLAN.md H1-H9 | 大部分已落地（Chapter 去冗余、TTS 锁分离、debounce、memo 化等） | 本轮不再重复 |
| OPTIMIZATION_PLAN.md M1-M15 | 部分已落地 | 本轮关注新发现的问题 |
| refactor-plan.md P0-1 (epub.rs 拆分) | 已完成 | 本轮 M1 关注 parse() 函数本身的进一步拆分 |
| refactor-plan.md P0-2 (reader.rs 拆分) | 已完成 | 本轮关注拆分后各文件的锁优化 |
| refactor-plan.md P0-3 (JSON 存储统一) | 已完成 | 本轮 M11 关注 DB 约束完善 |
| refactor-plan.md P0-4 (OPF 去重) | 已完成 | 本轮 L13 关注残留的重复实现 |
| refactor-plan.md P1-7 (api.ts 拆分) | 未执行 | 建议仍可执行，本轮不重复 |
| refactor-plan.md P1-8 (HTML 移出 src) | 已完成 | — |
