import { useState, useEffect, useRef } from 'react'
import {
  onDragDropEvent,
  filterBookFiles,
  libraryImport,
  libraryList,
} from '../services/api'
import useAppStore from '../store/useAppStore'
import './DropOverlay.css'

type OverlayState = 'idle' | 'hover' | 'dropping' | 'success'

export default function DropOverlay() {
  const [state, setState] = useState<OverlayState>('idle')
  const [fileCount, setFileCount] = useState(0)
  const setBooks = useAppStore(s => s.setBooks)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const clearTimer = () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
  }

  useEffect(() => {
    let unlisten: (() => void) | null = null

    onDragDropEvent((event) => {
      if (event === 'enter') {
        clearTimer()
        setState('hover')
      } else if (event === 'leave') {
        clearTimer()
        setState('idle')
      } else if (typeof event === 'object' && event.type === 'drop') {
        const bookFiles = filterBookFiles(event.paths)
        if (bookFiles.length === 0) {
          setState('idle')
          return
        }
        setFileCount(bookFiles.length)
        setState('dropping')

        libraryImport(bookFiles)
          .then(async () => {
            const items = await libraryList()
            setBooks(items)
            setState('success')
            clearTimer()
            timerRef.current = setTimeout(() => setState('idle'), 2000)
          })
          .catch((err) => {
            console.error('拖拽导入失败:', err)
            setState('idle')
          })
      }
    }).then(fn => { unlisten = fn })

    return () => {
      unlisten?.()
      clearTimer()
    }
  }, [setBooks])

  if (state === 'idle') return null

  return (
    <div className="drop-overlay">
      <div className="drop-overlay-bg" />
      <div className={`drop-zone ${state === 'dropping' ? 'dropping' : ''}`}>
        <div className="drop-flash" />
        {state === 'success' ? (
          <div className="drop-success">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            <div className="drop-success-text">导入成功</div>
            <div className="drop-success-count">已添加 {fileCount} 本书籍</div>
          </div>
        ) : (
          <>
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
          </>
        )}
      </div>
    </div>
  )
}
