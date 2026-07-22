import { useEffect } from 'react'
import { RouterProvider } from 'react-router-dom'
import router from './router'
import { settingsLoad } from '../services/api'
import useAppStore from '../store/useAppStore'

function App() {
  useEffect(() => {
    settingsLoad().then(settings => {
      useAppStore.getState().setSettings(settings)
    }).catch(() => {})
  }, [])

  // 禁用应用内 HTML5 拖拽，防止干扰 Tauri 原生 DragDrop
  useEffect(() => {
    const preventDrag = (e: DragEvent) => {
      e.preventDefault()
    }
    document.addEventListener('dragover', preventDrag)
    document.addEventListener('drop', preventDrag)
    return () => {
      document.removeEventListener('dragover', preventDrag)
      document.removeEventListener('drop', preventDrag)
    }
  }, [])

  return <RouterProvider router={router} />
}

export default App
