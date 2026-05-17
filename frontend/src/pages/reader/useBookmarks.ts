import { useCallback, useEffect } from 'react'
import { bookmarkAdd, bookmarkList, bookmarkRemove } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { findVisibleParagraphIndex } from './readerUtils'

export function useBookmarks(
  bookId: string | undefined,
  currentChapterIndex: number,
  contentRef: React.RefObject<HTMLDivElement | null>,
) {
  const { setBookmarks } = useAppStore()
  const bookmarks = useAppStore(s => s.reader.bookmarks)

  const loadBookmarks = useCallback(async () => {
    if (!bookId) return
    try {
      const list = await bookmarkList(bookId)
      setBookmarks(list)
    } catch { /* non-critical */ }
  }, [bookId, setBookmarks])

  useEffect(() => {
    const timer = window.setTimeout(() => { void loadBookmarks() }, 0)
    return () => window.clearTimeout(timer)
  }, [loadBookmarks])

  const currentBookmark = bookmarks.find(b => b.chapter_index === currentChapterIndex)

  const toggleBookmark = useCallback(async () => {
    if (!bookId) return
    const current = useAppStore.getState().reader.bookmarks.find(b => b.chapter_index === currentChapterIndex)
    if (current) {
      try {
        await bookmarkRemove(bookId, current.id)
        setBookmarks(useAppStore.getState().reader.bookmarks.filter(b => b.id !== current.id))
      } catch { /* non-critical */ }
    } else {
      try {
        const el = contentRef.current
        const paraIndex = el ? findVisibleParagraphIndex(el) : null
        const bm = await bookmarkAdd(bookId, currentChapterIndex, paraIndex ?? undefined)
        setBookmarks([...useAppStore.getState().reader.bookmarks, bm])
      } catch { /* non-critical */ }
    }
  }, [bookId, currentChapterIndex, setBookmarks, contentRef])

  return { currentBookmark, toggleBookmark }
}
