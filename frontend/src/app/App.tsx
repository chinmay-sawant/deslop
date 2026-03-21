import { HomePage } from '../features/home/HomePage'
import { useTheme } from '../shared/lib/useTheme'

export default function App() {
  const { theme, toggleTheme } = useTheme()

  return (
    <div className="page-viewport">
      <div className="page-shell">
        <HomePage theme={theme} onToggleTheme={toggleTheme} />
      </div>
    </div>
  )
}