import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { Account, Theme } from '../types'
import { authApi } from '../tauri/api'

interface AuthState {
  accounts: Account[]
  currentAccount: Account | null
  isLoading: boolean
  loadAccounts: () => Promise<void>
  loginOffline: (username: string) => Promise<Account>
  loginMicrosoft: () => Promise<Account>
  logout: (accountId: string) => Promise<void>
  setCurrentAccount: (accountId: string) => Promise<void>
  refreshToken: (accountId: string) => Promise<Account>
}

export const useAuthStore = create<AuthState>()((set) => ({
  accounts: [],
  currentAccount: null,
  isLoading: false,

  loadAccounts: async () => {
    set({ isLoading: true })
    try {
      const accounts = await authApi.getAccounts()
      const current = await authApi.getCurrentAccount().catch(() => null)
      set({ accounts, currentAccount: current ?? accounts[0] ?? null })
    } catch {
      // Backend unavailable (browser preview) — stay empty.
    } finally {
      set({ isLoading: false })
    }
  },

  loginOffline: async (username) => {
    const account = await authApi.loginOffline(username)
    set((state) => ({
      accounts: [...state.accounts.filter((a) => a.id !== account.id), account],
      currentAccount: account,
    }))
    return account
  },

  loginMicrosoft: async () => {
    const account = await authApi.loginMicrosoft()
    set((state) => ({
      accounts: [...state.accounts.filter((a) => a.id !== account.id), account],
      currentAccount: account,
    }))
    return account
  },

  logout: async (accountId) => {
    await authApi.logout(accountId)
    set((state) => {
      const accounts = state.accounts.filter((a) => a.id !== accountId)
      return {
        accounts,
        currentAccount:
          state.currentAccount?.id === accountId ? (accounts[0] ?? null) : state.currentAccount,
      }
    })
  },

  setCurrentAccount: async (accountId) => {
    const account = await authApi.setCurrentAccount(accountId)
    set({ currentAccount: account })
  },

  refreshToken: async (accountId) => {
    const account = await authApi.refreshToken(accountId)
    set((state) => ({
      accounts: state.accounts.map((a) => (a.id === account.id ? account : a)),
      currentAccount: state.currentAccount?.id === account.id ? account : state.currentAccount,
    }))
    return account
  },
}))

type ResolvedTheme = 'light' | 'dark'

interface ThemeState {
  theme: Theme
  resolvedTheme: ResolvedTheme
  setTheme: (theme: Theme) => void
  initializeTheme: () => void
}

function resolveTheme(theme: Theme): ResolvedTheme {
  if (theme !== 'system') return theme
  if (typeof window === 'undefined') return 'dark'
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}

function applyTheme(resolved: ResolvedTheme) {
  document.documentElement.classList.toggle('dark', resolved === 'dark')
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      theme: 'dark',
      resolvedTheme: 'dark',
      setTheme: (theme) => {
        const resolvedTheme = resolveTheme(theme)
        applyTheme(resolvedTheme)
        set({ theme, resolvedTheme })
      },
      initializeTheme: () => {
        const resolvedTheme = resolveTheme(get().theme)
        applyTheme(resolvedTheme)
        set({ resolvedTheme })
      },
    }),
    { name: 'theme-storage' }
  )
)
