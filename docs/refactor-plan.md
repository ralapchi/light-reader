# 代码结构整改方案

> 基于 2026-06-26 全量代码审查，按软件设计原则（单一职责、消除重复、不过度设计）整理。

## P0 — 结构性问题

### 1. `parser/parsers/epub.rs`（1535 行）拆分

**问题**：20+ 方法全在一个 struct impl 里，混合 OPF 元数据解析、HTML 内容提取、TOC 树构建三个独立关注点。

**方案**：拆为 4 个文件，`EpubParser` struct 保留在主文件，子模块方法通过 `impl EpubParser` 分文件实现。

| 新文件 | 职责 | 迁出方法 |
|--------|------|----------|
| `epub_parser.rs` | 主结构 + `parse()` 入口 + 辅助函数 | `new`, `parse`, `is_non_body_spine_item`, `split_href` 等 |
| `epub_metadata.rs` | OPF/封面/manifest 解析 | `parse_container_xml`, `parse_opf_file`, `parse_opf_metadata`, `get_full_path` |
| `epub_content.rs` | HTML 内容提取与清洗 | `extract_html_with_positions`, `extract_title`, `clean_title`, `is_xml_noise`, `is_cover_only_text` |
| `epub_toc.rs` | 目录解析与树构建 | `parse_nav_document`, `parse_nav_with_filter`, `parse_ncx`, `build_toc_tree`, `find_parent_mut`, `build_toc_title_map` |

公共辅助函数 `is_img_tag`, `read_img_attrs`, `record_anchor` 保留在 `epub_parser.rs`。

**验证**：`cargo check` 通过，`parser/parsers/tests.rs` 全部测试通过。

---

### 2. `tauri_api/commands/reader.rs`（813 行）拆分

**问题**：10 个 command + 辅助函数混在一起，涵盖书籍加载、章节导航、图片处理、进度管理四个职责。

**方案**：按功能域拆为 4 个文件。

| 新文件 | 包含的 command / 函数 |
|--------|----------------------|
| `reader_book.rs` | `reader_get_book`, `reader_open_book` |
| `reader_chapter.rs` | `reader_get_chapter`, `reader_go_to_chapter`, `reader_resolve_href`, `reader_get_link_preview`, `split_href`, `resolve_chapter_index` |
| `reader_image.rs` | `reader_chapter_image`, `reader_chapter_image_path`, `reader_chapter_images`, `resolve_chapter_image_cache_path_blocking`, `extract_image_asset_info` |
| `reader_progress.rs` | `reader_save_progress`, `reader_get_progress`, `reader_flush_progress`, `flush_dirty_progress_to_db`, `flush_dirty_progress_states`, `progress_to_dto` |

原 `reader.rs` 删除，`commands/mod.rs` 中改为 `mod reader_book; mod reader_chapter; ...` 并 `pub use` 所有 command 函数。

**验证**：`cargo check` 通过，所有 `#[tauri::command]` 注册无遗漏。

---

### 3. 旧 JSON 存储与数据库并存 — 统一走数据库

**问题**：`bookmark_store.rs`、`library_store.rs`、`progress_store.rs` 三个 JSON 存储仍在使用，与数据库形成双写和回退逻辑。

**当前引用**：

| 文件 | 行号 | 用途 |
|------|------|------|
| `bootstrap.rs` | 33 | DB 为空时回退读 JSON `library_store::load()` |
| `library.rs` | 13-14 | 删除书籍时清 JSON 书签 + 进度 |
| `reader.rs` | 775 | `flush_dirty_progress_states` 写 JSON |
| `library_service_impl.rs` | 81 | `save_index` 写 JSON |

**方案**：逐项移除双写和回退，统一走数据库 trait。

1. `bootstrap.rs`：移除 `_ => storage::library_store::load()` 回退，DB 为空时返回空 `LibraryIndex`
2. `library_service_impl.rs`：`save_index` 方法改为仅通过 `db.books()` 持久化（或直接移除，由 `bootstrap::flush_library` 负责）
3. `tauri_api/commands/reader.rs`：`flush_dirty_progress_states` 中移除 `progress_store::save` 调用，仅保留 DB 路径
4. `tauri_api/commands/library.rs`：`library_remove` 中移除 `bookmark_store::save` 和 `progress_store::delete`，改用 `db.bookmarks().clear_for_book()` 和 DB 对应操作
5. 删除 `storage/bookmark_store.rs`、`storage/library_store.rs`、`storage/progress_store.rs`
6. `storage/mod.rs` 移除对应 `pub mod` 声明

