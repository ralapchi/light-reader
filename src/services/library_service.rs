use crate::domain::app_error::AppResult;
use crate::domain::library_item::LibraryItem;

/// Service trait for library management.
///
/// Frontend calls these as Tauri commands; the eframe compat layer
/// implements the same operations via controller::dispatch.
pub trait LibraryService {
    /// List all books in the library index.
    fn list_books(&self) -> Vec<LibraryItem>;

    /// Import one or more book files into the library.
    fn import_books(&self, paths: Vec<String>) -> AppResult<Vec<LibraryItem>>;

    /// Open a book by its stable ID (parse + load into reader state).
    fn open_book(&self, book_id: &str) -> AppResult<()>;

    /// Remove a book from the library index.
    fn remove_book(&self, book_id: &str) -> AppResult<()>;

    /// Search books by title or author.
    fn search(&self, query: &str) -> Vec<LibraryItem>;

    /// Repair a book's source path (e.g. after file was moved).
    fn repair_path(&self, book_id: &str, new_path: &str) -> AppResult<()>;
}
