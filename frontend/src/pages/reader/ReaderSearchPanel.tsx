import type { ReactNode } from 'react'
import type { SearchHitDto } from '../../services/api'

interface ReaderSearchPanelProps {
  onClose: () => void
  onInput: (query: string) => void
  onResultClick: (hit: SearchHitDto) => void
  query: string
  results: SearchHitDto[]
  visible: boolean
}

function renderSearchContext(context: string, query: string): ReactNode {
  const idx = context.indexOf(query)
  if (idx < 0) return context
  return (
    <>
      {context.slice(0, idx)}
      <mark>{context.slice(idx, idx + query.length)}</mark>
      {context.slice(idx + query.length)}
    </>
  )
}

export default function ReaderSearchPanel({
  onClose,
  onInput,
  onResultClick,
  query,
  results,
  visible,
}: ReaderSearchPanelProps) {
  const trimmedQuery = query.trim()

  return (
    <div className={`search-overlay ${visible ? 'visible' : ''}`} onClick={(e) => {
      if (e.target === e.currentTarget) onClose()
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
            value={query}
            onChange={(e) => onInput(e.target.value)}
            autoFocus={visible}
          />
        </div>
        <div className="search-results">
          {trimmedQuery && results.length === 0 ? (
            <div className="search-hint">未找到匹配结果</div>
          ) : results.length > 0 ? (
            results.map((hit, i) => (
              <div
                key={`${hit.chapter_index}-${i}`}
                className="search-result-item"
                onClick={() => onResultClick(hit)}
              >
                <div className="search-result-context">{renderSearchContext(hit.context, trimmedQuery)}</div>
                <div className="search-result-meta">{hit.chapter_title} · {hit.progress_hint}</div>
              </div>
            ))
          ) : (
            <div className="search-hint">输入关键词搜索本书内容</div>
          )}
        </div>
      </div>
    </div>
  )
}
