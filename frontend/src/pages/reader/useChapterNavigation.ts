import { useCallback, useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { readerChapterImage, readerGetChapter, readerSaveProgress } from '../../services/api'
import type { ReaderBookDto, ReaderBlockDto, SearchHitDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { scrollToOffset, scrollToParagraph } from './readerUtils'

export function useChapterNavigation(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  contentRef: React.RefObject<HTMLDivElement | null>,
  handleCloseSearch: () => void,
) {
  const navigate = useNavigate()
  const {
    setCurrentChapter,
    setProgressPercent,
    closeToc,
  } = useAppStore()
  const currentChapter = useAppStore(s => s.reader.currentChapter)
  const currentChapterIndex = useAppStore(s => s.reader.currentChapterIndex)

  // ── Chapter image loading ──────────────────────────────

  const [imageCache, setImageCache] = useState<Record<string, string>>({})
  const loadedImageRef = useRef<Set<string>>(new Set())

  const loadChapterImages = useCallback(async (blocks: ReaderBlockDto[]) => {
    if (!bookId) return
    const imageBlocks = blocks.filter(b => b.type === 'image')
    if (imageBlocks.length === 0) return
    const updates: Record<string, string> = {}
    await Promise.allSettled(
      imageBlocks.map(async (b) => {
        if (b.type !== 'image') return
        if (loadedImageRef.current.has(b.asset_id)) return
        try {
          const dataUri = await readerChapterImage(bookId, b.asset_id)
          if (dataUri) updates[b.asset_id] = dataUri
        } finally {
          loadedImageRef.current.add(b.asset_id)
        }
      })
    )
    if (Object.keys(updates).length > 0) {
      setImageCache(prev => ({ ...prev, ...updates }))
    }
  }, [bookId])

  // ── Navigation ────────────────────────────────────────

  const goToChapter = useCallback(async (
    index: number,
    scrollOffset?: number | null,
    options?: { saveProgress?: boolean },
  ) => {
    try {
      const chapter = await readerGetChapter(index)
      setCurrentChapter(index, chapter)
      closeToc()
      handleCloseSearch()
      loadChapterImages(chapter.blocks)
      const bookPct = book ? Math.min(1, index / book.chapter_count) : 0
      setProgressPercent(bookPct)
      if (bookId && options?.saveProgress !== false) {
        readerSaveProgress({
          book_id: bookId,
          chapter_index: index,
          progress_percent: bookPct,
          paragraph_index: null,
          scroll_offset: 0,
        }).catch(() => { /* non-critical */ })
      }
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          const el = contentRef.current
          if (!el) return
          scrollToOffset(el, scrollOffset && scrollOffset > 0 ? scrollOffset : 0)
        })
      })
    } catch (e) {
      console.error('加载章节失败:', e)
    }
  }, [setCurrentChapter, closeToc, handleCloseSearch, loadChapterImages, bookId, book, setProgressPercent, contentRef])

  const handleSearchResultClick = useCallback((hit: SearchHitDto) => {
    handleCloseSearch()
    goToChapter(hit.chapter_index, null, { saveProgress: false }).then(() => {
      if (hit.paragraph_index != null) {
        requestAnimationFrame(() => {
          const content = contentRef.current
          if (!content) return
          scrollToParagraph(content, hit.paragraph_index)
        })
      }
    })
  }, [handleCloseSearch, goToChapter, contentRef])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      if (!book) {
        navigate(`/loading/${bookId}`)
        return
      }
      const pending = useAppStore.getState().reader.pendingNavTarget
      if (pending) {
        const targetChapter = Math.min(pending.chapter_index, (book.chapter_count ?? 1) - 1)
        useAppStore.getState().setPendingNavTarget(null)
        const alreadyLoaded = currentChapter && currentChapter.chapter_index === targetChapter
        if (alreadyLoaded) {
          if (pending.scroll_offset && pending.scroll_offset > 0) {
            requestAnimationFrame(() => {
              requestAnimationFrame(() => {
                const el = contentRef.current
                if (!el) return
                scrollToOffset(el, pending.scroll_offset!)
              })
            })
          } else if (pending.paragraph_index != null) {
            requestAnimationFrame(() => {
              const content = contentRef.current
              if (!content) return
              scrollToParagraph(content, pending.paragraph_index!)
            })
          }
        } else {
          goToChapter(targetChapter, pending.scroll_offset, { saveProgress: false }).then(() => {
            if (pending.paragraph_index != null && (!pending.scroll_offset || pending.scroll_offset <= 0)) {
              requestAnimationFrame(() => {
                const content = contentRef.current
                if (!content) return
                scrollToParagraph(content, pending.paragraph_index!)
              })
            }
          })
        }
      } else if (!currentChapter) {
        goToChapter(currentChapterIndex)
      }
    }, 0)
    return () => window.clearTimeout(timer)
  }, [book, bookId, currentChapter, currentChapterIndex, goToChapter, navigate, contentRef])

  return {
    goToChapter,
    handleSearchResultClick,
    imageCache,
    goBackToLibrary: () => navigate('/'),
    goToPreviousChapter: () => currentChapterIndex > 0 && goToChapter(currentChapterIndex - 1),
    goToNextChapter: () => book && currentChapterIndex < book.chapter_count - 1 && goToChapter(currentChapterIndex + 1),
  }
}
