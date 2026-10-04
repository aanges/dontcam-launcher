import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Play, Download, Square, ChevronRight } from 'lucide-react'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { ConsolePanel } from '../components/ConsolePanel'
import { LogoMark } from '../components/Logo'
import { useAuthStore } from '../store/authStore'
import { useProfileStore } from '../store/profileStore'
import { useVersionStore } from '../store/versionStore'
import { useLaunchStore } from '../store/launchStore'
import { useSettingsStore } from '../store/settingsStore'
import { useDontcamStore } from '../store/dontcamStore'
import { javaApi, utilsApi } from '../tauri/api'
import { getMemoryArgs, parseJvmArgs, formatDate } from '../utils/helpers'
import type { Profile } from '../types'

export function HomePage() {
  const { currentAccount } = useAuthStore()
  const { profiles, selectedProfile, selectProfile, loadProfiles } = useProfileStore()
  const { installedVersions, manifest, loadInstalledVersions, loadManifest, installModded, isInstalling } = useVersionStore()
  const { launchStatus, launchGame, killGame } = useLaunchStore()
  const { settings } = useSettingsStore()
  const checkDontcamUpdate = useDontcamStore((s) => s.checkDontcamUpdate)
  const dontcamChecking = useDontcamStore((s) => s.checking)

  const [launchError, setLaunchError] = useState<string | null>(null)
  const [launchingId, setLaunchingId] = useState<string | null>(null)
  const [installError, setInstallError] = useState<string | null>(null)
  const [installWarning, setInstallWarning] = useState<string | null>(null)
  const [dontcamOffline, setDontcamOffline] = useState<string | null>(null)

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
  const busy = launchingId !== null || isInstalling !== null || dontcamChecking !== null

  const doLaunch = async (profile: Profile | null, versionId: string) => {
    if (!currentAccount) {
      setLaunchError('Add an account first (Accounts page).')
      return
    }
    if (!versionId) return
    setLaunchError(null)
    setDontcamOffline(null)
    setLaunchingId(versionId)
    try {
      // DontCam freshness check before EVERY start (launchGame re-checks
      // too — the backend dedups back-to-back checks, so this is cheap).
      // Offline never blocks the game: we play on the local jar.
      const res = await checkDontcamUpdate(profile?.version_id ?? versionId, profile?.id)
      if (res.offline) setDontcamOffline(res.message)
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
                <>Pick a version in Versions or create a profile in Profiles to get started.</>
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
                  loading={launchingId !== null || dontcamChecking !== null}
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

      {launchError && <div className="alert alert-red">{launchError}</div>}
      {installError && <div className="alert alert-red">Install failed: {installError}</div>}
      {installWarning && <div className="alert alert-yellow">{installWarning}</div>}
      {dontcamOffline && <div className="alert alert-yellow">{dontcamOffline}</div>}
      {!currentAccount && (
        <div className="alert alert-yellow">
          No account selected. Go to <Link to="/accounts" className="font-bold underline">Accounts</Link> and add an offline or Microsoft account.
        </div>
      )}

      <ConsolePanel />

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
    </div>
  )
}
