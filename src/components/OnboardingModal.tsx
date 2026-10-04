import { useState } from 'react'
import { UserPlus, MonitorSmartphone } from 'lucide-react'
import { Modal } from './ui/Modal'
import { Button } from './ui/Button'
import { Input } from './ui/Input'
import { useAuthStore } from '../store/authStore'
import { LogoTile } from './Logo'

interface OnboardingModalProps {
  open: boolean
  onClose: () => void
}

export function OnboardingModal({ open, onClose }: OnboardingModalProps) {
  const { loginOffline, loginMicrosoft } = useAuthStore()
  const [username, setUsername] = useState('')
  const [busy, setBusy] = useState<'offline' | 'microsoft' | null>(null)
  const [error, setError] = useState<string | null>(null)

  const doOffline = async () => {
    if (!username.trim()) return
    setBusy('offline')
    setError(null)
    try {
      await loginOffline(username.trim())
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(null)
    }
  }

  const doMicrosoft = async () => {
    setBusy('microsoft')
    setError(null)
    try {
      await loginMicrosoft()
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(null)
    }
  }

  return (
    <Modal isOpen={open} onClose={onClose} title="Welcome to DontCam Client" description="Add an account to start playing.">
      <div className="space-y-5">
        <div className="flex items-center gap-3">
          <LogoTile className="h-12 w-12 !rounded-2xl" markClassName="h-full w-full" />
          <p className="text-sm text-slate-400">
            Offline accounts work instantly. Microsoft accounts open a browser login with a local callback.
          </p>
        </div>

        <div className="space-y-3">
          <Input
            label="Offline username"
            placeholder="Steve"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') void doOffline()
            }}
            autoFocus
          />
          <Button className="w-full" onClick={() => void doOffline()} disabled={!username.trim()} loading={busy === 'offline'}>
            <UserPlus className="h-4 w-4 mr-2" /> Continue offline
          </Button>
        </div>

        <div className="flex items-center gap-3 text-xs font-bold uppercase tracking-[0.2em] text-slate-600">
          <span className="h-px flex-1 bg-white/10" /> or <span className="h-px flex-1 bg-white/10" />
        </div>

        <Button variant="secondary" className="w-full" onClick={() => void doMicrosoft()} loading={busy === 'microsoft'}>
          <MonitorSmartphone className="h-4 w-4 mr-2" /> Login with Microsoft
        </Button>

        {error && <div className="alert alert-red">{error}</div>}
      </div>
    </Modal>
  )
}
