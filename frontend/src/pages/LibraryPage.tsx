import { useEffect, useState, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { libraryList, libraryImport, librarySearch, libraryCover, libraryRemove, libraryRemoveBatch, assetUrl } from '../services/api'
import { open } from '@tauri-apps/plugin-dialog'
import type { LibraryBookCardDto } from '../services/api'
import useAppStore from '../store/useAppStore'
import { coverColor } from '../utils/cover'
import './LibraryPage.css'

function formatProgress(item: LibraryBookCardDto): string {
  if (item.progress_percent >= 1) return `100% · ${item.format.toUpperCase()}`
  if (item.progress_percent > 0) return `${Math.round(item.progress_percent * 100)}% · ${item.format.toUpperCase()}`
  return `未开始 · ${item.format.toUpperCase()}`
}

function lastChapterInfo(item: LibraryBookCardDto): string {
  if (item.chapter_count === 0) return ''
  const ch = Math.max(1, Math.round(item.progress_percent * item.chapter_count))
  return `第 ${ch} 章`
}

function LibraryPage() {
  const navigate = useNavigate()
  const { books, setBooks, startOpening, setSidebarFooter } = useAppStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [isSearching, setIsSearching] = useState(false)
  const [coverImages, setCoverImages] = useState<Record<string, string>>({})
  const [selectMode, setSelectMode] = useState(false)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [deleteConfirm, setDeleteConfirm] = useState<{
    open: boolean; bookIds: string[]; title: string; message: string; deleteFiles: boolean
  }>({ open: false, bookIds: [], title: '', message: '', deleteFiles: false })

  // Load cover images via libraryCover API (returns base64 data URI)
  const loadCovers = useCallback(async (items: LibraryBookCardDto[]) => {
    const results = await Promise.allSettled(
      items.map(async (item) => {
        const uri = await libraryCover(item.book_id)
        return { bookId: item.book_id, uri }
      })
    )
    const covers: Record<string, string> = {}
    for (const r of results) {
      if (r.status === 'fulfilled' && r.value.uri) {
        covers[r.value.bookId] = r.value.uri
      }
    }
    setCoverImages(prev => ({ ...prev, ...covers }))
  }, [])

  const loadBooks = useCallback(async () => {
    try {
      const items = await libraryList()
      setBooks(items)
      // Clean up stale covers for books no longer in the list
      setCoverImages(prev => {
        const validIds = new Set(items.map(i => i.book_id))
        const next: Record<string, string> = {}
        for (const [id, uri] of Object.entries(prev)) {
          if (validIds.has(id)) next[id] = uri
        }
        return next
      })
      loadCovers(items)
    } catch (e) {
      console.error('加载书库失败:', e)
    }
  }, [setBooks, loadCovers])

  useEffect(() => {
    let cancelled = false

    async function loadInitialBooks() {
      try {
        const items = await libraryList()
        if (cancelled) return
        setBooks(items)
        setCoverImages(prev => {
          const validIds = new Set(items.map(i => i.book_id))
          const next: Record<string, string> = {}
          for (const [id, uri] of Object.entries(prev)) {
            if (validIds.has(id)) next[id] = uri
          }
          return next
        })
        await loadCovers(items)
      } catch (e) {
        if (!cancelled) console.error('加载书库失败:', e)
      }
    }

    void loadInitialBooks()
    return () => {
      cancelled = true
    }
  }, [setBooks, loadCovers])

  useEffect(() => {
    setSidebarFooter(`${books.length} 本藏书`)
  }, [books.length, setSidebarFooter])

  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query)
    if (!query.trim()) {
      setIsSearching(false)
      loadBooks()
      return
    }
    setIsSearching(true)
    try {
      const results = await librarySearch(query)
      setBooks(results)
      loadCovers(results)
    } catch (e) {
      console.error('搜索失败:', e)
    }
  }, [loadBooks, setBooks, loadCovers])

  const handleImport = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: '电子书', extensions: ['epub', 'txt'] }],
      })
      if (!selected) return
      const paths = Array.isArray(selected) ? selected : [selected]
      if (paths.length > 0) {
        await libraryImport(paths)
        loadBooks()
      }
    } catch (e) {
      console.error('导入失败:', e)
    }
  }, [loadBooks])

  const handleOpenBook = useCallback((bookId: string) => {
    if (selectMode) return
    const book = books.find(b => b.book_id === bookId)
    if (book) {
      // Use coverImages (base64 data URI) if loaded, otherwise fall back to
      // file path via convertFileSrc to avoid waiting for async loadCovers
      const cover = coverImages[bookId] ?? (book.cover_url ? assetUrl(book.cover_url) : null)
      startOpening(bookId, book.title, book.author, cover)
    }
    navigate(`/loading/${bookId}`)
  }, [navigate, books, coverImages, startOpening, selectMode])

  const toggleSelect = useCallback((bookId: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setSelectedIds(prev => {
      const next = new Set(prev)
      if (next.has(bookId)) next.delete(bookId)
      else next.add(bookId)
      return next
    })
  }, [])

  const handleDeleteSingle = useCallback((bookId: string, e: React.MouseEvent) => {
    e.stopPropagation()
    const book = books.find(b => b.book_id === bookId)
    const title = book?.title ?? '该书籍'
    setDeleteConfirm({
      open: true,
      bookIds: [bookId],
      title: '移除书籍',
      message: `将从书架移除「${title}」，阅读进度和缓存也将一并清除。`,
      deleteFiles: false,
    })
  }, [books])

  const handleDeleteBatch = useCallback(() => {
    const ids = Array.from(selectedIds)
    if (ids.length === 0) return
    setDeleteConfirm({
      open: true,
      bookIds: ids,
      title: '批量移除',
      message: `将从书架移除 ${ids.length} 本书籍，所有阅读进度和缓存也将一并清除。`,
      deleteFiles: false,
    })
  }, [selectedIds])

  const handleDeleteConfirm = useCallback(async () => {
    const { bookIds, deleteFiles } = deleteConfirm
    try {
      if (bookIds.length === 1) {
        await libraryRemove(bookIds[0], deleteFiles)
      } else {
        await libraryRemoveBatch(bookIds, deleteFiles)
      }
    } catch (e) {
      console.error('删除失败:', e)
    }
    setDeleteConfirm(prev => ({ ...prev, open: false }))
    setSelectedIds(new Set())
    setSelectMode(false)
    loadBooks()
  }, [deleteConfirm, loadBooks])

  // "Continue reading" = books with some progress, sorted by last_opened_at
  const continueReading = books
    .filter(b => b.progress_percent > 0 && b.progress_percent < 1 && b.last_opened_at)
    .sort((a, b) => {
      const da = a.last_opened_at ?? ''
      const db = b.last_opened_at ?? ''
      return db.localeCompare(da)
    })
    .slice(0, 3)

  return (
    <main className="library-main">
        <div className="library-header">
          <h1>书架</h1>
          <div className="header-actions">
            <div className="search-box">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
              </svg>
              <input
                type="text"
                placeholder="搜索书籍..."
                value={searchQuery}
                onChange={e => handleSearch(e.target.value)}
              />
            </div>
            <button className={`btn-secondary ${selectMode ? 'active' : ''}`} onClick={() => { setSelectMode(!selectMode); setSelectedIds(new Set()) }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="9 11 12 14 22 4" />
                <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
              </svg>
              {selectMode ? '取消' : '管理'}
            </button>
            <button className="btn-primary" onClick={handleImport}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              导入书籍
            </button>
          </div>
        </div>

        {/* Continue Reading */}
        {continueReading.length > 0 && !isSearching && (
          <>
            <div className="section-title">继续阅读</div>
            <div className="continue-reading">
              {continueReading.map(item => (
                <div
                  key={item.book_id}
                  className="continue-card"
                  onClick={() => handleOpenBook(item.book_id)}
                >
                  <div className={`continue-cover ${coverColor(item.book_id)}`}>
                    {coverImages[item.book_id] ? (
                      <img src={coverImages[item.book_id]} alt={item.title} />
                    ) : (
                      <div className="placeholder">{item.title[0]}</div>
                    )}
                  </div>
                  <div className="continue-info">
                    <div className="continue-title">{item.title}</div>
                    <div className="continue-author">{item.author ?? '未知作者'}</div>
                    <div className="continue-progress-bar">
                      <div
                        className="continue-progress-fill"
                        style={{ width: `${Math.round(item.progress_percent * 100)}%` }}
                      />
                    </div>
                    <div className="continue-progress-text">
                      {lastChapterInfo(item)} · {Math.round(item.progress_percent * 100)}%
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </>
        )}

        {/* All Books */}
        <div className="section-title">{isSearching ? '搜索结果' : '全部书籍'}</div>
        {books.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
              </svg>
            </div>
            <div className="empty-state-text">
              {isSearching ? '没有找到匹配的书籍' : '书库为空，点击"导入书籍"开始'}
            </div>
          </div>
        ) : (
          <div className="book-grid">
            {books.map(item => (
              <div
                key={item.book_id}
                className={`book-card ${selectMode && selectedIds.has(item.book_id) ? 'selected' : ''}`}
                onClick={() => handleOpenBook(item.book_id)}
              >
                <div className={`book-cover ${coverColor(item.book_id)}`}>
                  {coverImages[item.book_id] ? (
                    <img src={coverImages[item.book_id]} alt={item.title} />
                  ) : (
                    <div className="placeholder">{item.title[0]}</div>
                  )}
                  {selectMode ? (
                    <button
                      className={`cover-checkbox ${selectedIds.has(item.book_id) ? 'checked' : ''}`}
                      onClick={(e) => toggleSelect(item.book_id, e)}
                    >
                      {selectedIds.has(item.book_id) && (
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      )}
                    </button>
                  ) : (
                    <button
                      className="cover-delete"
                      onClick={(e) => handleDeleteSingle(item.book_id, e)}
                      title="移除"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                  )}
                </div>
                <div className="book-title">{item.title}</div>
                <div className="book-author">{item.author ?? '未知作者'}</div>
                <div className="book-progress">{formatProgress(item)}</div>
              </div>
            ))}
          </div>
        )}

        {/* Batch delete action bar */}
        {selectMode && selectedIds.size > 0 && (
          <div className="batch-action-bar">
            <span className="batch-count">已选择 {selectedIds.size} 本书籍</span>
            <div className="batch-actions">
              <button className="btn-danger" onClick={handleDeleteBatch}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <polyline points="3 6 5 6 21 6" />
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                </svg>
                批量移除
              </button>
            </div>
          </div>
        )}

        {/* Delete Confirm Dialog */}
        {deleteConfirm.open && (
          <>
            <div className="modal-backdrop" onClick={() => setDeleteConfirm(prev => ({ ...prev, open: false }))} />
            <div className="delete-modal">
              <div className="delete-modal-title">{deleteConfirm.title}</div>
              <div className="delete-modal-message">{deleteConfirm.message}</div>
              <label className="delete-modal-checkbox">
                <input
                  type="checkbox"
                  checked={deleteConfirm.deleteFiles}
                  onChange={e => setDeleteConfirm(prev => ({ ...prev, deleteFiles: e.target.checked }))}
                />
                <span>同时删除本地源文件</span>
              </label>
              <div className="delete-modal-hint">阅读进度和缓存文件将在移除后一并清除，此操作不可撤销。</div>
              {deleteConfirm.deleteFiles && (
                <div className="delete-modal-warning">本地源文件删除后将无法恢复。</div>
              )}
              <div className="delete-modal-actions">
                <button className="btn-secondary" onClick={() => setDeleteConfirm(prev => ({ ...prev, open: false }))}>取消</button>
                <button className="btn-danger" onClick={handleDeleteConfirm}>确认移除</button>
              </div>
            </div>
          </>
        )}
      </main>
  )
}

export default LibraryPage
