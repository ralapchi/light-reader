import { Outlet } from 'react-router-dom'
import Sidebar from './Sidebar'
import useAppStore from '../store/useAppStore'
import './MainLayout.css'

export default function MainLayout() {
  const sidebarFooter = useAppStore(s => s.sidebarFooter)
  return (
    <div className="main-layout">
      <Sidebar footerText={sidebarFooter} />
      <Outlet />
    </div>
  )
}
