import { Terminal, Trash2, ChevronDown, ChevronUp } from 'lucide-react'
import { Card } from './ui/Card'
import { Button } from './ui/Button'
import { useConsoleStore, type ConsoleKind } from '../store/consoleStore'
import { cn } from '../utils/helpers'

const KIND_STYLES: Record<ConsoleKind, string> = {
  info: 'text-slate-400',
  progress: 'text-primary-300',
  'game-out': 'text-slate-200',
  'game-err': 'text-orange-300',
  error: 'text-red-300',
  success: 'text-emerald-300',
}

export function ConsolePanel() {
  const { lines, open, setOpen, clear } = useConsoleStore()

  return (
    <Card padding="none" className="overflow-hidden">
      <div className="flex items-center justify-between gap-3 border-b border-white/[0.08] px-5 py-3">
        <button
          className="flex items-center gap-2 font-display text-sm font-bold text-white"
          onClick={() => setOpen(!open)}
          aria-expanded={open}
        >
          <Terminal className="h-4 w-4 text-primary-300" />
          Console
          <span className="pill">{lines.length}</span>
          {open ? <ChevronUp className="h-4 w-4 text-slate-500" /> : <ChevronDown className="h-4 w-4 text-slate-500" />}
        </button>
        <Button variant="ghost" size="sm" onClick={clear} aria-label="Clear console">
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
      {open && (
        <div className="h-44 overflow-y-auto bg-black/50 p-4 font-mono text-xs leading-relaxed">
          {lines.length === 0 ? (
            <p className="text-slate-600">No output yet. Launch the game or install a version to see logs here.</p>
          ) : (
            lines.map((line) => (
              <div key={line.id} className="flex gap-2">
                <span className="shrink-0 text-slate-600">{line.time}</span>
                <span className={cn('break-all whitespace-pre-wrap', KIND_STYLES[line.kind])}>{line.text}</span>
              </div>
            ))
          )}
        </div>
      )}
    </Card>
  )
}
