# 数据库存储层规格文档

> Light Reader — 桌面电子书阅读器 (Tauri 2 + React 19)

## 1. 概述

将 Light Reader 的持久化存储从 JSON 文件迁移到数据库。默认使用内置 SQLite，同时设计可插拔的存储抽象层，后期支持用户配置 PostgreSQL、MySQL 等外部数据库。

### 1.1 动机

| 现状问题 | 数据库方案 |
|----------|-----------|
| 每本书一个 JSON 文件，`load_all` 需遍历目录 | 单文件/远程数据库，索引查询 O(log n) |
| 原子写依赖 rename，无事务保证 | ACID 事务，断电安全 |
| 无法高效做聚合查询（总阅读时长、连续天数） | SQL 聚合函数原生支持 |
| 同步困难，需逐文件比对 | 基于数据库变更日志或行级同步 |
| 无全文搜索能力（当前搜索在内存中进行） | FTS5（SQLite）/ 全文索引（PG/MySQL） |

### 1.2 设计原则

1. **可插拔后端**：通过 Trait 抽象存储接口，SQLite 为默认实现，后期可插入 PG/MySQL 实现
2. **二进制缓存不动**：封面图片、章节图片、TTS 音频仍用文件系统（所有后端通用）
3. **设置文件保留**：`settings.json` 保持不变（包含 API 密钥等敏感信息，便于独立加密/备份）
4. **向前兼容**：首次启动自动从 JSON 迁移到当前后端
5. **统一 SQL 方言**：表结构使用标准 SQL，避免 SQLite 特有语法（如 `CHECK (id = 1)` 改用应用层约束）
6. **用户可配置**：`settings.json` 中配置数据库类型和连接信息

---

## 2. 技术选型

### 2.1 后端类型枚举

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseBackend {
    Sqlite,      // 默认，内置，零配置
    Postgres,    // 远程/本地 PostgreSQL
    Mysql,       // 远程/本地 MySQL
}
```

### 2.2 Rust 依赖

| crate | 用途 | 条件 |
|-------|------|------|
| `rusqlite` | SQLite 绑定，Bundled 特性 | 默认 |
| `r2d2` + `r2d2_sqlite` | SQLite 连接池 | 默认 |
| `tokio-postgres` / `sqlx::postgres` | PostgreSQL 驱动 | `db-postgres` feature |
| `mysql` / `sqlx::mysql` | MySQL 驱动 | `db-mysql` feature |

Cargo.toml：
```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.24"

# 可选后端（按需启用）
tokio-postgres = { version = "0.7", optional = true }
r2d2_postgres = { version = "0.5", optional = true }
mysql = { version = "25", optional = true }
r2d2_mysql = { version = "18", optional = true }

[features]
default = []
db-postgres = ["tokio-postgres", "r2d2_postgres"]
db-mysql = ["mysql", "r2d2_mysql"]
```

### 2.3 默认数据库文件位置

```
{app_data_dir}/reader.db          ← SQLite 主数据库
{app_data_dir}/reader.db-wal      ← WAL 模式日志（自动生成）
{app_data_dir}/reader.db-shm      ← WAL 共享内存（自动生成）
```

PostgreSQL/MySQL 使用远程连接，本地无数据文件。

使用 WAL 模式（`PRAGMA journal_mode=WAL`）提升并发读写性能。

---

## 3. 数据库表设计

### 3.1 ER 关系总览

```
┌─────────────┐     ┌──────────────────┐     ┌──────────────┐
│   books     │────<│ reading_progress  │     │  bookmarks   │
│             │────<│                  │     │              │
│             │────<│                  │     │              │
└─────────────┘     └──────────────────┘     └──────────────┘
       │
       │              ┌──────────────────┐
       └────────────<│  book_tags       │
                      └──────────────────┘

┌──────────────────┐
│ reading_sessions │   ← 独立表，统计模块使用
└──────────────────┘

┌──────────────────┐
│ stats_aggregates │   ← 单行缓存表
└──────────────────┘
```

### 3.2 books（书库）

对应当前 `LibraryIndex.items[]` + `LibraryIndex.last_selected_book_id`。

```sql
CREATE TABLE books (
    book_id         TEXT PRIMARY KEY,          -- "book-{16hex}" 确定性哈希
    title           TEXT NOT NULL,
    author          TEXT,
    format          TEXT NOT NULL,             -- "Epub" | "Txt"
    source_path     TEXT NOT NULL,             -- 原始文件绝对路径
    cover_ext       TEXT,                      -- 封面文件扩展名（"png"/"jpg"等），NULL=无封面
    chapter_count   INTEGER NOT NULL DEFAULT 0,
    file_health     TEXT NOT NULL DEFAULT 'Ok',-- "Ok"|"Missing"|"Moved"|"ParseWarning"

    -- 阅读统计快照（原 ReadingStatsSnapshot）
    total_read_seconds  INTEGER NOT NULL DEFAULT 0,
    last_read_at    TEXT,                      -- RFC3339
    bookmark_count  INTEGER NOT NULL DEFAULT 0,
    last_chapter_index INTEGER,

    -- 进度（冗余字段，原 progress_percent）
    progress_percent REAL NOT NULL DEFAULT 0.0,

    -- 时间戳
    imported_at     TEXT NOT NULL,             -- RFC3339
    last_opened_at  TEXT,                      -- RFC3339
    updated_at      TEXT NOT NULL              -- RFC3339，每次写入更新
);