**验证**：`cargo check` 通过，无对已删除模块的引用。

---

### 4. `services/asset_service_impl.rs`（276 行）— OPF 解析去重

**问题**：`extract_opf_path`（111 行）和 `parse_opf_cover`（128 行）是 EPUB OPF 解析逻辑，与 `parser/parsers/epub.rs` 中的 `parse_opf_file`、`parse_opf_metadata` 存在功能重复。

**方案**：

1. 在 `parser/` 下新建 `opf_utils.rs`，提取公共 OPF 解析函数：
   - `pub fn extract_opf_path(container_xml: &str) -> Option<String>`
   - `pub fn parse_opf_cover(content: &str) -> Option<String>`
   - `pub fn extract_attr(tag: &str, attr: &str) -> Option<String>`
2. `asset_service_impl.rs` 改为调用 `parser::opf_utils::*`
3. `parser/parsers/epub.rs` 中对应的 OPF 解析方法也改为调用 `opf_utils`（或保留内部实现，视具体重叠程度）

**验证**：`cargo check` 通过，封面提取功能正常。

---

## P1 — 规模优化（可选）

### 5. `domain/chapter_builder.rs`（392 行）

全是 `pub(crate)` 函数，内部逻辑完整。当前大小可接受。如后续需扩展，可将段落类型推断独立为 `paragraph_infer.rs`。

**暂不改动**。

### 6. `tauri_api/commands/tts.rs`（474 行）

6 个 TTS command。如后续不再扩展可保持现状。如需拆分，按播放控制 vs 配置/缓存分离。

**暂不改动**。

### 7. 前端 `services/api.ts`（381 行，25+ 函数）拆分

按功能域拆为独立模块，`api.ts` 做 re-export：

```
frontend/src/services/api/
  library.ts      ← libraryList, libraryImport, libraryRemove, libraryRemoveBatch, librarySearch, libraryCover, libraryFlushIndex
  reader.ts       ← readerOpenBook, readerGetChapter, readerChapterImage, readerChapterImages, readerSaveProgress, readerGetProgress, readerFlushProgress, readerResolveHref, readerGetLinkPreview
  search.ts       ← searchInBook, bookmarkList, bookmarkListAll, bookmarkAdd, bookmarkRemove
  settings.ts     ← settingsLoad, settingsSave, ttsConfigLoad, ttsConfigSave
  index.ts        ← re-export all + 类型定义
```

**验证**：`npm run build` 通过，所有 import 路径更新。

### 8. 前端 `statistic-page-*.html` 移出 src

`statistic-page-preview.html`（390 行）和 `statistic-page-v2.html`（552 行）是设计稿静态 HTML，不应在 `frontend/src/pages/` 中。

**方案**：移至 `docs/design/`。

---

## P2 — 不动

| 项目 | 原因 |
|------|------|
| `domain/` 27 个文件（7~392 行） | 小文件是好的领域建模，合并反而降低可读性 |
| `tauri_api/dto.rs` 12 个 DTO（182 行） | 行数可控，按功能拆分收益不大 |
| `tauri_api/commands/dto_convert.rs`（176 行） | 纯转换逻辑，内聚 |
| `storage/sqlite/books_repo.rs`（247 行） | 单一职责，大小合理 |
| CSS 文件（500~1000 行） | CSS 拆分收益低，用 CSS Modules 或 scoped 更实际 |
| `useTwoPageNavigation.ts`（399 行） | 双页导航逻辑复杂度本身高，拆分反而增加认知负担 |

---

## 执行顺序

1. **P0-3**：统一存储路径（先移除 JSON 双写，再删旧模块）
2. **P0-4**：OPF 解析去重
3. **P0-1**：epub.rs 拆分
4. **P0-2**：reader.rs 拆分
5. **P1-7**：前端 api.ts 拆分
6. **P1-8**：HTML 设计稿移出 src

每步完成后 `cargo check` / `npm run build` 验证，确保不引入回归。
