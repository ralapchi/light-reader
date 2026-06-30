use super::super::dto::*;
use super::dto_convert::{bookmark_to_dto, snap_to_char_boundary};
use super::BookSession;

#[tauri::command]
pub fn search_in_book(
    query: String,
    state: tauri::State<'_, BookSession>,
) -> Result<Vec<SearchHitDto>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;

    let mut hits = Vec::new();
    for chapter in &book.chapters {
        let text_para_count = chapter.text_paragraphs().count();
        for para in chapter.text_paragraphs() {
            let text_lower = para.text.to_lowercase();
            let query_lower = query.to_lowercase();
            if let Some(pos) = text_lower.find(&query_lower) {
                let raw_start = pos.saturating_sub(30);
                let start = snap_to_char_boundary(&para.text, raw_start);
                let raw_end = (pos + query_lower.len() + 30).min(para.text.len());
                let end = snap_to_char_boundary(&para.text, raw_end);
                let mut context = String::new();
                if start > 0 {
                    context.push_str("...");
                }
                context.push_str(&para.text[start..end]);
                if end < para.text.len() {
                    context.push_str("...");
                }

                let progress_hint = format!(
                    "约 {}% 处",
                    ((para.index as f32 / text_para_count.max(1) as f32) * 100.0) as u32
                );

                hits.push(SearchHitDto {
                    chapter_index: chapter.index,
                    chapter_title: chapter.title.clone(),
                    context,
                    progress_hint,
                    paragraph_index: para.index,
                });
                if hits.len() >= 50 {
                    return Ok(hits);
                }
            }
        }
    }
    Ok(hits)
}

#[tauri::command]
pub fn bookmark_list(
    book_id: String,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<Vec<BookmarkDto>, String> {
    let items = db.bookmarks().list(&book_id)?;
    Ok(items.into_iter().map(bookmark_to_dto).collect())
}

#[tauri::command]
pub fn bookmark_list_all(
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<Vec<BookmarkDto>, String> {
    let items = db.bookmarks().list_all()?;
    Ok(items.into_iter().map(bookmark_to_dto).collect())
}

#[tauri::command]
pub fn bookmark_add(
    book_id: String,
    chapter_index: usize,
    paragraph_index: Option<usize>,
    note: Option<String>,
    state: tauri::State<'_, BookSession>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<BookmarkDto, String> {
    use crate::domain::bookmark::Bookmark;

    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;

    let chapter = book
        .chapters
        .get(chapter_index)
        .ok_or_else(|| format!("章节 {} 不存在", chapter_index))?;

    let snippet = match paragraph_index {
        Some(pi) => chapter
            .text_paragraphs()
            .find(|p| p.index == pi)
            .map(|p| {
                let s = &p.text;
                if s.len() > 80 {
                    let end = snap_to_char_boundary(s, 80);
                    s[..end].to_string()
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default(),
        None => chapter.title.clone(),
    };

    let bm = Bookmark {
        id: uuid::Uuid::new_v4().to_string(),
        book_id,
        chapter_index,
        paragraph_index,
        title: chapter.title.clone(),
        snippet,
        created_at: chrono::Utc::now().to_rfc3339(),
        note,
    };

    db.bookmarks().add(&bm)?;

    Ok(bookmark_to_dto(bm))
}

#[tauri::command]
pub fn bookmark_remove(
    book_id: String,
    bookmark_id: String,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    db.bookmarks().remove(&book_id, &bookmark_id)
}
