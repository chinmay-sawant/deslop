import { Routes, Route } from 'react-router-dom'
import { HomePage } from '../features/home/HomePage'
import { DocsPage } from '../features/docs/DocsPage'
import { Header } from '../features/home/components/Header'
import { useTheme } from '../shared/lib/useTheme'

export default function App() {
  const { theme, toggleTheme } = useTheme()

  return (
    <div className="page-viewport">
      <div className="page-shell flex flex-col min-h-screen">
        <Header theme={theme} onToggleTheme={toggleTheme} />
        <div className="flex-1">
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/docs" element={<DocsPage />} />
          </Routes>
        </div>
      </div>
    </div>
  )
}