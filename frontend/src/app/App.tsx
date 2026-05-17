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

  return <RouterProvider router={router} />
}

export default App
