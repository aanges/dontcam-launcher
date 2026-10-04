import { invoke } from '@tauri-apps/api/core'

export const windowApi = {
  minimize: (): Promise<void> => invoke('minimize_window'),
  toggleMaximize: (): Promise<void> => invoke('toggle_maximize'),
  close: (): Promise<void> => invoke('close_window'),
}