-- 索引
CREATE INDEX idx_books_last_opened ON books(last_opened_at DESC);
CREATE INDEX idx_books_imported    ON books(imported_at DESC);
CREATE INDEX idx_books_source_path ON books(source_path);

-- 元数据表（单行）
CREATE TABLE app_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- 存储 last_selected_book_id 等全局状态
INSERT INTO app_meta (key, value) VALUES ('schema_version', '1');
INSERT INTO app_meta (key, value) VALUES ('last_selected_book_id', '');
```

### 3.3 reading_progress（阅读进度）

对应当前 `progress/{book_id}.json`，每本书最多一条记录。

```sql
CREATE TABLE reading_progress (
    book_id         TEXT PRIMARY KEY REFERENCES books(book_id) ON DELETE CASCADE,
    chapter_index   INTEGER NOT NULL DEFAULT 0,
    paragraph_index INTEGER,                   -- NULL = 未记录
    scroll_offset   REAL    NOT NULL DEFAULT 0.0,  -- 0.0-1.0
    progress_percent REAL   NOT NULL DEFAULT 0.0,  -- 0.0-1.0
    last_read_at    TEXT,                      -- RFC3339

    -- 锚点（原 ReaderAnchor）
    anchor_chapter_id TEXT,
    anchor_block_id   TEXT,
    anchor_char_offset INTEGER,

    -- 时间统计
    session_read_seconds INTEGER NOT NULL DEFAULT 0,
    total_read_seconds   INTEGER NOT NULL DEFAULT 0,

    -- 写入控制
    revision        INTEGER NOT NULL DEFAULT 0,
    dirty           INTEGER NOT NULL DEFAULT 0  -- 0=clean, 1=dirty
);
```

### 3.4 bookmarks（书签）

对应当前 `bookmarks/{book_id}.json`。

```sql
CREATE TABLE bookmarks (
    id              TEXT PRIMARY KEY,          -- UUID v4
    book_id         TEXT NOT NULL REFERENCES books(book_id) ON DELETE CASCADE,
    chapter_index   INTEGER NOT NULL,
    paragraph_index INTEGER,
    title           TEXT NOT NULL,             -- 章节标题
    snippet         TEXT NOT NULL,             -- 段落前 80 字符
    created_at      TEXT NOT NULL,             -- RFC3339
    note            TEXT                       -- 用户备注
);

CREATE INDEX idx_bookmarks_book ON bookmarks(book_id);
```

### 3.5 book_tags（书籍标签）

新增，支持统计模块的标签云。

```sql
CREATE TABLE book_tags (
    book_id TEXT NOT NULL REFERENCES books(book_id) ON DELETE CASCADE,
    tag     TEXT NOT NULL,
    PRIMARY KEY (book_id, tag)
);

CREATE INDEX idx_book_tags_tag ON book_tags(tag);
```

### 3.6 reading_sessions（阅读会话日志）

对应统计模块的 `sessions/{uuid}.json`，追加式不可变记录。

```sql
CREATE TABLE reading_sessions (
    session_id    TEXT PRIMARY KEY,            -- UUID v4
    book_id       TEXT NOT NULL REFERENCES books(book_id) ON DELETE CASCADE,
    started_at    TEXT NOT NULL,               -- RFC3339
    ended_at      TEXT NOT NULL,               -- RFC3339
    active_seconds INTEGER NOT NULL,
    chapter_start INTEGER NOT NULL,
    chapter_end   INTEGER NOT NULL,
    nav_events    INTEGER NOT NULL DEFAULT 0,
    device_id     TEXT                        -- 未来多设备标识
);

