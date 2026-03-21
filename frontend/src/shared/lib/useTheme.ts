import { useEffect, useState } from 'react'

export type Theme = 'dark' | 'light'

export const THEME_STORAGE_KEY = 'deslop-theme'

const THEME_META_COLORS: Record<Theme, string> = {
  dark: '#07110d',
  light: '#f3f5f8',
}

function isTheme(value: string | null): value is Theme {
  return value === 'dark' || value === 'light'
}

export function getStoredTheme(): Theme {
  if (typeof window === 'undefined') {
    return 'dark'
  }

  const storedTheme = window.localStorage.getItem(THEME_STORAGE_KEY)
  return isTheme(storedTheme) ? storedTheme : 'dark'
}

export function applyTheme(theme: Theme) {
  if (typeof document === 'undefined') {
    return
  }

  const root = document.documentElement
  root.dataset.theme = theme
  root.classList.toggle('light', theme === 'light')

  const themeColor = document.querySelector('meta[name="theme-color"]')
  themeColor?.setAttribute('content', THEME_META_COLORS[theme])
}

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(() => getStoredTheme())

  useEffect(() => {
    applyTheme(theme)
    window.localStorage.setItem(THEME_STORAGE_KEY, theme)
  }, [theme])

  return {
    theme,
    toggleTheme: () => setTheme((currentTheme) => (currentTheme === 'dark' ? 'light' : 'dark')),
  }
}