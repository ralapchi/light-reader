import { useCallback, useEffect, useRef, useState } from 'react'
import { flushSync } from 'react-dom'
import { useNavigate } from 'react-router-dom'
import { readerChapterImage, readerGetChapter, readerResolveHref, readerSaveProgress } from '../../services/api'
import type { ReaderBookDto, ReaderBlockDto, SearchHitDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { captureVisibleParagraph, scrollToAnchor, scrollToOffset, scrollToParagraph, scrollToParagraphTwoPage } from './readerUtils'

interface FootnoteReturnEntry {
  chapterIndex: number
  paragraphIndex: number | null
  scrollOffset: number
}

interface NavigateToHrefOptions {
  showReturn?: boolean
}

export function useChapterNavigation(
  bookId: string | undefined,
  book: ReaderBookDto | null,
  contentRef: React.RefObject<HTMLDivElement | null>,
  handleCloseSearch: () => void,
  readingMode?: string,
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

  // ── Footnote return stack ──────────────────────────────

  const [footnoteReturn, setFootnoteReturn] = useState<FootnoteReturnEntry | null>(null)

  // ── Navigation ────────────────────────────────────────

  const goToChapter = useCallback(async (
    index: number,
    scrollOffset?: number | null,
    options?: { saveProgress?: boolean },
  ) => {
    try {
      const chapter = await readerGetChapter(index)
      flushSync(() => {
        setCurrentChapter(index, chapter)
      })
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
        const el = contentRef.current
        if (!el) return
        if (readingMode === 'TwoPage' && scrollOffset == null) {
          // In two-page mode, no scroll offset - start at first spread
        } else if (scrollOffset && scrollOffset > 0) {
          scrollToOffset(el, scrollOffset)
        } else {
          scrollToOffset(el, 0)
        }
      })
    } catch (e) {
      console.error('加载章节失败:', e)
    }
  }, [setCurrentChapter, closeToc, handleCloseSearch, loadChapterImages, bookId, book, setProgressPercent, contentRef, readingMode])

  const navigateToHref = useCallback(async (
    href: string,
    fallbackIndex?: number | null,
    options?: NavigateToHrefOptions,
  ) => {
    // Save current position for potential footnote return
    const currentEl = contentRef.current
    const twoPage = readingMode === 'TwoPage'
    const shouldShowReturn = options?.showReturn !== false
    const savedReturn: FootnoteReturnEntry = {
      chapterIndex: currentChapterIndex,
      paragraphIndex: currentEl ? captureVisibleParagraph(currentEl, twoPage) : null,
      scrollOffset: currentEl ? (currentEl.scrollTop || 0) : 0,
    }

    try {
      const resolved = await readerResolveHref(href, currentChapterIndex)
      if (!resolved) {
        if (fallbackIndex != null) {
          goToChapter(fallbackIndex)
        }
        return
      }

      const targetChapter = resolved.chapter_index
      if (targetChapter !== currentChapterIndex) {
        if (shouldShowReturn) setFootnoteReturn(savedReturn)
        if (resolved.paragraph_index != null) {
          useAppStore.getState().setPendingNavTarget({
            chapter_index: targetChapter,
            paragraph_index: resolved.paragraph_index,
            scroll_offset: null,
          })
        }
        goToChapter(targetChapter, null, { saveProgress: false }).then(() => {
          if (resolved.paragraph_index != null) {
            requestAnimationFrame(() => {
              const content = contentRef.current
              if (!content) return
              if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, resolved.paragraph_index!)
              else scrollToParagraph(content, resolved.paragraph_index!)
            })
          }
        })
      } else {
        // Same chapter: scroll to paragraph after paint
        if (shouldShowReturn) setFootnoteReturn(savedReturn)
        const targetPara = resolved.paragraph_index
        if (targetPara != null) {
          requestAnimationFrame(() => {
            const content = contentRef.current
            if (!content) return
            if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, targetPara)
            else scrollToParagraph(content, targetPara)
          })
        }
      }
    } catch {
      if (fallbackIndex != null) goToChapter(fallbackIndex)
    }
  }, [currentChapterIndex, goToChapter, contentRef, readingMode])

  const returnFromFootnote = useCallback(() => {
    if (!footnoteReturn) return
    const { chapterIndex, paragraphIndex, scrollOffset } = footnoteReturn
    setFootnoteReturn(null)
    if (chapterIndex !== currentChapterIndex) {
      // Use pendingNavTarget for reliable scroll restoration after chapter load
      useAppStore.getState().setPendingNavTarget({
        chapter_index: chapterIndex,
        paragraph_index: readingMode === 'TwoPage' ? paragraphIndex : null,
        scroll_offset: readingMode === 'TwoPage' ? null : scrollOffset,
      })
      goToChapter(chapterIndex, null, { saveProgress: false })
    } else {
      const el = contentRef.current
      if (!el) return
      if (readingMode === 'TwoPage') {
        if (paragraphIndex != null) scrollToParagraphTwoPage(el, paragraphIndex)
      } else {
        scrollToOffset(el, scrollOffset)
      }
    }
  }, [footnoteReturn, currentChapterIndex, goToChapter, contentRef, readingMode])

  const handleSearchResultClick = useCallback((hit: SearchHitDto) => {
    handleCloseSearch()
    goToChapter(hit.chapter_index, null, { saveProgress: false }).then(() => {
      if (hit.paragraph_index != null) {
        requestAnimationFrame(() => {
          const content = contentRef.current
          if (!content) return
          if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, hit.paragraph_index)
          else scrollToParagraph(content, hit.paragraph_index)
        })
      }
    })
  }, [handleCloseSearch, goToChapter, contentRef, readingMode])

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
          if (readingMode === 'TwoPage') {
            // Two-page: prefer paragraph_index
            if (pending.paragraph_index != null) {
              requestAnimationFrame(() => {
                const content = contentRef.current
                if (!content) return
                scrollToParagraphTwoPage(content, pending.paragraph_index!)
              })
            }
          } else if (pending.anchor) {
            requestAnimationFrame(() => {
              const el = contentRef.current
              if (!el) return
              scrollToAnchor(el, pending.anchor!)
            })
          } else if (pending.scroll_offset && pending.scroll_offset > 0) {
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
            if (pending.anchor && readingMode !== 'TwoPage') {
              requestAnimationFrame(() => {
                const content = contentRef.current
                if (!content) return
                scrollToAnchor(content, pending.anchor!)
              })
            } else if (pending.paragraph_index != null && (!pending.scroll_offset || pending.scroll_offset <= 0)) {
              requestAnimationFrame(() => {
                const content = contentRef.current
                if (!content) return
                if (readingMode === 'TwoPage') scrollToParagraphTwoPage(content, pending.paragraph_index!)
                else scrollToParagraph(content, pending.paragraph_index!)
              })
            }
          })
        }
      } else if (!currentChapter) {
        goToChapter(currentChapterIndex)
      }
    }, 0)
    return () => window.clearTimeout(timer)
  }, [book, bookId, currentChapter, currentChapterIndex, goToChapter, navigate, contentRef, readingMode])

  const clearFootnoteReturn = useCallback(() => setFootnoteReturn(null), [])

  return {
    goToChapter,
    navigateToHref,
    returnFromFootnote,
    footnoteReturn,
    clearFootnoteReturn,
    handleSearchResultClick,
    imageCache,
    goBackToLibrary: () => navigate('/'),
    goToPreviousChapter: () => currentChapterIndex > 0 && goToChapter(currentChapterIndex - 1),
    goToNextChapter: () => book && currentChapterIndex < book.chapter_count - 1 && goToChapter(currentChapterIndex + 1),
  }
}