CREATE INDEX idx_sessions_book  ON reading_sessions(book_id);
CREATE INDEX idx_sessions_ended ON reading_sessions(ended_at DESC);
```

### 3.7 stats_aggregates（聚合缓存）

对应统计模块的 `stats_cache.json`，单行缓存表。使用固定 key `'default'` 代替 `CHECK (id = 1)` 以兼容所有数据库。

```sql
CREATE TABLE stats_aggregates (
    id                  TEXT PRIMARY KEY DEFAULT 'default',  -- 固定单行
    total_active_seconds INTEGER NOT NULL DEFAULT 0,
    daily_seconds       TEXT NOT NULL DEFAULT '{}',   -- JSON: {"2026-06-18": 3600, ...}
    per_book_seconds    TEXT NOT NULL DEFAULT '{}',   -- JSON: {"book-xxx": 7200, ...}
    hourly_seconds      TEXT NOT NULL DEFAULT '{}',   -- JSON: {"0": 120, "1": 0, ...}
    active_dates        TEXT NOT NULL DEFAULT '[]',   -- JSON: ["2026-06-18", ...]
    books_completed     INTEGER NOT NULL DEFAULT 0,
    total_nav_events    INTEGER NOT NULL DEFAULT 0,
    computed_at         TEXT NOT NULL                  -- RFC3339
);
```

### 3.8 完整建表 SQL

以下为标准 SQL，兼容 SQLite / PostgreSQL / MySQL。各后端差异（占位符、UPSERT 语法等）封装在 repo 实现中。

```sql
-- SQLite 需要额外的 PRAGMA（在连接层执行，不写入 schema 文件）：
-- PRAGMA journal_mode=WAL;
-- PRAGMA foreign_keys=ON;
-- PRAGMA busy_timeout=5000;

