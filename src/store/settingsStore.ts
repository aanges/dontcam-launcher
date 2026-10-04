import { create } from 'zustand'
import type { Settings } from '../types'
import { settingsApi } from '../tauri/api'

export const DEFAULT_SETTINGS: Settings = {
  language: 'en-US',
  java: {
    auto_detect: true,
    jvm_args: '',
    memory_allocation: { min: 1, max: 4, unit: 'GB' },
  },
  game: {
    auto_close_launcher: false,
    keep_launcher_open: true,
    fullscreen: false,
  },
  network: {
    download_threads: 4,
  },
  ui: {
    show_snapshots: false,
    show_old_versions: true,
    show_alpha_beta: false,
    sort_versions_by: 'newest_first',
    compact_mode: false,
    animations: true,
  },
  advanced: {
    debug_logging: false,
    console_enabled: true,
    custom_game_args: [],
    verify_downloads: true,
  },
}

interface SettingsState {
  settings: Settings
  isLoading: boolean
  loadSettings: () => Promise<void>
  updateSettings: (patch: Partial<Settings>) => Promise<void>
  resetSettings: () => Promise<void>
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  isLoading: false,

  loadSettings: async () => {
    set({ isLoading: true })
    try {
      const settings = await settingsApi.get()
      set({ settings })
    } catch {
      // Backend unavailable — keep defaults.
    } finally {
      set({ isLoading: false })
    }
  },

  updateSettings: async (patch) => {
    const merged: Settings = { ...get().settings, ...patch }
    set({ settings: merged })
    try {
      const saved = await settingsApi.update(merged)
      set({ settings: saved })
    } catch {
      // Optimistic local update stays when the backend is unreachable.
    }
  },

  resetSettings: async () => {
    try {
      const settings = await settingsApi.reset()
      set({ settings })
    } catch {
      set({ settings: DEFAULT_SETTINGS })
    }
  },
}))
