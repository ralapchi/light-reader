import { useState, useEffect, useRef, useCallback } from 'react'
import {
  onDragDropEvent,
  filterBookFiles,
  libraryImport,
} from '../services/api'
import useAppStore from '../store/useAppStore'
import { mergeImportedBooks, getImportToastMessage, loadBookCovers } from '../utils/importUtils'
import './DropOverlay.css'

type OverlayState = 'idle' | 'hover' | 'dropping'

const HOVER_TIMEOUT_MS = 8000

export default function DropOverlay() {
  const [state, setState] = useState<OverlayState>('idle')
  const [fileCount, setFileCount] = useState(0)
  const setBooks = useAppStore(s => s.setBooks)
  const showToast = useAppStore(s => s.showToast)
  const booksRef = useRef(useAppStore.getState().books)
  const isDroppingRef = useRef(false)
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // 同步订阅 books 以保持 booksRef 最新
  useEffect(() => {
    return useAppStore.subscribe(s => { booksRef.current = s.books })
  }, [])

  const clearTimer = useCallback(() => {
    if (hoverTimerRef.current) {
      clearTimeout(hoverTimerRef.current)
      hoverTimerRef.current = null
    }
  }, [])

  useEffect(() => {
    let unlisten: (() => void) | null = null

    onDragDropEvent((event) => {
      const { type, paths } = event

      if (type === 'enter') {
        // enter 事件携带 paths，用它来判断是否为外部文件拖入
        if (paths && paths.length > 0) {
          clearTimer()
          setState('hover')
          // 兜底：防止 leave 事件丢失时 hover 状态卡住
          hoverTimerRef.current = setTimeout(() => {
            setState(prev => prev === 'hover' ? 'idle' : prev)
          }, HOVER_TIMEOUT_MS)
        }
      } else if (type === 'leave') {
        clearTimer()
        if (!isDroppingRef.current) {
          setState('idle')
        }
      } else if (type === 'drop') {
        // 防重复：导入进行中时忽略新的拖拽
        if (isDroppingRef.current) return

        clearTimer()

        if (!paths || paths.length === 0) {
          setState('idle')
          return
        }

        const bookFiles = filterBookFiles(paths)
        if (bookFiles.length === 0) {
          setState('idle')
          return
        }

        setFileCount(bookFiles.length)
        setState('dropping')
        isDroppingRef.current = true

        libraryImport(bookFiles)
          .then((result) => {
            // 合并导入结果到 store
            const merged = mergeImportedBooks(booksRef.current, result.imported)
            setBooks(merged)

            // 显示 toast
            const toast = getImportToastMessage(result)
            showToast(toast)

            // 异步加载封面
            loadBookCovers(result.imported)
          })
          .catch((err) => {
            console.error('拖拽导入失败:', err)
            const msg = err instanceof Error ? err.message : String(err)
            showToast({ type: 'error', message: '导入失败', detail: msg || '请检查文件格式是否受支持' })
          })
          .finally(() => {
            isDroppingRef.current = false
            setState('idle')
          })
      }
    }).then(fn => { unlisten = fn })

    return () => {
      unlisten?.()
      clearTimer()
    }
  }, [setBooks, showToast, clearTimer])

  if (state === 'idle') return null

  return (
    <div className="drop-overlay">
      <div className="drop-overlay-bg" />
      <div className={`drop-zone ${state === 'dropping' ? 'dropping' : ''}`}>
        <div className="drop-flash" />
        <div className="drop-icon">
          <div className="drop-icon-ring" />
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
            <line x1="12" y1="8" x2="12" y2="16" />
            <polyline points="8 12 12 8 16 12" />
          </svg>
        </div>
        <div className="drop-text-main">
          {state === 'dropping' ? '正在导入...' : '拖拽文件到此处导入'}
        </div>
        <div className="drop-text-sub">
          {state === 'dropping'
            ? `处理 ${fileCount} 个文件`
            : <>支持 <span>EPUB</span>、<span>TXT</span> 格式</>
          }
        </div>
      </div>
    </div>
  )
}
