import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import useAppStore from '../store/useAppStore'
import { readerGetChapter, readerChapterImage, readerSaveProgress, searchInBook, bookmarkList, bookmarkAdd, bookmarkRemove, settingsSave, ttsStart, ttsPause, ttsResume, ttsStop, onTtsPlaying, onTtsPaused, onTtsStopped, onTtsBuffering, onTtsFinished, onTtsError } from '../services/api'
import type { TocItemDto, ReaderBlockDto, SearchHitDto } from '../services/api'
import { COMPACT_READER_FONTS, READER_THEMES, findReaderTheme, readerFontFamily } from '../utils/readerOptions'
import './ReaderPage.css'

function RenderBlock({ block, imageCache, paragraphStyle, highlight }: { block: ReaderBlockDto; imageCache: Record<string, string>; paragraphStyle?: React.CSSProperties; highlight?: boolean }) {
  if (block.type === 'separator') {
    return <p className="reader-paragraph separator" style={paragraphStyle}>***</p>
  }
  if (block.type === 'image') {
    return (
      <figure className="reader-image">
        {imageCache[block.asset_id] && (
          <img src={imageCache[block.asset_id]} alt={block.alt_text ?? ''} />
        )}
        {block.caption && <figcaption>{block.caption}</figcaption>}
      </figure>
    )
  }
  // paragraph
  const cls = `reader-paragraph ${block.kind === 'quote' ? 'quote' : 'indent'}${highlight ? ' tts-highlight' : ''}`
  return <p className={cls} style={paragraphStyle} data-para-index={block.index}>{block.text}</p>
}

function flattenToc(items: TocItemDto[]): TocItemDto[] {
  const out: TocItemDto[] = []
  for (const item of items) {
    out.push(item)
    if (item.children.length > 0) out.push(...flattenToc(item.children))
  }
  return out
}

function blockKey(block: ReaderBlockDto, fallbackIndex: number): string {
  if (block.type === 'separator') return `sep-${fallbackIndex}`
  return `${block.type}-${block.index}`
}

function blockParagraphIndex(block: ReaderBlockDto): number | null {
  return block.type === 'paragraph' ? block.index : null
}

// ── Scroll helpers ──

function scrollToOffset(el: HTMLElement, offset: number) {
  const prev = el.style.scrollBehavior
  el.style.scrollBehavior = 'auto'
  el.scrollTop = offset
  el.style.scrollBehavior = prev
}

