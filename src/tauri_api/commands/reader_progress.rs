use super::super::dto::*;

#[tauri::command]
pub fn reader_save_progress(
    mut progress: SaveProgressDto,
    index_state: tauri::State<'_, super::LibraryIndexState>,
    progress_state: tauri::State<'_, super::ProgressState>,
    dirty_progress_state: tauri::State<'_, super::DirtyProgressState>,
    progress_revision_state: tauri::State<'_, super::ProgressRevisionState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    progress.progress_percent = progress.progress_percent.clamp(0.0, 1.0);
    let normalized_offset = progress.scroll_offset.map(|offset| offset.clamp(0.0, 1.0));
    let incoming_revision = progress.revision.unwrap_or(0);
    {
        let mut revisions = progress_revision_state.lock().map_err(|e| e.to_string())?;
        let current_revision = revisions.get(&progress.book_id).copied().unwrap_or(0);
        if incoming_revision < current_revision {
            log::info!(
                "忽略过期进度: book={}, incoming_rev={}, current_rev={}, incoming_ch={}, incoming_pct={:.0}%",
                progress.book_id,
                incoming_revision,
                current_revision,
                progress.chapter_index,
                progress.progress_percent * 100.0
            );
            return Ok(());
        }
        revisions.insert(progress.book_id.clone(), incoming_revision);
    }
    log::info!(
        "更新内存进度: book={}, rev={}, ch={}, pct={:.0}%",
        progress.book_id,
        incoming_revision,
        progress.chapter_index,
        progress.progress_percent * 100.0
    );
    use crate::domain::reading_progress::ReadingProgress;
    let existing = {
        let progress_map = progress_state.lock().map_err(|e| e.to_string())?;
        progress_map.get(&progress.book_id).cloned()
    };
    // DB 加载在锁外进行
    let existing = if existing.is_some() {
        existing
    } else {
        let saved = db.progress().load(&progress.book_id).ok().flatten();
        if let Some(ref saved) = saved {
            let mut progress_map = progress_state.lock().map_err(|e| e.to_string())?;
            progress_map.insert(progress.book_id.clone(), saved.clone());
        }
        saved
    };
    let same_chapter_existing = existing
        .as_ref()
        .filter(|p| p.chapter_index == progress.chapter_index);
    let clear = progress.clear_position.unwrap_or(false);
    let rp = ReadingProgress {
        book_id: progress.book_id.clone(),
        chapter_index: progress.chapter_index,
        paragraph_index: if clear {
            None
        } else if progress.paragraph_index.is_none() && normalized_offset.is_none() {
            same_chapter_existing.and_then(|p| p.paragraph_index)
        } else {
            progress.paragraph_index
        },
        scroll_offset: normalized_offset
            .or_else(|| same_chapter_existing.map(|p| p.scroll_offset))
            .unwrap_or(0.0),
        progress_percent: progress.progress_percent,
        last_read_at: chrono::Utc::now().to_rfc3339(),
        session_read_seconds: existing
            .as_ref()
            .map(|p| p.session_read_seconds)
            .unwrap_or(0),
        total_read_seconds: existing.as_ref().map(|p| p.total_read_seconds).unwrap_or(0),
        anchor: if clear {
            None
        } else {
            progress.anchor.map(|a| crate::domain::reader_anchor::ReaderAnchor {
                chapter_id: a.chapter_id,
                block_id: a.block_id,
                char_offset: a.char_offset,
            })
        },
    };
    progress_state
        .lock()
        .map_err(|e| e.to_string())?
        .insert(rp.book_id.clone(), rp);
    dirty_progress_state
        .lock()
        .map_err(|e| e.to_string())?
        .insert(progress.book_id.clone());

    // Update the cached library index (no disk I/O per page turn).
    let mut index = index_state.lock().map_err(|e| e.to_string())?;
    if let Some(item) = index
        .items
        .iter_mut()
        .find(|i| i.book_id == progress.book_id)
    {
        item.progress_percent = progress.progress_percent;
        item.last_opened_at = Some(chrono::Utc::now().to_rfc3339());
    }

    Ok(())
}

/// Load saved reading progress for a book (chapter + scroll offset).
#[tauri::command]
pub fn reader_get_progress(
    book_id: String,
    progress_state: tauri::State<'_, super::ProgressState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<Option<SaveProgressDto>, String> {
    let progress = {
        let mut progress_map = progress_state.lock().map_err(|e| e.to_string())?;
        if let Some(progress) = progress_map.get(&book_id) {
            Some(progress.clone())
        } else {
            let saved = db.progress().load(&book_id)?;
            if let Some(progress) = saved.as_ref() {
                progress_map.insert(book_id, progress.clone());
            }
            saved
        }
    };
    Ok(progress.map(progress_to_dto))
}

#[tauri::command]
pub fn reader_flush_progress(
    progress_state: tauri::State<'_, super::ProgressState>,
    dirty_progress_state: tauri::State<'_, super::DirtyProgressState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    flush_dirty_progress_to_db(progress_state.inner(), dirty_progress_state.inner(), db.inner())
}

/// Flush dirty progress entries from in-memory cache to the database.
pub fn flush_dirty_progress_to_db(
    progress_state: &super::ProgressState,
    dirty_progress_state: &super::DirtyProgressState,
    db: &Box<dyn crate::storage::traits::DatabaseBackend>,
) -> Result<(), String> {
    let dirty_ids: Vec<String> = dirty_progress_state
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .cloned()
        .collect();
    if dirty_ids.is_empty() {
        return Ok(());
    }

    let entries = {
        let progress_map = progress_state.lock().map_err(|e| e.to_string())?;
        dirty_ids
            .iter()
            .filter_map(|book_id| progress_map.get(book_id).map(|progress| (book_id.clone(), progress.clone())))
            .collect::<Vec<_>>()
    };

    let saved_ids: Vec<String> = entries.iter().map(|(id, _)| id.clone()).collect();
    db.progress().save_batch(&entries)?;

    if !saved_ids.is_empty() {
        let mut dirty = dirty_progress_state.lock().map_err(|e| e.to_string())?;
        for book_id in saved_ids {
            dirty.remove(&book_id);
        }
    }

    Ok(())
}

fn progress_to_dto(rp: crate::domain::reading_progress::ReadingProgress) -> SaveProgressDto {
    SaveProgressDto {
        book_id: rp.book_id,
        chapter_index: rp.chapter_index,
        progress_percent: rp.progress_percent,
        paragraph_index: rp.paragraph_index,
        scroll_offset: Some(rp.scroll_offset),
        anchor: rp.anchor.map(|a| ReaderAnchorDto {
            chapter_id: a.chapter_id,
            block_id: a.block_id,
            char_offset: a.char_offset,
        }),
        clear_position: None,
        revision: None,
    }
}
