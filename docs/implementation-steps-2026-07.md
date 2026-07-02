# 代码优化实施步骤与修改计划

> 对应方案：`docs/code-optimization-plan-2026-07.md`
> 状态：待审核

---

## 阶段一：TTS 分段与缓存修正（H2, H4, H5）

### 步骤 1.1 — 修复分段器字节长度问题（H2）

**文件：** `src/tts/segmenter.rs`

**当前代码（第 34 行）：**
```rust
if !current_text.is_empty() && current_text.len() + para_text.len() + 1 > max_chars {
```

**修改为：**
```rust
if !current_text.is_empty()
    && current_text.chars().count() + para_text.chars().count() + 1 > max_chars
{
```

**同时修改测试（第 122-146 行 `long_paragraph_triggers_split`）：**

现有测试用 ASCII 字符（`"A".repeat(30)`），字节长度 == 字符数，无法覆盖中文场景。新增一个中文分段测试：

```rust
#[test]
fn chinese_text_uses_char_count_not_byte_count() {
    // 3 个中文段落，每段 10 字符 = 30 字符 / 90 字节
    let paras: Vec<Paragraph> = (0..3)
        .map(|i| Paragraph {
            index: i,
            text: "你好世界测试".to_string(), // 6 字符 = 18 字节
            kind: ParagraphKind::Body,
            indent_level: 0,
            source_line_hint: None,
            links: Vec::new(),
        })
        .collect();
    // max_chars=13: 按字符数应分 2 段 (6+6+1=13 ≤ 13, 第三段触发分割)
    // 按字节数会分 3 段 (18+18+1=37 > 13 每次都触发)
    let segments = segment_chapter(0, &paras, 13);
    assert_eq!(segments.len(), 2, "中文分段应按字符数而非字节数");
    assert_eq!(segments[0].char_count, 13); // 6 + '\n' + 6 = 13 字符
}
```

**验证：** `cargo test segmenter` 通过，新测试验证中文按字符数分段。

---

### 步骤 1.2 — 修复 `tts_max_text_length` 硬编码（H4）

**文件：** `src/tauri_api/commands/tts.rs`

**当前代码（第 91-95 行）：**
```rust
/// Get the max text length from a provider for the given config.
/// TODO: query provider dynamically when multiple providers are supported.
fn tts_max_text_length(_config: &TtsConfig) -> usize {
    200 // Xiaomi provider limit
}
```

**修改为：**
```rust
/// Get the max text length for the configured provider.
fn tts_max_text_length(config: &TtsConfig) -> usize {
    match config.provider {
        TtsProviderKind::Xiaomi => 500,
    }
}
```

需在文件顶部添加 `use crate::tts::types::TtsProviderKind;`（如未引入）。

**验证：** `cargo check` 通过。分段长度从 ~200 提升到 ~500 字符。

---

### 步骤 1.3 — `synthesize_blocking` 增加缓存检查（H5）

**文件：** `src/tts/synthesis_service.rs`

**当前代码（第 70-89 行）：**
```rust
pub fn synthesize_blocking(
    request: &TtsRequest,
    config: &TtsConfig,
    voice_id: &str,
    cache: &TtsCache,
) -> Result<TtsResponse, TtsError> {
    let provider = create_provider_from_config(config);
    let resp = provider.synthesize(request, config)?;
    let path = cache.segment_path(
        &provider_cache_label(config),
        &request.book_id,
        request.chapter_index,
        request.segment_index,
        voice_id,
        "pcm16",
    );
    let _ = cache.write(&path, &resp.audio_bytes);
    cache.prune_if_over_limit();
    Ok(resp)
}
```

**修改为：**
```rust
pub fn synthesize_blocking(
    request: &TtsRequest,
    config: &TtsConfig,
    voice_id: &str,
    cache: &TtsCache,
) -> Result<TtsResponse, TtsError> {
    let provider_label = provider_cache_label(config);
    let path = cache.segment_path(
        &provider_label,
        &request.book_id,
        request.chapter_index,
        request.segment_index,
        voice_id,
        "pcm16",
    );

    // Check cache first — avoids redundant API calls during prefetch
    if cache.exists(&path) {
        match cache.read(&path) {
            Ok(audio_bytes) => {
                log::info!(
                    "TTS 缓存命中: book={} ch={} seg={}",
                    &request.book_id,
                    request.chapter_index,
                    request.segment_index
                );
                return Ok(TtsResponse {
                    audio_bytes,
                    media_type: "audio/pcm16".to_string(),
                    duration_ms: None,
                });
            }
            Err(e) => {
                log::warn!("TTS 缓存读取失败，重新合成: {}", e);
            }
        }
    }

    // Synthesize via provider
    let provider = create_provider_from_config(config);
    let resp = provider.synthesize(request, config)?;

    // Write to cache (best-effort)
    if let Err(e) = cache.write(&path, &resp.audio_bytes) {
        log::warn!("TTS 缓存写入失败: {}", e);
    }
    cache.prune_if_over_limit();
    Ok(resp)
}
```

**同时清理 `synthesize_and_play` 中的重复缓存检查（`tts.rs` 第 126-141 行）。**

`synthesize_and_play` 目前先手动检查缓存路径再调用 `synthesize_blocking`。现在 `synthesize_blocking` 已内置缓存检查，`synthesize_and_play` 中的手动检查变为冗余。简化为：

