import { createBrowserRouter } from 'react-router-dom'
import MainLayout from '../components/MainLayout'
import LibraryPage from '../pages/LibraryPage'
import LoadingPage from '../pages/LoadingPage'
import ReaderPage from '../pages/ReaderPage'
import BookmarkPage from '../pages/BookmarkPage'
import SettingsPage from '../pages/SettingsPage'

const router = createBrowserRouter([
  {
    element: <MainLayout />,
    children: [
      { index: true, element: <LibraryPage /> },
      { path: 'bookmarks', element: <BookmarkPage /> },
      { path: 'settings', element: <SettingsPage /> },
    ],
  },
  { path: '/loading/:bookId', element: <LoadingPage /> },
  { path: '/reader/:bookId', element: <ReaderPage /> },
])

export default router
