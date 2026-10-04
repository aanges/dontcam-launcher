import { create } from 'zustand'
import type { LaunchStatus, LaunchOptions } from '../types'
import { launchApi } from '../tauri/api'

interface LaunchState {
  launchStatus: LaunchStatus
  launchGame: (options: LaunchOptions) => Promise<void>
  killGame: () => Promise<void>
  getStatus: () => Promise<void>
  setStatus: (status: LaunchStatus) => void
}

export const useLaunchStore = create<LaunchState>((set) => ({
  launchStatus: 'idle',
  
  launchGame: async (options) => {
    set({ launchStatus: 'launching' })
    try {
      await launchApi.launch(options)
      set({ launchStatus: 'running' })
    } catch (error) {
      set({ launchStatus: { error: error instanceof Error ? error.message : 'Launch failed' } })
      throw error
    }
  },
  
  killGame: async () => {
    await launchApi.kill()
    set({ launchStatus: 'idle' })
  },
  
  getStatus: async () => {
    const status = await launchApi.getStatus()
    set({ launchStatus: status })
  },

  setStatus: (status) => set({ launchStatus: status }),
}))