```rust
// 原第 126-141 行替换为直接调用
let (audio_bytes, media_type) =
    match TtsSynthesisService::synthesize_blocking(&request, config, &voice_id, cache) {
        Ok(resp) => (resp.audio_bytes, resp.media_type),
        Err(e) => {
            let msg = format!("TTS 合成失败: {}", e);
            log::error!("{}", msg);
            emitter.tts_error(&TtsError {
                book_id: Some(book_id.to_string()),
                error_message: msg.clone(),
            });
            return Err(msg);
        }
    };
```

删除 `segment_path` 手动构造和 `segment_path.exists()` 检查块。

**验证：** `cargo check` 通过。重复播放同一段章节时，日志中不出现新的合成请求。

---

## 阶段二：阻塞调用与 TXT 解析（H3, H6, H7）

### 步骤 2.1 — `tts_start` 改为 async + spawn_blocking（H3）

**文件：** `src/tauri_api/commands/tts.rs`

**当前签名（第 323-329 行）：**
```rust
#[tauri::command]
pub fn tts_start(
    chapter_index: usize,
    book_state: tauri::State<'_, BookSession>,
    tts_state: tauri::State<'_, TtsSessionLock>,
    app: tauri::AppHandle,
) -> Result<(), String> {
```

**修改为：**
```rust
#[tauri::command]
pub async fn tts_start(
    chapter_index: usize,
    book_state: tauri::State<'_, BookSession>,
    tts_state: tauri::State<'_, TtsSessionLock>,
    app: tauri::AppHandle,
) -> Result<(), String> {
```

**第 381-397 行的 `synthesize_and_play` 调用改为 spawn_blocking：**

```rust
// 原第 381-397 行
// Synthesize and play segment 0
let seg = segment.clone();
let bid = book_id.clone();
let cfg = config.clone();
let c = Arc::clone(&cache);
let ptx = playback_tx.clone();
let app_clone = app.clone();

let synth_result = tauri::async_runtime::spawn_blocking(move || {
    synthesize_and_play(&seg, &bid, chapter_index, &cfg, &c, &ptx, &app_clone)
})
.await
.map_err(|e| format!("TTS 合成任务失败: {}", e))?;

if let Err(e) = synth_result {
    let mut guard = tts_state.lock().map_err(|lock_err| lock_err.to_string())?;
    guard.stop_flag.store(true, Ordering::Relaxed);
    guard.playback_state.status = PlaybackStatus::Error(e.clone());
    guard.playback_tx = None;
    guard.is_playing_flag.store(false, Ordering::Relaxed);
    return Err(e);
}
```

注意：`synthesize_and_play` 的参数需要 `Clone` 或 `move`。`Segment` 已实现 `Clone`，`TtsConfig` 已实现 `Clone`，`Arc<TtsCache>` 可 `clone`，`mpsc::Sender` 可 `clone`，`tauri::AppHandle` 可 `clone`。

**验证：** TTS 启动时 UI 翻页/设置操作不卡顿。`cargo check` 通过。

---

### 步骤 2.2 — TXT 解析改用 BufReader 逐行读取（H6）

**文件：** `src/parser/parsers/txt.rs`

**当前 `parse` 方法（第 185-233 行）** 先 `read_to_string` 再 `lines().collect()`。

**修改 `parse` 方法：**

```rust
fn parse(&self, path: &str) -> AppResult<ParseResult> {
    let file = File::open(path).map_err(|e| {
        let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件打开失败", e.to_string());
        err.recoverable = true;
        err
    })?;

    // 逐行读取，避免大文件全部读入内存
    let reader = std::io::BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件读取失败", e.to_string());
            err.recoverable = true;
            err
        })?;

    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    // 检测是否包含章节
    let has_chapters = line_refs
        .iter()
        .any(|line| Self::is_chapter_line(line).is_some());

    let (content, chapter_titles) = if has_chapters {
        Self::split_by_chapters(line_refs)
    } else {
        let full_text = line_refs.join("\n");
        let content = vec![full_text.trim().to_string()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        let chapter_titles = vec!["文本文件".to_string()];
        (content, chapter_titles)
    };

    Ok(ParseResult {
        content,
        chapter_titles,
        spine_hrefs: Vec::new(),
        toc: None,
        metadata: None,
        warnings: Vec::new(),
        cover_image: None,
        cover_media_type: None,
        image_assets: Vec::new(),
        chapter_image_blocks: Vec::new(),
        chapter_links: Vec::new(),
        chapter_anchors: Vec::new(),
        chapter_heading_flags: Vec::new(),
    })
}
```

注意：`BufReader::lines()` 逐行读取，每行一个 `String`。虽然 `lines` Vec 仍持有所有行的文本，但避免了 `read_to_string` 的双倍内存（完整文件 String + lines Vec）。如需进一步优化为真正流式，需要重构 `split_by_chapters` 为迭代器模式，但这会增加复杂度，暂不进行。

同时将 `use std::io::Read;` 改为 `use std::io::BufRead;`，并添加 `use std::io::BufReader;`。

**验证：** `cargo test` 中 TXT 相关测试通过。手动测试大文件（50MB+）解析不 OOM。

---

### 步骤 2.3 — TXT 解析增加编码检测（H7）

**文件：** `Cargo.toml` 添加依赖：
```toml
encoding_rs = "0.8"
```

**文件：** `src/parser/parsers/txt.rs`

在 `parse` 方法中，文件打开后先读取前 4 字节检测 BOM，再尝试 UTF-8，失败则按 GBK 解码：

