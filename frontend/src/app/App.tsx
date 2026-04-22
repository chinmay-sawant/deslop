import { Routes, Route, Navigate } from 'react-router-dom'
import { HomePage } from '../features/home/HomePage'
import { DocsPage } from '../features/docs/DocsPage'
import { FindingsPage } from '../features/findings/FindingsPage'
import { Header } from '../features/home/components/Header'
import { useTheme } from '../shared/lib/useTheme'

export default function App() {
  const { theme, toggleTheme } = useTheme()

  return (
    <>
      <a href="#main-content" className="skip-link">
        Skip to main content
      </a>
      <div className="page-viewport">
        <div className="page-shell flex flex-col min-h-screen">
          <Header theme={theme} onToggleTheme={toggleTheme} />
          <div id="main-content" className="flex-1">
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route index element={<HomePage />} />
              <Route path="/docs" element={<DocsPage />} />
              <Route path="/findings" element={<FindingsPage />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </div>
        </div>
      </div>
    </>
  )
}
