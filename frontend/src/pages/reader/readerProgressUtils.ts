import type { SaveProgressDto } from '../../services/api'

export type ReadingPositionSource = 'single' | 'two-page' | 'explicit-navigation'

export interface ReadingPosition {
  bookId: string
  chapterIndex: number
  chapterOffset: number
  source: ReadingPositionSource
  revision: number
}

let nextReadingPositionRevision = 1

/** Chapter-level progress as a fraction of total book chapters. */
export function chapterProgressPercent(chapterIndex: number, chapterCount: number): number {
  if (chapterCount <= 0) return 0
  return Math.min(1, Math.max(0, chapterIndex) / chapterCount)
}

export function clampProgressOffset(offset: number): number {
  return Math.max(0, Math.min(1, Number.isFinite(offset) ? offset : 0))
}

export function chapterOffsetProgressPercent(
  chapterIndex: number,
  chapterCount: number,
  chapterOffset: number,
): number {
  if (chapterCount <= 0) return 0
  return Math.min(1, Math.max(0, chapterIndex + clampProgressOffset(chapterOffset)) / chapterCount)
}

export function createReadingPosition(
  bookId: string,
  chapterIndex: number,
  chapterOffset: number,
  source: ReadingPositionSource,
): ReadingPosition {
  return {
    bookId,
    chapterIndex,
    chapterOffset: clampProgressOffset(chapterOffset),
    source,
    revision: nextReadingPositionRevision++,
  }
}

export function readingPositionProgressPercent(
  position: ReadingPosition,
  chapterCount: number,
): number {
  return chapterOffsetProgressPercent(position.chapterIndex, chapterCount, position.chapterOffset)
}

export function readingPositionToSaveProgress(
  position: ReadingPosition,
  chapterCount: number,
): SaveProgressDto {
  return {
    book_id: position.bookId,
    chapter_index: position.chapterIndex,
    progress_percent: readingPositionProgressPercent(position, chapterCount),
    paragraph_index: null,
    scroll_offset: position.chapterOffset,
    anchor: null,
    clear_position: true,
    revision: position.revision,
  }
}

/** Build progress for an explicit chapter-start navigation. */
export function buildChapterOnlyProgress(
  bookId: string,
  chapterIndex: number,
  chapterCount: number,
): SaveProgressDto {
  return readingPositionToSaveProgress(
    createReadingPosition(bookId, chapterIndex, 0, 'explicit-navigation'),
    chapterCount,
  )
}