```rust
fn parse(&self, path: &str) -> AppResult<ParseResult> {
    let mut file = File::open(path).map_err(|e| {
        let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件打开失败", e.to_string());
        err.recoverable = true;
        err
    })?;

    // 读取文件内容，检测编码
    let mut raw_bytes = Vec::new();
    file.read_to_end(&mut raw_bytes).map_err(|e| {
        let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件读取失败", e.to_string());
        err.recoverable = true;
        err
    })?;

    let content_str = decode_text(&raw_bytes);

    // 后续逻辑不变，使用 content_str.lines() ...
}

/// 检测编码并解码为 UTF-8 字符串
fn decode_text(raw: &[u8]) -> String {
    // 检查 BOM
    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&raw[3..]).into_owned();
    }
    if raw.starts_with(&[0xFF, 0xFE]) || raw.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 LE/BE
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(raw);
        return cow.into_owned();
    }

    // 尝试 UTF-8
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }

    // 回退到 GBK（中文 TXT 最常见的非 UTF-8 编码）
    log::info!("文件非 UTF-8 编码，尝试 GBK 解码");
    let (cow, _, had_errors) = encoding_rs::GBK.decode(raw);
    if had_errors {
        log::warn!("GBK 解码存在不可识别字符");
    }
    cow.into_owned()
}
```

注意：此步骤将 `read_to_end` 读入全部字节。对于大文件 OOM 问题，理想方案是流式检测编码 + 流式解析。但 BOM 检测只需前几字节，GBK/UTF-8 判断也可通过前 N 字节采样。实际实现时可先读前 4KB 判断编码，再用 `BufReader` 配合 `encoding_rs::Decoder` 流式解码。但这增加复杂度，建议先实现简单版本（全量读取 + 编码检测），后续与 H6 的流式读取合并优化。

**验证：** GBK 编码的 TXT 文件解析后中文正常显示。`cargo test` 通过。

---

## 阶段三：前端 CSS 与可访问性（H8, H9, H10）

### 步骤 3.1 — 修复 CSS 类名冲突（H8）

**涉及文件：**
- `frontend/src/pages/LibraryPage.css`
- `frontend/src/pages/LoadingPage.css`
- `frontend/src/global.css`

**操作 1：将 `:root` 亮色变量从 `LibraryPage.css` 移到 `global.css`**

从 `LibraryPage.css` 第 3-18 行剪切 `:root { ... }` 块，粘贴到 `global.css` 顶部。

**操作 2：重命名冲突类名**

在 `LoadingPage.css` 中：
```css
/* 重命名 */
.book-title  →  .loading-book-title
.book-author →  .loading-book-author
.btn-secondary → .loading-btn-secondary
@keyframes fadeIn → @keyframes fadeInLoading
```

在 `LoadingPage.tsx` 中同步修改 `className` 引用。

在 `LibraryPage.css` 中：
```css
@keyframes fadeIn → @keyframes fadeInLibrary
```

在 `LibraryPage.tsx` 中同步修改引用（如有）。

**操作 3：删除重复的 `.cover-1` ~ `.cover-6`**

从 `LoadingPage.css` 中删除第 210-215 行的 `.cover-1` ~ `.cover-6`（与 `LibraryPage.css` 完全相同）。保留 `LibraryPage.css` 中的定义，或移到 `global.css`。

**操作 4：删除 4 个文件中的 Google Fonts `@import`**

从 `ReaderPage.css`、`LibraryPage.css`、`BookmarkPage.css`、`LoadingPage.css` 的第 1 行删除 `@import url('https://fonts.googleapis.com/...')`。

在 `frontend/index.html` 的 `<head>` 中添加：
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@400;500;700&family=Noto+Serif+SC:wght@400;700&display=swap" rel="stylesheet">
```

**验证：** `npm run build` 通过。各页面样式正常。DevTools 中字体只加载一次。

---

### 步骤 3.2 — 修复 7 个 Hook 的 Zustand selector（H9）

逐个文件修改，模式统一：

**文件 1：`frontend/src/pages/library/useLibraryPage.ts` 第 23 行**

```ts
// 之前
const { books, setBooks, startOpening, setSidebarFooter } = useAppStore()

// 之后
const books = useAppStore(s => s.books)
const setBooks = useAppStore(s => s.setBooks)
const startOpening = useAppStore(s => s.startOpening)
const setSidebarFooter = useAppStore(s => s.setSidebarFooter)
```

**文件 2：`frontend/src/pages/loading/useLoadingPage.ts` 第 17-18 行**

```ts
// 之前
const { books, opening, startOpening, setOpeningError, setReaderBook, setCurrentChapter, setProgressPercent } =
    useAppStore()

// 之后
const books = useAppStore(s => s.books)
const opening = useAppStore(s => s.opening)
const startOpening = useAppStore(s => s.startOpening)
const setOpeningError = useAppStore(s => s.setOpeningError)
const setReaderBook = useAppStore(s => s.setReaderBook)
const setCurrentChapter = useAppStore(s => s.setCurrentChapter)
const setProgressPercent = useAppStore(s => s.setProgressPercent)
```

**文件 3：`frontend/src/pages/reader/useChapterNavigation.ts` 第 25-29 行**

```ts
// 之前
const {
    setCurrentChapter,
    setProgressPercent,
    closeToc,
} = useAppStore()

// 之后
const setCurrentChapter = useAppStore(s => s.setCurrentChapter)
const setProgressPercent = useAppStore(s => s.setProgressPercent)
const closeToc = useAppStore(s => s.closeToc)
```

**文件 4：`frontend/src/pages/reader/useReadingProgress.ts` 第 21 行**

```ts
// 之前
const { setProgressPercent } = useAppStore()

