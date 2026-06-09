use super::super::dto::*;
use super::dto_convert::snap_to_char_boundary;
use super::ReaderSession;

#[tauri::command]
pub fn search_in_book(
    query: String,
    state: tauri::State<'_, ReaderSession>,
) -> Result<Vec<SearchHitDto>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    // Clone chapters inside the lock, then drop guard before searching.
    let chapters = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
        book.chapters.clone()
    };

    let mut hits = Vec::new();
    for chapter in &chapters {
        let text_para_count = chapter.text_paragraphs().count();
        for para in chapter.text_paragraphs() {
            if let Some(pos) = para.text.find(&query) {
                let raw_start = pos.saturating_sub(30);
                let start = snap_to_char_boundary(&para.text, raw_start);
                let raw_end = (pos + query.len() + 30).min(para.text.len());
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
pub fn bookmark_list(book_id: String) -> Result<Vec<BookmarkDto>, String> {
    let items = crate::storage::bookmark_store::load(&book_id);
    Ok(items
        .into_iter()
        .map(|b| BookmarkDto {
            id: b.id,
            book_id: b.book_id,
            chapter_index: b.chapter_index,
            paragraph_index: b.paragraph_index,
            title: b.title,
            snippet: b.snippet,
            created_at: b.created_at,
            note: b.note,
        })
        .collect())
}

#[tauri::command]
pub fn bookmark_list_all() -> Result<Vec<BookmarkDto>, String> {
    let items = crate::storage::bookmark_store::load_all();
    Ok(items
        .into_iter()
        .map(|b| BookmarkDto {
            id: b.id,
            book_id: b.book_id,
            chapter_index: b.chapter_index,
            paragraph_index: b.paragraph_index,
            title: b.title,
            snippet: b.snippet,
            created_at: b.created_at,
            note: b.note,
        })
        .collect())
}

#[tauri::command]
pub fn bookmark_add(
    book_id: String,
    chapter_index: usize,
    paragraph_index: Option<usize>,
    note: Option<String>,
    state: tauri::State<'_, ReaderSession>,
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
        book_id: book_id.clone(),
        chapter_index,
        paragraph_index,
        title: chapter.title.clone(),
        snippet: snippet.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        note: note.clone(),
    };

    let mut items = crate::storage::bookmark_store::load(&book_id);
    items.push(bm.clone());
    crate::storage::bookmark_store::save(&book_id, &items).map_err(|e| e.to_string())?;

    Ok(BookmarkDto {
        id: bm.id,
        book_id: bm.book_id,
        chapter_index: bm.chapter_index,
        paragraph_index: bm.paragraph_index,
        title: bm.title,
        snippet: bm.snippet,
        created_at: bm.created_at,
        note: bm.note,
    })
}

#[tauri::command]
pub fn bookmark_remove(book_id: String, bookmark_id: String) -> Result<(), String> {
    let mut items = crate::storage::bookmark_store::load(&book_id);
    let original_len = items.len();
    items.retain(|b| b.id != bookmark_id);
    if items.len() == original_len {
        return Err(format!("书签 {} 不存在", bookmark_id));
    }
    crate::storage::bookmark_store::save(&book_id, &items).map_err(|e| e.to_string())
}
