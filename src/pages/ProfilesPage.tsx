import { useEffect, useState } from 'react'
import { Plus, Edit, Trash2, Copy, Play, Gamepad2, Square } from 'lucide-react'
import { cn } from '../utils/helpers'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Input } from '../components/ui/Input'
import { Modal, ConfirmDialog } from '../components/ui/Modal'
import { Select, Checkbox } from '../components/ui/Select'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../components/ui/Tabs'
import { useProfileStore } from '../store/profileStore'
import { useVersionStore } from '../store/versionStore'
import { useAuthStore } from '../store/authStore'
import { useLaunchStore } from '../store/launchStore'
import { useSettingsStore } from '../store/settingsStore'
import { useDontcamStore } from '../store/dontcamStore'
import { javaApi, utilsApi } from '../tauri/api'
import { formatDate, getMemoryArgs, parseJvmArgs } from '../utils/helpers'
import type { Profile } from '../types'

export function ProfilesPage() {
  const { profiles, selectedProfile, loadProfiles, createProfile, updateProfile, deleteProfile, duplicateProfile, selectProfile } = useProfileStore()
  const { installedVersions, loadInstalledVersions } = useVersionStore()
  const { currentAccount, accounts } = useAuthStore()
  const { launchStatus, launchGame, killGame } = useLaunchStore()
  const { settings } = useSettingsStore()
  const checkDontcamUpdate = useDontcamStore((s) => s.checkDontcamUpdate)
  const dontcamChecking = useDontcamStore((s) => s.checking)

  const [showCreateModal, setShowCreateModal] = useState(false)
  const [editingProfile, setEditingProfile] = useState<Profile | null>(null)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null)
  const [newProfileName, setNewProfileName] = useState('')
  const [newProfileVersion, setNewProfileVersion] = useState('')
  const [newProfileAccount, setNewProfileAccount] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [dontcamOffline, setDontcamOffline] = useState<string | null>(null)

  useEffect(() => {
    loadProfiles()
    loadInstalledVersions()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const isRunning = launchStatus === 'running' || launchStatus === 'launching'

  const handleCreateProfile = async () => {
    if (!newProfileName.trim() || !newProfileVersion) return
    setError(null)
    try {
      const profile = await createProfile(newProfileName.trim(), newProfileVersion, newProfileAccount || currentAccount?.id)
      selectProfile(profile)
      setShowCreateModal(false)
      setNewProfileName('')
      setNewProfileVersion('')
      setNewProfileAccount('')
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  const handleUpdateProfile = async () => {
    if (editingProfile) {
      await updateProfile(editingProfile)
      setEditingProfile(null)
    }
  }

  const handleDeleteProfile = async () => {
    if (showDeleteConfirm) {
      await deleteProfile(showDeleteConfirm)
      setShowDeleteConfirm(null)
    }
  }

  const handleDuplicateProfile = async (profileId: string) => {
    const original = profiles.find((p) => p.id === profileId)
    const name = prompt('Enter name for duplicate profile:', `${original?.name ?? 'Profile'} (Copy)`)
    if (name) await duplicateProfile(profileId, name)
  }

  const handleLaunch = async (profile: Profile) => {
    const account = (profile.account_id && accounts.find((a) => a.id === profile.account_id)) || currentAccount
    if (!account) {
      setError('No account available. Add one on the Accounts page.')
      return
    }
    setError(null)
    setDontcamOffline(null)
    try {
      const res = await checkDontcamUpdate(profile.version_id, profile.id)
      if (res.offline) setDontcamOffline(res.message)
      const [java, gameDir] = await Promise.all([
        javaApi.resolveForVersion(profile.version_id).catch(() => null),
        utilsApi.getGameDir(profile.id).catch(() => ''),
      ])
      const memArgs = getMemoryArgs(settings.java.memory_allocation.min, settings.java.memory_allocation.max, settings.java.memory_allocation.unit)
      await launchGame({
        profile_id: profile.id,
        version_id: profile.version_id,
        account,
        java_path: (java?.path as unknown as string) ?? 'java',
        jvm_args: [...memArgs, ...parseJvmArgs(settings.java.jvm_args), ...parseJvmArgs(profile.java_args)],
        game_args: profile.game_args,
        game_dir: gameDir,
        resolution: profile.resolution ?? undefined,
      })
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <div className="animate-fade-in space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="eyebrow">Instances</p>
          <h1 className="h1 mt-1">Profiles</h1>
          <p className="sub">Separate versions, mods and settings per profile.</p>
        </div>
        <div className="flex items-center gap-2">
          {isRunning && (
            <Button variant="destructive" onClick={() => void killGame()}>
              <Square className="h-4 w-4 mr-2" /> Stop game
            </Button>
          )}
          <Button onClick={() => setShowCreateModal(true)}>
            <Plus className="h-4 w-4 mr-2" /> Create Profile
          </Button>
        </div>
      </div>

      {error && <div className="alert alert-red">{error}</div>}
      {dontcamOffline && <div className="alert alert-yellow">{dontcamOffline}</div>}

      {profiles.length === 0 ? (
        <Card padding="lg" className="text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-3xl border border-white/10 bg-white/[0.04]">
            <Gamepad2 className="h-8 w-8 text-slate-500" />
          </div>
          <h3 className="font-display text-xl font-bold text-white">No profiles yet</h3>
          <p className="mx-auto mt-1 max-w-sm text-sm text-slate-400">Create your first profile to start playing with custom settings</p>
          <Button onClick={() => setShowCreateModal(true)} size="lg" className="mt-6">
            <Plus className="h-4 w-4 mr-2" /> Create Profile
          </Button>
        </Card>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {profiles.map((profile) => {
            const selected = selectedProfile?.id === profile.id
            return (
              <Card
                key={profile.id}
                padding="md"
                className={cn('card-lift cursor-pointer', selected && '!border-primary-400/50 shadow-[0_0_32px_rgba(46,155,255,0.15)]')}
                onClick={() => selectProfile(profile)}
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-primary-400 to-violet-600 font-display text-lg font-bold text-white">
                    {profile.name.charAt(0).toUpperCase()}
                  </div>
                  <div className="min-w-0 flex-1">
                    <h3 className="truncate font-display font-bold text-white">{profile.name}</h3>
                    <p className="truncate font-mono text-xs text-slate-400">{profile.version_id}</p>
                  </div>
                  {selected && <span className="pill-volt shrink-0">Active</span>}
                </div>
                <div className="mt-3 flex flex-wrap gap-1.5">
                  <span className="pill">{profile.mod_loader === 'none' ? 'vanilla' : profile.mod_loader}</span>
                  {profile.dontcam_mod && <span className="pill-violet">dontcam mod</span>}
                  <span className="pill">{profile.last_played ? `Played ${formatDate(profile.last_played)}` : 'Never played'}</span>
                </div>
                <div className="mt-4 flex items-center gap-1 border-t border-white/[0.07] pt-3" onClick={(e) => e.stopPropagation()}>
                  <Button size="sm" className="flex-1" onClick={() => void handleLaunch(profile)} disabled={isRunning} loading={dontcamChecking !== null}>
                    <Play className="h-4 w-4 mr-1" /> Play
                  </Button>
                  <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={() => setEditingProfile(profile)} aria-label={`Edit ${profile.name}`}>
                    <Edit className="h-4 w-4" />
                  </Button>
                  <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={() => void handleDuplicateProfile(profile.id)} aria-label="Duplicate">
                    <Copy className="h-4 w-4" />
                  </Button>
                  <Button variant="ghost" size="sm" className="!h-8 !w-8 !p-0" onClick={() => setShowDeleteConfirm(profile.id)} aria-label="Delete">
                    <Trash2 className="h-4 w-4 text-red-400" />
                  </Button>
                </div>
              </Card>
            )
          })}
        </div>
      )}

      <Modal isOpen={showCreateModal} onClose={() => setShowCreateModal(false)} title="Create New Profile" size="lg">
        <div className="space-y-4">
          <Input label="Profile Name" placeholder="My Awesome Profile" value={newProfileName} onChange={(e) => setNewProfileName(e.target.value)} autoFocus />
          <Select
            label="Version"
            placeholder="Select Minecraft version"
            value={newProfileVersion}
            onChange={(e) => setNewProfileVersion(e.target.value)}
            options={installedVersions.map((v) => ({ value: v.id, label: `${v.id} (${v.version_type})` }))}
          />
          <Select
            label="Account (Optional)"
            value={newProfileAccount}
            onChange={(e) => setNewProfileAccount(e.target.value)}
            options={[{ value: '', label: 'Use current account' }, ...accounts.map((a) => ({ value: a.id, label: `${a.username} (${a.account_type})` }))]}
          />
          <div className="flex justify-end gap-3 pt-2">
            <Button variant="secondary" onClick={() => setShowCreateModal(false)}>Cancel</Button>
            <Button onClick={() => void handleCreateProfile()} disabled={!newProfileName.trim() || !newProfileVersion}>Create Profile</Button>
          </div>
        </div>
      </Modal>

      {editingProfile && (
        <Modal isOpen={true} onClose={() => setEditingProfile(null)} title={`Edit ${editingProfile.name}`} size="xl">
          <Tabs defaultValue="overview" variant="enclosed">
            <TabsList aria-label="Edit profile sections">
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="settings">Settings</TabsTrigger>
            </TabsList>
            <TabsContent value="overview">
              <div className="space-y-4 pt-1">
                <Input label="Profile Name" value={editingProfile.name} onChange={(e) => setEditingProfile({ ...editingProfile, name: e.target.value })} />
                <Select
                  label="Version"
                  value={editingProfile.version_id}
                  onChange={(e) => setEditingProfile({ ...editingProfile, version_id: e.target.value })}
                  options={installedVersions.map((v) => ({ value: v.id, label: v.id }))}
                />
                <Select
                  label="Mod loader"
                  value={editingProfile.mod_loader}
                  onChange={(e) => setEditingProfile({ ...editingProfile, mod_loader: e.target.value as Profile['mod_loader'] })}
                  options={[
                    { value: 'none', label: 'None (Vanilla)' },
                    { value: 'fabric', label: 'Fabric' },
                    { value: 'forge', label: 'Forge' },
                    { value: 'quilt', label: 'Quilt' },
                    { value: 'neoforge', label: 'NeoForge' },
                  ]}
                />
                <Select
                  label="Account"
                  value={editingProfile.account_id || ''}
                  onChange={(e) => setEditingProfile({ ...editingProfile, account_id: e.target.value || undefined })}
                  options={[{ value: '', label: 'Default' }, ...accounts.map((a) => ({ value: a.id, label: `${a.username} (${a.account_type})` }))]}
                />
                <label className="flex cursor-pointer items-start gap-3 rounded-2xl border border-white/[0.08] bg-white/[0.03] p-4">
                  <input
                    type="checkbox"
                    checked={editingProfile.dontcam_mod}
                    onChange={(e) => setEditingProfile({ ...editingProfile, dontcam_mod: e.target.checked })}
                    className="mt-1 h-5 w-5 shrink-0 cursor-pointer appearance-none rounded-md border border-white/20 bg-black/50 transition-all checked:border-primary-400 checked:bg-primary-400 checked:bg-[url('data:image/svg+xml,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 20 20%27 fill=%27none%27 stroke=%27%23000%27 stroke-width=%273%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpath d=%27M4 10l4 4 8-8%27/%3e%3c/svg%3e')] checked:bg-center checked:bg-no-repeat"
                  />
                  <span>
                    <span className="block font-display text-sm font-bold text-white">DontCam Client Mod</span>
                    <span className="mt-0.5 block text-[13px] leading-relaxed text-slate-400">
                      Auto-installs the bundled DontCam mod into this instance on launch, wherever a port exists.
                    </span>
                  </span>
                </label>
                <div className="flex justify-end gap-3 border-t border-white/[0.08] pt-4">
                  <Button variant="secondary" onClick={() => setEditingProfile(null)}>Cancel</Button>
                  <Button onClick={() => void handleUpdateProfile()}>Save Changes</Button>
                </div>
              </div>
            </TabsContent>
            <TabsContent value="settings">
              <div className="space-y-4 pt-1">
                <Input
                  label="JVM Arguments"
                  value={editingProfile.java_args}
                  onChange={(e) => setEditingProfile({ ...editingProfile, java_args: e.target.value })}
                  helperText="Additional JVM arguments (memory is handled in Settings)"
                />
                <Input
                  label="Game arguments (space separated)"
                  value={editingProfile.game_args.join(' ')}
                  onChange={(e) => setEditingProfile({ ...editingProfile, game_args: e.target.value.split(' ').filter(Boolean) })}
                />
                <div className="grid grid-cols-2 gap-4">
                  <Input
                    label="Window Width"
                    type="number"
                    value={editingProfile.resolution?.width ?? ''}
                    placeholder="854"
                    onChange={(e) =>
                      setEditingProfile({
                        ...editingProfile,
                        resolution: { width: parseInt(e.target.value) || 854, height: editingProfile.resolution?.height ?? 480, fullscreen: editingProfile.resolution?.fullscreen ?? false },
                      })
                    }
                  />
                  <Input
                    label="Window Height"
                    type="number"
                    value={editingProfile.resolution?.height ?? ''}
                    placeholder="480"
                    onChange={(e) =>
                      setEditingProfile({
                        ...editingProfile,
                        resolution: { width: editingProfile.resolution?.width ?? 854, height: parseInt(e.target.value) || 480, fullscreen: editingProfile.resolution?.fullscreen ?? false },
                      })
                    }
                  />
                </div>
                <Checkbox
                  label="Fullscreen"
                  checked={editingProfile.resolution?.fullscreen ?? false}
                  onChange={(e) =>
                    setEditingProfile({
                      ...editingProfile,
                      resolution: { width: editingProfile.resolution?.width ?? 854, height: editingProfile.resolution?.height ?? 480, fullscreen: e.target.checked },
                    })
                  }
                />
                <div className="flex justify-end gap-3 border-t border-white/[0.08] pt-4">
                  <Button variant="secondary" onClick={() => setEditingProfile(null)}>Cancel</Button>
                  <Button onClick={() => void handleUpdateProfile()}>Save Changes</Button>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </Modal>
      )}

      <ConfirmDialog
        isOpen={!!showDeleteConfirm}
        onClose={() => setShowDeleteConfirm(null)}
        onConfirm={() => void handleDeleteProfile()}
        title="Delete Profile"
        message="Are you sure you want to delete this profile? This action cannot be undone."
        confirmText="Delete"
        variant="danger"
      />
    </div>
  )
}
