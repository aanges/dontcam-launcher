import { Download, X } from 'lucide-react'
import { Modal } from './ui/Modal'
import { Button } from './ui/Button'
import { useUpdateStore } from '../store/updateStore'

export function UpdateDialog() {
  const { status, version, notes, progress, installUpdate, dismiss } = useUpdateStore()

  const open = status === 'available' || status === 'downloading' || status === 'installing'
  const busy = status === 'downloading' || status === 'installing'

  return (
    <Modal
      isOpen={open}
      onClose={() => {
        if (!busy) dismiss()
      }}
      title={`Dostępna aktualizacja ${version ?? ''}`}
      size="md"
      showCloseButton={!busy}
      closeOnOverlayClick={false}
      closeOnEscape={!busy}
    >
      <div className="space-y-4">
        <div className="flex items-center gap-3 rounded-2xl border border-primary-400/25 bg-primary-400/[0.07] p-4">
          <span className="font-display text-2xl font-bold text-primary-300">{version}</span>
          <p className="text-[13px] leading-relaxed text-slate-300">
            Znaleziono nową wersję launchera. Pobieranie z GitHub Releases, instalacja przy restarcie aplikacji.
          </p>
        </div>

        {notes && (
          <div className="max-h-40 overflow-y-auto rounded-xl border border-white/[0.08] bg-black/40 p-3 font-mono text-xs leading-relaxed text-slate-400 whitespace-pre-wrap">
            {notes}
          </div>
        )}

        {busy && (
          <div>
            <div className="mb-1.5 flex justify-between text-xs font-semibold text-slate-400">
              <span>{status === 'installing' ? 'Instalowanie…' : 'Pobieranie…'}</span>
              <span className="font-mono text-primary-300">{progress}%</span>
            </div>
            <div className="h-2.5 overflow-hidden rounded-full bg-white/[0.08]">
              <div
                className="h-full rounded-full bg-gradient-to-r from-primary-500 via-primary-400 to-violet-500 shadow-[0_0_16px_rgba(46,155,255,0.5)] transition-all"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {!busy && (
          <div className="flex justify-end gap-3 pt-1">
            <Button variant="secondary" onClick={dismiss} disabled={busy}>
              <X className="h-4 w-4 mr-2" /> Nie
            </Button>
            <Button onClick={() => void installUpdate()} disabled={busy}>
              <Download className="h-4 w-4 mr-2" /> Tak, instaluj
            </Button>
          </div>
        )}
      </div>
    </Modal>
  )
}