// 之后
const setProgressPercent = useAppStore(s => s.setProgressPercent)
```

**文件 5：`frontend/src/pages/reader/useTtsReader.ts` 第 18 行**

```ts
// 之前
const { setTtsState, resetTts } = useAppStore()

// 之后
const setTtsState = useAppStore(s => s.setTtsState)
const resetTts = useAppStore(s => s.resetTts)
```

**文件 6：`frontend/src/pages/reader/useReaderSearch.ts` 第 11 行**

```ts
// 之前
const { toggleSearch, closeSearch } = useAppStore()

// 之后
const toggleSearch = useAppStore(s => s.toggleSearch)
const closeSearch = useAppStore(s => s.closeSearch)
```

**文件 7：`frontend/src/pages/reader/useBookmarks.ts` 第 11 行**

```ts
// 之前
const { setBookmarks } = useAppStore()

// 之后
const setBookmarks = useAppStore(s => s.setBookmarks)
```

**验证：** `npm run build` 通过。React DevTools Profiler 中 Library 页面在 TTS 状态变化时不重渲染。

---

### 步骤 3.3 — 可点击 div 改为 button（H10）

**文件 1：`frontend/src/components/Sidebar.tsx` 第 61-66 行**

```tsx
// 之前
<div
    key={item.id}
    className={`sidebar-item ${activeId === item.id ? 'active' : ''}`}
    onClick={() => item.path && navigate(item.path)}
    title={collapsed ? item.label : undefined}
>

// 之后
<button
    type="button"
    key={item.id}
    className={`sidebar-item ${activeId === item.id ? 'active' : ''}`}
    onClick={() => item.path && navigate(item.path)}
    title={collapsed ? item.label : undefined}
>
```

关闭标签 `</div>` → `</button>`。在 `Sidebar.css` 中给 `.sidebar-item` 添加 `all: unset;` 或调整样式以适配 `<button>` 元素的默认样式。

**文件 2：`frontend/src/pages/reader/ReaderTocPanel.tsx` — 目录项**

同模式将 `<div onClick>` 改为 `<button type="button" onClick>`。

**文件 3：`frontend/src/pages/LibraryPage.tsx` — 书籍卡片**

书籍卡片较复杂（包含子元素），改用带 ARIA 属性的方案：
```tsx
<div
    key={item.book_id}
    role="button"
    tabIndex={0}
    className="book-card"
    onClick={() => handleOpenBook(item.book_id)}
    onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault()
            handleOpenBook(item.book_id)
        }
    }}
>
```

**文件 4：`frontend/src/pages/BookmarkPage.tsx` — 书签条目**

同 LibraryPage 方案，添加 `role="button"` + `tabIndex={0}` + `onKeyDown`。

**文件 5：`frontend/src/pages/LoadingPage.tsx` — 返回按钮**

`<div onClick>` 改为 `<button type="button" onClick>`。

**验证：** 键盘 Tab 可导航到所有可点击元素，Enter/Space 可触发操作。

---

## 阶段四：EPUB 解析器可读性（M1, M2, M3, M4, M5）

### 步骤 4.1 — `parse()` 方法拆分（M1）

**文件：** `src/parser/parsers/epub/epub_parser.rs`

将 `parse()` 方法（第 123-503 行）拆分为 4 个方法：

```rust
impl EpubParser {
    pub fn parse(&mut self, path: &str) -> AppResult<ParseResult> {
        let zip = self.open_archive(path)?;
        self.parse_container_and_opf(&zip)?;
        self.parse_toc(&zip)?;
        self.build_chapters_from_spine(&zip)?;
        self.extract_cover(&zip)?;
        Ok(self.finalize())
    }

    fn parse_container_and_opf(&mut self, zip: &ZipArchive<...>) -> AppResult<()> {
        // 迁入：container.xml 解析（原 ~136-170 行）
        // 迁入：OPF manifest/spine/metadata 解析（原 ~172-238 行）
    }

    fn parse_toc(&mut self, zip: &ZipArchive<...>) -> AppResult<()> {
        // 迁入：nav.xhtml 解析（原 ~160-175 行）
        // 迁入：toc.ncx 解析（原 ~175-238 行）
    }

    fn build_chapters_from_spine(&mut self, zip: &ZipArchive<...>) -> AppResult<()> {
        // 迁入：spine 遍历 + HTML 提取（原 ~240-410 行）
    }

    fn extract_cover(&mut self, zip: &ZipArchive<...>) -> AppResult<()> {
        // 迁入：封面提取（原 ~438-486 行）
    }
}
```

每个方法 60-100 行。`parse()` 变为编排方法，约 10 行。

**验证：** `cargo test` 全部 EPUB 测试通过。

---

### 步骤 4.2 — 图片资源创建去重（M2）

**文件：** `src/parser/parsers/epub/epub_parser.rs`

提取辅助方法：

```rust
impl EpubParser {
    /// Get existing or create new image asset, returning the asset_id.
    fn get_or_create_image_asset(
        &mut self,
        img_src: &str,
        chapter_dir: &str,
        is_inline: bool,
    ) -> String {
        let img_full_path = if chapter_dir.is_empty() {
            img_src.to_string()
        } else {
            crate::parser::epub_assets::resolve_path(chapter_dir, img_src)
        };

        // Check if already registered
        if let Some(existing) = self.image_assets.iter().find(|a| a.asset_path == img_full_path) {
            return existing.asset_id.clone();
        }

        // Create new asset
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        img_full_path.hash(&mut hasher);
        let asset_id = format!("img-{:016x}", hasher.finish());

        let media_type = crate::parser::epub_assets::media_type_from_href(&img_full_path);
        self.image_assets.push(BookImageAsset {
            asset_id: asset_id.clone(),
            asset_path: img_full_path,
            media_type,
            is_inline,
        });

        asset_id
    }
}
```

原第 292-346 行和 354-410 行的内联图片处理和图片块处理改为调用此方法。

**验证：** EPUB 图片提取功能正常。

---

### 步骤 4.3 — 格式检测统一（M3）

**文件：** `src/domain/book_format.rs`

添加 `from_path` 方法：

```rust
impl BookFormat {
    pub fn from_path(path: &str) -> Option<Self> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())?;
        match ext.as_str() {
            "epub" => Some(Self::Epub),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }
}
```

**文件：** 修改三处调用：

`src/parser/parsers/factory.rs` 第 26-29 行：
```rust
// 之前
if path.ends_with(".epub") { ... } else if path.ends_with(".txt") { ... }

