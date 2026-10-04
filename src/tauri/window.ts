import { invoke } from '@tauri-apps/api/core'

export const windowApi = {
  minimize: (): Promise<void> => invoke('minimize_window'),
  maximize: (): Promise<void> => invoke('maximize_window'),
  unmaximize: (): Promise<void> => invoke('unmaximize_window'),
  toggleMaximize: (): Promise<void> => invoke('toggle_maximize'),
  close: (): Promise<void> => invoke('close_window'),
  setTitle: (title: string): Promise<void> => invoke('set_window_title', { title }),
  center: (): Promise<void> => invoke('center_window'),
  show: (): Promise<void> => invoke('show_window'),
  hide: (): Promise<void> => invoke('hide_window'),
}

export function useWindowControls() {
  return {
    minimize: windowApi.minimize,
    maximize: windowApi.toggleMaximize,
    close: windowApi.close,
  }
}
