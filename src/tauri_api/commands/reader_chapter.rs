use crate::domain::book::Book;

use super::super::dto::*;
use super::dto_convert::block_to_dto;
use super::BookSession;

/// Split an href into (file_part, fragment). Fragment is the part after '#'.
fn split_href(href: &str) -> (String, Option<String>) {
    match href.split_once('#') {
        Some((f, frag)) => (f.to_string(), Some(frag.to_string())),
        None => (href.to_string(), None),
    }
}

/// Resolve the chapter index for a given file_part within a book.
/// Falls back to `from_chapter` for fragment-only or empty file_part hrefs.
fn resolve_chapter_index(book: &Book, file_part: &str, from_chapter: usize, href: &str) -> Option<usize> {
    if file_part.is_empty() || href.starts_with('#') {
        return Some(from_chapter);
    }

    let target_file = file_part.rsplit('/').next().unwrap_or(file_part);
    if target_file.is_empty() {
        return None;
    }

    // Primary match: source_href matches file_part, /target_file, or target_file
    book.chapters.iter().position(|ch| {
        ch.source_href
            .as_ref()
            .map(|h| h == file_part || h.ends_with(&format!("/{}", target_file)) || h == target_file)
            .unwrap_or(false)
    })
    .or_else(|| {
        // Fallback: match by filename only
        book.chapters.iter().position(|ch| {
            ch.source_href
                .as_ref()
                .map(|h| {
                    let ch_file = h.rsplit('/').next().unwrap_or(h);
                    ch_file == target_file
                })
                .unwrap_or(false)
        })
    })
}

#[tauri::command]
pub fn reader_get_chapter(
    chapter_index: usize,
    state: tauri::State<'_, BookSession>,
) -> Result<ReaderChapterDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let chapter = book.chapters.get(chapter_index).ok_or_else(|| {
        format!(
            "章节 {} 不存在 (共 {} 章)",
            chapter_index,
            book.chapters.len()
        )
    })?;

    Ok(ReaderChapterDto {
        chapter_index,
        title: chapter.title.clone(),
        blocks: chapter
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| block_to_dto(b, i))
            .collect(),
        char_count: chapter.char_count,
    })
}

/// Resolve an EPUB href (optionally with fragment) to chapter_index and paragraph_index.
#[tauri::command]
pub fn reader_resolve_href(
    href: String,
    from_chapter_index: Option<usize>,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<ReaderResolvedLinkDto>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let (file_part, fragment) = split_href(&href);

    let chapter_index = match resolve_chapter_index(book, &file_part, from_chapter_index.unwrap_or(0), &href) {
        Some(ci) => ci,
        None => return Ok(None),
    };

    if chapter_index >= book.chapters.len() {
        return Ok(None);
    }

    // 2. If no fragment, just return chapter_index
    let fragment = match fragment {
        Some(f) => f,
        None => {
            return Ok(Some(ReaderResolvedLinkDto {
                chapter_index,
                paragraph_index: None,
                block_index: None,
                scroll_offset: None,
            }));
        }
    };

    // 3. Look up the anchor within the chapter
    let chapter = &book.chapters[chapter_index];
    let paragraph_index = chapter
        .anchors
        .iter()
        .find(|(id, _)| id == &fragment)
        .map(|(_, pi)| *pi);

    Ok(Some(ReaderResolvedLinkDto {
        chapter_index,
        paragraph_index,
        block_index: paragraph_index, // blocks are 1:1 with paragraphs in current model
        scroll_offset: None,
    }))
}

/// Get link preview: resolve href and extract target paragraph text in one IPC call.
#[tauri::command]
pub fn reader_get_link_preview(
    href: String,
    from_chapter_index: usize,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<LinkPreviewDto>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let (file_part, fragment) = split_href(&href);

    let chapter_index = match resolve_chapter_index(book, &file_part, from_chapter_index, &href) {
        Some(ci) => ci,
        None => return Ok(None),
    };

    if chapter_index >= book.chapters.len() {
        return Ok(None);
    }

    let chapter = &book.chapters[chapter_index];

    // Resolve paragraph_index from fragment
    let paragraph_index = fragment.as_ref().and_then(|frag| {
        chapter
            .anchors
            .iter()
            .find(|(id, _)| id == frag)
            .map(|(_, pi)| *pi)
    });

    // Extract paragraph text (trimmed)
    let text = paragraph_index
        .and_then(|pi| {
            chapter.blocks.iter().find_map(|b| match b {
                crate::domain::chapter_block::ChapterBlock::Paragraph(p)
                | crate::domain::chapter_block::ChapterBlock::Heading(p)
                | crate::domain::chapter_block::ChapterBlock::Quote(p)
                    if p.index == pi =>
                {
                    Some(p.text.trim().to_string())
                }
                _ => None,
            })
        })
        .unwrap_or_default();

    Ok(Some(LinkPreviewDto {
        chapter_index,
        paragraph_index,
        text,
        title: None,
    }))
}

#[tauri::command]
pub fn reader_go_to_chapter(
    chapter_index: usize,
    state: tauri::State<'_, BookSession>,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_mut().ok_or("没有打开的书籍")?;
    if chapter_index >= book.chapters.len() {
        return Err(format!(
            "章节 {} 不存在 (共 {} 章)",
            chapter_index,
            book.chapters.len()
        ));
    }
    // Session state chapter index is managed by the frontend; the backend just validates.
    Ok(())
}
