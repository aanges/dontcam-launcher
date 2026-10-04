import { useEffect, useState, type ReactNode } from 'react'
import { useThemeStore } from '../store/authStore'

// Kept minimal on purpose: the zustand store is the source of truth.
export function ThemeProvider({ children }: { children: ReactNode }) {
  const { initializeTheme } = useThemeStore()
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
    initializeTheme()

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => initializeTheme()
    mediaQuery.addEventListener('change', handleChange)

    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [initializeTheme])

  if (!mounted) {
    return <>{children}</>
  }

  return <>{children}</>
}

export function useTheme() {
  const resolvedTheme = useThemeStore((s) => s.resolvedTheme)
  const setTheme = useThemeStore((s) => s.setTheme)
  return {
    theme: resolvedTheme,
    toggleTheme: () => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark'),
    setTheme,
  }
}
