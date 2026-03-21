import { HomePage } from '../features/home/HomePage'
import { useTheme } from '../shared/lib/useTheme'

export default function App() {
  const { theme, toggleTheme } = useTheme()

  return <HomePage theme={theme} onToggleTheme={toggleTheme} />
}