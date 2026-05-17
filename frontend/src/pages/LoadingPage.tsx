import { coverColor } from '../utils/cover'
import { useLoadingPage } from './loading/useLoadingPage'
import './LoadingPage.css'

function LoadingPage() {
  const { author, bookId, coverUrl, handleBack, openBook, opening, stageText, title } = useLoadingPage()

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
