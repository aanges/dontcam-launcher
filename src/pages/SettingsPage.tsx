import { useEffect, useState } from 'react'
import { Save, RotateCcw, FolderOpen, Cpu, Monitor, Network, Wrench, Download, Check, RefreshCw } from 'lucide-react'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Input, Textarea } from '../components/ui/Input'
import { Select, Checkbox } from '../components/ui/Select'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../components/ui/Tabs'
import { useSettingsStore } from '../store/settingsStore'
import { useThemeStore } from '../store/authStore'
import { useUpdateStore } from '../store/updateStore'
import { useConsoleStore } from '../store/consoleStore'
import { javaApi, utilsApi } from '../tauri/api'
import type { JavaInstallation, Settings } from '../types'

function SectionTitle({ icon: Icon, title, hint }: { icon: React.ComponentType<{ className?: string }>; title: string; hint?: string }) {
  return (
    <div className="mb-4">
      <h3 className="flex items-center gap-2 font-display font-bold text-white">
        <span className="flex h-8 w-8 items-center justify-center rounded-xl border border-primary-400/25 bg-primary-400/10">
          <Icon className="h-4 w-4 text-primary-300" />
        </span>
        {title}
      </h3>
      {hint && <p className="mt-1.5 text-[13px] text-slate-500">{hint}</p>}
    </div>
  )
}

