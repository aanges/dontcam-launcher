import { useEffect, useRef } from 'react'
import { Trash2, ChevronDown, ChevronRight, TerminalSquare } from 'lucide-react'
import { cn } from '../utils/helpers'
import { useConsoleStore, type ConsoleKind } from '../store/consoleStore'
import { useLaunchStore } from '../store/launchStore'
import { useVersionStore } from '../store/versionStore'
import { Button } from './ui/Button'

const lineColors: Record<ConsoleKind, string> = {
  info: 'text-slate-400',
  progress: 'text-sky-400',
  'game-out': 'text-slate-200',
  'game-err': 'text-red-400',
  error: 'text-red-400 font-semibold',
  success: 'text-primary-300',
}

export function statusText(status: unknown, installing: string | null): { label: string; live: boolean } {
  if (installing) return { label: `Installing ${installing}…`, live: true }
  if (typeof status === 'string') {
    if (status === 'running') return { label: 'Game running', live: true }
    if (status === 'launching' || status === 'preparing') return { label: 'Launching…', live: true }
    if (status === 'downloading_assets' || status === 'downloading_libraries')
      return { label: 'Downloading…', live: true }
    return { label: 'Idle', live: false }
  }
  return { label: 'Attention needed', live: true }
}

export function ConsolePanel() {
  const { lines, open, setOpen, clear } = useConsoleStore()
  const launchStatus = useLaunchStore((s) => s.launchStatus)
  const isInstalling = useVersionStore((s) => s.isInstalling)
  const bodyRef = useRef<HTMLDivElement>(null)

  const status = statusText(launchStatus, isInstalling)

  useEffect(() => {
    const el = bodyRef.current
    if (el && open) el.scrollTop = el.scrollHeight
  }, [lines, open])

  return (
    <div className="panel overflow-hidden !rounded-[22px]">
      <div className="flex items-center gap-3 border-b border-white/[0.07] bg-black/30 px-4 py-2.5">
        <button
          onClick={() => setOpen(!open)}
          className="flex items-center gap-2 font-display text-sm font-bold text-slate-200 transition-colors hover:text-white"
          aria-expanded={open}
        >
          {open ? <ChevronDown className="h-4 w-4 text-slate-500" /> : <ChevronRight className="h-4 w-4 text-slate-500" />}
          <TerminalSquare className="h-4 w-4 text-primary-400" />
          Console
        </button>
        <span
          className={cn(
            'flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-bold',
            status.live
              ? 'border-primary-400/30 bg-primary-400/10 text-primary-200'
              : 'border-white/10 bg-white/[0.04] text-slate-400'
          )}
        >
          <span className={cn('h-1.5 w-1.5 rounded-full', status.live ? 'animate-pulse bg-primary-400' : 'bg-slate-500')}>
          </span>
          {status.label}
        </span>
        <span className="ml-auto font-mono text-[11px] text-slate-500">{lines.length} lines</span>
        <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={clear} aria-label="Clear console">
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
      {open && (
        <div
          ref={bodyRef}
          className="h-96 overflow-y-auto bg-black/60 px-4 py-3 font-mono text-[13px] leading-relaxed"
        >
          {lines.length === 0 ? (
            <p className="text-slate-600">No activity yet — install or launch something and watch it here.</p>
          ) : (
            lines.map((line) => (
              <div key={line.id} className="flex gap-2 whitespace-pre-wrap break-all">
                <span className="shrink-0 select-none text-slate-600">{line.time}</span>
                <span className={lineColors[line.kind]}>{line.text}</span>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}
