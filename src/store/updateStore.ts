import { create } from 'zustand'
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { useConsoleStore } from './consoleStore'

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'installing'
  | 'error'

interface UpdateState {
  status: UpdateStatus
  version: string | null
  notes: string | null
  progress: number
  error: string | null
  checkForUpdates: (silent?: boolean) => Promise<void>
  installUpdate: () => Promise<void>
  dismiss: () => void
}

let pendingUpdate: Update | null = null
let lastLoggedStep = -1

export const useUpdateStore = create<UpdateState>((set, get) => ({
  status: 'idle',
  version: null,
  notes: null,
  progress: 0,
  error: null,

  checkForUpdates: async (silent = true) => {
    const push = useConsoleStore.getState().push
    if (get().status === 'checking' || get().status === 'downloading') return
    set({ status: 'checking', error: null })
    if (!silent) push('info', 'Sprawdzanie aktualizacji…')
    try {
      const update = await check()
      if (update) {
        pendingUpdate = update
        set({ status: 'available', version: update.version, notes: update.body ?? null })
        push('success', `Dostępna aktualizacja ${update.version} (zainstalowana: ${update.currentVersion}).`)
      } else {
        pendingUpdate = null
        set({ status: 'idle', version: null, notes: null })
        push('info', 'Launcher aktualny — brak nowych wersji.')
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error)
      pendingUpdate = null
      set({ status: 'error', error: msg })
      push('error', `Sprawdzanie aktualizacji nie powiodło się: ${msg}`)
      // silent check must not leave the dialog open — back to idle shortly
      if (silent) set({ status: 'idle' })
    }
  },

  installUpdate: async () => {
    const push = useConsoleStore.getState().push
    const update = pendingUpdate
    if (!update) {
      set({ status: 'error', error: 'Brak pobranej aktualizacji.' })
      return
    }
    set({ status: 'downloading', progress: 0, error: null })
    lastLoggedStep = -1
    push('info', `Pobieranie aktualizacji ${update.version}…`)
    let downloaded = 0
    let contentLength = 0
    const onEvent = (event: DownloadEvent) => {
      if (event.event === 'Started') {
        contentLength = event.data.contentLength ?? 0
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength
        if (contentLength > 0) {
          const pct = Math.min(100, Math.floor((downloaded / contentLength) * 100))
          set({ progress: pct })
          const step = Math.floor(pct / 10)
          if (step !== lastLoggedStep) {
            lastLoggedStep = step
            push('progress', `Pobieranie aktualizacji ${update.version}: ${pct}%`)
          }
        }
      } else if (event.event === 'Finished') {
        set({ progress: 100, status: 'installing' })
        push('success', `Pobieranie ukończone (${formatBytes(downloaded)}). Instalowanie…`)
      }
    }
    try {
      await update.downloadAndInstall(onEvent)
      push('success', `Aktualizacja ${update.version} zainstalowana. Restart launchera…`)
      pendingUpdate = null
      // On Windows the installer exits the app itself; relaunch is a no-op fallback.
      await relaunch().catch(() => undefined)
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error)
      set({ status: 'error', error: msg })
      push('error', `Instalacja aktualizacji nie powiodła się: ${msg}`)
    }
  },

  dismiss: () => {
    // User said "Nie" — release backend resources, stay on current version.
    if (pendingUpdate) {
      pendingUpdate.close().catch(() => undefined)
      pendingUpdate = null
    }
    set({ status: 'idle', version: null, notes: null, progress: 0, error: null })
  },
}))

function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)))
  return `${(bytes / 1024 ** i).toFixed(1)} ${units[i]}`
}
