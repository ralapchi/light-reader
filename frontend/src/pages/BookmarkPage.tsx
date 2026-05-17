import { useEffect, useState, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { bookmarkListAll, bookmarkRemove, libraryList } from '../services/api'
import type { BookmarkDto, LibraryBookCardDto } from '../services/api'
import { confirm } from '@tauri-apps/plugin-dialog'
import useAppStore from '../store/useAppStore'
import './BookmarkPage.css'

interface BookGroup {
  bookId: string
  title: string
  bookmarks: BookmarkDto[]
}

function BookmarkPage() {
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
        setGroups(Array.from(grouped.values()))
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

  return (
    <main className="bookmark-main">
        <div className="bookmark-page-header">
          <h1>书签</h1>
        </div>

        {loading ? (
          <div className="bookmark-loading">加载中...</div>
        ) : groups.length === 0 ? (
          <div className="bookmark-page-empty">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
            </svg>
            <div>暂无书签</div>
            <div className="bookmark-page-empty-hint">在阅读时按 Ctrl+B 添加书签</div>
          </div>
        ) : (
          <div className="bookmark-groups">
            {groups.map(group => (
              <div key={group.bookId} className="bookmark-group">
                <div className="bookmark-group-title">{group.title}</div>
                <div className="bookmark-group-list">
                  {group.bookmarks.map(bm => (
                    <div
                      key={bm.id}
                      className="bookmark-entry"
                      onClick={() => handleBookmarkClick(bm)}
                    >
                      <div className="bookmark-entry-icon">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" strokeWidth="2">
                          <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
                        </svg>
                      </div>
                      <div className="bookmark-entry-content">
                        <div className="bookmark-entry-title">{bm.title}</div>
                        <div className="bookmark-entry-snippet">{bm.snippet}</div>
                        <div className="bookmark-entry-meta">第 {bm.chapter_index + 1} 章</div>
                      </div>
                      <button
                        className="bookmark-entry-delete"
                        onClick={(e) => { e.stopPropagation(); handleDelete(bm) }}
                        title="删除书签"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <line x1="18" y1="6" x2="6" y2="18" />
                          <line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </main>
  )
}

export default BookmarkPage
