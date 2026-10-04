import { Minus, Maximize2, X } from 'lucide-react'
import { Button } from '../ui/Button'
import { LogoTile } from '../Logo'
import { invoke } from '@tauri-apps/api/core'

export function TitleBar() {
  const handleMinimize = () => {
    invoke('minimize_window').catch(console.error)
  }

  const handleMaximize = () => {
    invoke('toggle_maximize').catch(console.error)
  }

  const handleClose = () => {
    invoke('close_window').catch(console.error)
  }

  return (
    <header className="drag-region z-50 flex h-12 shrink-0 items-center justify-between border-b border-white/[0.07] bg-black/60 pl-4 pr-2 backdrop-blur-xl">
      <div className="flex items-center gap-3">
        <LogoTile className="h-7 w-7 !rounded-[10px]" markClassName="h-full w-full" />
        <div className="hidden items-baseline gap-2 sm:flex">
          <h1 className="font-display text-[13px] font-bold uppercase tracking-[0.24em] text-white">
            DontCam
          </h1>
          <span className="font-display text-[11px] font-bold uppercase tracking-[0.24em] text-primary-400">
            Client
          </span>
        </div>
      </div>

      <div className="no-drag flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-10 !rounded-lg text-slate-400 hover:!bg-white/10 hover:!text-white"
          onClick={handleMinimize}
          aria-label="Minimize"
        >
          <Minus className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-10 !rounded-lg text-slate-400 hover:!bg-white/10 hover:!text-white"
          onClick={handleMaximize}
          aria-label="Maximize"
        >
          <Maximize2 className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-10 !rounded-lg text-slate-400 hover:!bg-red-500 hover:!text-white"
          onClick={handleClose}
          aria-label="Close"
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </header>
  )
}