CREATE TABLE IF NOT EXISTS app_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS books (
    book_id             TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    author              TEXT,
    format              TEXT NOT NULL,
    source_path         TEXT NOT NULL,
    cover_ext           TEXT,
    chapter_count       INTEGER NOT NULL DEFAULT 0,
    file_health         TEXT NOT NULL DEFAULT 'Ok',
    total_read_seconds  INTEGER NOT NULL DEFAULT 0,
    last_read_at        TEXT,
    bookmark_count      INTEGER NOT NULL DEFAULT 0,
    last_chapter_index  INTEGER,
    progress_percent    REAL NOT NULL DEFAULT 0.0,
    imported_at         TEXT NOT NULL,
    last_opened_at      TEXT,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_books_last_opened ON books(last_opened_at DESC);
CREATE INDEX IF NOT EXISTS idx_books_imported    ON books(imported_at DESC);

CREATE TABLE IF NOT EXISTS reading_progress (
    book_id              TEXT PRIMARY KEY REFERENCES books(book_id) ON DELETE CASCADE,
    chapter_index        INTEGER NOT NULL DEFAULT 0,
    paragraph_index      INTEGER,
    scroll_offset        REAL NOT NULL DEFAULT 0.0,
    progress_percent     REAL NOT NULL DEFAULT 0.0,
    last_read_at         TEXT,
    anchor_chapter_id    TEXT,
    anchor_block_id      TEXT,
    anchor_char_offset   INTEGER,
    session_read_seconds INTEGER NOT NULL DEFAULT 0,
    total_read_seconds   INTEGER NOT NULL DEFAULT 0,
    revision             INTEGER NOT NULL DEFAULT 0,
    dirty                INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS bookmarks (
    id              TEXT PRIMARY KEY,
    book_id         TEXT NOT NULL REFERENCES books(book_id) ON DELETE CASCADE,
    chapter_index   INTEGER NOT NULL,
    paragraph_index INTEGER,
    title           TEXT NOT NULL,
    snippet         TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    note            TEXT
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_book ON bookmarks(book_id);

CREATE TABLE IF NOT EXISTS book_tags (
    book_id TEXT NOT NULL REFERENCES books(book_id) ON DELETE CASCADE,
    tag     TEXT NOT NULL,
    PRIMARY KEY (book_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_book_tags_tag ON book_tags(tag);

CREATE TABLE IF NOT EXISTS reading_sessions (
    session_id     TEXT PRIMARY KEY,
    book_id        TEXT NOT NULL REFERENCES books(book_id) ON DELETE CASCADE,
    started_at     TEXT NOT NULL,
    ended_at       TEXT NOT NULL,
    active_seconds INTEGER NOT NULL,
    chapter_start  INTEGER NOT NULL,
    chapter_end    INTEGER NOT NULL,
    nav_events     INTEGER NOT NULL DEFAULT 0,
    device_id      TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_book  ON reading_sessions(book_id);
CREATE INDEX IF NOT EXISTS idx_sessions_ended ON reading_sessions(ended_at DESC);

CREATE TABLE IF NOT EXISTS stats_aggregates (
    id                   TEXT PRIMARY KEY DEFAULT 'default',
    total_active_seconds INTEGER NOT NULL DEFAULT 0,
    daily_seconds        TEXT NOT NULL DEFAULT '{}',
    per_book_seconds     TEXT NOT NULL DEFAULT '{}',
    hourly_seconds       TEXT NOT NULL DEFAULT '{}',
    active_dates         TEXT NOT NULL DEFAULT '[]',
    books_completed      INTEGER NOT NULL DEFAULT 0,
    total_nav_events     INTEGER NOT NULL DEFAULT 0,
    computed_at          TEXT NOT NULL
);
```

---

## 4. 存储文件对照

### 4.1 迁移到 SQLite 的数据

| 原文件 | 目标表 | 迁移方式 |
|--------|--------|---------|
| `library_index.json` → `items[]` | `books` | 逐条 INSERT |
| `library_index.json` → `last_selected_book_id` | `app_meta` | INSERT |
| `progress/{book_id}.json` | `reading_progress` | 逐文件 INSERT |
| `bookmarks/{book_id}.json` | `bookmarks` | 逐文件、逐条 INSERT |
| `stats_cache.json` | `stats_aggregates` | 单行 INSERT |
| `sessions/{uuid}.json` | `reading_sessions` | 逐文件 INSERT |
| `book_tags.json` | `book_tags` | 逐条 INSERT |

### 4.2 保留为文件的数据

| 路径 | 原因 |
|------|------|
| `settings.json` | 含 API 密钥，便于独立加密/备份 |
| `cache/covers/{book_id}.{ext}` | 二进制图片，文件系统更高效 |
| `cache/images/{book_id}/{asset_id}.{ext}` | 二进制图片 |
| `cache/tts/{provider}/...` | 二进制音频，500MB 上限 |
| `logs/reader.log` | 日志文件 |

---

## 5. 数据库访问层设计

### 5.1 核心思路：Trait 抽象 + 后端实现

```
命令层 (Tauri commands)
  ↓ 调用
仓储 Trait (storage/traits.rs)       ← 定义纯接口，零实现
  ↑ 实现
SQLite 实现 (storage/sqlite/)        ← 默认后端
PostgreSQL 实现 (storage/postgres/)  ← feature-gated
MySQL 实现 (storage/mysql/)          ← feature-gated
```

命令层只依赖 Trait，不感知具体后端。通过运行时配置选择后端实现。

### 5.2 模块结构

```
src/storage/
  traits.rs               ← 所有仓储 Trait 定义（核心接口）
  factory.rs              ← 根据配置创建后端实例
  migrate.rs              ← JSON → 数据库迁移逻辑（后端无关）

  sqlite/
    mod.rs                ← re-export
    connection.rs         ← SQLite 连接池、PRAGMA
    books_repo.rs         ← impl BooksRepo for SqliteBooksRepo
    progress_repo.rs
    bookmarks_repo.rs
    tags_repo.rs
    sessions_repo.rs
    aggregates_repo.rs
    schema.rs             ← 建表 SQL

  postgres/               ← [feature = "db-postgres"]
    mod.rs
    connection.rs
    books_repo.rs
    ...

  mysql/                  ← [feature = "db-mysql"]
    mod.rs
    connection.rs
    books_repo.rs
    ...

  mod.rs                  ← re-export traits + factory
  paths.rs                ← 保留，新增 db_path()
  util.rs                 ← 保留（settings.json 仍需原子写）
```

### 5.3 仓储 Trait 定义（`traits.rs`）

所有 Trait 方法使用 `&self`，返回 `Result<T, String>`，参数和返回类型均为领域模型类型（与后端无关）。

```rust
use crate::domain::*;

// ── 书籍 ──

pub trait BooksRepo: Send + Sync {
    fn upsert(&self, item: &LibraryItem) -> Result<(), String>;
    fn delete(&self, book_id: &str) -> Result<(), String>;
    fn delete_batch(&self, book_ids: &[&str]) -> Result<(), String>;
    fn list_all(&self) -> Result<Vec<LibraryItem>, String>;
    fn get(&self, book_id: &str) -> Result<Option<LibraryItem>, String>;
    fn search(&self, query: &str) -> Result<Vec<LibraryItem>, String>;
    fn update_progress(&self, book_id: &str, percent: f32, last_opened_at: &str) -> Result<(), String>;
    fn update_stats(&self, book_id: &str, stats: &ReadingStatsSnapshot) -> Result<(), String>;
    fn get_last_selected(&self) -> Result<Option<String>, String>;
    fn set_last_selected(&self, book_id: &str) -> Result<(), String>;
}

// ── 阅读进度 ──

pub trait ProgressRepo: Send + Sync {
    fn load(&self, book_id: &str) -> Result<Option<ReadingProgress>, String>;
    fn save(&self, book_id: &str, progress: &ReadingProgress) -> Result<(), String>;
    fn mark_dirty(&self, book_id: &str, progress: &ReadingProgress, revision: u64) -> Result<(), String>;
    fn flush_dirty(&self) -> Result<Vec<String>, String>;
    fn load_all(&self) -> Result<Vec<(String, ReadingProgress)>, String>;
}

// ── 书签 ──

pub trait BookmarksRepo: Send + Sync {
    fn list(&self, book_id: &str) -> Result<Vec<Bookmark>, String>;
    fn list_all(&self) -> Result<Vec<Bookmark>, String>;
    fn add(&self, bookmark: &Bookmark) -> Result<(), String>;
    fn remove(&self, book_id: &str, bookmark_id: &str) -> Result<(), String>;
    fn clear_for_book(&self, book_id: &str) -> Result<(), String>;
}

// ── 标签 ──

pub trait TagsRepo: Send + Sync {
    fn get_tags(&self, book_id: &str) -> Result<Vec<String>, String>;
    fn set_tags(&self, book_id: &str, tags: &[String]) -> Result<(), String>;
    fn all_tags(&self) -> Result<Vec<(String, u32)>, String>;
}

// ── 阅读会话 ──

pub trait SessionsRepo: Send + Sync {
    fn save(&self, session: &ReadingSession) -> Result<(), String>;
    fn load_all(&self) -> Result<Vec<ReadingSession>, String>;
    fn load_since(&self, date: &str) -> Result<Vec<ReadingSession>, String>;
    fn load_for_book(&self, book_id: &str) -> Result<Vec<ReadingSession>, String>;
}

// ── 聚合缓存 ──

pub trait AggregatesRepo: Send + Sync {
    fn load(&self) -> Result<Option<ReadingAggregates>, String>;
    fn save(&self, agg: &ReadingAggregates) -> Result<(), String>;
}

// ── 数据库管理 ──

pub trait DatabaseBackend: Send + Sync {
    fn books(&self) -> &dyn BooksRepo;
    fn progress(&self) -> &dyn ProgressRepo;
    fn bookmarks(&self) -> &dyn BookmarksRepo;
    fn tags(&self) -> &dyn TagsRepo;
    fn sessions(&self) -> &dyn SessionsRepo;
    fn aggregates(&self) -> &dyn AggregatesRepo;

    /// 执行 schema 迁移（建表等）
    fn migrate(&self) -> Result<(), String>;
}
```

### 5.4 工厂函数（`factory.rs`）

根据配置创建对应的后端实例：

```rust
use crate::domain::settings::DatabaseConfig;

pub fn create_backend(config: &DatabaseConfig, data_dir: &Path) -> Result<Box<dyn DatabaseBackend>, String> {
    match config.backend {
        DatabaseBackend::Sqlite => {
            let db_path = config.path.as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| data_dir.join("reader.db"));
            let backend = SqliteBackend::open(&db_path)?;
            Ok(Box::new(backend))
        }
        #[cfg(feature = "db-postgres")]
        DatabaseBackend::Postgres => {
            let conn_str = config.connection_string.as_ref()
                .ok_or("PostgreSQL requires connection_string")?;
            let backend = PostgresBackend::connect(conn_str)?;
            Ok(Box::new(backend))
        }
        #[cfg(feature = "db-mysql")]
        DatabaseBackend::Mysql => {
            let conn_str = config.connection_string.as_ref()
                .ok_or("MySQL requires connection_string")?;
            let backend = MysqlBackend::connect(conn_str)?;
            Ok(Box::new(backend))
        }
        #[allow(unreachable_patterns)]
        _ => Err(format!("Unsupported backend: {:?}", config.backend)),
    }
}
```

### 5.5 SQLite 后端实现（`sqlite/`）

#### connection.rs

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type SqlitePool = Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: &Path) -> Result<SqlitePool, String> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)
        .map_err(|e| e.to_string())?;

    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        PRAGMA busy_timeout=5000;
    ").map_err(|e| e.to_string())?;

    Ok(pool)
}
```

#### 每个 repo 实现示例（books_repo.rs）

```rust
pub struct SqliteBooksRepo {
    pool: SqlitePool,
}

impl SqliteBooksRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
}

impl BooksRepo for SqliteBooksRepo {
    fn upsert(&self, item: &LibraryItem) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO books (...) VALUES (...) ON CONFLICT(book_id) DO UPDATE SET ...",
            rusqlite::params![...],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
    // ... 其余方法
}
```

#### SQLite 后端顶层结构

```rust
pub struct SqliteBackend {
    pool: SqlitePool,
    books: SqliteBooksRepo,
    progress: SqliteProgressRepo,
    bookmarks: SqliteBookmarksRepo,
    tags: SqliteTagsRepo,
    sessions: SqliteSessionsRepo,
    aggregates: SqliteAggregatesRepo,
}

impl SqliteBackend {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        let pool = create_pool(db_path)?;
        let backend = Self {
            books: SqliteBooksRepo::new(pool.clone()),
            progress: SqliteProgressRepo::new(pool.clone()),
            bookmarks: SqliteBookmarksRepo::new(pool.clone()),
            tags: SqliteTagsRepo::new(pool.clone()),
            sessions: SqliteSessionsRepo::new(pool.clone()),
            aggregates: SqliteAggregatesRepo::new(pool.clone()),
            pool,
        };
        backend.migrate()?;
        Ok(backend)
    }
}

impl DatabaseBackend for SqliteBackend {
    fn books(&self) -> &dyn BooksRepo { &self.books }
    fn progress(&self) -> &dyn ProgressRepo { &self.progress }
    fn bookmarks(&self) -> &dyn BookmarksRepo { &self.bookmarks }
    fn tags(&self) -> &dyn TagsRepo { &self.tags }
    fn sessions(&self) -> &dyn SessionsRepo { &self.sessions }
    fn aggregates(&self) -> &dyn AggregatesRepo { &self.aggregates }

    fn migrate(&self) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute_batch(SCHEMA_SQL).map_err(|e| e.to_string())?;
        Ok(())
    }
}
```

### 5.6 PostgreSQL / MySQL 后端（预留）

```rust
// postgres/backend.rs
pub struct PostgresBackend {
    pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
    // ... 各 repo 实例
}

impl PostgresBackend {
    pub fn connect(conn_str: &str) -> Result<Self, String> { ... }
}

impl DatabaseBackend for PostgresBackend {
    fn books(&self) -> &dyn BooksRepo { &self.books }
    // ...
}

// mysql/backend.rs
pub struct MysqlBackend { ... }
impl DatabaseBackend for MysqlBackend { ... }
```

后端间的 SQL 差异：
| 差异点 | SQLite | PostgreSQL | MySQL |
|--------|--------|------------|-------|
| 占位符 | `?` | `$1, $2, ...` | `?` |
| UPSERT | `ON CONFLICT DO UPDATE` | `ON CONFLICT DO UPDATE` | `ON DUPLICATE KEY UPDATE` |
| 布尔值 | `INTEGER 0/1` | `BOOLEAN` | `TINYINT(1)` |
| JSON 字段 | `TEXT` + 应用层序列化 | `JSONB`（原生索引） | `JSON` |
| 自增 ID | `INTEGER PRIMARY KEY` (隐式) | `SERIAL` / `GENERATED` | `AUTO_INCREMENT` |

这些差异封装在各后端的 repo 实现中，对命令层完全透明。

### 5.7 Tauri State 集成

```rust
// main.rs
pub struct AppState {
    pub db: Box<dyn DatabaseBackend>,      // 后端无关，运行时多态
    pub library: Mutex<LibraryIndex>,      // 仍保留内存缓存，启动时从 DB 加载
    pub progress: ProgressState,           // HashMap，仍保留内存缓存
    pub dirty_progress: DirtyProgressState,
    pub reader: Mutex<Option<ReaderState>>,
    pub tts: Mutex<TtsSession>,
}
```

命令层通过 `db.books()`、`db.progress()` 等方法访问存储，完全不感知底层是 SQLite 还是 PostgreSQL。

---

## 6. JSON → 数据库迁移

### 6.1 迁移触发

在 `factory::create_backend()` 返回后：
1. 调用 `backend.migrate()` 确保表结构存在
2. 检查 `app_meta` 表中 `schema_version` 是否存在
3. 如果不存在（首次打开新数据库），执行 JSON → 数据库完整迁移
4. 迁移成功后写入 `schema_version = 1`

### 6.2 迁移流程（`migrate.rs`）

迁移逻辑使用 Trait 接口，与具体后端无关：

```rust
pub fn migrate_from_json(db: &dyn DatabaseBackend, data_dir: &Path) -> Result<(), String> {

    // 1. 迁移 library_index.json → books 表
    if let Ok(index) = library_store::load() {
        for item in &index.items {
            db.books().upsert(item)?;
        }
        if let Some(id) = &index.last_selected_book_id {
            db.books().set_last_selected(id)?;
        }
    }

    // 2. 迁移 progress/*.json → reading_progress 表
    let progress_dir = data_dir.join("progress");
    if progress_dir.exists() {
        for entry in fs::read_dir(&progress_dir).flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(file) = serde_json::from_str::<ProgressFile>(&data) {
                        db.progress().save(&file.progress.book_id, &file.progress)?;
                    }
                }
            }
        }
    }

    // 3. 迁移 bookmarks/*.json → bookmarks 表
    let bm_dir = data_dir.join("bookmarks");
    if bm_dir.exists() {
        for entry in fs::read_dir(&bm_dir).flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(file) = serde_json::from_str::<BookmarksFile>(&data) {
                        for bm in &file.items {
                            db.bookmarks().add(bm)?;
                        }
                    }
                }
            }
        }
    }

    // 4. 迁移 stats_cache.json → stats_aggregates 表
    if let Ok(agg) = stats_store::load_aggregates() {
        db.aggregates().save(&agg)?;
    }

    // 5. 迁移 sessions/*.json → reading_sessions 表
    let sessions_dir = data_dir.join("sessions");
    if sessions_dir.exists() {
        for entry in fs::read_dir(&sessions_dir).flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<ReadingSession>(&data) {
                        db.sessions().save(&session)?;
                    }
                }
            }
        }
    }

    // 6. 迁移 book_tags.json → book_tags 表
    if let Ok(tags) = tag_store::load_all_tags() {
        for bt in &tags {
            db.tags().set_tags(&bt.book_id, &bt.tags)?;
        }
    }

    // 7. 归档旧文件
    archive_json_files(data_dir)?;

    Ok(())
}
```

### 6.3 归档策略

迁移成功后，将旧 JSON 文件移动到 `{app_data_dir}/archive/json_migration_{timestamp}/`：

```
archive/
  json_migration_20260623/
    library_index.json
    progress/
    bookmarks/
    stats_cache.json
    sessions/
    book_tags.json
```

不删除原始文件，保留一个版本供回滚。

### 6.4 回滚机制

如果数据库损坏或用户需要回滚：
1. 删除 `reader.db`
2. 将 `archive/json_migration_*/` 下的文件移回原位
3. 下次启动时重新执行迁移

---

## 7. 命令层适配

### 7.1 需要修改的命令

| 命令 | 当前实现 | 修改后 |
|------|---------|--------|
| `library_list` | 从内存 `LibraryIndex` 读取 | 不变（内存缓存仍有效） |
| `library_import` | `LibraryIndex` + 写 JSON | 同时写入 `db.books().upsert()` |
| `library_remove` | 删 JSON 文件 | `db.books().delete()` + CASCADE |
| `library_flush_index` | 写 `library_index.json` | `db.books().upsert()` 批量 |
| `reader_save_progress` | 内存 HashMap | 不变（仍用内存缓存 + dirty 标记） |
| `reader_flush_progress` | 写 `progress/*.json` | `db.progress().flush_dirty()` |
| `bookmark_add` | 读写 `bookmarks/*.json` | `db.bookmarks().add()` |
| `bookmark_remove` | 读写 `bookmarks/*.json` | `db.bookmarks().remove()` |
| `bookmark_list` | 扫描文件 | `db.bookmarks().list()` |
| `bookmark_list_all` | 扫描所有文件 | `db.bookmarks().list_all()` |
| `settings_load` | 读 `settings.json` | 不变 |
| `settings_save` | 写 `settings.json` | 不变 |
| `tts_config_load/save` | 读写 `settings.json` | 不变 |

### 7.2 窗口关闭处理

```rust
// main.rs - on_window_event
CloseRequested => {
    // 1. flush 脏进度到数据库
    flush_dirty_progress_to_db(&db, &dirty_progress);
    // 2. 保存 library 到数据库
    flush_library_to_db(&db, &library);
    // 3. 不再需要写 JSON 文件
}
```

---

## 8. 性能考虑

### 8.1 读写策略

| 数据类型 | 读策略 | 写策略 |
|----------|--------|--------|
| books | 内存缓存 + DB | 每次变更同步写 DB |
| progress | 内存 HashMap + dirty 标记 | 30s / 窗口关闭时 flush 到 DB |
| bookmarks | 直接读 DB（低频操作） | 直接写 DB |
| settings | JSON 文件（不变） | JSON 文件（不变） |
| sessions | 直接写 DB（追加式） | session 结束时写 DB |
| aggregates | 内存缓存 | session 结束时更新 |

### 8.2 WAL 模式优势

- 读操作不阻塞写操作
- 写操作不阻塞读操作
- 适合"频繁读、偶尔写"的桌面应用场景

### 8.3 连接池大小

默认 5 个连接。Tauri 命令是同步执行的（`#[tauri::command]`），单窗口场景下 2-3 个连接足够，5 个留有余量。

---

## 9. 数据库配置

### 9.1 settings.json 扩展

```json
{
  "version": 1,
  "reader_settings": { ... },
  "database": {
    "backend": "sqlite",
    "path": null,
    "connection_string": null,
    "max_connections": 5
  },
  ...
}
```

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `backend` | `"sqlite"` \| `"postgres"` \| `"mysql"` | `"sqlite"` | 数据库后端类型 |
| `path` | `string?` | `null` | SQLite 数据库文件路径；`null` = `{app_data_dir}/reader.db` |
| `connection_string` | `string?` | `null` | PG/MySQL 连接字符串，如 `postgres://user:pass@host/db` |
| `max_connections` | `number` | `5` | 连接池大小 |

### 9.2 Rust 配置结构

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub backend: DatabaseBackend,
    pub path: Option<String>,
    pub connection_string: Option<String>,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 { 5 }

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: DatabaseBackend::Sqlite,
            path: None,
            connection_string: None,
            max_connections: 5,
        }
    }
}
```

### 9.3 初始化逻辑

```rust
// main.rs
let settings = settings_store::load();
let db = storage::factory::create_backend(&settings.database, &app_data_dir)?;

// 注册为 Tauri State
app.manage(AppState {
    db,
    ...
});
```

### 9.4 后端切换流程

用户在设置页面修改数据库配置后：
1. 前端调用 `settings_save` 保存新配置
2. 提示用户"数据库配置已更改，需要重启应用生效"
3. 下次启动时使用新配置初始化数据库连接

不支持运行时热切换后端（风险高、收益低）。

### 9.5 各后端配置示例

**SQLite（默认，零配置）：**
```json
{ "backend": "sqlite" }
```

**SQLite 自定义路径：**
```json
{ "backend": "sqlite", "path": "/Users/me/Documents/reader.db" }
```

**PostgreSQL：**
```json
{
  "backend": "postgres",
  "connection_string": "postgres://reader:password@localhost:5432/reader_db"
}
```

**MySQL：**
```json
{
  "backend": "mysql",
  "connection_string": "mysql://reader:password@localhost:3306/reader_db"
}
```

---

## 10. 文件清单

### 10.1 新建文件

| 文件 | 说明 |
|------|------|
| `src/storage/traits.rs` | 所有仓储 Trait 定义 |
| `src/storage/factory.rs` | 后端工厂函数 |
| `src/storage/migrate.rs` | JSON → 数据库迁移逻辑 |
| `src/storage/sqlite/mod.rs` | SQLite 模块入口 |
| `src/storage/sqlite/connection.rs` | SQLite 连接池、PRAGMA |
| `src/storage/sqlite/schema.rs` | 建表 SQL |
| `src/storage/sqlite/books_repo.rs` | impl BooksRepo |
| `src/storage/sqlite/progress_repo.rs` | impl ProgressRepo |
| `src/storage/sqlite/bookmarks_repo.rs` | impl BookmarksRepo |
| `src/storage/sqlite/tags_repo.rs` | impl TagsRepo |
| `src/storage/sqlite/sessions_repo.rs` | impl SessionsRepo |
| `src/storage/sqlite/aggregates_repo.rs` | impl AggregatesRepo |
| `src/storage/postgres/` | PostgreSQL 后端（feature-gated，后期实现） |
| `src/storage/mysql/` | MySQL 后端（feature-gated，后期实现） |

### 10.2 修改文件

| 文件 | 修改内容 |
|------|---------|
| `Cargo.toml` | 新增 `rusqlite`, `r2d2`, `r2d2_sqlite`；可选 `tokio-postgres`, `mysql` |
| `src/main.rs` | 通过 `factory::create_backend` 初始化 DB，注册为 Tauri State |
| `src/storage/mod.rs` | 注册 `traits`, `factory`, `migrate`, `sqlite` 模块 |
| `src/storage/paths.rs` | 新增 `db_path()` |
| `src/tauri_api/commands/reader.rs` | `reader_flush_progress` 改用 `db.progress()` Trait |
| `src/tauri_api/commands/bookmarks.rs` | 书签命令改用 `db.bookmarks()` Trait |
| `src/tauri_api/commands/library.rs` | 库命令改用 `db.books()` Trait |
| `src/domain/settings.rs` | `SettingsFile` 新增 `DatabaseConfig` 字段 |

### 10.3 不变文件

| 文件 | 原因 |
|------|------|
| `src/storage/settings_store.rs` | settings.json 保持不变 |
| `src/storage/util.rs` | 原子写仍用于 settings.json |
| `src/tts/cache.rs` | TTS 缓存仍用文件系统 |
| `src/services/asset_service_impl.rs` | 图片缓存仍用文件系统 |
| 前端所有文件 | API 接口不变，前端无感知 |

---

## 11. 验证标准

1. **首次启动迁移**：已有 JSON 数据自动迁移到 SQLite，旧文件归档到 `archive/`
2. **数据完整性**：迁移后 books、progress、bookmarks、tags、sessions 记录数与原 JSON 文件一致
3. **CRUD 正常**：导入书籍、保存进度、添加/删除书签、标签编辑功能正常
4. **性能无退化**：书库列表加载、章节切换、进度保存的响应时间与迁移前持平或更快
5. **断电安全**：强制终止应用后重启，数据库无损坏（WAL 自动恢复）
6. **CASCADE 删除**：删除书籍时，关联的 progress、bookmarks、tags、sessions 自动清理
7. **空库启动**：无历史数据的全新安装正常初始化空数据库
8. **后端切换**：修改 `settings.json` 中 `database.backend` 后重启，能正常连接新后端
9. **Trait 隔离**：命令层代码中无任何 SQLite/PG/MySQL 特定类型引用（编译时 `#[cfg]` 除外）
