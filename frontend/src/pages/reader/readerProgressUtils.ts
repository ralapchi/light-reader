import type { SaveProgressDto } from '../../services/api'

/** Chapter-level progress as a fraction of total book chapters. */
export function chapterProgressPercent(chapterIndex: number, chapterCount: number): number {
  if (chapterCount <= 0) return 0
  return Math.min(1, Math.max(0, chapterIndex) / chapterCount)
}

/** Build a chapter-level SaveProgressDto (no paragraph/scroll/anchor). */
export function buildChapterOnlyProgress(
  bookId: string,
  chapterIndex: number,
  chapterCount: number,
): SaveProgressDto {
  return {
    book_id: bookId,
    chapter_index: chapterIndex,
    progress_percent: chapterProgressPercent(chapterIndex, chapterCount),
    paragraph_index: null,
    scroll_offset: null,
    anchor: null,
    clear_position: true,
  }
}
