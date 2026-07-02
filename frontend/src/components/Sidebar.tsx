import { useNavigate, useLocation } from 'react-router-dom'
import useAppStore from '../store/useAppStore'
import './Sidebar.css'

interface SidebarProps {
  footerText: string
}

const NAV_ITEMS = [
  {
    section: '书库',
    items: [
      { id: 'library', label: '全部', path: '/', icon: <><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" /><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" /></> },
      { id: 'bookmarks', label: '书签', path: '/bookmarks', icon: <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" /> },
    ],
  },
  {
    section: '设置',
    items: [
      { id: 'settings', label: '设置', path: '/settings', icon: <><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></> },
    ],
  },
]

function pathToActive(pathname: string): string {
  if (pathname === '/') return 'library'
  if (pathname === '/bookmarks') return 'bookmarks'
  if (pathname === '/settings') return 'settings'
  return ''
}

const COLLAPSE_ICON = <polyline points="15 18 9 12 15 6" />
const EXPAND_ICON = <polyline points="9 18 15 12 9 6" />

export default function Sidebar({ footerText }: SidebarProps) {
  const navigate = useNavigate()
  const location = useLocation()
  const activeId = pathToActive(location.pathname)
  const collapsed = useAppStore(s => s.sidebarCollapsed)
  const toggleSidebar = useAppStore(s => s.toggleSidebar)

  return (
    <nav className={`main-sidebar ${collapsed ? 'collapsed' : ''}`}>
      <div className="sidebar-top">
        <div className="sidebar-brand">轻看</div>
        <button
          type="button"
          className="sidebar-collapse-btn"
          onClick={toggleSidebar}
          aria-label={collapsed ? '展开侧边栏' : '折叠侧边栏'}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            {collapsed ? EXPAND_ICON : COLLAPSE_ICON}
          </svg>
        </button>
      </div>
      {NAV_ITEMS.map(section => (
        <div className="sidebar-section" key={section.section}>
          <div className="sidebar-section-title">{section.section}</div>
          {section.items.map(item => (
            <button
              type="button"
              key={item.id}
              className={`sidebar-item ${activeId === item.id ? 'active' : ''}`}
              onClick={() => item.path && navigate(item.path)}
              title={collapsed ? item.label : undefined}
            >
              <svg className="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                {item.icon}
              </svg>
              <span className="sidebar-item-label">{item.label}</span>
            </button>
          ))}
        </div>
      ))}
      <div className="sidebar-spacer" />
      <div className="sidebar-footer">{footerText}</div>
    </nav>
  )
}