export function SettingsPage() {
  const { settings, loadSettings, updateSettings, resetSettings, isLoading } = useSettingsStore()
  const { theme, setTheme } = useThemeStore()
  const [tab, setTab] = useState('general')
  const [javaList, setJavaList] = useState<JavaInstallation[]>([])
  const [javaLoading, setJavaLoading] = useState(false)
  const [downloadingJava, setDownloadingJava] = useState<number | null>(null)
  const [javaError, setJavaError] = useState<string | null>(null)
  const [saved, setSaved] = useState(false)
  const [appDataDir, setAppDataDir] = useState('')
  const updateStatus = useUpdateStore((s) => s.status)
  const updateVersion = useUpdateStore((s) => s.version)
  const updateError = useUpdateStore((s) => s.error)
  const checkForUpdates = useUpdateStore((s) => s.checkForUpdates)

  useEffect(() => {
    loadSettings()
    utilsApi.getAppDataDir().then(setAppDataDir).catch(() => undefined)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const refreshJava = async () => {
    setJavaLoading(true)
    try {
      const list = await javaApi.detect()
      setJavaList(list)
    } finally {
      setJavaLoading(false)
    }
  }

  const downloadJavaMajor = async (major: number) => {
    setDownloadingJava(major)
    setJavaError(null)
    try {
      await javaApi.download(major, 'eclipse_adoptium', 'x64')
      await refreshJava()
    } catch (e) {
      setJavaError(e instanceof Error ? e.message : String(e))
    } finally {
      setDownloadingJava(null)
    }
  }

  useEffect(() => {
    refreshJava()
  }, [])

  const save = async () => {
    // settings are saved on every change already; this just gives feedback
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  const setUi = (patch: Partial<Settings['ui']>) => updateSettings({ ui: { ...settings.ui, ...patch } })
  const setJava = (patch: Partial<Settings['java']>) => updateSettings({ java: { ...settings.java, ...patch } })
  const setGame = (patch: Partial<Settings['game']>) => updateSettings({ game: { ...settings.game, ...patch } })
  const setAdvanced = (patch: Partial<Settings['advanced']>) => updateSettings({ advanced: { ...settings.advanced, ...patch } })

  return (
    <div className="animate-fade-in space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="eyebrow">Tuning</p>
          <h1 className="h1 mt-1">Settings</h1>
          <p className="sub">Configure launcher, Java and game defaults</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" onClick={() => void resetSettings()}>
            <RotateCcw className="h-4 w-4 mr-2" /> Reset
          </Button>
          <Button onClick={() => void save()} loading={isLoading}>
            <Save className="h-4 w-4 mr-2" /> {saved ? 'Saved!' : 'Save'}
          </Button>
        </div>
      </div>

      <Tabs value={tab} onChange={setTab} variant="enclosed">
        <TabsList aria-label="Settings sections">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="java">Java & Memory</TabsTrigger>
          <TabsTrigger value="game">Game</TabsTrigger>
          <TabsTrigger value="network">Network</TabsTrigger>
          <TabsTrigger value="advanced">Advanced</TabsTrigger>
        </TabsList>

        <TabsContent value="general">
          <div className="grid gap-4">
            <Card padding="md">
              <SectionTitle icon={Download} title="Aktualizacje launchera" hint="Launcher sprawdza GitHub Releases (aanges/dontcam-launcher) automatycznie przy starcie. Okno instalacji pojawia się tylko, gdy nowa wersja istnieje." />
              <div className="flex flex-wrap items-center gap-3">
                <Button
                  variant="secondary"
                  size="sm"
                  loading={updateStatus === 'checking'}
                  onClick={() => {
                    useConsoleStore.getState().push('info', 'Ręczne sprawdzanie aktualizacji…')
                    void checkForUpdates(false)
                  }}
                >
                  <RefreshCw className="h-4 w-4 mr-2" /> Sprawdź aktualizacje
                </Button>
                {updateStatus === 'checking' && <span className="pill">sprawdzanie…</span>}
                {updateStatus === 'available' && <span className="pill-volt">dostępna: {updateVersion}</span>}
                {updateStatus === 'idle' && <span className="pill">aktualny</span>}
                {updateStatus === 'error' && <span className="pill">{updateError ?? 'błąd sprawdzania'}</span>}
              </div>
            </Card>
            <Card padding="md">
              <SectionTitle icon={Monitor} title="Appearance" />
              <div className="grid gap-4 sm:grid-cols-2">
                <Select
                  label="Theme"
                  value={theme}
                  onChange={(e) => setTheme(e.target.value as typeof theme)}
                  options={[
                    { value: 'dark', label: 'Dark' },
                    { value: 'light', label: 'Light' },
                    { value: 'system', label: 'System' },
                  ]}
                />
                <Select
                  label="Language"
                  value={settings.language}
                  onChange={(e) => void updateSettings({ language: e.target.value })}
                  options={[
                    { value: 'en-US', label: 'English (US)' },
                    { value: 'pl-PL', label: 'Polski' },
                  ]}
                />
              </div>
              <div className="mt-4 grid gap-2 sm:grid-cols-2">
                <Checkbox label="Compact mode" checked={settings.ui.compact_mode} onChange={(e) => void setUi({ compact_mode: e.target.checked })} />
                <Checkbox label="Animations" checked={settings.ui.animations} onChange={(e) => void setUi({ animations: e.target.checked })} />
              </div>
            </Card>
            <Card padding="md">
              <SectionTitle icon={FolderOpen} title="Installation folder" hint="Vanilla-style layout next to your .minecraft: versions, libraries, assets, instances, profiles." />
              <p className="mb-3 break-all font-mono text-[11px] text-slate-500">{appDataDir || 'Loading...'}</p>
              <div className="flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" disabled={!appDataDir} onClick={() => appDataDir && void utilsApi.openFolder(appDataDir)}>
                  Open folder
                </Button>
                <Button variant="secondary" size="sm" disabled={!appDataDir} onClick={() => appDataDir && void utilsApi.openFolder(`${appDataDir}\\versions`)}>
                  Open versions
                </Button>
                <Button variant="secondary" size="sm" disabled={!appDataDir} onClick={() => appDataDir && void utilsApi.openFolder(`${appDataDir}\\instances`)}>
                  Open instances
                </Button>
              </div>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="java">
          <div className="grid gap-4">
            <Card padding="md">
              <SectionTitle icon={Cpu} title="Memory" hint="1.8.9/1.12.2 need Java 8 • 1.20.1 needs Java 17 • 1.21.1+ needs Java 21. The launcher picks the best installed Java automatically." />
              <div className="grid gap-4 sm:grid-cols-3">
                <Input
                  label="Min memory"
                  type="number"
                  min={256}
                  value={settings.java.memory_allocation.min}
                  onChange={(e) => void setJava({ memory_allocation: { ...settings.java.memory_allocation, min: parseInt(e.target.value) || 1 } })}
                />
                <Input
                  label="Max memory"
                  type="number"
                  min={512}
                  value={settings.java.memory_allocation.max}
                  onChange={(e) => void setJava({ memory_allocation: { ...settings.java.memory_allocation, max: parseInt(e.target.value) || 4 } })}
                />
                <Select
                  label="Unit"
                  value={settings.java.memory_allocation.unit}
                  onChange={(e) => void setJava({ memory_allocation: { ...settings.java.memory_allocation, unit: e.target.value as 'MB' | 'GB' } })}
                  options={[
                    { value: 'MB', label: 'MB' },
                    { value: 'GB', label: 'GB' },
                  ]}
                />
              </div>
            </Card>
            <Card padding="md">
              <div className="mb-4 flex items-center justify-between">
                <h3 className="font-display font-bold text-white">Java installations</h3>
                <Button variant="secondary" size="sm" onClick={() => void refreshJava()} loading={javaLoading}>Detect</Button>
              </div>
              {javaList.length === 0 ? (
                <p className="rounded-xl border border-dashed border-white/10 p-4 text-center text-sm text-slate-500">No Java found. Install Eclipse Temurin 8 + 17 + 21, then click Detect.</p>
              ) : (
                <div className="space-y-2">
                  {javaList.map((j, i) => (
                    <div key={i} className="rounded-xl border border-white/[0.07] bg-black/40 p-3">
                      <p className="font-display text-sm font-bold text-white">Java {j.version.major} <span className="font-mono font-normal text-slate-400">• {j.version.full}</span></p>
                      <p className="mt-0.5 break-all font-mono text-[11px] text-slate-500">{j.path} • {j.vendor} • {j.source}</p>
                    </div>
                  ))}
                </div>
              )}
              <div className="mt-5 border-t border-white/[0.07] pt-4">
                <h4 className="mb-2 flex items-center gap-2 font-display text-sm font-bold text-white"><Download className="h-4 w-4 text-primary-300" /> Download Java (Eclipse Temurin)</h4>
                <div className="flex flex-wrap gap-2">
                  {[8, 17, 21].map((major) => {
                    const hasIt = javaList.some((j) => j.version.major === major)
                    const downloading = downloadingJava === major
                    return (
                      <Button
                        key={major}
                        variant={hasIt ? 'ghost' : 'secondary'}
                        size="sm"
                        disabled={hasIt || downloadingJava !== null}
                        loading={downloading}
                        onClick={() => void downloadJavaMajor(major)}
                      >
                        {hasIt ? <><Check className="h-4 w-4 mr-1 text-primary-300" /> Java {major}</> : <><Download className="h-4 w-4 mr-1" /> Java {major}</>}
                      </Button>
                    )
                  })}
                </div>
                {javaError && <p className="mt-2 text-xs text-red-400">Download failed: {javaError}</p>}
              </div>
              <div className="mt-4">
                <Input
                  label="Custom Java path (optional)"
                  placeholder="C:\Program Files\Eclipse Adoptium\jdk-21\bin\java.exe"
                  value={settings.java.custom_java_path ?? ''}
                  onChange={(e) => void setJava({ custom_java_path: e.target.value || undefined })}
                />
              </div>
              <div className="mt-4">
                <Textarea
                  label="Extra JVM args"
                  value={settings.java.jvm_args}
                  rows={3}
                  onChange={(e) => void setJava({ jvm_args: e.target.value })}
                />
              </div>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="game">
          <Card padding="md">
            <SectionTitle icon={Monitor} title="Game defaults" />
            <div className="grid gap-4 sm:grid-cols-2">
              <Input
                label="Custom game dir (optional)"
                placeholder="Leave empty for per-profile instances"
                value={settings.game.default_game_dir ?? ''}
                onChange={(e) => void setGame({ default_game_dir: e.target.value || undefined })}
              />
              <Select
                label="Close launcher behaviour"
                value={settings.game.keep_launcher_open ? 'keep' : 'close'}
                onChange={(e) => void setGame({ keep_launcher_open: e.target.value === 'keep', auto_close_launcher: e.target.value === 'close' })}
                options={[
                  { value: 'keep', label: 'Keep launcher open' },
                  { value: 'close', label: 'Close on launch' },
                ]}
              />
            </div>
            <div className="mt-4 grid gap-4 sm:grid-cols-3">
              <Input label="Width" type="number" value={settings.game.custom_resolution?.width ?? ''} placeholder="854" onChange={(e) => void setGame({ custom_resolution: e.target.value ? { width: parseInt(e.target.value) || 854, height: settings.game.custom_resolution?.height ?? 480, fullscreen: settings.game.fullscreen } : undefined })} />
              <Input label="Height" type="number" value={settings.game.custom_resolution?.height ?? ''} placeholder="480" onChange={(e) => void setGame({ custom_resolution: e.target.value ? { width: settings.game.custom_resolution?.width ?? 854, height: parseInt(e.target.value) || 480, fullscreen: settings.game.fullscreen } : undefined })} />
              <div className="flex items-end pb-2">
                <Checkbox label="Fullscreen" checked={settings.game.fullscreen} onChange={(e) => void setGame({ fullscreen: e.target.checked })} />
              </div>
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="network">
          <Card padding="md">
            <SectionTitle icon={Network} title="Downloads" />
            <div className="grid gap-4 sm:grid-cols-2">
              <Input label="Download threads" type="number" value={settings.network.download_threads} onChange={(e) => void updateSettings({ network: { download_threads: parseInt(e.target.value) || 4 } })} />
              <div className="flex items-end pb-2">
                <Checkbox label="Verify downloads (SHA1)" checked={settings.advanced.verify_downloads} onChange={(e) => void setAdvanced({ verify_downloads: e.target.checked })} />
              </div>
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="advanced">
          <Card padding="md">
            <SectionTitle icon={Wrench} title="Advanced" />
            <div className="space-y-4">
              <Textarea
                label="Custom game args (space separated)"
                value={settings.advanced.custom_game_args.join(' ')}
                onChange={(e) => void setAdvanced({ custom_game_args: e.target.value.split(' ').filter(Boolean) })}
              />
              <Input
                label="Pre-launch command"
                value={settings.advanced.pre_launch_command ?? ''}
                onChange={(e) => void setAdvanced({ pre_launch_command: e.target.value || undefined })}
              />
              <Input
                label="Post-exit command"
                value={settings.advanced.post_exit_command ?? ''}
                onChange={(e) => void setAdvanced({ post_exit_command: e.target.value || undefined })}
              />
              <div className="flex flex-wrap gap-x-8 gap-y-2 border-t border-white/[0.07] pt-4">
                <Checkbox label="Debug logging" checked={settings.advanced.debug_logging} onChange={(e) => void setAdvanced({ debug_logging: e.target.checked })} />
                <Checkbox label="Show console" checked={settings.advanced.console_enabled} onChange={(e) => void setAdvanced({ console_enabled: e.target.checked })} />
              </div>
            </div>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
