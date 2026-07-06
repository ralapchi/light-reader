import type { LibraryBookCardDto } from '../services/api'
import { coverColor } from '../utils/cover'
import { useLibraryPage } from './library/useLibraryPage'
import TagEditor from '../components/TagEditor'
import './LibraryPage.css'

function formatProgress(item: LibraryBookCardDto): string {
  if (item.progress_percent >= 1) return `100% · ${item.format.toUpperCase()}`
  if (item.progress_percent > 0) return `${Math.round(item.progress_percent * 100)}% · ${item.format.toUpperCase()}`
  return `未开始 · ${item.format.toUpperCase()}`
}

function lastChapterInfo(item: LibraryBookCardDto): string {
  if (item.chapter_count === 0) return ''
  const ch = Math.max(1, Math.round(item.progress_percent * item.chapter_count))
  return `第 ${ch} 章`
}

function LibraryPage() {
  const {
    books,
    closeDeleteConfirm,
    continueReading,
    coverImages,
    deleteConfirm,
    editingBookId,
    handleDeleteBatch,
    handleDeleteConfirm,
    handleDeleteSingle,
    handleCloseTagEditor,
    handleEditTags,
    handleImport,
    handleOpenBook,
    handleSearch,
    isSearching,
    searchQuery,
    selectedIds,
    selectMode,
    setDeleteFiles,
    toggleSelect,
    toggleSelectMode,
  } = useLibraryPage()

  return (
    <main className="library-main">
        <div className="library-header">
          <h1>书架</h1>
          <div className="header-actions">
            <div className="search-box">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
              </svg>
              <input
                type="text"
                placeholder="搜索书籍..."
                value={searchQuery}
                onChange={e => handleSearch(e.target.value)}
              />
            </div>
            <button className={`btn-secondary ${selectMode ? 'active' : ''}`} onClick={toggleSelectMode}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="9 11 12 14 22 4" />
                <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
              </svg>
              {selectMode ? '取消' : '管理'}
            </button>
            <button className="btn-primary" onClick={handleImport}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              导入书籍
            </button>
          </div>
        </div>

        {/* Continue Reading */}
        {continueReading.length > 0 && !isSearching && (
          <>
            <div className="section-title">继续阅读</div>
            <div className="continue-reading">
              {continueReading.map(item => (
                <div
                  key={item.book_id}
                  className="continue-card"
                  role="button"
                  tabIndex={0}
                  onClick={() => handleOpenBook(item.book_id)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault()
                      handleOpenBook(item.book_id)
                    }
                  }}
                >
                  <div className={`continue-cover ${coverColor(item.book_id)}`}>
                    {coverImages[item.book_id] ? (
                      <img src={coverImages[item.book_id]} alt={item.title} />
                    ) : (
                      <div className="placeholder">{item.title[0]}</div>
                    )}
                  </div>
                  <div className="continue-info">
                    <div className="continue-title">{item.title}</div>
                    <div className="continue-author">{item.author ?? '未知作者'}</div>
                    <div className="continue-progress-bar">
                      <div
                        className="continue-progress-fill"
                        style={{ width: `${Math.round(item.progress_percent * 100)}%` }}
                      />
                    </div>
                    <div className="continue-progress-text">
                      {lastChapterInfo(item)} · {Math.round(item.progress_percent * 100)}%
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </>
        )}

        {/* All Books */}
        <div className="section-title">{isSearching ? '搜索结果' : '书籍'}</div>
        {books.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
              </svg>
            </div>
            <div className="empty-state-text">
              {isSearching ? '没有找到匹配的书籍' : '书库为空，点击"导入书籍"开始'}
            </div>
          </div>
        ) : (
          <div className="book-grid">
            {books.map(item => (
              <div
                key={item.book_id}
                className={`book-card ${selectMode && selectedIds.has(item.book_id) ? 'selected' : ''}`}
                role="button"
                tabIndex={0}
                onClick={() => handleOpenBook(item.book_id)}
                onDoubleClick={(e) => {
                  e.preventDefault()
                  e.stopPropagation()
                  handleEditTags(item.book_id)
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault()
                    handleOpenBook(item.book_id)
                  }
                }}
              >
                <div className={`book-cover ${coverColor(item.book_id)}`}>
                  {coverImages[item.book_id] ? (
                    <img src={coverImages[item.book_id]} alt={item.title} />
                  ) : (
                    <div className="placeholder">{item.title[0]}</div>
                  )}
                  {selectMode ? (
                    <button
                      className={`cover-checkbox ${selectedIds.has(item.book_id) ? 'checked' : ''}`}
                      onClick={(e) => { e.stopPropagation(); toggleSelect(item.book_id) }}
                    >
                      {selectedIds.has(item.book_id) && (
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      )}
                    </button>
                  ) : (
                    <button
                      className="cover-delete"
                      onClick={(e) => { e.stopPropagation(); handleDeleteSingle(item.book_id) }}
                      title="移除"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                  )}
                </div>
                <div className="book-progress">{formatProgress(item)}</div>
              </div>
            ))}
          </div>
        )}

        {/* Tag Editor Panel (shown when a book is double-clicked) */}
        {editingBookId && (() => {
          const book = books.find(b => b.book_id === editingBookId)
          if (!book) return null
          return (
            <>
              <div className="modal-backdrop" onClick={handleCloseTagEditor} />
              <div className="tag-editor-modal">
                <div className="tag-editor-modal-header">
                  <div className="tag-editor-modal-book">
                    <div className={`tag-editor-modal-cover ${coverColor(book.book_id)}`}>
                      {coverImages[book.book_id] ? (
                        <img src={coverImages[book.book_id]} alt={book.title} />
                      ) : (
                        <div className="placeholder">{book.title[0]}</div>
                      )}
                    </div>
                    <div>
                      <div className="tag-editor-modal-title">{book.title}</div>
                      <div className="tag-editor-modal-author">{book.author ?? '未知作者'} · {book.format.toUpperCase()}</div>
                    </div>
                  </div>
                  <button className="tag-editor-modal-close" onClick={handleCloseTagEditor}>
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </div>
                <div className="tag-editor-modal-body">
                  <TagEditor bookId={editingBookId} />
                </div>
              </div>
            </>
          )
        })()}

        {/* Batch delete action bar */}
        {selectMode && selectedIds.size > 0 && (
          <div className="batch-action-bar">
            <span className="batch-count">已选择 {selectedIds.size} 本书籍</span>
            <div className="batch-actions">
              <button className="btn-danger" onClick={handleDeleteBatch}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <polyline points="3 6 5 6 21 6" />
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                </svg>
                批量移除
              </button>
            </div>
          </div>
        )}

        {/* Delete Confirm Dialog */}
        {deleteConfirm.open && (
          <>
            <div className="modal-backdrop" onClick={closeDeleteConfirm} />
            <div className="delete-modal">
              <div className="delete-modal-title">{deleteConfirm.title}</div>
              <div className="delete-modal-message">{deleteConfirm.message}</div>
              <label className="delete-modal-checkbox">
                <input
                  type="checkbox"
                  checked={deleteConfirm.deleteFiles}
                  onChange={e => setDeleteFiles(e.target.checked)}
                />
                <span>同时删除本地源文件</span>
              </label>
              <div className="delete-modal-hint">阅读进度和缓存文件将在移除后一并清除，此操作不可撤销。</div>
              {deleteConfirm.deleteFiles && (
                <div className="delete-modal-warning">本地源文件删除后将无法恢复。</div>
              )}
              <div className="delete-modal-actions">
                <button className="btn-secondary" onClick={closeDeleteConfirm}>取消</button>
                <button className="btn-danger" onClick={handleDeleteConfirm}>确认移除</button>
              </div>
            </div>
          </>
        )}
      </main>
  )
}

export default LibraryPage
