import { create } from 'zustand'
import { listen } from '@tauri-apps/api/event'
import { useLaunchStore } from './launchStore'
import type { LaunchStatus } from '../types'

export type ConsoleKind = 'info' | 'progress' | 'game-out' | 'game-err' | 'error' | 'success'

export interface ConsoleLine {
  id: number
  time: string
  kind: ConsoleKind
  text: string
}

let nextId = 1
let initialized = false

interface ConsoleState {
  lines: ConsoleLine[]
  open: boolean
  setOpen: (open: boolean) => void
  push: (kind: ConsoleKind, text: string) => void
  clear: () => void
}

export const useConsoleStore = create<ConsoleState>((set) => ({
  lines: [],
  open: true,
  setOpen: (open) => set({ open }),
  push: (kind, text) =>
    set((state) => ({
      lines: [
        ...state.lines.slice(-399),
        { id: nextId++, time: new Date().toLocaleTimeString(), kind, text },
      ],
    })),
  clear: () => set({ lines: [] }),
}))

interface DownloadProgress {
  version_id: string
  stage: string
  current: number
  total: number
}

interface GameLog {
  stream: string
  line: string
}

/** Subscribe to backend events once. Safe to call in browser (failures ignored). */
export function initLauncherEvents() {
  if (initialized) return
  initialized = true

  const push = (kind: ConsoleKind, text: string) => useConsoleStore.getState().push(kind, text)

  // App version first (proves which build is running — incl. after an update).
  import('@tauri-apps/api/app')
    .then(({ getVersion }) => getVersion())
    .then((v) => push('info', `DontCam Client v${v} — console ready`))
    .catch(() => push('info', 'DontCam Client console ready'))

  listen<DownloadProgress>('download-progress', (event) => {
    const { version_id, stage, current, total } = event.payload
    if (stage === 'done') push('success', `[${version_id}] download complete`)
    else push('progress', `[${version_id}] ${stage} ${current}/${total}`)
  }).catch(() => undefined)

  listen<LaunchStatus>('launch-status', (event) => {
    const status = event.payload
    useLaunchStore.getState().setStatus(status)
    if (typeof status === 'string') {
      if (status === 'running') push('success', 'Game is running — have fun!')
      else if (status === 'idle') push('info', 'Back to idle')
      else push('info', `Status: ${status}`)
    } else {
      const obj = status as { error?: string; crashed?: string }
      if (obj.error) push('error', `Launch failed: ${obj.error}`)
      else if (obj.crashed) push('error', `Game closed: ${obj.crashed}`)
    }
  }).catch(() => undefined)

  listen<GameLog>('game-log', (event) => {
    push(event.payload.stream === 'stderr' ? 'game-err' : 'game-out', event.payload.line)
  }).catch(() => undefined)
}
