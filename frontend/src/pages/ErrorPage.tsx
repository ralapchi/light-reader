import { useRouteError, isRouteErrorResponse, useNavigate } from 'react-router-dom'
import './ErrorPage.css'

export default function ErrorPage() {
  const error = useRouteError()
  const navigate = useNavigate()

  let title = '出错了'
  let message = '发生了未知错误'

  if (isRouteErrorResponse(error)) {
    title = `${error.status} ${error.statusText}`
    message = error.data?.message || '页面不存在或无法访问'
  } else if (error instanceof Error) {
    message = error.message
  }

  return (
    <div className="error-page">
      <div className="error-content">
        <div className="error-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        </div>
        <h1 className="error-title">{title}</h1>
        <p className="error-message">{message}</p>
        <div className="error-actions">
          <button className="error-btn" onClick={() => navigate('/')}>
            返回书架
          </button>
          <button className="error-btn secondary" onClick={() => window.location.reload()}>
            刷新页面
          </button>
        </div>
      </div>
    </div>
  )
}
