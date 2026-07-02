use crate::domain::bookmark::Bookmark;
use crate::domain::library_item::LibraryItem;
use crate::domain::reading_aggregates::ReadingAggregates;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::reading_session::ReadingSession;

// -- Books --

pub trait BooksRepo: Send + Sync {
    fn upsert(&self, item: &LibraryItem) -> Result<(), String>;
    fn delete(&self, book_id: &str) -> Result<(), String>;
    fn delete_batch(&self, book_ids: &[&str]) -> Result<(), String>;
    fn list_all(&self) -> Result<Vec<LibraryItem>, String>;
    fn get_last_selected(&self) -> Result<Option<String>, String>;
    fn set_last_selected(&self, book_id: &str) -> Result<(), String>;
}

// -- Reading Progress --

pub trait ProgressRepo: Send + Sync {
    fn load(&self, book_id: &str) -> Result<Option<ReadingProgress>, String>;
    fn save_batch(&self, entries: &[(String, ReadingProgress)]) -> Result<(), String>;
}

// -- Bookmarks --

pub trait BookmarksRepo: Send + Sync {
    fn list(&self, book_id: &str) -> Result<Vec<Bookmark>, String>;
    fn list_all(&self) -> Result<Vec<Bookmark>, String>;
    fn add(&self, bookmark: &Bookmark) -> Result<(), String>;
    fn remove(&self, book_id: &str, bookmark_id: &str) -> Result<(), String>;
}

// -- Tags (stats feature, not yet wired to commands) --

#[allow(dead_code)]
pub trait TagsRepo: Send + Sync {
    fn get_tags(&self, book_id: &str) -> Result<Vec<String>, String>;
    fn set_tags(&self, book_id: &str, tags: &[String]) -> Result<(), String>;
    fn all_tags(&self) -> Result<Vec<(String, u32)>, String>;
}

// -- Reading Sessions (stats feature, not yet wired to commands) --

#[allow(dead_code)]
pub trait SessionsRepo: Send + Sync {
    fn save(&self, session: &ReadingSession) -> Result<(), String>;
    fn load_all(&self) -> Result<Vec<ReadingSession>, String>;
    fn load_since(&self, date: &str) -> Result<Vec<ReadingSession>, String>;
    fn load_for_book(&self, book_id: &str) -> Result<Vec<ReadingSession>, String>;
}

// -- Aggregates (stats feature, not yet wired to commands) --

#[allow(dead_code)]
pub trait AggregatesRepo: Send + Sync {
    fn load(&self) -> Result<Option<ReadingAggregates>, String>;
    fn save(&self, agg: &ReadingAggregates) -> Result<(), String>;
}

// -- Database Backend (composite trait) --

pub trait DatabaseBackend: Send + Sync {
    fn books(&self) -> &dyn BooksRepo;
    fn progress(&self) -> &dyn ProgressRepo;
    fn bookmarks(&self) -> &dyn BookmarksRepo;
    #[allow(dead_code)]
    fn tags(&self) -> &dyn TagsRepo;
    #[allow(dead_code)]
    fn sessions(&self) -> &dyn SessionsRepo;
    #[allow(dead_code)]
    fn aggregates(&self) -> &dyn AggregatesRepo;

    /// Run schema migrations (create tables, etc.)
    fn migrate(&self) -> Result<(), String>;
}
