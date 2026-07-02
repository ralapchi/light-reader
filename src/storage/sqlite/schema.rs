/// Full schema SQL for the Light Reader database.
/// Uses standard SQL compatible with SQLite / PostgreSQL / MySQL.
/// SQLite-specific PRAGMAs are executed in connection.rs, not here.
pub const SCHEMA_SQL: &str = "
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
    book_id              TEXT PRIMARY KEY,
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
    dirty                INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS bookmarks (
    id              TEXT PRIMARY KEY,
    book_id         TEXT NOT NULL,
    chapter_index   INTEGER NOT NULL,
    paragraph_index INTEGER,
    title           TEXT NOT NULL,
    snippet         TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    note            TEXT,
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_book ON bookmarks(book_id);

CREATE TABLE IF NOT EXISTS book_tags (
    book_id TEXT NOT NULL,
    tag     TEXT NOT NULL,
    PRIMARY KEY (book_id, tag),
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_book_tags_tag ON book_tags(tag);

CREATE TABLE IF NOT EXISTS reading_sessions (
    session_id     TEXT PRIMARY KEY,
    book_id        TEXT NOT NULL,
    started_at     TEXT NOT NULL,
    ended_at       TEXT NOT NULL,
    active_seconds INTEGER NOT NULL,
    chapter_start  INTEGER NOT NULL,
    chapter_end    INTEGER NOT NULL,
    nav_events     INTEGER NOT NULL DEFAULT 0,
    device_id      TEXT,
    FOREIGN KEY (book_id) REFERENCES books(book_id) ON DELETE CASCADE
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

INSERT OR IGNORE INTO app_meta (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO app_meta (key, value) VALUES ('last_selected_book_id', '');
";
