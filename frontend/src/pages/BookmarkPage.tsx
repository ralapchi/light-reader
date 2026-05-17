import { useBookmarkPage } from './bookmarks/useBookmarkPage'
import './BookmarkPage.css'

function BookmarkPage() {
  const { groups, handleBookmarkClick, handleDelete, loading } = useBookmarkPage()

  return (
    <main className="bookmark-main">
        <div className="bookmark-page-header">
          <h1>书签</h1>
        </div>

        {loading ? (
          <div className="bookmark-loading">加载中...</div>
        ) : groups.length === 0 ? (
          <div className="bookmark-page-empty">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
            </svg>
            <div>暂无书签</div>
            <div className="bookmark-page-empty-hint">在阅读时按 Ctrl+B 添加书签</div>
          </div>
        ) : (
          <div className="bookmark-groups">
            {groups.map(group => (
              <div key={group.bookId} className="bookmark-group">
                <div className="bookmark-group-title">{group.title}</div>
                <div className="bookmark-group-list">
                  {group.bookmarks.map(bm => (
                    <div
                      key={bm.id}
                      className="bookmark-entry"
                      onClick={() => handleBookmarkClick(bm)}
                    >
                      <div className="bookmark-entry-icon">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" strokeWidth="2">
                          <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
                        </svg>
                      </div>
                      <div className="bookmark-entry-content">
                        <div className="bookmark-entry-title">{bm.title}</div>
                        <div className="bookmark-entry-snippet">{bm.snippet}</div>
                        <div className="bookmark-entry-meta">第 {bm.chapter_index + 1} 章</div>
                      </div>
                      <button
                        className="bookmark-entry-delete"
                        onClick={(e) => { e.stopPropagation(); handleDelete(bm) }}
                        title="删除书签"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <line x1="18" y1="6" x2="6" y2="18" />
                          <line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </main>
  )
}

export default BookmarkPage
