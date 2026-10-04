import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { Settings } from '../types'
import { settingsApi } from '../tauri/api'

const defaultSettings: Settings = {
  theme: 'system',
  language: 'en-US',
  java: {
    auto_detect: true,
    preferred_version: '21',
    custom_java_path: undefined,
    jvm_args: '-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200',
    memory_allocation: { min: 1, max: 4, unit: 'GB' },
  },
  game: {
    default_game_dir: undefined,
    auto_close_launcher: false,
    keep_launcher_open: true,
    custom_resolution: undefined,
    fullscreen: false,
    vsync: true,
    fov: 70,
    render_distance: 12,
    max_fps: 0,
    enable_mods: true,
    enable_resource_packs: true,
  },
  network: {
    proxy_enabled: false,
    proxy_host: '',
    proxy_port: 8080,
    proxy_username: undefined,
    proxy_password: undefined,
    download_threads: 4,
    bandwidth_limit: undefined,
  },
  ui: {
    show_snapshots: false,
    show_old_versions: true,
    show_alpha_beta: false,
    sort_versions_by: 'newest_first',
    compact_mode: false,
    animations: true,
    background_blur: true,
    news_enabled: true,
  },
  advanced: {
    debug_logging: false,
    console_enabled: false,
    custom_game_args: [],
    environment_variables: {},
    pre_launch_command: undefined,
    post_exit_command: undefined,
    verify_downloads: true,
    parallel_downloads: 4,
  },
}

interface SettingsState {
  settings: Settings
  isLoading: boolean
  loadSettings: () => Promise<void>
  updateSettings: (settings: Partial<Settings>) => Promise<void>
  resetSettings: () => Promise<void>
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      settings: defaultSettings,
      isLoading: false,
      
      loadSettings: async () => {
        set({ isLoading: true })
        try {
          const settings = await settingsApi.get()
          set({ settings, isLoading: false })
        } catch {
          set({ settings: defaultSettings, isLoading: false })
        }
      },
      
      updateSettings: async (partialSettings) => {
        const newSettings = { ...get().settings, ...partialSettings }
        set({ settings: newSettings })
        await settingsApi.update(newSettings)
      },
      
      resetSettings: async () => {
        const settings = await settingsApi.reset()
        set({ settings })
      },
    }),
    {
      name: 'settings-storage',
      partialize: (state) => ({ settings: state.settings }),
    }
  )
)