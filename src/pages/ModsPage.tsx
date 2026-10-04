import { useEffect, useState } from 'react'
import { Download, Puzzle, Box, Search, Layers } from 'lucide-react'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Select } from '../components/ui/Select'
import { Modal } from '../components/ui/Modal'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../components/ui/Tabs'
import { useVersionStore } from '../store/versionStore'
import { useProfileStore } from '../store/profileStore'
import { useConsoleStore } from '../store/consoleStore'
import { dontcamApi, utilsApi, type DontcamRelease } from '../tauri/api'
import type { ModLoaderVersion } from '../types'

const MC_VERSIONS = ['1.21.1', '1.20.1', '1.19.4', '1.18.2', '1.17.1', '1.16.5', '1.12.2', '1.8.9']

/** Loader is fixed per MC line — NEVER change MC versions or loaders here. */
export function loaderForMc(mc: string): 'forge' | 'fabric' {
  return mc === '1.8.9' || mc === '1.12.2' ? 'forge' : 'fabric'
}

export function ModsPage() {
  const { getModLoaderVersions, installModLoader, getInstalledModLoaders } = useVersionStore()
  const { profiles } = useProfileStore()

  const [tab, setTab] = useState('browse')
  const [mcVersion, setMcVersion] = useState('1.20.1')
  const [modLoader, setModLoader] = useState<'forge' | 'fabric' | 'quilt' | 'neoforge'>('fabric')
  const [loaderVersions, setLoaderVersions] = useState<ModLoaderVersion[]>([])
  const [installed, setInstalled] = useState<Awaited<ReturnType<typeof getInstalledModLoaders>>>([])
  const [loading, setLoading] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedProfile, setSelectedProfile] = useState('')
  const [installTarget, setInstallTarget] = useState<ModLoaderVersion | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [modsFolder, setModsFolder] = useState('')

  // DontCam mod section (private aanges/MODY repo)
  const [dontcamMc, setDontcamMc] = useState('1.20.1')
  const [dontcamReleases, setDontcamReleases] = useState<DontcamRelease[]>([])
  const [dontcamLoading, setDontcamLoading] = useState(false)
  const [dontcamInstalling, setDontcamInstalling] = useState(false)
  const [dontcamError, setDontcamError] = useState<string | null>(null)
  const [dontcamPath, setDontcamPath] = useState<string | null>(null)

  const loadLoaders = async () => {
    setLoading(true)
    setError(null)
    try {
      const versions = await getModLoaderVersions(modLoader, mcVersion)
      setLoaderVersions(versions)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  const loadInstalled = async () => {
    try {
      setInstalled(await getInstalledModLoaders())
    } catch {
      setInstalled([])
    }
  }

  useEffect(() => {
    void loadLoaders()
    void loadInstalled()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [modLoader, mcVersion])

  useEffect(() => {
    setDontcamLoading(true)
    dontcamApi
      .list()
      .then(setDontcamReleases)
      .catch((e) => setDontcamError(e instanceof Error ? e.message : String(e)))
      .finally(() => setDontcamLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    if (profiles.length > 0 && !selectedProfile) setSelectedProfile(profiles[0].id)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [profiles])

  useEffect(() => {
    if (!selectedProfile) {
      setModsFolder('')
      return
    }
    utilsApi
      .getGameDir(selectedProfile)
      .then((dir) => setModsFolder(`${dir}\\mods`))
      .catch(() => setModsFolder(''))
  }, [selectedProfile])

  const handleInstall = async () => {
    if (!installTarget) return
    setError(null)
    try {
      const gameDir = selectedProfile ? await utilsApi.getGameDir(selectedProfile).catch(() => '') : ''
      await installModLoader(installTarget, gameDir)
      setInstallTarget(null)
      await loadInstalled()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  const filtered = loaderVersions.filter((v) => v.id.toLowerCase().includes(searchQuery.toLowerCase()))

  return (
    <div className="animate-fade-in space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="eyebrow">Loaders</p>
          <h1 className="h1 mt-1">Mods & Mod Loaders</h1>
          <p className="sub">Install Fabric, Forge or Quilt for your Minecraft version</p>
        </div>
        <Card padding="sm" className="flex flex-wrap items-end gap-3">
          <div className="w-36">
            <Select value={mcVersion} onChange={(e) => setMcVersion(e.target.value)} options={MC_VERSIONS.map((v) => ({ value: v, label: v }))} />
          </div>
          <div className="w-36">
            <Select
              value={modLoader}
              onChange={(e) => setModLoader(e.target.value as typeof modLoader)}
              options={[
                { value: 'fabric', label: 'Fabric' },
                { value: 'forge', label: 'Forge' },
                { value: 'quilt', label: 'Quilt' },
                { value: 'neoforge', label: 'NeoForge' },
              ]}
            />
          </div>
          <Button variant="secondary" onClick={() => void loadLoaders()} loading={loading}>
            <Search className="h-4 w-4 mr-2" /> Load
          </Button>
        </Card>
      </div>

      {error && (
        <div className="alert alert-red">{error}</div>
      )}

      <Card padding="md" className="!border-primary-400/25">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="eyebrow">Official mod</p>
            <h2 className="mt-1 font-display text-xl font-bold tracking-tight text-white">DontCam Mod</h2>
            <p className="sub">
              Pobiera właściwy jar z prywatnego repo <span className="font-mono">aanges/MODY</span> do{' '}
              <span className="font-mono">instances/&lt;profil&gt;/mods/</span>, nadpisując starą wersję. Loader dobierany
              automatycznie ({loaderForMc(dontcamMc)}) i instalowany, gdy go brakuje.
            </p>
          </div>
          <div className="flex flex-wrap items-end gap-3">
            <div className="w-36">
              <Select
                label="Wersja MC"
                value={dontcamMc}
                onChange={(e) => { setDontcamMc(e.target.value); setDontcamPath(null); setDontcamError(null) }}
                options={MC_VERSIONS.map((v) => ({ value: v, label: `${v} (${loaderForMc(v)})` }))}
              />
            </div>
            <div className="min-w-44 flex-1 sm:max-w-64">
              <Select
                label="Profil"
                value={selectedProfile}
                onChange={(e) => setSelectedProfile(e.target.value)}
                options={profiles.map((p) => ({ value: p.id, label: `${p.name} (${p.version_id})` }))}
              />
            </div>
            <Button
              loading={dontcamInstalling}
              disabled={dontcamInstalling || !selectedProfile}
              onClick={() => void (async () => {
                const push = useConsoleStore.getState().push
                setDontcamInstalling(true)
                setDontcamError(null)
                setDontcamPath(null)
                push('info', `Instalowanie DontCam dla MC ${dontcamMc} (profil ${selectedProfile || '—'})…`)
                try {
                  const dest = await dontcamApi.install(dontcamMc, selectedProfile || undefined)
                  setDontcamPath(dest)
                  push('success', `DontCam dla ${dontcamMc} gotowy: ${dest}`)
                } catch (e) {
                  const msg = e instanceof Error ? e.message : String(e)
                  setDontcamError(msg)
                  push('error', `Instalacja DontCam nie powiodła się: ${msg}`)
                } finally {
                  setDontcamInstalling(false)
                }
              })()}
            >
              <Download className="h-4 w-4 mr-2" /> Instaluj DontCam
            </Button>
          </div>
        </div>
        {dontcamLoading && <p className="mt-3 text-xs text-slate-500">Ładowanie listy buildów z MODY…</p>}
        {!dontcamLoading && dontcamReleases.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1.5">
            {dontcamReleases.map((r) => (
              <button
                key={r.mc_version}
                onClick={() => { setDontcamMc(r.mc_version); setDontcamPath(null); setDontcamError(null) }}
                className={r.mc_version === dontcamMc ? 'pill-volt' : 'pill'}
                title={`Folder ${r.folder} • ${r.source}${r.assets.length ? ` • ${r.assets.join(', ')}` : ''}`}
              >
                {r.mc_version} • {r.loader}
              </button>
            ))}
          </div>
        )}
        {dontcamError && <div className="alert alert-red mt-3">{dontcamError}</div>}
        {dontcamPath && <div className="alert alert-green mt-3">Zainstalowano: <span className="font-mono">{dontcamPath}</span>. Możesz startować grę przez launch_game.</div>}
        {!selectedProfile && <p className="mt-3 text-xs text-yellow-200/80">Wybierz profil, aby zainstalować moda do jego folderu mods.</p>}
      </Card>

      <Tabs value={tab} onChange={setTab} variant="enclosed">
        <TabsList aria-label="Mods sections">
          <TabsTrigger value="browse">Browse ({loaderVersions.length})</TabsTrigger>
          <TabsTrigger value="installed">Installed ({installed.length})</TabsTrigger>
          <TabsTrigger value="howto">How to add mods</TabsTrigger>
        </TabsList>

        <TabsContent value="browse">
          <div className="relative mb-4 max-w-md">
            <Search className="absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
            <input
              type="text"
              placeholder="Search loader versions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="h-11 w-full rounded-xl border border-white/10 bg-black/50 pl-11 pr-4 text-[15px] text-white placeholder:text-slate-600 transition-all focus:border-primary-400/60 focus:outline-none focus:ring-2 focus:ring-primary-400/20"
            />
          </div>

          {loading ? (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {[...Array(6)].map((_, i) => (
                <Card key={i} padding="md" className="animate-pulse">
                  <div className="mb-3 h-4 w-20 rounded bg-white/10" />
                  <div className="mb-4 h-6 w-3/4 rounded bg-white/10" />
                  <div className="h-10 rounded-xl bg-white/10" />
                </Card>
              ))}
            </div>
          ) : filtered.length === 0 ? (
            <Card padding="lg" className="text-center">
              <Puzzle className="mx-auto mb-4 h-12 w-12 text-slate-600" />
              <h3 className="font-display text-lg font-bold text-white">No versions found</h3>
              <p className="mt-1 text-sm text-slate-400">Try a different Minecraft version or mod loader</p>
            </Card>
          ) : (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {filtered.map((version) => (
                <Card key={version.id} padding="md" className="card-lift">
                  <span className="pill-violet mb-2 capitalize">{version.mod_loader}</span>
                  <h3 className="mb-1 break-all font-mono text-sm font-bold text-white">{version.id}</h3>
                  <p className="mb-4 text-[13px] text-slate-400">Minecraft {version.mc_version}</p>
                  <Button className="w-full" size="sm" onClick={() => setInstallTarget(version)}>
                    <Download className="h-4 w-4 mr-2" /> Install
                  </Button>
                </Card>
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="installed">
          {installed.length === 0 ? (
            <Card padding="lg" className="text-center">
              <Box className="mx-auto mb-4 h-12 w-12 text-slate-600" />
              <h3 className="font-display text-lg font-bold text-white">No mod loaders installed yet</h3>
              <p className="mt-1 text-sm text-slate-400">Install one from the Browse tab</p>
            </Card>
          ) : (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {installed.map((item) => (
                <Card key={item.id} padding="md" className="card-lift">
                  <span className="pill-volt mb-2 capitalize">{item.mod_loader}</span>
                  <h3 className="break-all font-mono text-sm font-bold text-white">{item.id}</h3>
                  <p className="mt-1 text-[13px] text-slate-400">MC {item.mc_version}</p>
                </Card>
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="howto">
          <Card padding="md">
            <h3 className="flex items-center gap-2 font-display font-bold text-white">
              <Layers className="h-4 w-4 text-primary-300" /> Adding mods manually
            </h3>
            <ol className="mt-4 space-y-3 text-sm leading-relaxed text-slate-300">
              {[
                'Install a mod loader above (Fabric recommended for 1.20.1/1.21.1, Forge for 1.12.2, any for 1.8.9 with Forge).',
                'Select a profile below and open its mods folder.',
                'Download mods (.jar) matching your Minecraft + loader version from Modrinth or CurseForge.',
                'Drop the .jar files into the mods folder and launch the game with the modded version.',
              ].map((step, i) => (
                <li key={i} className="flex gap-3">
                  <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary-400 font-display text-xs font-bold text-white">
                    {i + 1}
                  </span>
                  <span className="pt-0.5">{step}</span>
                </li>
              ))}
            </ol>
            <div className="mt-5 flex flex-col gap-3 sm:flex-row">
              <Select
                label="Profile"
                value={selectedProfile}
                onChange={(e) => setSelectedProfile(e.target.value)}
                options={profiles.map((p) => ({ value: p.id, label: `${p.name} (${p.version_id})` }))}
              />
              <div className="flex items-end">
                <Button variant="secondary" disabled={!modsFolder} onClick={() => modsFolder && void utilsApi.openFolder(modsFolder.replace('\\mods', ''))}>
                  Open game folder
                </Button>
              </div>
            </div>
            {modsFolder && <p className="mt-3 break-all font-mono text-[11px] text-slate-500">Mods folder: {modsFolder}</p>}
          </Card>
        </TabsContent>
      </Tabs>

      {installTarget && (
        <Modal isOpen={true} onClose={() => setInstallTarget(null)} title={`Install ${installTarget.id}`} size="md">
          <p className="mb-4 text-sm leading-relaxed text-slate-300">
            This will install <strong className="font-mono text-white">{installTarget.id}</strong> for Minecraft {installTarget.mc_version}.
          </p>
          <Select
            label="Target Profile (optional)"
            value={selectedProfile}
            onChange={(e) => setSelectedProfile(e.target.value)}
            options={[{ value: '', label: 'Global install' }, ...profiles.map((p) => ({ value: p.id, label: p.name }))]}
          />
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setInstallTarget(null)}>Cancel</Button>
            <Button onClick={() => void handleInstall()}>Install</Button>
          </div>
        </Modal>
      )}
    </div>
  )
}
