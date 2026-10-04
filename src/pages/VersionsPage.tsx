import { useEffect, useMemo, useState } from 'react'
import { Trash2, RefreshCw, Filter, Search, Box, FolderOpen, ChevronDown, ChevronRight, Check } from 'lucide-react'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Select, Checkbox } from '../components/ui/Select'
import { useVersionStore } from '../store/versionStore'
import { useSettingsStore } from '../store/settingsStore'
import { useProfileStore } from '../store/profileStore'
import { formatDate } from '../utils/helpers'
import { modLoaderApi, utilsApi } from '../tauri/api'
import type { VersionInfo, Profile } from '../types'
import type { Settings } from '../types'

function groupKey(v: VersionInfo): string {
  const m = v.id.match(/^(\d+\.\d+)/)
  return m ? m[1] : 'other'
}

function compareGroups(a: string, b: string): number {
  if (a === 'other') return 1
  if (b === 'other') return -1
  const pa = a.split('.').map(Number)
  const pb = b.split('.').map(Number)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const d = (pb[i] ?? 0) - (pa[i] ?? 0)
    if (d !== 0) return d
  }
  return 0
}

/** Loader badge for installed modded profiles (e.g. fabric-loader-0.15.11-1.20.1). */
function loaderBadge(id: string): string | null {
  const lower = id.toLowerCase()
  if (lower.includes('neoforge')) return 'NeoForge'
  if (lower.includes('forge')) return 'Forge'
  if (lower.includes('fabric')) return 'Fabric'
  if (lower.includes('quilt')) return 'Quilt'
  return null
}

