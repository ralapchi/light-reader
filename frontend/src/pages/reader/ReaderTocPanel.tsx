import type { TocItemDto } from '../../services/api'

interface ReaderTocPanelProps {
  currentChapterIndex: number
  items: TocItemDto[]
  onClose: () => void
  onGoToChapter: (index: number) => void
  progressDisplay: string
  progressPercent: number
  visible: boolean
}

export default function ReaderTocPanel({
  currentChapterIndex,
  items,
  onClose,
  onGoToChapter,
  progressDisplay,
  progressPercent,
  visible,
}: ReaderTocPanelProps) {
  return (
    <div className={`toc-overlay ${visible ? 'visible' : ''}`}>
      <div className="toc-backdrop" onClick={onClose} />
      <div className="toc-panel">
        <div className="toc-header">
          <span className="toc-header-title">目录</span>
          <button className="toc-close" onClick={onClose}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div className="toc-list">
          {items.map((item, i) => (
            <div
              key={item.id || i}
              className={`toc-item ${item.chapter_index === currentChapterIndex ? 'active' : ''}`}
              style={{ paddingLeft: `${12 + item.depth * 12}px` }}
              onClick={() => item.chapter_index != null && onGoToChapter(item.chapter_index)}
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
  )
}
