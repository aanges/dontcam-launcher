import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Play, Plus, Gamepad2, Users, Package, Clock, Download, Trash2, Square, ChevronRight } from 'lucide-react'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Modal, ConfirmDialog } from '../components/ui/Modal'
import { Select } from '../components/ui/Select'
import { Input } from '../components/ui/Input'
import { ConsolePanel } from '../components/ConsolePanel'
import { LogoMark } from '../components/Logo'
import { useAuthStore } from '../store/authStore'
import { useProfileStore } from '../store/profileStore'
import { useVersionStore } from '../store/versionStore'
import { useLaunchStore } from '../store/launchStore'
import { useSettingsStore } from '../store/settingsStore'
import { javaApi, utilsApi } from '../tauri/api'
import { getMemoryArgs, parseJvmArgs, formatDate } from '../utils/helpers'
import type { Profile } from '../types'

const FEATURED_VERSIONS = ['1.21.1', '1.20.1', '1.12.2', '1.8.9']

export function HomePage() {
  const { currentAccount, accounts } = useAuthStore()
  const { profiles, selectedProfile, selectProfile, loadProfiles, createProfile } = useProfileStore()
  const { installedVersions, manifest, loadInstalledVersions, loadManifest, installModded, uninstallVersion, isInstalling } = useVersionStore()
  const { launchStatus, launchGame, killGame } = useLaunchStore()
  const { settings } = useSettingsStore()

  const [showCreateProfile, setShowCreateProfile] = useState(false)
  const [newProfileName, setNewProfileName] = useState('')
  const [selectedVersion, setSelectedVersion] = useState('')
  const [showUninstallConfirm, setShowUninstallConfirm] = useState<string | null>(null)
  const [launchError, setLaunchError] = useState<string | null>(null)
  const [launchingId, setLaunchingId] = useState<string | null>(null)
  const [installError, setInstallError] = useState<string | null>(null)
  const [installWarning, setInstallWarning] = useState<string | null>(null)

  useEffect(() => {
    loadProfiles()
    loadInstalledVersions()
    loadManifest()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const isRunning = launchStatus === 'running' || launchStatus === 'launching'
  const installedIds = new Set(installedVersions.map((v) => v.id))

  const quickProfile = selectedProfile ?? profiles[0] ?? null
  const quickVersionId = quickProfile?.version_id ?? installedVersions[0]?.id ?? manifest?.latest.release ?? ''
  const quickInstalled = installedIds.has(quickVersionId)
  const busy = launchingId !== null || isInstalling !== null

  const doLaunch = async (profile: Profile | null, versionId: string) => {
    if (!currentAccount) {
      setLaunchError('Add an account first (Accounts page).')
      return
    }
    if (!versionId) return
    setLaunchError(null)
    setLaunchingId(versionId)
    try {
      const [java, gameDir] = await Promise.all([
        javaApi.resolveForVersion(versionId).catch(() => null),
        utilsApi.getGameDir(profile?.id).catch(() => ''),
      ])
      const memArgs = getMemoryArgs(
        settings.java.memory_allocation.min,
        settings.java.memory_allocation.max,
        settings.java.memory_allocation.unit
      )
      await launchGame({
        profile_id: profile?.id ?? '',
        version_id: versionId,
        account: currentAccount,
        java_path: (java?.path as unknown as string) ?? 'java',
        jvm_args: [...memArgs, ...parseJvmArgs(settings.java.jvm_args), ...parseJvmArgs(profile?.java_args ?? '')],
        game_args: [...(profile?.game_args ?? [])],
        game_dir: gameDir,
        resolution: profile?.resolution ?? settings.game.custom_resolution ?? undefined,
      })
    } catch (error) {
      setLaunchError(error instanceof Error ? error.message : String(error))
    } finally {
      setLaunchingId(null)
    }
  }

  const handleQuickPlay = async () => {
    if (!quickVersionId) return
    if (quickProfile) selectProfile(quickProfile)
    if (!quickInstalled) {
      // Install & Play: vanilla + loader + mod, then launch. Watch the console below.
      setInstallError(null)
      setInstallWarning(null)
      try {
        const res = await installModded(quickVersionId, false, quickProfile?.id)
        if (res.loader_error) {
          setInstallWarning(`Vanilla ready, but loader failed: ${res.loader_error}`)
        }
      } catch (error) {
        setInstallError(error instanceof Error ? error.message : String(error))
        return
      }
    }
    await doLaunch(quickProfile, quickVersionId)
  }

  const ensureProfileForVersion = async (versionId: string): Promise<Profile | null> => {
    const match = profiles.find((p) => p.version_id === versionId) ?? null
    if (match) {
      if (selectedProfile?.id !== match.id) selectProfile(match)
      return match
    }
    try {
      const created = await createProfile(`Minecraft ${versionId}`, versionId, currentAccount?.id)
      selectProfile(created)
      return created
    } catch {
      return selectedProfile ?? profiles[0] ?? null
    }
  }

  const handleInstallFeatured = async (versionId: string) => {
    setInstallError(null)
    setInstallWarning(null)
    try {
      const profile = await ensureProfileForVersion(versionId)
      const res = await installModded(versionId, false, profile?.id)
      if (res.loader_error) {
        setInstallWarning(`Vanilla ready, but loader failed: ${res.loader_error}`)
      }
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error))
    }
  }

  const handlePlayFeatured = async (versionId: string) => {
    const profile = await ensureProfileForVersion(versionId)
    await doLaunch(profile, versionId)
  }

  const confirmUninstall = async () => {
    if (showUninstallConfirm) {
      await uninstallVersion(showUninstallConfirm)
      setShowUninstallConfirm(null)
    }
  }

  const handleCreateProfile = async () => {
    if (!newProfileName.trim() || !selectedVersion) return
    const profile = await createProfile(newProfileName.trim(), selectedVersion, currentAccount?.id)
    selectProfile(profile)
    setShowCreateProfile(false)
    setNewProfileName('')
    setSelectedVersion('')
  }

  return (
    <div className="animate-fade-in space-y-6">
      {/* Hero */}
      <section className="panel relative overflow-hidden !rounded-[28px] p-6 sm:p-10">
        <div className="bg-grid absolute inset-0 opacity-70" />
        <div className="orb left-[8%] top-[-60px] h-64 w-64 bg-primary-400/[0.13]" />
        <div className="orb bottom-[-90px] right-[12%] h-72 w-72 bg-violet-600/[0.16]" />
        <LogoMark className="pointer-events-none absolute -bottom-16 -right-8 h-72 w-72 opacity-[0.07]" />
        <div className="relative flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
          <div className="max-w-2xl">
            <div className="flex flex-wrap items-center gap-2.5">
              <span className="h-2 w-2 animate-pulse rounded-full bg-primary-400 shadow-[0_0_12px_rgba(46,155,255,0.9)]" />
              <span className="eyebrow">DontCam Client</span>
              {currentAccount && (
                <span className="pill !py-1">
                  {currentAccount.username} • {currentAccount.account_type}
                </span>
              )}
              {isRunning && <span className="pill-volt !py-1">Running</span>}
            </div>
            <h1 className="mt-4 font-display text-4xl font-bold leading-[1.02] tracking-tight text-white sm:text-6xl">
              Ready to mine{currentAccount ? <>, <span className="text-gradient">{currentAccount.username}</span></> : ''}?
            </h1>
            <p className="sub mt-3 max-w-xl !text-[15px]">
              {quickProfile ? (
                <>Profile <strong className="text-white">{quickProfile.name}</strong> • version <strong className="text-white">{quickVersionId || '—'}</strong> • {quickProfile.mod_loader === 'none' ? 'vanilla' : quickProfile.mod_loader} {quickInstalled ? 'is ready to launch' : 'will be downloaded first'}</>
              ) : (
                <>Pick a version below or create a profile to get started.</>
              )}
            </p>
            <div className="mt-6 flex flex-wrap items-center gap-4">
              {isRunning ? (
                <Button size="lg" variant="destructive" onClick={() => void killGame()} className="!rounded-2xl">
                  <Square className="h-5 w-5" />
                  Stop game
                </Button>
              ) : quickInstalled ? (
                <Button
                  size="lg"
                  onClick={() => void handleQuickPlay()}
                  disabled={!currentAccount || busy || !quickVersionId}
                  loading={launchingId !== null}
                  className="!rounded-2xl !px-8"
                >
                  <Play className="h-5 w-5 fill-current" />
                  Play {quickVersionId}
                </Button>
              ) : (
                <Button
                  size="lg"
                  onClick={() => void handleQuickPlay()}
                  disabled={!currentAccount || busy || !quickVersionId}
                  loading={isInstalling !== null}
                  className="!rounded-2xl !px-8"
                >
                  <Download className="h-5 w-5" />
                  Install & Play {quickVersionId}
                </Button>
              )}
              <Link
                to="/versions"
                className="group inline-flex items-center gap-1 font-display text-sm font-bold text-slate-400 transition-colors hover:text-primary-300"
              >
                Choose a different version
                <ChevronRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
              </Link>
            </div>
          </div>
          {quickProfile && (
            <div className="panel-deep w-full max-w-xs shrink-0 p-5">
              <p className="eyebrow !text-[10px]">Active profile</p>
              <div className="mt-3 flex items-center gap-3">
                <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-primary-400 to-violet-600 font-display text-lg font-bold text-white shadow-[0_0_24px_rgba(46,155,255,0.35)]">
                  {quickProfile.name.charAt(0).toUpperCase()}
                </div>
                <div className="min-w-0">
                  <p className="truncate font-display font-bold text-white">{quickProfile.name}</p>
                  <p className="truncate font-mono text-xs text-slate-400">{quickVersionId}</p>
                </div>
              </div>
              <div className="mt-3 flex flex-wrap gap-1.5">
                <span className="pill-volt">{quickProfile.mod_loader === 'none' ? 'vanilla' : quickProfile.mod_loader}</span>
                <span className={quickInstalled ? 'pill-volt' : 'pill'}>{quickInstalled ? 'ready' : 'not installed'}</span>
              </div>
            </div>
          )}
        </div>
      </section>

      {launchError && (
        <div className="alert alert-red">{launchError}</div>
      )}
      {installError && (
        <div className="alert alert-red">Install failed: {installError}</div>
      )}
      {installWarning && (
        <div className="alert alert-yellow">{installWarning}</div>
      )}
      {!currentAccount && (
        <div className="alert alert-yellow">
          No account selected. Go to <Link to="/accounts" className="font-bold underline">Accounts</Link> and add an offline or Microsoft account.
        </div>
      )}

      <ConsolePanel />

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard title="Accounts" value={accounts.length} icon={Users} />
        <StatCard title="Profiles" value={profiles.length} icon={Gamepad2} />
        <StatCard title="Versions installed" value={installedVersions.length} icon={Package} />
        <StatCard title="Status" value={typeof launchStatus === 'string' ? launchStatus : 'error'} icon={Clock} />
      </div>

      <section>
        <div className="mb-4 flex items-center justify-between">
          <div>
            <p className="eyebrow">Quick start</p>
            <h2 className="mt-1 font-display text-xl font-bold tracking-tight text-white">Featured versions</h2>
          </div>
          <Link to="/versions" className="font-display text-sm font-bold text-primary-300 hover:text-primary-200 hover:underline">
            Browse all
          </Link>
        </div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {FEATURED_VERSIONS.map((versionId, i) => {
            const installed = installedIds.has(versionId)
            const installing = isInstalling === versionId
            const art = [
              'from-sky-500 via-primary-500 to-emerald-700',
              'from-violet-600 via-indigo-600 to-dark-900',
              'from-amber-500 via-orange-600 to-rose-700',
              'from-emerald-500 via-teal-600 to-primary-700',
            ][i % 4]
            return (
              <Card key={versionId} padding="none" className="card-lift overflow-hidden">
                <div className={`relative h-28 bg-gradient-to-br ${art} flex items-end p-4`}>
                  <div className="bg-grid absolute inset-0 opacity-60" />
                  <span className="relative font-display text-[32px] font-bold leading-none tracking-tight text-white/95 drop-shadow">
                    {versionId}
                  </span>
                  {installed && (
                    <span className="absolute right-3 top-3 rounded-full border border-primary-300/40 bg-black/50 px-2 py-0.5 text-[11px] font-bold text-primary-200">
                      INSTALLED
                    </span>
                  )}
                </div>
                <div className="p-4">
                  <p className="mb-4 text-[13px] text-slate-400">
                    {versionId === '1.8.9' ? 'Java 8 • PvP classic' : versionId === '1.12.2' ? 'Java 8 • modded classic' : versionId === '1.20.1' ? 'Java 17 • mods' : 'Java 21 • latest'}
                  </p>
                  <div className="flex gap-2">
                    {installed ? (
                      <>
                        <Button size="sm" className="flex-1" disabled={isRunning || !currentAccount} onClick={() => void handlePlayFeatured(versionId)}>
                          <Play className="h-4 w-4" /> Play
                        </Button>
                        <Button size="sm" variant="ghost" onClick={() => setShowUninstallConfirm(versionId)} aria-label={`Uninstall ${versionId}`}>
                          <Trash2 className="h-4 w-4 text-red-400" />
                        </Button>
                      </>
                    ) : (
                      <Button size="sm" variant="secondary" className="flex-1" disabled={installing || busy} loading={installing} onClick={() => void handleInstallFeatured(versionId)}>
                        <Download className="h-4 w-4" /> Install
                      </Button>
                    )}
                  </div>
                </div>
              </Card>
            )
          })}
        </div>
      </section>

      {profiles.length > 0 && (
        <section>
          <div className="mb-4 flex items-center justify-between">
            <div>
              <p className="eyebrow">Instances</p>
              <h2 className="mt-1 font-display text-xl font-bold tracking-tight text-white">Profiles</h2>
            </div>
            <Link to="/profiles" className="font-display text-sm font-bold text-primary-300 hover:text-primary-200 hover:underline">
              View all
            </Link>
          </div>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {profiles.slice(0, 4).map((profile) => (
              <Card
                key={profile.id}
                padding="md"
                className="card-lift cursor-pointer"
                onClick={() => void doLaunch(profile, profile.version_id)}
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-primary-400 to-violet-600 font-display text-lg font-bold text-white shadow-[0_0_24px_rgba(46,155,255,0.3)]">
                    {profile.name.charAt(0).toUpperCase()}
                  </div>
                  <div className="min-w-0 flex-1">
                    <h3 className="truncate font-display font-bold text-white">{profile.name}</h3>
                    <p className="truncate font-mono text-xs text-slate-400">{profile.version_id}</p>
                    <p className="mt-0.5 text-xs text-slate-500">
                      {profile.last_played ? `Last played: ${formatDate(profile.last_played)}` : 'Never played'}
                    </p>
                  </div>
                  <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-white/10 bg-white/[0.05] text-primary-300">
                    <Play className="h-4 w-4 fill-current" />
                  </span>
                </div>
              </Card>
            ))}
          </div>
        </section>
      )}

      <section>
        <Card padding="md" className="flex flex-col gap-4 !border-primary-400/20 !bg-gradient-to-r !from-primary-400/[0.08] !to-transparent sm:flex-row sm:items-center">
          <div className="flex-1">
            <h3 className="font-display font-bold text-white">Create a new profile</h3>
            <p className="mt-0.5 text-sm text-slate-400">Separate versions, mods, resource packs and settings per profile.</p>
          </div>
          <Button onClick={() => setShowCreateProfile(true)}>
            <Plus className="h-4 w-4 mr-2" /> Create Profile
          </Button>
        </Card>
      </section>

      <Modal isOpen={showCreateProfile} onClose={() => setShowCreateProfile(false)} title="Create New Profile">
        <div className="space-y-4">
          <Input
            label="Profile name"
            placeholder="My Awesome Profile"
            value={newProfileName}
            onChange={(e) => setNewProfileName(e.target.value)}
            autoFocus
          />
          <Select
            label="Version"
            placeholder="Select version"
            value={selectedVersion}
            onChange={(e) => setSelectedVersion(e.target.value)}
            options={[...installedVersions.map((v) => ({ value: v.id, label: `${v.id}` })), ...FEATURED_VERSIONS.filter((v) => !installedIds.has(v)).map((v) => ({ value: v, label: `${v} (will install)` }))]}
          />
          <div className="flex justify-end gap-3 pt-2">
            <Button variant="secondary" onClick={() => setShowCreateProfile(false)}>Cancel</Button>
            <Button onClick={() => void handleCreateProfile()} disabled={!newProfileName.trim() || !selectedVersion}>Create</Button>
          </div>
        </div>
      </Modal>

      <ConfirmDialog
        isOpen={!!showUninstallConfirm}
        onClose={() => setShowUninstallConfirm(null)}
        onConfirm={() => void confirmUninstall()}
        title="Uninstall Version"
        message={`Are you sure you want to uninstall ${showUninstallConfirm}? This action cannot be undone.`}
        confirmText="Uninstall"
        variant="danger"
      />
    </div>
  )
}

function StatCard({ title, value, icon: Icon }: { title: string; value: string | number; icon: React.ComponentType<{ className?: string }> }) {
  return (
    <Card padding="md" className="card-lift">
      <div className="flex items-center justify-between">
        <div>
          <p className="eyebrow !text-[10px]">{title}</p>
          <p className="mt-1.5 font-display text-[26px] font-bold capitalize leading-none text-white">{value}</p>
        </div>
        <div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-primary-400/25 bg-primary-400/10">
          <Icon className="h-6 w-6 text-primary-300" />
        </div>
      </div>
    </Card>
  )
}
