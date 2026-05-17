import type { BookmarkDto, ReaderBookDto } from '../../services/api'

interface ReaderToolbarProps {
  book: ReaderBookDto | null
  chapterTitle: string
  currentBookmark?: BookmarkDto
  onBack: () => void
  onNextChapter: () => void
  onPreviousChapter: () => void
  onToggleBookmark: () => void
  onToggleSearch: () => void
  onToggleToc: () => void
  onTtsStop: () => void
  onTtsToggle: () => void
  ttsStatus: string
}

export default function ReaderToolbar({
  book,
  chapterTitle,
  currentBookmark,
  onBack,
  onNextChapter,
  onPreviousChapter,
  onToggleBookmark,
  onToggleSearch,
  onToggleToc,
  onTtsStop,
  onTtsToggle,
  ttsStatus,
}: ReaderToolbarProps) {
  return (
    <div className="toolbar">
      <div className="toolbar-left">
        <button className="toolbar-btn" onClick={onBack}>
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
        <button className="toolbar-btn" onClick={onToggleToc}>
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
        <button className="toolbar-btn" onClick={onToggleSearch}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          搜索
        </button>
        <button className={`toolbar-btn ${currentBookmark ? 'bookmarked' : ''}`} onClick={onToggleBookmark}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill={currentBookmark ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="2">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
          </svg>
          书签
        </button>
        <button className={`toolbar-btn ${ttsStatus === 'playing' ? 'tts-active' : ''}`} onClick={onTtsToggle} disabled={ttsStatus === 'buffering'}>
          {ttsStatus === 'playing' ? (
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
          {ttsStatus === 'playing' ? '暂停' : ttsStatus === 'paused' ? '继续' : ttsStatus === 'buffering' ? '加载中' : '听书'}
        </button>
        {ttsStatus !== 'idle' && (
          <button className="toolbar-btn" onClick={onTtsStop} title="停止听书">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            </svg>
          </button>
        )}
        <div className="toolbar-divider" />
        <button className="toolbar-btn" onClick={onPreviousChapter}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>
        <button className="toolbar-btn" onClick={onNextChapter}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
      </div>
    </div>
  )
}
