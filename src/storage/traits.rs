use crate::domain::bookmark::Bookmark;
use crate::domain::library_item::{LibraryItem, ReadingStatsSnapshot};
use crate::domain::reading_aggregates::ReadingAggregates;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::reading_session::ReadingSession;

// -- Books --

pub trait BooksRepo: Send + Sync {
    fn upsert(&self, item: &LibraryItem) -> Result<(), String>;
    fn delete(&self, book_id: &str) -> Result<(), String>;
    fn delete_batch(&self, book_ids: &[&str]) -> Result<(), String>;
    fn list_all(&self) -> Result<Vec<LibraryItem>, String>;
    fn get(&self, book_id: &str) -> Result<Option<LibraryItem>, String>;
    fn search(&self, query: &str) -> Result<Vec<LibraryItem>, String>;
    fn update_progress(
        &self,
        book_id: &str,
        percent: f32,
        last_opened_at: &str,
    ) -> Result<(), String>;
    fn update_stats(&self, book_id: &str, stats: &ReadingStatsSnapshot) -> Result<(), String>;
    fn get_last_selected(&self) -> Result<Option<String>, String>;
    fn set_last_selected(&self, book_id: &str) -> Result<(), String>;
}

// -- Reading Progress --

pub trait ProgressRepo: Send + Sync {
    fn load(&self, book_id: &str) -> Result<Option<ReadingProgress>, String>;
    fn save(&self, book_id: &str, progress: &ReadingProgress) -> Result<(), String>;
    fn save_batch(&self, entries: &[(String, ReadingProgress)]) -> Result<(), String>;
    fn mark_dirty(
        &self,
        book_id: &str,
        progress: &ReadingProgress,
        revision: u64,
    ) -> Result<(), String>;
    fn flush_dirty(&self) -> Result<Vec<String>, String>;
    fn load_all(&self) -> Result<Vec<(String, ReadingProgress)>, String>;
}

// -- Bookmarks --

pub trait BookmarksRepo: Send + Sync {
    fn list(&self, book_id: &str) -> Result<Vec<Bookmark>, String>;
    fn list_all(&self) -> Result<Vec<Bookmark>, String>;
    fn add(&self, bookmark: &Bookmark) -> Result<(), String>;
    fn remove(&self, book_id: &str, bookmark_id: &str) -> Result<(), String>;
    fn clear_for_book(&self, book_id: &str) -> Result<(), String>;
}

// -- Tags --

pub trait TagsRepo: Send + Sync {
    fn get_tags(&self, book_id: &str) -> Result<Vec<String>, String>;
    fn set_tags(&self, book_id: &str, tags: &[String]) -> Result<(), String>;
    fn all_tags(&self) -> Result<Vec<(String, u32)>, String>;
}

// -- Reading Sessions --

pub trait SessionsRepo: Send + Sync {
    fn save(&self, session: &ReadingSession) -> Result<(), String>;
    fn load_all(&self) -> Result<Vec<ReadingSession>, String>;
    fn load_since(&self, date: &str) -> Result<Vec<ReadingSession>, String>;
    fn load_for_book(&self, book_id: &str) -> Result<Vec<ReadingSession>, String>;
}

// -- Aggregates --

pub trait AggregatesRepo: Send + Sync {
    fn load(&self) -> Result<Option<ReadingAggregates>, String>;
    fn save(&self, agg: &ReadingAggregates) -> Result<(), String>;
}

// -- Database Backend (composite trait) --

pub trait DatabaseBackend: Send + Sync {
    fn books(&self) -> &dyn BooksRepo;
    fn progress(&self) -> &dyn ProgressRepo;
    fn bookmarks(&self) -> &dyn BookmarksRepo;
    fn tags(&self) -> &dyn TagsRepo;
    fn sessions(&self) -> &dyn SessionsRepo;
    fn aggregates(&self) -> &dyn AggregatesRepo;

    /// Run schema migrations (create tables, etc.)
    fn migrate(&self) -> Result<(), String>;
}
