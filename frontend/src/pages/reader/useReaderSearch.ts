import { useCallback, useEffect, useRef, useState } from 'react'
import { searchInBook } from '../../services/api'
import type { SearchHitDto } from '../../services/api'
import useAppStore from '../../store/useAppStore'

export function useReaderSearch() {
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<SearchHitDto[]>([])
  const timerRef = useRef<ReturnType<typeof setTimeout>>(null)
  const searchIdRef = useRef(0)
  const { toggleSearch, closeSearch } = useAppStore()
  const showSearch = useAppStore(s => s.reader.showSearch)

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
      searchIdRef.current++
    }
  }, [])

  const clearState = useCallback(() => {
    setSearchQuery('')
    setSearchResults([])
  }, [])

  const handleClose = useCallback(() => {
    clearState()
    closeSearch()
  }, [clearState, closeSearch])

  const handleToggle = useCallback(() => {
    if (showSearch) clearState()
    toggleSearch()
  }, [showSearch, clearState, toggleSearch])

  const handleInput = useCallback((q: string) => {
    setSearchQuery(q)
    if (timerRef.current) clearTimeout(timerRef.current)
    if (!q.trim()) {
      setSearchResults([])
      return
    }
    const sid = ++searchIdRef.current
    timerRef.current = setTimeout(async () => {
      try {
        const hits = await searchInBook(q.trim())
        if (searchIdRef.current === sid) {
          setSearchResults(hits)
        }
      } catch {
        if (searchIdRef.current === sid) {
          setSearchResults([])
        }
      }
    }, 300)
  }, [])

  return { searchQuery, searchResults, handleInput, handleClose, handleToggle }
}