function scrollToParagraph(container: HTMLElement, paraIndex: number) {
  const paras = container.querySelectorAll('.reader-paragraph')
  const target = paras[paraIndex]
  if (target) (target as HTMLElement).scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function findVisibleParagraphIndex(container: HTMLElement): number | null {
  const scrollTop = container.scrollTop
  const blocks = container.querySelectorAll('.reader-paragraph, .reader-image')
  for (let i = blocks.length - 1; i >= 0; i--) {
    if ((blocks[i] as HTMLElement).offsetTop <= scrollTop + 100) return i
  }
  return null
}

function ReaderPage() {
  const { bookId } = useParams<{ bookId: string }>()
  const navigate = useNavigate()
  const contentRef = useRef<HTMLDivElement>(null)
  const [imageCache, setImageCache] = useState<Record<string, string>>({})
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<SearchHitDto[]>([])
  const searchTimerRef = useRef<ReturnType<typeof setTimeout>>(null)
  const [activePanel, setActivePanel] = useState<'theme' | 'font' | 'format' | null>(null)

  const {
    reader,
    toggleToc, toggleSearch, closeToc, closeSearch,
    setCurrentChapter, setProgressPercent,
    setBookmarks, setSettings, setTtsState, resetTts,
  } = useAppStore()

  const { book, currentChapterIndex, currentChapter, progressPercent, showToc, showSearch, bookmarks, settings, tts } = reader

  // Save progress to backend
  const saveProgress = useCallback((pct?: number, _force?: boolean, paragraphIndex?: number | null, scrollOffset?: number) => {
    if (!bookId) return
    readerSaveProgress({
      book_id: bookId,
      chapter_index: currentChapterIndex,
      progress_percent: pct ?? progressPercent,
      paragraph_index: paragraphIndex ?? null,
      scroll_offset: scrollOffset ?? null,
    }).catch(() => { /* non-critical */ })
  }, [bookId, currentChapterIndex, progressPercent])

  const clearSearchState = useCallback(() => {
    setSearchQuery('')
    setSearchResults([])
  }, [])

  const handleToggleSearch = useCallback(() => {
    if (showSearch) clearSearchState()
    toggleSearch()
  }, [showSearch, clearSearchState, toggleSearch])

  const handleCloseSearch = useCallback(() => {
    clearSearchState()
    closeSearch()
  }, [clearSearchState, closeSearch])

  // Debounced search
  const handleSearchInput = useCallback((q: string) => {
    setSearchQuery(q)
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current)
    if (!q.trim()) {
      setSearchResults([])
      return
    }
    searchTimerRef.current = setTimeout(async () => {
      try {
        const hits = await searchInBook(q.trim())
        setSearchResults(hits)
      } catch {
        setSearchResults([])
      }
    }, 300)
  }, [])

  // Load images for current chapter via readerChapterImage (returns base64 data URI)
  const loadChapterImages = useCallback(async (blocks: ReaderBlockDto[]) => {
    const imageBlocks = blocks.filter(b => b.type === 'image')
    if (imageBlocks.length === 0) return
    const updates: Record<string, string> = {}
    await Promise.allSettled(
      imageBlocks.map(async (b) => {
        if (b.type !== 'image') return
        if (imageCache[b.asset_id]) return
        try {
          const dataUri = await readerChapterImage(bookId!, b.asset_id)
          if (dataUri) updates[b.asset_id] = dataUri
        } catch { /* skip */ }
      })
    )
    if (Object.keys(updates).length > 0) {
      setImageCache(prev => ({ ...prev, ...updates }))
    }
  }, [imageCache, bookId])

  // Go to a chapter
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

      // 恢复滚动位置 — 用 scrollToOffset 绕过 CSS scroll-behavior: smooth
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
  }, [setCurrentChapter, closeToc, handleCloseSearch, loadChapterImages, bookId, book, setProgressPercent])

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
  }, [handleCloseSearch, goToChapter])

  // Load first chapter on mount if not loaded
  useEffect(() => {
    const timer = window.setTimeout(() => {
      if (!book) {
        navigate(`/loading/${bookId}`)
        return
      }
      // 检查 pendingNavTarget
      const pending = useAppStore.getState().reader.pendingNavTarget
      if (pending) {
        const targetChapter = Math.min(pending.chapter_index, (book.chapter_count ?? 1) - 1)
        useAppStore.getState().setPendingNavTarget(null)
        const alreadyLoaded = currentChapter && currentChapter.chapter_index === targetChapter
        if (alreadyLoaded) {
          // Chapter already loaded by LoadingPage — skip redundant IPC, just restore scroll
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
  }, [book, bookId, currentChapter, currentChapterIndex, goToChapter, navigate])

  // Save current reading position (paragraph index + scroll offset)
  const saveCurrentPosition = useCallback(() => {
    const el = contentRef.current
    if (!el || !book) return
    const scrollTop = el.scrollTop
    const scrollHeight = el.scrollHeight - el.clientHeight
    if (scrollHeight <= 0) return
    const chapterPct = scrollTop / scrollHeight
    const bookPct = Math.min(1, (currentChapterIndex + chapterPct) / book.chapter_count)
    const paraIndex = findVisibleParagraphIndex(el)
    saveProgress(bookPct, true, paraIndex, scrollTop)
  }, [book, currentChapterIndex, saveProgress])

  // Keep a stable ref so the effect below doesn't re-run on every scroll
  const saveRef = useRef(saveCurrentPosition)

  useEffect(() => {
    saveRef.current = saveCurrentPosition
  }, [saveCurrentPosition])

  // Save progress on page leave / visibility change / window close
  useEffect(() => {
    const handleVisibility = () => {
      if (document.hidden) saveRef.current()
    }
    const handleBeforeUnload = () => saveRef.current()
    document.addEventListener('visibilitychange', handleVisibility)
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => {
      document.removeEventListener('visibilitychange', handleVisibility)
      window.removeEventListener('beforeunload', handleBeforeUnload)
      saveRef.current()
    }
  }, [])

  // Handle scroll progress (book-level: chapter offset + intra-chapter scroll)
  const handleScroll = useCallback(() => {
    const el = contentRef.current
    if (!el) return
    const scrollTop = el.scrollTop
    const scrollHeight = el.scrollHeight - el.clientHeight
    if (scrollHeight > 0 && book) {
      const chapterPct = scrollTop / scrollHeight
      const bookPct = Math.min(1, (currentChapterIndex + chapterPct) / book.chapter_count)
      setProgressPercent(bookPct)
    }
  }, [setProgressPercent, book, currentChapterIndex])

  // Load bookmarks for current book
  const loadBookmarks = useCallback(async () => {
    if (!bookId) return
    try {
      const list = await bookmarkList(bookId)
      setBookmarks(list)
    } catch { /* non-critical */ }
  }, [bookId, setBookmarks])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void loadBookmarks()
    }, 0)
    return () => window.clearTimeout(timer)
  }, [loadBookmarks])

  // Check if current chapter is bookmarked
  const currentBookmark = bookmarks.find(b => b.chapter_index === currentChapterIndex)

  // ── TTS control ──
  const handleTtsToggle = useCallback(async () => {
    if (tts.status === 'playing') {
      await ttsPause()
    } else if (tts.status === 'paused') {
      await ttsResume()
    } else {
      await ttsStart(currentChapterIndex)
    }
  }, [tts.status, currentChapterIndex])

  const handleTtsStop = useCallback(async () => {
    await ttsStop()
  }, [])

  // Toggle bookmark for current chapter
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
  }, [bookId, currentChapterIndex, setBookmarks])

  // Keyboard shortcut: Ctrl/Cmd+B to toggle bookmark
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault()
        toggleBookmark()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [toggleBookmark])

  // ── TTS event listeners ──
  useEffect(() => {
    const unsubs: (() => void)[] = []
    Promise.all([
      onTtsPlaying(p => setTtsState({ status: 'playing', paragraph_indices: p.paragraph_indices, segment_index: p.segment_index, total_segments: p.total_segments, error: null })),
      onTtsPaused(() => setTtsState({ status: 'paused' })),
      onTtsStopped(() => resetTts()),
      onTtsBuffering(p => setTtsState({ status: 'buffering', segment_index: p.segment_index })),
      onTtsFinished(() => setTtsState({ status: 'finished', paragraph_indices: [] })),
      onTtsError(p => setTtsState({ status: 'error', error: p.error_message })),
    ]).then(fns => unsubs.push(...fns.map(u => u)))
    return () => unsubs.forEach(u => u())
  }, [setTtsState, resetTts])

  // Auto-scroll highlighted paragraph into view
  const prevSegRef = useRef(-1)
  useEffect(() => {
    if (tts.status !== 'playing') { prevSegRef.current = -1; return }
    if (tts.paragraph_indices.length === 0) return
    if (tts.segment_index === prevSegRef.current) return
    prevSegRef.current = tts.segment_index
    requestAnimationFrame(() => {
      const content = contentRef.current
      if (!content) return
      scrollToParagraph(content, tts.paragraph_indices[0])
    })
  }, [tts.status, tts.paragraph_indices, tts.segment_index])

  const flatToc = book ? flattenToc(book.toc) : []
  const chapterTitle = currentChapter?.title ?? ''
  const progressDisplay = `${Math.round(progressPercent * 100)}%`

  const updateAndSave = useCallback((partial: Partial<typeof settings>) => {
    // If adjusting font/format in original mode, auto-switch to light theme
    if (settings.theme === 'original' && !partial.theme && (partial.font_family || partial.font_size || partial.line_height || partial.paragraph_spacing)) {
      partial = { ...partial, theme: 'light' }
    }
    setSettings(partial)
    const next = { ...settings, ...partial }
    settingsSave(next).catch(() => {})
  }, [settings, setSettings])

  const isOriginal = settings.theme === 'original'
  const currentTheme = findReaderTheme(settings.theme)
  const readerStyle: React.CSSProperties = {
    '--bg': currentTheme.bg,
    '--text-primary': currentTheme.text,
    '--accent': currentTheme.accent,
    '--border': currentTheme.border,
    '--surface': currentTheme.surface,
    '--text-secondary': currentTheme.textSec,
    '--text-tertiary': currentTheme.textTer,
    '--surface-hover': currentTheme.hover,
    '--accent-soft': currentTheme.accentSoft,
  } as React.CSSProperties

  // Original mode: no style overrides, let EPUB CSS take effect
  const contentStyle: React.CSSProperties = isOriginal ? {} : {
    fontFamily: readerFontFamily(settings.font_family),
    fontSize: `${settings.font_size}px`,
    lineHeight: settings.line_height,
  }

  const paragraphStyle: React.CSSProperties = isOriginal ? {} : {
    marginBottom: `${settings.paragraph_spacing}em`,
  }

  return (
    <div className="reader-app" style={readerStyle}>
      {/* Hover trigger zone */}
      <div className="toolbar-trigger" />

      {/* Top Toolbar */}
      <div className="toolbar">
        <div className="toolbar-left">
          <button className="toolbar-btn" onClick={() => navigate('/')}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="15 18 9 12 15 6" />
            </svg>
            书架
          </button>
          <div className="toolbar-divider" />
          <span className="toolbar-title">
            {book?.title ?? chapterTitle}
          </span>
        </div>
        <div className="toolbar-right">
          <button className="toolbar-btn" onClick={toggleToc}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="8" y1="6" x2="21" y2="6" />
              <line x1="8" y1="12" x2="21" y2="12" />
              <line x1="8" y1="18" x2="21" y2="18" />
              <line x1="3" y1="6" x2="3.01" y2="6" />
              <line x1="3" y1="12" x2="3.01" y2="12" />
              <line x1="3" y1="18" x2="3.01" y2="18" />
            </svg>
            目录
          </button>
          <button className="toolbar-btn" onClick={handleToggleSearch}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            搜索
          </button>
          <button className={`toolbar-btn ${currentBookmark ? 'bookmarked' : ''}`} onClick={toggleBookmark}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill={currentBookmark ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="2">
              <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
            </svg>
            书签
          </button>
          <button className={`toolbar-btn ${tts.status === 'playing' ? 'tts-active' : ''}`} onClick={handleTtsToggle} disabled={tts.status === 'buffering'}>
            {tts.status === 'playing' ? (
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="6" y="4" width="4" height="16" />
                <rect x="14" y="4" width="4" height="16" />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" y1="19" x2="12" y2="23" />
                <line x1="8" y1="23" x2="16" y2="23" />
              </svg>
            )}
            {tts.status === 'playing' ? '暂停' : tts.status === 'paused' ? '继续' : tts.status === 'buffering' ? '加载中' : '听书'}
          </button>
          {tts.status !== 'idle' && (
            <button className="toolbar-btn" onClick={handleTtsStop} title="停止听书">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              </svg>
            </button>
          )}
          <div className="toolbar-divider" />
          <button
            className="toolbar-btn"
            onClick={() => currentChapterIndex > 0 && goToChapter(currentChapterIndex - 1)}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </button>
          <button
            className="toolbar-btn"
            onClick={() => book && currentChapterIndex < book.chapter_count - 1 && goToChapter(currentChapterIndex + 1)}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="9 18 15 12 9 6" />
            </svg>
          </button>
        </div>
      </div>

      {/* Floating TOC Panel */}
      <div className={`toc-overlay ${showToc ? 'visible' : ''}`}>
        <div className="toc-backdrop" onClick={closeToc} />
        <div className="toc-panel">
          <div className="toc-header">
            <span className="toc-header-title">目录</span>
            <button className="toc-close" onClick={closeToc}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
          <div className="toc-list">
            {flatToc.map((item, i) => (
              <div
                key={item.id || i}
                className={`toc-item ${item.chapter_index === currentChapterIndex ? 'active' : ''}`}
                style={{ paddingLeft: `${12 + item.depth * 12}px` }}
                onClick={() => item.chapter_index != null && goToChapter(item.chapter_index)}
              >
                <span className="chapter-num">{(item.chapter_index ?? i) + 1}</span>
                {item.title}
              </div>
            ))}
          </div>
          <div className="toc-progress">
            <div className="toc-progress-bar">
              <div className="toc-progress-fill" style={{ width: `${Math.round(progressPercent * 100)}%` }} />
            </div>
            <span className="toc-progress-text">第 {currentChapterIndex + 1} 章 · {progressDisplay}</span>
          </div>
        </div>
      </div>

      {/* Search Overlay */}
      <div className={`search-overlay ${showSearch ? 'visible' : ''}`} onClick={(e) => {
        if (e.target === e.currentTarget) handleCloseSearch()
      }}>
        <div className="search-panel">
          <div className="search-input-area">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
              className="search-input"
              type="text"
              placeholder="在本书中搜索..."
              value={searchQuery}
              onChange={(e) => handleSearchInput(e.target.value)}
              autoFocus={showSearch}
            />
          </div>
          <div className="search-results">
            {searchQuery.trim() && searchResults.length === 0 ? (
              <div className="search-hint">未找到匹配结果</div>
            ) : searchResults.length > 0 ? (
              searchResults.map((hit, i) => {
                // Highlight the matched text in context
                const q = searchQuery.trim()
                const idx = hit.context.indexOf(q)
                let rendered: ReactNode = hit.context
                if (idx >= 0) {
                  rendered = (
                    <>
                      {hit.context.slice(0, idx)}
                      <mark>{hit.context.slice(idx, idx + q.length)}</mark>
                      {hit.context.slice(idx + q.length)}
                    </>
                  )
                }
                return (
                  <div
                    key={`${hit.chapter_index}-${i}`}
                    className="search-result-item"
                    onClick={() => handleSearchResultClick(hit)}
                  >
                    <div className="search-result-context">{rendered}</div>
                    <div className="search-result-meta">{hit.chapter_title} · {hit.progress_hint}</div>
                  </div>
                )
              })
            ) : (
              <div className="search-hint">输入关键词搜索本书内容</div>
            )}
          </div>
        </div>
      </div>

      {/* Reading Progress Bar */}
      <div className="reading-progress-track">
        <div className="reading-progress-fill" style={{ width: `${Math.round(progressPercent * 100)}%` }} />
      </div>

      {/* Reader Content */}
      <div className="reader-content" ref={contentRef} onScroll={handleScroll}>
        <div className="reader-column" style={contentStyle}>
          {currentChapter && (
            <>
              <h1 className="chapter-title">{currentChapter.title}</h1>
              {currentChapter.blocks.map((block, i) => (
                <RenderBlock
                  key={blockKey(block, i)}
                  block={block}
                  imageCache={imageCache}
                  paragraphStyle={paragraphStyle}
                  highlight={blockParagraphIndex(block) === tts.paragraph_indices[0]}
                />
              ))}
            </>
          )}
        </div>
      </div>

      {/* Bottom Status Bar */}
      <div className="status-bar">
        <span>第 {currentChapterIndex + 1} 章 · {chapterTitle}</span>
        <span>
          {tts.status !== 'idle' && (
            <span className="tts-status-badge">
              {tts.status === 'playing' ? '听书中' : tts.status === 'paused' ? '已暂停' : tts.status === 'buffering' ? '合成中' : tts.status === 'error' ? '出错' : ''}
              {tts.total_segments > 0 && ` ${tts.segment_index + 1}/${tts.total_segments}`}
            </span>
          )}
          {progressDisplay}
        </span>
      </div>

      {/* Bottom-right settings buttons */}
      <div className="demo-controls">
        {/* Panel popup */}
        {activePanel && (
          <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
            {activePanel === 'theme' && (
              <div className="theme-options">
                {READER_THEMES.map(t => (
                  <button
                    key={t.id}
                    className={`theme-swatch ${settings.theme === t.id ? 'active' : ''}`}
                    onClick={() => updateAndSave({ theme: t.id })}
                    title={t.label}
                  >
                    <span className="swatch-color" style={{ background: t.bg, boxShadow: `inset 0 0 0 1px ${t.border}` }} />
                    <span className="swatch-label">{t.label}</span>
                  </button>
                ))}
              </div>
            )}
            {activePanel === 'font' && (
              <div className="font-options">
                <div className="option-row">
                  <span className="option-label">字体</span>
                  <div className="option-group">
                    {COMPACT_READER_FONTS.map(f => (
                      <button
                        key={f.id}
                        className={`option-chip ${settings.font_family === f.id ? 'active' : ''}`}
                        onClick={() => updateAndSave({ font_family: f.id })}
                      >{f.compactLabel}</button>
                    ))}
                  </div>
                </div>
                <div className="option-row">
                  <span className="option-label">字号 {settings.font_size}px</span>
                  <input
                    type="range" min={12} max={28} step={1}
                    value={settings.font_size}
                    onChange={e => updateAndSave({ font_size: Number(e.target.value) })}
                    className="option-slider"
                  />
                </div>
              </div>
            )}
            {activePanel === 'format' && (
              <div className="format-options">
                <div className="option-row">
                  <span className="option-label">行距 {settings.line_height.toFixed(2)}</span>
                  <input
                    type="range" min={1.2} max={2.5} step={0.05}
                    value={settings.line_height}
                    onChange={e => updateAndSave({ line_height: Number(e.target.value) })}
                    className="option-slider"
                  />
                </div>
                <div className="option-row">
                  <span className="option-label">段间距 {settings.paragraph_spacing.toFixed(1)}em</span>
                  <input
                    type="range" min={0.4} max={3} step={0.1}
                    value={settings.paragraph_spacing}
                    onChange={e => updateAndSave({ paragraph_spacing: Number(e.target.value) })}
                    className="option-slider"
                  />
                </div>
              </div>
            )}
          </div>
        )}
        <div className="demo-btn-row">
          <button
            className={`demo-btn ${activePanel === 'theme' ? 'active' : ''}`}
            onClick={() => setActivePanel(activePanel === 'theme' ? null : 'theme')}
          >
            主题
          </button>
          <button
            className={`demo-btn ${activePanel === 'font' ? 'active' : ''}`}
            onClick={() => setActivePanel(activePanel === 'font' ? null : 'font')}
          >
            字体
          </button>
          <button
            className={`demo-btn ${activePanel === 'format' ? 'active' : ''}`}
            onClick={() => setActivePanel(activePanel === 'format' ? null : 'format')}
          >
            格式
          </button>
        </div>
      </div>

      {/* Click outside to close panel */}
      {activePanel && (
        <div className="panel-backdrop" onClick={() => setActivePanel(null)} />
      )}
    </div>
  )
}

export default ReaderPage
