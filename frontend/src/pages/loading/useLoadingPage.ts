import { useCallback, useEffect, useRef, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { libraryCover, readerGetChapter, readerGetProgress, readerOpenBook } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { useTauriEvent } from '../../hooks/useTauriEvent'
import type { BookOpeningFailed, BookOpeningProgress } from '../../hooks/useTauriEvent'

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return '打开书籍失败'
}

export function useLoadingPage() {
  const { bookId } = useParams<{ bookId: string }>()
  const navigate = useNavigate()
  const { books, opening, startOpening, setOpeningError, setReaderBook, setCurrentChapter, setProgressPercent } =
    useAppStore()
  const [stageText, setStageText] = useState('准备中...')
  const [fallbackCover, setFallbackCover] = useState<string | null>(null)
  const openingRef = useRef(false)

  useTauriEvent<BookOpeningProgress>('book-opening-progress', (payload) => {
    if (payload.book_id === bookId && payload.progress_text) {
      setStageText(payload.progress_text)
    }
  })

  useTauriEvent<BookOpeningFailed>('book-opening-failed', (payload) => {
    if (payload.book_id === bookId) {
      setOpeningError(payload.error_message)
    }
  })

  const openBook = useCallback(async () => {
    if (!bookId || openingRef.current) return
    openingRef.current = true

    const book = books.find(b => b.book_id === bookId)
    if (opening.status === 'idle') {
      startOpening(bookId, book?.title ?? '未知书籍', book?.author ?? null, book?.cover_url ?? null)
    }

    try {
      const startTime = Date.now()
      const [readerBook, saved] = await Promise.all([
        readerOpenBook(bookId),
        readerGetProgress(bookId),
      ])

      const elapsed = Date.now() - startTime
      const remaining = Math.max(0, 600 - elapsed)
      if (remaining > 0) await new Promise(r => setTimeout(r, remaining))

      const resumeChapter = saved?.chapter_index ?? 0
      const clamped = Math.min(resumeChapter, readerBook.chapter_count - 1)
      const chapter = await readerGetChapter(clamped)
      setCurrentChapter(clamped, chapter)
      if (saved) {
        setProgressPercent(saved.progress_percent)
      }

      const existing = useAppStore.getState().reader.pendingNavTarget
      if (!existing) {
        useAppStore.getState().setPendingNavTarget({
          chapter_index: clamped,
          paragraph_index: null,
          scroll_offset: null,
          anchor: null,
        })
      }

      setReaderBook(readerBook)
      navigate(`/reader/${bookId}`)
    } catch (e: unknown) {
      setOpeningError(errorMessage(e))
      openingRef.current = false
    }
  }, [bookId, books, opening.status, startOpening, setOpeningError, setReaderBook, setCurrentChapter, setProgressPercent, navigate])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void openBook()
    }, 0)
    return () => window.clearTimeout(timer)
  }, [openBook])

  useEffect(() => {
    if (!opening.coverUrl && bookId) {
      libraryCover(bookId).then(uri => {
        if (uri) setFallbackCover(uri)
      }).catch(() => {})
    }
  }, [bookId, opening.coverUrl])

  const title = opening.title || books.find(b => b.book_id === bookId)?.title || ''
  const author = opening.author || books.find(b => b.book_id === bookId)?.author || ''

  return {
    author,
    bookId,
    coverUrl: opening.coverUrl || fallbackCover,
    handleBack: () => navigate('/'),
    openBook,
    opening,
    stageText,
    title,
  }
}
