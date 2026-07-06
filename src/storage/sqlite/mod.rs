pub mod aggregates_repo;
pub mod bookmarks_repo;
pub mod books_repo;
pub mod connection;
pub mod progress_repo;
pub mod schema;
pub mod sessions_repo;
pub mod tag_groups_repo;
pub mod tags_repo;

use std::path::Path;

use connection::{create_pool, SqlitePool};
use schema::SCHEMA_SQL;

use crate::storage::traits::{
    AggregatesRepo, BookmarksRepo, BooksRepo, DatabaseBackend, ProgressRepo, SessionsRepo,
    TagGroupsRepo, TagsRepo,
};

use aggregates_repo::SqliteAggregatesRepo;
use bookmarks_repo::SqliteBookmarksRepo;
use books_repo::SqliteBooksRepo;
use progress_repo::SqliteProgressRepo;
use sessions_repo::SqliteSessionsRepo;
use tag_groups_repo::SqliteTagGroupsRepo;
use tags_repo::SqliteTagsRepo;

pub struct SqliteBackend {
    pool: SqlitePool,
    books: SqliteBooksRepo,
    progress: SqliteProgressRepo,
    bookmarks: SqliteBookmarksRepo,
    tags: SqliteTagsRepo,
    tag_groups: SqliteTagGroupsRepo,
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
            tag_groups: SqliteTagGroupsRepo::new(pool.clone()),
            sessions: SqliteSessionsRepo::new(pool.clone()),
            aggregates: SqliteAggregatesRepo::new(pool.clone()),
            pool,
        };
        backend.migrate()?;
        Ok(backend)
    }
}

impl DatabaseBackend for SqliteBackend {
    fn books(&self) -> &dyn BooksRepo {
        &self.books
    }

    fn progress(&self) -> &dyn ProgressRepo {
        &self.progress
    }

    fn bookmarks(&self) -> &dyn BookmarksRepo {
        &self.bookmarks
    }

    fn tags(&self) -> &dyn TagsRepo {
        &self.tags
    }

    fn tag_groups(&self) -> &dyn TagGroupsRepo {
        &self.tag_groups
    }

    fn sessions(&self) -> &dyn SessionsRepo {
        &self.sessions
    }

    fn aggregates(&self) -> &dyn AggregatesRepo {
        &self.aggregates
    }

    fn migrate(&self) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute_batch(SCHEMA_SQL).map_err(|e| e.to_string())?;
        Ok(())
    }
}
