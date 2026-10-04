import { create } from 'zustand'
import { dontcamApi, type DontcamCheckResult } from '../tauri/api'
import { useConsoleStore } from './consoleStore'

interface DontcamState {
  /** mcVersion currently being checked (blocks Play buttons while set). */
  checking: string | null
  lastResult: DontcamCheckResult | null
  checkDontcamUpdate: (mcVersion: string, profileId?: string) => Promise<DontcamCheckResult>
}

export const useDontcamStore = create<DontcamState>((set) => ({
  checking: null,
  lastResult: null,

  checkDontcamUpdate: async (mcVersion, profileId) => {
    const push = useConsoleStore.getState().push
    set({ checking: mcVersion })
    push('info', `Sprawdzanie DontCam dla ${mcVersion}…`)
    try {
      const res = await dontcamApi.check(mcVersion, profileId)
      set({ lastResult: res })
      if (res.offline) {
        push('info', `DontCam offline: ${res.message}`)
      } else if (res.updated) {
        push('success', `DontCam zaktualizowany: ${res.version}`)
      } else {
        push('info', `DontCam aktualny (${res.version || mcVersion}).`)
      }
      return res
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error)
      push('error', `Check DontCam nie powiódł się: ${msg}`)
      throw error
    } finally {
      set({ checking: null })
    }
  },
}))
