import { useEffect, useCallback, useState, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import useAppStore from '../store/useAppStore'
import { readerOpenBook, readerGetChapter, readerGetProgress, libraryCover } from '../services/api'
import { useTauriEvent } from '../hooks/useTauriEvent'
import type { BookOpeningProgress, BookOpeningFailed } from '../hooks/useTauriEvent'
import { coverColor } from '../utils/cover'
import './LoadingPage.css'

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return '打开书籍失败'
}

function LoadingPage() {
  const { bookId } = useParams<{ bookId: string }>()
  const navigate = useNavigate()
  const { books, opening, startOpening, setOpeningError, setReaderBook, setCurrentChapter, setProgressPercent } =
    useAppStore()
  const [stageText, setStageText] = useState('准备中...')
  const [fallbackCover, setFallbackCover] = useState<string | null>(null)
  const openingRef = useRef(false)

  // Listen to backend progress events → update stage text
  useTauriEvent<BookOpeningProgress>('book-opening-progress', (payload) => {
    if (payload.book_id === bookId && payload.progress_text) {
      setStageText(payload.progress_text)
    }
  })

  // Listen to backend failure events → show error
  useTauriEvent<BookOpeningFailed>('book-opening-failed', (payload) => {
    if (payload.book_id === bookId) {
      setOpeningError(payload.error_message)
    }
  })

  const openBook = useCallback(async () => {
    if (!bookId || openingRef.current) return
    openingRef.current = true

    const book = books.find(b => b.book_id === bookId)
    // Only initialize opening state if not already set (e.g. by LibraryPage with real cover)
    if (opening.status === 'idle') {
      startOpening(bookId, book?.title ?? '未知书籍', book?.author ?? null, book?.cover_url ?? null)
    }

    try {
      const startTime = Date.now()

      const [readerBook, saved] = await Promise.all([
        readerOpenBook(bookId),
        readerGetProgress(bookId),
      ])

      // Enforce minimum 600ms for smooth UX (only wait the remainder)
      const elapsed = Date.now() - startTime
      const remaining = Math.max(0, 600 - elapsed)
      if (remaining > 0) await new Promise(r => setTimeout(r, remaining))

      // Resume from last reading position
      const resumeChapter = saved?.chapter_index ?? 0
      const clamped = Math.min(resumeChapter, readerBook.chapter_count - 1)
      const chapter = await readerGetChapter(clamped)
      setCurrentChapter(clamped, chapter)
      if (saved) {
        setProgressPercent(saved.progress_percent)
      }
      // 只在没有 pendingNavTarget 时才设置（书签/搜索跳转会先设置 pendingNavTarget）
      const existing = useAppStore.getState().reader.pendingNavTarget
      if (!existing) {
        useAppStore.getState().setPendingNavTarget({
          chapter_index: clamped,
          paragraph_index: saved?.paragraph_index ?? null,
          scroll_offset: saved?.scroll_offset ?? null,
        })
      }

      setReaderBook(readerBook)
      navigate(`/reader/${bookId}`)
    } catch (e: unknown) {
      setOpeningError(errorMessage(e))
    }
  }, [bookId, books, opening.status, startOpening, setOpeningError, setReaderBook, setCurrentChapter, setProgressPercent, navigate])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void openBook()
    }, 0)
    return () => window.clearTimeout(timer)
  }, [openBook])

  // Fallback: fetch cover via libraryCover API (returns base64 data URI)
  useEffect(() => {
    if (!opening.coverUrl && bookId) {
      libraryCover(bookId).then(uri => {
        if (uri) setFallbackCover(uri)
      }).catch(() => {})
    }
  }, [bookId, opening.coverUrl])

  const handleBack = useCallback(() => {
    navigate('/')
  }, [navigate])

  const title = opening.title || books.find(b => b.book_id === bookId)?.title || ''
  const author = opening.author || books.find(b => b.book_id === bookId)?.author || ''

  const coverUrl = opening.coverUrl || fallbackCover

  return (
    <div className="loading-layout">
      <div className="back-hint" onClick={handleBack}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <polyline points="15 18 9 12 15 6" />
        </svg>
        返回书架
      </div>

      {opening.status !== 'error' ? (
        <div className="loading-screen">
          {coverUrl ? (
            <div className="cover-wrapper">
              <img src={coverUrl} alt={title} className="cover-img" />
            </div>
          ) : (
            <div className={`cover-wrapper ${coverColor(bookId ?? '')}`}>
              <div className="cover-placeholder">{title ? title[0] : '?'}</div>
            </div>
          )}
          <div className="book-meta">
            <div className="book-title">{title}</div>
            <div className="book-author">{author || '未知作者'}</div>
          </div>
          <div className="spinner-area">
            <div className="spinner" />
            <div className="status-text">{stageText}</div>
          </div>
        </div>
      ) : (
        <div className="error-state">
          <div className="error-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
          </div>
          <div className="error-title">打开失败</div>
          <div className="error-detail">{opening.errorMessage}</div>
          <div className="error-actions">
            <button className="btn-secondary" onClick={handleBack}>返回书架</button>
            <button className="btn-accent" onClick={openBook}>重新打开</button>
          </div>
        </div>
      )}
    </div>
  )
}

export default LoadingPage
