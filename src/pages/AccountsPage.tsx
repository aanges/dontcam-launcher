import { useEffect, useState } from 'react'
import { Plus, RefreshCw, User, ShieldCheck, LogOut, Info } from 'lucide-react'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Input } from '../components/ui/Input'
import { Modal, ConfirmDialog } from '../components/ui/Modal'
import { useAuthStore } from '../store/authStore'
import { formatDateTime } from '../utils/helpers'

export function AccountsPage() {
  const { accounts, currentAccount, isLoading, error, loadAccounts, loginOffline, loginMicrosoft, logout, setCurrentAccount, refreshAccount } = useAuthStore()
  const [showOfflineModal, setShowOfflineModal] = useState(false)
  const [username, setUsername] = useState('')
  const [deleteId, setDeleteId] = useState<string | null>(null)
  const [busyId, setBusyId] = useState<string | null>(null)

  useEffect(() => {
    loadAccounts()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const handleOfflineLogin = async () => {
    if (!username.trim()) return
    try {
      await loginOffline(username.trim())
      setShowOfflineModal(false)
      setUsername('')
    } catch {
      // error is in store
    }
  }

  const handleMicrosoftLogin = async () => {
    try {
      await loginMicrosoft()
    } catch {
      // error shown below; browser window was opened for OAuth
    }
  }

  const handleRefresh = async (id: string) => {
    setBusyId(id)
    try {
      await refreshAccount(id)
    } finally {
      setBusyId(null)
    }
  }

  return (
    <div className="animate-fade-in space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="eyebrow">Identity</p>
          <h1 className="h1 mt-1">Accounts</h1>
          <p className="sub">Manage Microsoft and offline accounts</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" onClick={() => setShowOfflineModal(true)}>
            <Plus className="h-4 w-4 mr-2" /> Offline
          </Button>
          <Button onClick={() => void handleMicrosoftLogin()} loading={isLoading}>
            <User className="h-4 w-4 mr-2" /> Microsoft login
          </Button>
        </div>
      </div>

      {error && (
        <div className="alert alert-red">{error}</div>
      )}

      {accounts.length === 0 ? (
        <Card padding="lg" className="text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-3xl border border-white/10 bg-white/[0.04]">
            <User className="h-8 w-8 text-slate-500" />
          </div>
          <h3 className="font-display text-xl font-bold text-white">No accounts yet</h3>
          <p className="mx-auto mt-1 max-w-md text-sm text-slate-400">
            Add an offline account for quick testing, or sign in with Microsoft to play on premium servers.
          </p>
          <div className="mt-6 flex justify-center gap-3">
            <Button variant="secondary" onClick={() => setShowOfflineModal(true)}>Add offline</Button>
            <Button onClick={() => void handleMicrosoftLogin()}>Sign in with Microsoft</Button>
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {accounts.map((account) => {
            const isCurrent = currentAccount?.id === account.id
            const microsoft = account.account_type === 'microsoft'
            return (
              <Card
                key={account.id}
                padding="md"
                className={isCurrent ? '!border-primary-400/50 shadow-[0_0_32px_rgba(46,155,255,0.15)]' : 'card-lift'}
              >
                <div className="flex items-start gap-3">
                  <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-primary-400 to-violet-600 font-display text-lg font-bold text-white">
                    {account.username.charAt(0).toUpperCase()}
                  </div>
                  <div className="min-w-0 flex-1">
                    <h3 className="truncate font-display font-bold text-white">{account.username}</h3>
                    <p className="mt-1 flex flex-wrap items-center gap-1.5 text-xs capitalize text-slate-400">
                      <span className={microsoft ? 'pill-volt' : 'pill'}>
                        {microsoft ? <ShieldCheck className="h-3 w-3" /> : <User className="h-3 w-3" />}
                        {account.account_type}
                      </span>
                      {isCurrent && <span className="pill-volt">active</span>}
                    </p>
                    <p className="mt-1.5 truncate font-mono text-[11px] text-slate-500">{account.uuid}</p>
                    <p className="text-[11px] text-slate-500">Last used: {formatDateTime(account.last_used)}</p>
                  </div>
                </div>
                <div className="mt-4 flex items-center gap-2 border-t border-white/[0.07] pt-3">
                  {!isCurrent && (
                    <Button size="sm" variant="secondary" className="flex-1" onClick={() => void setCurrentAccount(account)}>
                      Use
                    </Button>
                  )}
                  {microsoft && (
                    <Button size="sm" variant="ghost" className="!h-8 !w-8 !p-0" disabled={busyId === account.id} loading={busyId === account.id} onClick={() => void handleRefresh(account.id)} aria-label="Refresh token">
                      <RefreshCw className="h-4 w-4" />
                    </Button>
                  )}
                  <Button size="sm" variant="ghost" className="!h-8 !w-8 !p-0" onClick={() => setDeleteId(account.id)} aria-label="Remove account">
                    <LogOut className="h-4 w-4 text-red-400" />
                  </Button>
                </div>
              </Card>
            )
          })}
        </div>
      )}

      <Modal isOpen={showOfflineModal} onClose={() => setShowOfflineModal(false)} title="Add offline account">
        <div className="space-y-4">
          <Input
            label="Username"
            placeholder="Steve"
            value={username}
            maxLength={16}
            onChange={(e) => setUsername(e.target.value)}
            helperText="Letters, numbers and underscore, max 16 chars. Works on offline/cracked servers."
            autoFocus
            onKeyDown={(e) => {
              if (e.key === 'Enter') void handleOfflineLogin()
            }}
          />
          <div className="flex justify-end gap-3">
            <Button variant="secondary" onClick={() => setShowOfflineModal(false)}>Cancel</Button>
            <Button onClick={() => void handleOfflineLogin()} disabled={!username.trim()} loading={isLoading}>Add account</Button>
          </div>
        </div>
      </Modal>

      <ConfirmDialog
        isOpen={!!deleteId}
        onClose={() => setDeleteId(null)}
        onConfirm={() => {
          if (deleteId) void logout(deleteId)
          setDeleteId(null)
        }}
        title="Remove account"
        message="Are you sure you want to remove this account from the launcher?"
        confirmText="Remove"
        variant="danger"
      />

      <Card padding="md" className="flex gap-3">
        <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-white/10 bg-white/[0.05]">
          <Info className="h-4 w-4 text-primary-300" />
        </span>
        <div>
          <h3 className="font-display font-bold text-white">How Microsoft login works</h3>
          <p className="mt-1 text-sm leading-relaxed text-slate-400">
            Clicking Microsoft login opens an in-app sign-in window. Sign in, approve access, and the window closes itself when done.
            Your tokens are stored locally in the app data folder and refreshed automatically.
          </p>
        </div>
      </Card>
    </div>
  )
}
