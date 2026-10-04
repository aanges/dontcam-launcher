import { useState } from 'react'
import { ShieldCheck } from 'lucide-react'
import { Modal } from './ui/Modal'
import { Button } from './ui/Button'
import { Input } from './ui/Input'
import { LogoTile } from './Logo'
import { useAuthStore } from '../store/authStore'

/** First-run popup: no account exists yet. Microsoft first (green), offline as plain text below. */
export function OnboardingModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { isLoading, error, loginMicrosoft, loginOffline } = useAuthStore()
  const [showOffline, setShowOffline] = useState(false)
  const [username, setUsername] = useState('')

  const handleMicrosoft = async () => {
    try {
      await loginMicrosoft()
    } catch {
      // error is in the store and shown below
    }
  }

  const handleOffline = async () => {
    if (!username.trim()) return
    try {
      await loginOffline(username.trim())
    } catch {
      // error is in the store and shown below
    }
  }

  return (
    <Modal isOpen={open} onClose={onClose} size="md" showCloseButton>
      <div className="flex flex-col items-center gap-5 py-2 text-center">
        <div className="relative">
          <div className="absolute -inset-3 rounded-[28px] bg-primary-400/20 blur-xl" />
          <LogoTile className="relative h-20 w-20 !rounded-[24px]" markClassName="h-full w-full" />
        </div>
        <div>
          <p className="eyebrow">First run</p>
          <h2 className="mt-1 font-display text-2xl font-bold tracking-tight text-white">
            Welcome to DontCam Client
          </h2>
          <p className="mt-1 text-sm text-slate-400">
            Add an account to start playing Minecraft.
          </p>
        </div>

        <div className="w-full flex flex-col gap-3">
          <button
            onClick={() => void handleMicrosoft()}
            disabled={isLoading}
            className="w-full flex items-center justify-center gap-2.5 px-4 py-3.5 rounded-2xl bg-green-600 hover:bg-green-500 active:scale-[0.99] text-white font-display font-bold tracking-wide shadow-[0_0_28px_rgba(22,163,74,0.45)] transition-all disabled:opacity-60"
          >
            {isLoading ? (
              <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
            ) : (
              <ShieldCheck className="h-5 w-5" />
            )}
            Continue with Microsoft
          </button>

          {!showOffline ? (
            <button
              onClick={() => setShowOffline(true)}
              className="text-sm font-semibold text-slate-400 underline decoration-white/20 underline-offset-4 transition-colors hover:text-primary-300"
            >
              or continue offline
            </button>
          ) : (
            <div className="flex flex-col gap-2 pt-1">
              <Input
                placeholder="Username (letters, numbers, _)"
                value={username}
                maxLength={16}
                onChange={(e) => setUsername(e.target.value)}
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') void handleOffline()
                }}
              />
              <Button onClick={() => void handleOffline()} disabled={!username.trim()} loading={isLoading}>
                Add offline account
              </Button>
            </div>
          )}
        </div>

        {error && (
          <p className="break-words text-sm text-red-400">{error}</p>
        )}
        <p className="text-xs leading-relaxed text-slate-500">
          Offline works on non-premium servers. Microsoft is required for premium servers and the owner crown.
        </p>
      </div>
    </Modal>
  )
}