// 之后
match BookFormat::from_path(path) {
    Some(BookFormat::Epub) => Box::new(EpubParser::new()),
    Some(BookFormat::Txt) => Box::new(TxtParser::new()),
    None => return Err(...),
}
```

`src/services/library_service_impl.rs` 第 13-17 行和 `src/services/reader_service_impl.rs` 第 33-37 行同样替换为 `BookFormat::from_path(path)`。

**验证：** `cargo test` 通过。`.EPUB` 大小写变体正确识别。

---

### 步骤 4.4 — HTML 标签精确匹配（M4）

**文件：** `src/parser/parsers/epub/epub_content.rs`

第 306-308 行：
```rust
// 之前
if name.starts_with(&b"h1"[..])
    || name.starts_with(&b"h2"[..])
    || name.starts_with(&b"h3"[..])

// 之后
if name.eq_ignore_ascii_case(b"h1")
    || name.eq_ignore_ascii_case(b"h2")
    || name.eq_ignore_ascii_case(b"h3")
```

第 317 行：
```rust
// 之前
if name.starts_with(&b"title"[..])

// 之后
if name.eq_ignore_ascii_case(b"title")
```

第 149 行（`attr_name.starts_with(&b"style"[..])`）：
```rust
// 之前
if attr_name.starts_with(&b"style"[..])

// 之后
if attr_name.eq_ignore_ascii_case(b"style")
```

**验证：** `cargo test` 通过。含非标准标签名（如 `h1foo`）的 EPUB 不被误解析。

---

### 步骤 4.5 — `extract_html_with_positions` 状态提取为 struct（M5）

**文件：** `src/parser/parsers/epub/epub_content.rs`

这是较大的重构。将 20+ 个局部变量提取为 struct：

```rust
struct HtmlExtractorState {
    current_para: String,
    para_links: Vec<TextLink>,
    current_para_start: usize,
    paragraphs: Vec<Paragraph>,
    img_blocks: Vec<(isize, BookImageBlock)>,
    inline_images: Vec<(usize, String)>,
    // ... 其余变量
}

impl HtmlExtractorState {
    fn flush_para(&mut self) {
        if self.current_para.trim().is_empty() && self.para_links.is_empty() {
            self.current_para.clear();
            self.current_para_start = 0;
            return;
        }
        // 迁入 flush_para 逻辑
    }

    fn handle_a_attrs(&mut self, e: &BytesStart) {
        // 迁入 handle_a_attrs 逻辑
    }
}
```

`extract_html_with_positions` 变为：
```rust
fn extract_html_with_positions(...) -> (Vec<Paragraph>, Vec<(isize, BookImageBlock)>, Vec<(usize, String)>) {
    let mut state = HtmlExtractorState::new();
    let mut reader = quick_xml::Reader::from_str(html);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(ref e)) => {
                state.handle_start(e);
            }
            Ok(Event::End(ref e)) => {
                state.handle_end(e);
            }
            Ok(Event::Text(ref e)) => {
                state.handle_text(e);
            }
            Ok(Event::Empty(ref e)) => {
                state.handle_empty(e);
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    state.finish()
}
```

**此步骤改动量大，建议在步骤 4.1-4.4 验证通过后单独进行。**

**验证：** `cargo test` 全部 EPUB 测试通过。解析结果与重构前完全一致。

---

## 阶段五：锁优化、DB 约束、搜索（M6, M7, M8, M11, M12）

### 步骤 5.1 — SQLite 添加外键约束（M11）

**文件：** `src/storage/sqlite/connection.rs` 第 11-13 行

```rust
// 之前
"PRAGMA journal_mode=WAL;
 PRAGMA busy_timeout=10000;"

// 之后
"PRAGMA journal_mode=WAL;
 PRAGMA busy_timeout=10000;
 PRAGMA foreign_keys=ON;"
```

**文件：** `src/storage/sqlite/schema.rs`

在各子表添加 FOREIGN KEY 约束：

```sql
-- reading_progress 表（第 32 行 CREATE TABLE 内追加）
CREATE TABLE IF NOT EXISTS reading_progress (
    book_id              TEXT PRIMARY KEY,
    -- ... 现有字段 ...
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);

-- bookmarks 表（第 48 行）
CREATE TABLE IF NOT EXISTS bookmarks (
    -- ... 现有字段 ...
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);

-- book_tags 表（第 61 行）
CREATE TABLE IF NOT EXISTS book_tags (
    book_id TEXT NOT NULL,
    tag     TEXT NOT NULL,
    PRIMARY KEY (book_id, tag),
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);

