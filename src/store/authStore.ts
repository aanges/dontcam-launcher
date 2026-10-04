import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { Account, Theme } from '../types'
import { authApi, settingsApi } from '../tauri/api'
import type { Settings } from '../types'

// ---------- Auth ----------

interface AuthState {
  accounts: Account[]
  currentAccount: Account | null
  isLoading: boolean
  error: string | null
  loadAccounts: () => Promise<void>
  loginOffline: (username: string) => Promise<Account>
  loginMicrosoft: () => Promise<Account>
  logout: (accountId: string) => Promise<void>
  setCurrentAccount: (account: Account | null) => Promise<void>
  refreshAccount: (accountId: string) => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      accounts: [],
      currentAccount: null,
      isLoading: false,
      error: null,

      loadAccounts: async () => {
        set({ isLoading: true, error: null })
        try {
          const [accounts, current] = await Promise.all([
            authApi.getAccounts().catch(() => [] as Account[]),
            authApi.getCurrentAccount().catch(() => null),
          ])
          set({
            accounts,
            currentAccount: current ?? accounts[0] ?? null,
            isLoading: false,
          })
        } catch (e) {
          set({ isLoading: false, error: e instanceof Error ? e.message : String(e) })
        }
      },

      loginOffline: async (username) => {
        set({ isLoading: true, error: null })
        try {
          const account = await authApi.loginOffline(username)
          set((state) => ({
            accounts: [
              ...state.accounts.filter((a) => a.id !== account.id && !(a.account_type === 'offline' && a.username === account.username)),
              account,
            ],
            currentAccount: account,
            isLoading: false,
          }))
          return account
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e)
          set({ isLoading: false, error: msg })
          throw e
        }
      },

      loginMicrosoft: async () => {
        set({ isLoading: true, error: null })
        try {
          const account = await authApi.loginMicrosoft()
          set((state) => ({
            accounts: [...state.accounts.filter((a) => a.id !== account.id), account],
            currentAccount: account,
            isLoading: false,
          }))
          return account
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e)
          set({ isLoading: false, error: msg })
          throw e
        }
      },

      logout: async (accountId) => {
        await authApi.logout(accountId)
        set((state) => {
          const accounts = state.accounts.filter((a) => a.id !== accountId)
          return {
            accounts,
            currentAccount:
              state.currentAccount?.id === accountId
                ? accounts[0] ?? null
                : state.currentAccount,
          }
        })
      },

      setCurrentAccount: async (account) => {
        if (!account) {
          set({ currentAccount: null })
          return
        }
        try {
          const updated = await authApi.setCurrentAccount(account.id)
          set({ currentAccount: updated })
        } catch {
          set({ currentAccount: account })
        }
        void get
      },

      refreshAccount: async (accountId) => {
        const account = await authApi.refreshToken(accountId)
        set((state) => ({
          accounts: state.accounts.map((a) => (a.id === accountId ? account : a)),
          currentAccount:
            state.currentAccount?.id === accountId ? account : state.currentAccount,
        }))
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        accounts: state.accounts,
        currentAccount: state.currentAccount,
      }),
    }
  )
)

// ---------- Theme ----------

interface ThemeState {
  theme: Theme
  resolvedTheme: 'light' | 'dark'
  setTheme: (theme: Theme) => void
  initializeTheme: () => void
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      theme: 'dark',
      resolvedTheme: 'dark',

      setTheme: (theme) => {
        set({ theme })
        get().initializeTheme()
        // persist into backend settings too (best effort)
        settingsApi
          .get()
          .then((s) => settingsApi.update({ ...s, theme } as Settings))
          .catch(() => undefined)
      },

      initializeTheme: () => {
        const { theme } = get()
        let resolved: 'light' | 'dark' = 'dark'
        if (theme === 'system') {
          resolved = window.matchMedia('(prefers-color-scheme: dark)').matches
            ? 'dark'
            : 'light'
        } else if (theme === 'light') {
          resolved = 'light'
        } else if (theme === 'dark') {
          resolved = 'dark'
        } else {
          resolved = 'dark'
        }
        set({ resolvedTheme: resolved })
        document.documentElement.classList.toggle('dark', resolved === 'dark')
      },
    }),
    {
      name: 'theme-storage',
      partialize: (state) => ({ theme: state.theme }),
    }
  )
)
