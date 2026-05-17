import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { confirm } from '@tauri-apps/plugin-dialog'
import { bookmarkListAll, bookmarkRemove, libraryList } from '../../services/api'
import type { BookmarkDto, LibraryBookCardDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'

export interface BookGroup {
  bookId: string
  title: string
  bookmarks: BookmarkDto[]
}

function groupBookmarks(allBookmarks: BookmarkDto[], books: LibraryBookCardDto[]): BookGroup[] {
  const bookMap = new Map<string, LibraryBookCardDto>()
  for (const b of books) bookMap.set(b.book_id, b)

  const grouped = new Map<string, BookGroup>()
  for (const bm of allBookmarks) {
    if (!grouped.has(bm.book_id)) {
      const book = bookMap.get(bm.book_id)
      grouped.set(bm.book_id, {
        bookId: bm.book_id,
        title: book?.title ?? '未知书籍',
        bookmarks: [],
      })
    }
    grouped.get(bm.book_id)!.bookmarks.push(bm)
  }
  return Array.from(grouped.values())
}

export function useBookmarkPage() {
  const navigate = useNavigate()
  const setSidebarFooter = useAppStore(s => s.setSidebarFooter)
  const [groups, setGroups] = useState<BookGroup[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let cancelled = false

    async function loadBookmarks() {
      try {
        const [allBookmarks, books] = await Promise.all([
          bookmarkListAll(),
          libraryList(),
        ])
        if (cancelled) return
        setGroups(groupBookmarks(allBookmarks, books))
      } catch (e) {
        if (!cancelled) console.error('加载书签失败:', e)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void loadBookmarks()
    return () => {
      cancelled = true
    }
  }, [])

  const handleBookmarkClick = useCallback((bm: BookmarkDto) => {
    useAppStore.getState().setPendingNavTarget({
      chapter_index: bm.chapter_index,
      paragraph_index: bm.paragraph_index ?? null,
      scroll_offset: null,
    })
    navigate(`/loading/${bm.book_id}`)
  }, [navigate])

  const handleDelete = useCallback(async (bm: BookmarkDto) => {
    const ok = await confirm(
      `确定删除该书签？删除后无法恢复。`,
      { title: '删除书签', kind: 'warning' }
    )
    if (!ok) return
    try {
      await bookmarkRemove(bm.book_id, bm.id)
      setGroups(prev => prev
        .map(g => g.bookId === bm.book_id
          ? { ...g, bookmarks: g.bookmarks.filter(b => b.id !== bm.id) }
          : g
        )
        .filter(g => g.bookmarks.length > 0)
      )
    } catch { /* non-critical */ }
  }, [])

  const totalCount = groups.reduce((sum, g) => sum + g.bookmarks.length, 0)

  useEffect(() => {
    setSidebarFooter(`${totalCount} 个书签`)
  }, [totalCount, setSidebarFooter])

  return {
    groups,
    handleBookmarkClick,
    handleDelete,
    loading,
  }
}