-- reading_sessions 表（第 69 行）
CREATE TABLE IF NOT EXISTS reading_sessions (
    -- ... 现有字段 ...
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);
```

注意：由于使用 `CREATE TABLE IF NOT EXISTS`，已有数据库的表不会被修改。需要写迁移代码或要求用户重新导入。建议新增 schema_version 2 的迁移脚本，执行 `ALTER TABLE` 重建约束。简单方案：在 `bootstrap.rs` 中检测旧 schema 并执行迁移。

**验证：** `cargo check` 通过。删除书籍后，`bookmarks`、`reading_progress` 等表中无孤儿记录。

---

### 步骤 5.2 — 搜索循环内 `to_lowercase` 提前（M12）

**文件：** `src/tauri_api/commands/bookmark.rs` 第 18-54 行

```rust
// 之前（第 22 行，内层循环内）
let query_lower = query.to_lowercase();

// 修改为（提到循环外）
```

在 `search_in_book` 函数中，`for chapter` 循环之前添加：
```rust
let query_lower = query.to_lowercase();
```

删除第 22 行的 `let query_lower = query.to_lowercase();`。

第 23 行 `text_lower.find(&query_lower)` 不变（已使用提前计算的 `query_lower`）。

**验证：** 搜索功能正常。`cargo check` 通过。

---

### 步骤 5.3 — 持锁期间 DB I/O 优化（M6）

**文件：** `src/tauri_api/commands/reader_progress.rs` 第 39-47 行

```rust
// 之前
let existing = {
    let mut progress_map = progress_state.lock().map_err(|e| e.to_string())?;
    if !progress_map.contains_key(&progress.book_id) {
        if let Ok(Some(saved)) = db.progress().load(&progress.book_id) {
            progress_map.insert(progress.book_id.clone(), saved);
        }
    }
    progress_map.get(&progress.book_id).cloned()
};

// 之后：先检查内存缓存，未命中再从 DB 加载
let existing = {
    let progress_map = progress_state.lock().map_err(|e| e.to_string())?;
    if let Some(p) = progress_map.get(&progress.book_id) {
        Some(p.clone())
    } else {
        None
    }
};
// DB 加载在锁外进行
let existing = if existing.is_some() {
    existing
} else {
    let saved = db.progress().load(&progress.book_id).ok().flatten();
    if let Some(ref saved) = saved {
        let mut progress_map = progress_state.lock().map_err(|e| e.to_string())?;
        progress_map.insert(progress.book_id.clone(), saved.clone());
    }
    saved
};
```

**验证：** 翻页时进度保存正常。`cargo check` 通过。

---

### 步骤 5.4 — `library_*` command 改为 async（M7）

**文件：** `src/tauri_api/commands/library.rs`

逐个将同步 command 改为 async，阻塞 I/O 包入 `spawn_blocking`。

示例 — `library_list`：
```rust
// 之前
#[tauri::command]
pub fn library_list(
    index_state: tauri::State<'_, LibraryIndexState>,
) -> Result<Vec<LibraryBookCardDto>, String> {
    let index = index_state.lock().map_err(|e| e.to_string())?;
    Ok(index.items.iter().map(item_to_dto).collect())
}