export function VersionsPage() {
  const {
    manifest,
    installedVersions,
    filteredVersions,
    isLoading,
    loadManifest,
    loadInstalledVersions,
    uninstallVersion,
    filterVersions,
  } = useVersionStore()
  const { settings, updateSettings } = useSettingsStore()
  const { profiles, selectedProfile, selectProfile, updateProfile, createProfile } = useProfileStore()

  const [searchQuery, setSearchQuery] = useState('')
  const [showFilters, setShowFilters] = useState(false)
  const [baseDir, setBaseDir] = useState('')
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set())
  const [notice, setNotice] = useState<string | null>(null)
  const [pickingId, setPickingId] = useState<string | null>(null)

  useEffect(() => {
    loadManifest()
    loadInstalledVersions()
    utilsApi.getAppDataDir().then(setBaseDir).catch(() => undefined)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    filterVersions(
      settings.ui.show_snapshots,
      settings.ui.show_old_versions,
      settings.ui.show_alpha_beta,
      settings.ui.sort_versions_by
    )
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.ui.show_snapshots, settings.ui.show_old_versions, settings.ui.show_alpha_beta, settings.ui.sort_versions_by, manifest])

  const installedIds = useMemo(() => new Set(installedVersions.map((v) => v.id)), [installedVersions])

  const searched = useMemo(
    () => filteredVersions.filter((v) => v.id.toLowerCase().includes(searchQuery.toLowerCase())),
    [filteredVersions, searchQuery]
  )

  const groups = useMemo(() => {
    const map = new Map<string, VersionInfo[]>()
    for (const v of searched) {
      const key = groupKey(v)
      if (!map.has(key)) map.set(key, [])
      map.get(key)!.push(v)
    }
    return [...map.entries()].sort(([a], [b]) => compareGroups(a, b))
  }, [searched])

  const toggleGroup = (key: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  }

  /** Pick a version for the quick-play profile. Nothing is downloaded here —
   *  installation happens after pressing Install & Play on Home. */
  const handlePick = async (version: VersionInfo) => {
    setPickingId(version.id)
    setNotice(null)
    try {
      let profile = selectedProfile ?? profiles[0] ?? null
      // Auto-select the right loader for this line (Fabric 1.16+, Forge on legacy).
      let loader: Profile['mod_loader'] = profile?.mod_loader ?? 'none'
      try {
        const resolved = await modLoaderApi.resolveLoader(version.id)
        loader = resolved.mod_loader === 'liteloader' ? 'none' : resolved.mod_loader
      } catch {
        // offline / unknown line — keep current loader
      }
      if (!profile) {
        profile = await createProfile(`Minecraft ${version.id}`, version.id)
      }
      if (profile.version_id !== version.id || !profile.dontcam_mod || profile.mod_loader !== loader) {
        profile = { ...profile, version_id: version.id, dontcam_mod: true, mod_loader: loader }
        await updateProfile(profile)
      }
      selectProfile(profile)
      setNotice(
        installedIds.has(version.id)
          ? `Picked ${version.id} for profile "${profile.name}" — ready to play on Home.`
          : `Picked ${version.id} for profile "${profile.name}" — press Install & Play on Home to download it.`
      )
    } finally {
      setPickingId(null)
    }
  }

  const handleUninstall = async (versionId: string) => {
    if (confirm(`Are you sure you want to uninstall ${versionId}?`)) {
      await uninstallVersion(versionId)
    }
  }

  const versionPath = (id: string) => (baseDir ? `${baseDir}\\versions\\${id}` : '')
  const currentVersionId = (selectedProfile ?? profiles[0])?.version_id

  return (
    <div className="animate-fade-in space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="eyebrow">Library</p>
          <h1 className="h1 mt-1">Choose version</h1>
          <p className="sub">
            Pick a version line, then a version. Download happens after Install & Play on Home.{' '}
            {manifest && <span className="font-mono text-xs text-slate-500">latest: {manifest.latest.release}</span>}
          </p>
          {baseDir && (
            <p className="mt-1.5 break-all font-mono text-[11px] text-slate-500">
              {baseDir}
              <button
                className="ml-2 font-sans font-bold text-primary-300 underline-offset-2 hover:underline"
                onClick={() => void utilsApi.openFolder(baseDir)}
              >
                open
              </button>
            </p>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" onClick={() => setShowFilters(!showFilters)}>
            <Filter className="h-4 w-4 mr-2" />
            Filters
          </Button>
          <Button variant="secondary" onClick={() => { void loadManifest(); void loadInstalledVersions() }} loading={isLoading}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
        </div>
      </div>

      {notice && <div className="alert alert-green">{notice}</div>}

      {showFilters && (
        <Card padding="md" className="animate-slide-down">
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Select
              label="Sort inside groups"
              value={settings.ui.sort_versions_by}
              onChange={(e) => void updateSettings({ ui: { ...settings.ui, sort_versions_by: e.target.value as Settings['ui']['sort_versions_by'] } })}
              options={[
                { value: 'newest_first', label: 'Newest First' },
                { value: 'oldest_first', label: 'Oldest First' },
                { value: 'alphabetical', label: 'Alphabetical' },
                { value: 'release_type', label: 'Release Type' },
              ]}
            />
            <div className="flex flex-col justify-end gap-2">
              <Checkbox
                label="Show Snapshots"
                checked={settings.ui.show_snapshots}
                onChange={(e) => void updateSettings({ ui: { ...settings.ui, show_snapshots: e.target.checked } })}
              />
              <Checkbox
                label="Show Old Versions"
                checked={settings.ui.show_old_versions}
                onChange={(e) => void updateSettings({ ui: { ...settings.ui, show_old_versions: e.target.checked } })}
              />
              <Checkbox
                label="Show Alpha/Beta"
                checked={settings.ui.show_alpha_beta}
                onChange={(e) => void updateSettings({ ui: { ...settings.ui, show_alpha_beta: e.target.checked } })}
              />
            </div>
          </div>
        </Card>
      )}

      <div className="relative max-w-md">
        <Search className="absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
        <input
          type="text"
          placeholder="Search versions..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="h-11 w-full rounded-xl border border-white/10 bg-black/50 pl-11 pr-4 text-[15px] text-white placeholder:text-slate-600 transition-all focus:border-primary-400/60 focus:outline-none focus:ring-2 focus:ring-primary-400/20"
        />
      </div>

      {installedVersions.length > 0 && (
        <section>
          <div className="mb-3 flex items-baseline gap-3">
            <p className="eyebrow">On disk</p>
            <h2 className="font-display text-lg font-bold text-white">Installed ({installedVersions.length})</h2>
          </div>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {installedVersions.map((version) => (
              <Card key={version.id} padding="md" className="card-lift">
                <div className="mb-2 flex items-start justify-between gap-2">
                  <h3 className="flex min-w-0 items-center gap-2 font-display font-bold text-white">
                    <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-primary-400">
                      <Check className="h-3 w-3 text-black" strokeWidth={3} />
                    </span>
                    <span className="truncate font-mono text-sm">{version.id}</span>
                  </h3>
                  <div className="flex shrink-0 items-center gap-1">
                    <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={() => versionPath(version.id) && void utilsApi.openFolder(versionPath(version.id))} aria-label={`Open ${version.id} folder`}>
                      <FolderOpen className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={() => void handleUninstall(version.id)} aria-label={`Uninstall ${version.id}`}>
                      <Trash2 className="h-4 w-4 text-red-400" />
                    </Button>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className="pill capitalize">{version.version_type}</span>
                  {loaderBadge(version.id) && (
                    <span className="pill-volt">{loaderBadge(version.id)} • modded</span>
                  )}
                </div>
              </Card>
            ))}
          </div>
        </section>
      )}

      {groups.length === 0 ? (
        <Card padding="lg" className="text-center">
          <Box className="mx-auto mb-4 h-12 w-12 text-slate-600" />
          <h3 className="font-display text-lg font-bold text-white">No versions found</h3>
          <p className="mt-1 text-sm text-slate-400">Try adjusting your filters or search query</p>
        </Card>
      ) : (
        <div className="space-y-3">
          {groups.map(([key, items]) => {
            const isCollapsed = collapsed.has(key)
            const installedCount = items.filter((v) => installedIds.has(v.id)).length
            return (
              <Card key={key} padding="md">
                <button
                  className="flex w-full items-center justify-between gap-3"
                  onClick={() => toggleGroup(key)}
                  aria-expanded={!isCollapsed}
                >
                  <span className="flex min-w-0 items-center gap-3">
                    <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-white/10 bg-white/[0.05] text-slate-400">
                      {isCollapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                    </span>
                    <span className="font-display text-2xl font-bold tracking-tight text-white">{key === 'other' ? 'Other' : key}</span>
                    <span className="pill shrink-0">{items.length} versions</span>
                    {installedCount > 0 && (
                      <span className="pill-volt shrink-0">{installedCount} installed</span>
                    )}
                  </span>
                </button>
                {!isCollapsed && (
                  <div className="mt-3 divide-y divide-white/[0.06] border-t border-white/[0.06]">
                    {items.map((version) => {
                      const installed = installedIds.has(version.id)
                      const isCurrent = currentVersionId === version.id
                      const picking = pickingId === version.id
                      return (
                        <div key={version.id} className="flex items-center gap-3 py-3">
                          <span className={`shrink-0 rounded-md px-2 py-1 font-mono text-[11px] font-bold uppercase tracking-wide ${typeColor(version.type)}`}>
                            {version.type}
                          </span>
                          <div className="min-w-0 flex-1">
                            <p className="flex flex-wrap items-center gap-x-2 font-display font-bold text-white">
                              <span className="font-mono text-sm">{version.id}</span>
                              {installed && <span className="text-xs font-semibold text-primary-300">• Installed</span>}
                              {isCurrent && <span className="text-xs font-semibold text-violet-300">• Picked</span>}
                            </p>
                            <p className="mt-0.5 text-xs text-slate-500">Released {formatDate(version.release_time)}</p>
                          </div>
                          {installed && (
                            <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={() => void handleUninstall(version.id)} aria-label={`Uninstall ${version.id}`}>
                              <Trash2 className="h-4 w-4 text-red-400" />
                            </Button>
                          )}
                          {!isCurrent ? (
                            <Button size="sm" variant={installed ? 'secondary' : 'primary'} onClick={() => void handlePick(version)} loading={picking} disabled={picking}>
                              <Check className="h-4 w-4 mr-1" /> Pick
                            </Button>
                          ) : (
                            <span className="pill-volt">Selected</span>
                          )}
                        </div>
                      )
                    })}
                  </div>
                )}
              </Card>
            )
          })}
        </div>
      )}
    </div>
  )
}

function typeColor(type: string): string {
  switch (type) {
    case 'release':
      return 'bg-primary-400/15 text-primary-200 border border-primary-400/25'
    case 'snapshot':
      return 'bg-yellow-400/15 text-yellow-200 border border-yellow-400/25'
    case 'beta':
    case 'old_beta':
      return 'bg-sky-400/15 text-sky-200 border border-sky-400/25'
    case 'alpha':
    case 'old_alpha':
      return 'bg-violet-400/15 text-violet-200 border border-violet-400/25'
    default:
      return 'bg-white/[0.06] text-slate-300 border border-white/10'
  }
}
