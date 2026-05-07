use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: String,
    pub chapter_index: usize,
    pub paragraph_index: Option<usize>,
    pub scroll_offset: f32,
    pub progress_percent: f32,
    pub last_read_at: String,
    pub session_read_seconds: u64,
    pub total_read_seconds: u64,
}

impl ReadingProgress {
    /// 章节跳转时构造新进度，保留 paragraph_index、scroll_offset、阅读时长。
    pub fn for_chapter_jump(&self, chapter_index: usize, total_chapters: usize) -> Self {
        let progress_percent = if total_chapters == 0 {
            0.0
        } else {
            ((chapter_index + 1) as f32 / total_chapters as f32).clamp(0.0, 1.0)
        };
        Self {
            book_id: self.book_id.clone(),
            chapter_index,
            paragraph_index: self.paragraph_index,
            scroll_offset: self.scroll_offset,
            progress_percent,
            last_read_at: chrono::Utc::now().to_rfc3339(),
            session_read_seconds: self.session_read_seconds,
            total_read_seconds: self.total_read_seconds,
        }
    }
}