// 之后
#[tauri::command]
pub async fn library_list(
    index_state: tauri::State<'_, LibraryIndexState>,
) -> Result<Vec<LibraryBookCardDto>, String> {
    let items = {
        let index = index_state.lock().map_err(|e| e.to_string())?;
        index.items.clone()
    };
    // item_to_dto 可能做文件系统检查（cover 路径），放在 spawn_blocking 中
    tauri::async_runtime::spawn_blocking(move || {
        items.iter().map(item_to_dto).collect()
    })
    .await
    .map_err(|e| e.to_string())
}
```

其他 command 同理。注意 `library_import` 和 `library_remove` 涉及更多 I/O，改动更大。

**此步骤改动面大，建议逐 command 进行，每个改完即验证。**

**验证：** `cargo check` 通过。书库操作期间 UI 响应正常。

---

### 步骤 5.5 — TTS 自动推进改事件驱动（M8）

**文件：** `src/tauri_api/commands/tts.rs`

此为架构级改动。在播放线程中，当 sink 播放结束时通过 channel 通知：

```rust
// spawn_playback_thread 返回值增加 completion_rx
fn spawn_playback_thread() -> (mpsc::Sender<PlaybackCmd>, Arc<AtomicBool>, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel::<PlaybackCmd>();
    let (completion_tx, completion_rx) = mpsc::channel::<()>();
    let is_playing = Arc::new(AtomicBool::new(false));

    std::thread::spawn(move || {
        // ...
        loop {
            // 检测播放结束
            if playing_flag.load(Ordering::Relaxed) && p.is_empty() && !p.is_paused() {
                playing_flag.store(false, Ordering::Relaxed);
                let _ = completion_tx.send(()); // 通知完成
            }
            // ... 原有 recv_timeout 逻辑 ...
        }
    });

    (tx, is_playing, completion_rx)
}
```

`spawn_auto_advance_thread` 改为等待 `completion_rx.recv()` 而非轮询：

```rust
fn spawn_auto_advance_thread(
    // ... 原有参数 ...
    completion_rx: mpsc::Receiver<()>,
) {
    std::thread::spawn(move || {
        let mut current_seg_idx: usize = 0;
        loop {
            if poll_stop.load(Ordering::Relaxed) { break; }

            match completion_rx.recv_timeout(Duration::from_secs(60)) {
                Ok(()) => {
                    current_seg_idx += 1;
                    if current_seg_idx >= total_segments {
                        emitter.tts_finished(...);
                        break;
                    }
                    // 播放下一段
                    if let Some(next_seg) = segments.get(current_seg_idx) {
                        if synthesize_and_play(...).is_err() { break; }
                        emitter.tts_playing(...);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}
```

**此步骤改动 TTS 核心逻辑，建议在阶段一验证通过后单独进行，充分测试。**

**验证：** 段落切换无明显延迟。播放线程异常退出时自动推进线程能检测并停止。

---

## 阶段六：死代码、错误类型、前端重构（M9, M10, M13-M18）

### 步骤 6.1 — 删除死代码（M9）

**文件：** `src/storage/traits.rs`

评估 `TagsRepo`、`SessionsRepo`、`AggregatesRepo` 是否计划开发：
- 如计划开发统计功能：保留，在注释中标注 `// WIP: 统计功能待接入`
- 如不计划：删除 trait 定义 + 对应 sqlite 实现 + `DatabaseBackend` 中的 `tags()`/`sessions()`/`aggregates()` 方法

删除 `BooksRepo::get`、`search`、`update_progress`、`update_stats` 上的 `#[allow(dead_code)]` 标注和方法本身（如无调用）。

删除 `ProgressRepo::save`、`mark_dirty`、`flush_dirty`、`load_all` 上的 `#[allow(dead_code)]` 和方法（如无调用）。

**文件：** `src/tts/synthesis_service.rs` — 删除 `#[allow(dead_code)]` 标注的实例方法 `synthesize()`（第 91-145 行）和 `validate_config()`（第 48-54 行）。

**文件：** `src/tts/player.rs` — 删除 `#[allow(dead_code)]` 的 `len()` 方法。

**文件：** `src/tts/config.rs` — 删除空的 `impl TtsConfig {}` 块。

**验证：** `cargo check` 通过，无 warning。

---

### 步骤 6.2 — 枚举 Debug 格式化改为 Display（M13）

**文件：** `src/domain/paragraph_kind.rs`

添加 `as_str()` 方法：
```rust
impl ParagraphKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Body => "body",
            Self::Heading => "heading",
            Self::Quote => "quote",
        }
    }
}
```

（需确认 `ParagraphKind` 的变体名。当前代码中 `format!("{:?}", p.kind).to_lowercase()` 产出 `"body"`/`"heading"`/`"quote"`，需对应 `as_str` 返回值。）

**文件：** `src/tts/types.rs` — 为 `TtsProviderKind` 添加 `as_str()`：
```rust
impl TtsProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Xiaomi => "xiaomi",
            #[cfg(feature = "tts-aliyun")]
            Self::Aliyun => "aliyun",
        }
    }
}
```

**文件：** `src/tauri_api/commands/dto_convert.rs` — 替换所有 `format!("{:?}", ...).to_lowercase()`：
```rust
// 第 66 行
kind: format!("{:?}", p.kind).to_lowercase(),
// →
kind: p.kind.as_str().to_string(),

// 第 109 行
provider: format!("{:?}", config.provider).to_lowercase(),
// →
provider: config.provider.as_str().to_string(),
```

**文件：** `src/tts/synthesis_service.rs` 第 16 行：
```rust
fn provider_cache_label(config: &TtsConfig) -> String {
    config.provider.as_str().to_string()
}
```

**文件：** `src/tauri_api/commands/tts.rs` 第 128 行（`synthesize_and_play` 内）：
```rust
// 之前
&format!("{:?}", config.provider).to_lowercase()

// 之后
&config.provider.as_str().to_string()
```

**验证：** `cargo check` 通过。DTO 序列化结果不变。TTS 缓存路径不变。

---

### 步骤 6.3 — `useTwoPageNavigation.ts` 拆分（M18）

**文件：** `frontend/src/pages/reader/useTwoPageNavigation.ts`

拆分为 4 个 hook：

```
frontend/src/pages/reader/
  useSpreadNavigation.ts   — spread 索引管理和翻页逻辑（~150 行）
  useSpreadKeyboard.ts     — 键盘/滚轮事件处理（~80 行）
  useSpreadPreload.ts      — 章节预加载（~60 行）
  useVisibleChapterSync.ts — 可见章节同步（~40 行）
```

`useTwoPageNavigation` 变为组合这 4 个 hook 的薄封装（~50 行）。

此步骤改动量大且涉及复杂的交互逻辑，建议：
1. 先写新 hook 文件
2. 逐步将逻辑从 `useTwoPageNavigation` 迁移
3. 每迁出一块逻辑即测试
4. 最后删除原文件中的冗余代码

**验证：** 双页模式翻页、键盘导航、章节预加载、可见章节同步全部正常。

---

### 步骤 6.4 — 全局正则改为局部实例（M17）

**文件：**
- `frontend/src/pages/reader/ReaderBlock.tsx` 第 2 行（模块级 `INLINE_IMAGE_RE`）
- `frontend/src/pages/reader/readerUtils.ts` 第 4 行
- `frontend/src/pages/reader/useChapterImages.ts` 第 35 行

**方案：** 在每个文件中将模块级正则改为使用时创建局部实例：

```ts
// 之前（模块级）
const INLINE_IMAGE_RE = /\u{E000}(.+?)\u{E001}/gu

// 使用处
INLINE_IMAGE_RE.lastIndex = 0
while ((m = INLINE_IMAGE_RE.exec(text)) !== null) { ... }

// 之后（使用 matchAll，无需 lastIndex 管理）
const matches = [...text.matchAll(/\u{E000}(.+?)\u{E001}/gu)]
```

或定义工厂函数：
```ts
function matchInlineImages(text: string): IterableIterator<RegExpMatchArray> {
    return text.matchAll(/\u{E000}(.+?)\u{E001}/gu)[Symbol.iterator]()
}
```

在 `utils/` 下新建 `inlineImage.ts` 统一管理：
```ts
// frontend/src/utils/inlineImage.ts
export const INLINE_IMAGE_PATTERN = '\\u{E000}(.+?)\\u{E001}'
export function matchInlineImages(text: string): RegExpMatchArray[] {
    return [...text.matchAll(new RegExp(INLINE_IMAGE_PATTERN, 'gu'))]
}
```

三个文件改为 import 并调用 `matchInlineImages`。

**验证：** `npm run build` 通过。内联图片渲染正常。

---

## 阶段七：锦上添花（L1-L14）

### L1. 依赖升级

`Cargo.toml` 中 `zip` 0.6→0.7（需适配 API 变更）、`env_logger` 0.10→0.11、`dirs` 5→6。逐个升级，每次 `cargo check` + `cargo test`。

### L2. 删除未使用 feature flags

`Cargo.toml` 删除 `tts-aliyun`、`db-postgres`、`db-mysql`（如无计划实现）。同时删除代码中 `#[cfg(feature = "tts-aliyun")]` 等条件编译。

### L3. 删除未使用 npm 依赖

`frontend/package.json` 删除 `@tauri-apps/plugin-fs`、`@tauri-apps/plugin-shell`。`npm install` 后 `npm run build` 验证。

### L4. `reader_get_progress` 返回类型统一

`src/tauri_api/commands/reader_progress.rs` 第 107 行 `Option<SaveProgressDto>` → `Result<Option<SaveProgressDto>, String>`。前端 `api.ts` 中对应函数返回类型同步修改。

### L5. `settings_store::load()` 损坏文件备份

`src/storage/settings_store.rs` 第 76-85 行，在回退默认值前将损坏文件重命名为 `.bak`：
```rust
if let Ok(metadata) = std::fs::metadata(path) {
    let backup = format!("{}.bak", path);
    let _ = std::fs::rename(path, &backup);
    log::warn!("设置文件损坏，已备份到 {}", backup);
}
```

### L6. `.option-select` CSS 补充

在 `ReaderPage.css` 中添加 `.option-select` 样式，或将 `ReaderSettingsControls.tsx` 第 51 行的 `className="option-select"` 改为 `className="settings-select"`（复用已有样式）。

### L7. `.demo-controls` 重命名

CSS 和 TSX 中将 `.demo-controls` → `.settings-floating-panel`，`.demo-btn` → `.settings-floating-btn`。全局搜索替换。

### L8. `useRef` 类型修正

`useReaderSearch.ts` 第 9 行：`useRef<ReturnType<typeof setTimeout>>(null)` → `useRef<ReturnType<typeof setTimeout> | null>(null)`。

### L9. 哨兵常量提取

在 `src/parser.rs` 或新建 `src/parser/constants.rs` 中定义：
```rust
pub const INDENT_MARKER: &str = "\x01INDENT\x01";
pub const INLINE_IMG_PREFIX: char = '\u{E000}';
pub const INLINE_IMG_SUFFIX: char = '\u{E001}';
```

`epub_content.rs` 和 `chapter_builder.rs` 引用这些常量而非硬编码。

---

## 执行顺序总结

| 序号 | 步骤 | 改动量 | 风险 | 前置依赖 |
|------|------|--------|------|---------|
| 1.1 | segmenter 字节→字符 | 极小 | 低 | 无 |
| 1.2 | tts_max_text_length | 极小 | 低 | 无 |
| 1.3 | synthesize_blocking 缓存检查 | 小 | 低 | 无 |
| 2.1 | tts_start async | 小 | 中 | 1.3 |
| 2.2 | TXT BufReader | 小 | 中 | 无 |
| 2.3 | TXT 编码检测 | 中 | 中 | 2.2 |
| 3.1 | CSS 冲突修复 | 中 | 低 | 无 |
| 3.2 | Zustand selector | 小 | 低 | 无 |
| 3.3 | div→button 可访问性 | 中 | 低 | 无 |
| 4.1 | parse() 拆分 | 中 | 低 | 无 |
| 4.2 | 图片去重 | 小 | 低 | 4.1 |
| 4.3 | 格式检测统一 | 小 | 低 | 无 |
| 4.4 | 标签精确匹配 | 极小 | 低 | 无 |
| 4.5 | HtmlExtractorState | 大 | 中 | 4.1-4.4 |
| 5.1 | SQLite 外键 | 中 | 中 | 无 |
| 5.2 | 搜索 to_lowercase | 极小 | 低 | 无 |
| 5.3 | 持锁 DB I/O | 小 | 中 | 无 |
| 5.4 | library async | 大 | 中 | 无 |
| 5.5 | TTS 事件驱动 | 大 | 高 | 2.1 |
| 6.1 | 死代码清理 | 中 | 低 | 无 |
| 6.2 | Display 替换 Debug | 小 | 低 | 无 |
| 6.3 | useTwoPageNavigation 拆分 | 大 | 中 | 无 |
| 6.4 | 正则局部化 | 小 | 低 | 无 |
| 7.x | L1-L14 锦上添花 | 小 | 低 | 无 |

建议按表格顺序执行，每步完成后 `cargo check` + `cargo test` + `npm run build` 验证。步骤 4.5、5.4、5.5 改动量最大，建议单独分支开发。
