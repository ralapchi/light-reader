import { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { open } from '@tauri-apps/plugin-dialog'
import { assetUrl, libraryImport, libraryList, librarySearch } from '../../services/api'
import type { LibraryBookCardDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'
import { useCoverLoader } from './useCoverLoader'
import { useBookDeletion } from './useBookDeletion'

function continueReadingBooks(books: LibraryBookCardDto[]): LibraryBookCardDto[] {
  return books
    .filter(b => b.progress_percent > 0 && b.progress_percent < 1 && b.last_opened_at)
    .sort((a, b) => {
      const da = a.last_opened_at ?? ''
      const db = b.last_opened_at ?? ''
      return db.localeCompare(da)
    })
    .slice(0, 3)
}

export function useLibraryPage() {
  const navigate = useNavigate()
  const books = useAppStore(s => s.books)
  const setBooks = useAppStore(s => s.setBooks)
  const startOpening = useAppStore(s => s.startOpening)
  const setSidebarFooter = useAppStore(s => s.setSidebarFooter)
  const [searchQuery, setSearchQuery] = useState('')
  const [isSearching, setIsSearching] = useState(false)
  const [editingBookId, setEditingBookId] = useState<string | null>(null)

  const { coverImages, loadCovers, pruneCovers } = useCoverLoader()

  const loadBooks = useCallback(async () => {
    try {
      const items = await libraryList()
      setBooks(items)
      pruneCovers(items)
      loadCovers(items)
    } catch (e) {
      console.error('加载书库失败:', e)
    }
  }, [setBooks, loadCovers, pruneCovers])

  const deletion = useBookDeletion(books, loadBooks)

  useEffect(() => {
    if (books.length === 0) {
      loadBooks()
    } else {
      pruneCovers(books)
      loadCovers(books)
    }
  }, [loadBooks, pruneCovers, loadCovers, books.length])

  useEffect(() => {
    setSidebarFooter(`${books.length} 本藏书`)
  }, [books.length, setSidebarFooter])

  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query)
    if (!query.trim()) {
      setIsSearching(false)
      loadBooks()
      return
    }
    setIsSearching(true)
    try {
      const results = await librarySearch(query)
      setBooks(results)
      loadCovers(results)
    } catch (e) {
      console.error('搜索失败:', e)
    }
  }, [loadBooks, setBooks, loadCovers])

  const handleImport = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: '电子书', extensions: ['epub', 'txt'] }],
      })
      if (!selected) return
      const paths = Array.isArray(selected) ? selected : [selected]
      if (paths.length > 0) {
        await libraryImport(paths)
        loadBooks()
      }
    } catch (e) {
      console.error('导入失败:', e)
    }
  }, [loadBooks])

  const handleOpenBook = useCallback((bookId: string) => {
    if (deletion.selectMode) return
    const book = books.find(b => b.book_id === bookId)
    if (book) {
      const cover = coverImages[bookId] ?? (book.cover_url ? assetUrl(book.cover_url) : null)
      startOpening(bookId, book.title, book.author, cover)
    }
    navigate(`/loading/${bookId}`)
  }, [navigate, books, coverImages, startOpening, deletion.selectMode])

  const handleEditTags = useCallback((bookId: string) => {
    setEditingBookId(prev => prev === bookId ? null : bookId)
  }, [])

  const handleCloseTagEditor = useCallback(() => {
    setEditingBookId(null)
  }, [])

  return {
    books,
    continueReading: useMemo(() => continueReadingBooks(books), [books]),
    coverImages,
    deleteConfirm: deletion.deleteConfirm,
    editingBookId,
    handleDeleteBatch: deletion.handleDeleteBatch,
    handleDeleteConfirm: deletion.handleDeleteConfirm,
    handleDeleteSingle: deletion.handleDeleteSingle,
    handleCloseTagEditor,
    handleEditTags,
    handleImport,
    handleOpenBook,
    handleSearch,
    isSearching,
    searchQuery,
    selectedIds: deletion.selectedIds,
    selectMode: deletion.selectMode,
    closeDeleteConfirm: deletion.closeDeleteConfirm,
    setDeleteFiles: deletion.setDeleteFiles,
    toggleSelect: deletion.toggleSelect,
    toggleSelectMode: deletion.toggleSelectMode,
  }
}
