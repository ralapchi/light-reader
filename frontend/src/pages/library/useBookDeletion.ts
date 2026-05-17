import { useCallback, useState } from 'react'
import { libraryRemove, libraryRemoveBatch } from '../../services/api'
import type { LibraryBookCardDto } from '../../services/api'

export interface DeleteConfirmState {
  open: boolean
  bookIds: string[]
  title: string
  message: string
  deleteFiles: boolean
}

export function useBookDeletion(books: LibraryBookCardDto[], loadBooks: () => void) {
  const [selectMode, setSelectMode] = useState(false)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [deleteConfirm, setDeleteConfirm] = useState<DeleteConfirmState>({
    open: false,
    bookIds: [],
    title: '',
    message: '',
    deleteFiles: false,
  })

  const toggleSelect = useCallback((bookId: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev)
      if (next.has(bookId)) next.delete(bookId)
      else next.add(bookId)
      return next
    })
  }, [])

  const toggleSelectMode = useCallback(() => {
    setSelectMode(prev => !prev)
    setSelectedIds(new Set())
  }, [])

  const handleDeleteSingle = useCallback((bookId: string) => {
    const book = books.find(b => b.book_id === bookId)
    const title = book?.title ?? '该书籍'
    setDeleteConfirm({
      open: true,
      bookIds: [bookId],
      title: '移除书籍',
      message: `将从书架移除「${title}」，阅读进度和缓存也将一并清除。`,
      deleteFiles: false,
    })
  }, [books])

  const handleDeleteBatch = useCallback(() => {
    const ids = Array.from(selectedIds)
    if (ids.length === 0) return
    setDeleteConfirm({
      open: true,
      bookIds: ids,
      title: '批量移除',
      message: `将从书架移除 ${ids.length} 本书籍，所有阅读进度和缓存也将一并清除。`,
      deleteFiles: false,
    })
  }, [selectedIds])

  const handleDeleteConfirm = useCallback(async () => {
    const { bookIds, deleteFiles } = deleteConfirm
    try {
      if (bookIds.length === 1) {
        await libraryRemove(bookIds[0], deleteFiles)
      } else {
        await libraryRemoveBatch(bookIds, deleteFiles)
      }
    } catch (e) {
      console.error('删除失败:', e)
    }
    setDeleteConfirm(prev => ({ ...prev, open: false }))
    setSelectedIds(new Set())
    setSelectMode(false)
    loadBooks()
  }, [deleteConfirm, loadBooks])

  const closeDeleteConfirm = useCallback(() => {
    setDeleteConfirm(prev => ({ ...prev, open: false }))
  }, [])

  const setDeleteFiles = useCallback((deleteFiles: boolean) => {
    setDeleteConfirm(prev => ({ ...prev, deleteFiles }))
  }, [])

  return {
    selectMode,
    selectedIds,
    deleteConfirm,
    toggleSelect,
    toggleSelectMode,
    handleDeleteSingle,
    handleDeleteBatch,
    handleDeleteConfirm,
    closeDeleteConfirm,
    setDeleteFiles,
  }
}
