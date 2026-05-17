import { useEffect } from 'react'
import { Outlet } from 'react-router-dom'
import Sidebar from './Sidebar'
import useAppStore from '../store/useAppStore'
import './MainLayout.css'

export default function MainLayout() {
  const sidebarFooter = useAppStore(s => s.sidebarFooter)
  const appTheme = useAppStore(s => s.reader.settings.app_theme)

  useEffect(() => {
    const el = document.documentElement
    if (appTheme === 'system') {
      delete el.dataset.theme
    } else {
      el.dataset.theme = appTheme
    }
  }, [appTheme])

  return (
    <div className="main-layout">
      <Sidebar footerText={sidebarFooter} />
      <Outlet />
    </div>
  )
}
