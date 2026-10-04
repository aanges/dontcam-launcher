import { create } from 'zustand'
import type { VersionManifest, VersionInfo, GameVersion, ModLoaderVersion, InstalledModLoader, InstallModdedResult, JavaInstallation } from '../types'
import { versionApi, modLoaderApi, javaApi } from '../tauri/api'
import { useDontcamStore } from './dontcamStore'

/** Curated minor lines shown in the picker. Everything else (incl. pre-1.8) is hidden. */
export const CURATED_MINORS = ['1.8', '1.12', '1.16', '1.17', '1.18', '1.19', '1.20', '1.21']

const curatedRegexes = CURATED_MINORS.map((m) => new RegExp(`^${m.replace('.', '\\.')}(?:[.-]|$)`))

export function isCuratedVersion(id: string): boolean {
  return curatedRegexes.some((re) => re.test(id))
}

interface VersionState {
  manifest: VersionManifest | null
  installedVersions: GameVersion[]
  filteredVersions: VersionInfo[]
  isLoading: boolean
  isInstalling: string | null
  installProgress: Record<string, number>
  loadManifest: () => Promise<void>
  loadInstalledVersions: () => Promise<void>
  installVersion: (versionId: string, force?: boolean) => Promise<GameVersion>
  installModded: (versionId: string, force?: boolean, profileId?: string) => Promise<InstallModdedResult>
  uninstallVersion: (versionId: string) => Promise<void>
  getVersionDetails: (versionId: string) => Promise<GameVersion>
  filterVersions: (showSnapshots: boolean, showOld: boolean, showAlphaBeta: boolean, sortBy: string) => void
  getModLoaderVersions: (modLoader: string, mcVersion: string) => Promise<ModLoaderVersion[]>
  installModLoader: (version: ModLoaderVersion, gameDir: string) => Promise<InstalledModLoader>
  getInstalledModLoaders: () => Promise<InstalledModLoader[]>
  detectJava: () => Promise<JavaInstallation[]>
  getJavaInstallations: () => Promise<JavaInstallation[]>
  downloadJava: (version: number, vendor: string, architecture: string) => Promise<JavaInstallation>
}

export const useVersionStore = create<VersionState>((set, get) => ({
  manifest: null,
  installedVersions: [],
  filteredVersions: [],
  isLoading: false,
  isInstalling: null,
  installProgress: {},
  
  loadManifest: async () => {
    set({ isLoading: true })
    try {
      const manifest = await versionApi.fetchManifest()
      set({ manifest })
      get().filterVersions(false, true, false, 'newest_first')
    } catch (error) {
      console.error('Failed to load manifest:', error)
    } finally {
      set({ isLoading: false })
    }
  },
  
  loadInstalledVersions: async () => {
    try {
      const versions = await versionApi.getInstalled()
      set({ installedVersions: versions })
    } catch (error) {
      console.error('Failed to load installed versions:', error)
    }
  },
  
  installVersion: async (versionId, force = false) => {
    set({ isInstalling: versionId, installProgress: { ...get().installProgress, [versionId]: 0 } })
    try {
      const version = await versionApi.install(versionId, force)
      set((state) => ({
        installedVersions: state.installedVersions.some((v) => v.id === version.id)
          ? state.installedVersions.map((v) => (v.id === version.id ? version : v))
          : [...state.installedVersions, version],
        isInstalling: null,
        installProgress: { ...state.installProgress, [versionId]: 100 },
      }))
      return version
    } catch (error) {
      set({ isInstalling: null })
      throw error
    }
  },
  
  installModded: async (versionId, force = false, profileId) => {
    set({ isInstalling: versionId, installProgress: { ...get().installProgress, [versionId]: 0 } })
    try {
      const result = await modLoaderApi.installModded(versionId, force, profileId)
      await get().loadInstalledVersions()
      // Refresh the DontCam jar right after install (backend already staged
      // one — this only swaps it when MODY has something newer). Must not
      // fail the install when the network is down.
      try {
        await useDontcamStore.getState().checkDontcamUpdate(versionId, profileId)
      } catch {
        /* offline — staged mod stays */
      }
      set((state) => ({
        isInstalling: null,
        installProgress: { ...state.installProgress, [versionId]: 100 },
      }))
      return result
    } catch (error) {
      set({ isInstalling: null })
      throw error
    }
  },

  uninstallVersion: async (versionId) => {
    await versionApi.uninstall(versionId)
    set((state) => ({
      installedVersions: state.installedVersions.filter((v) => v.id !== versionId),
    }))
  },
  
  getVersionDetails: async (versionId) => {
    return versionApi.getDetails(versionId)
  },
  
  filterVersions: (showSnapshots, showOld, showAlphaBeta, sortBy) => {
    const { manifest } = get()
    if (!manifest) return

    const versions = manifest.versions.filter((v) => {
      if (!isCuratedVersion(v.id)) return false
      // 1.8 line: only 1.8.9 stays, every other 1.8.x is hidden.
      if (/^1\.8[.-]/.test(v.id) && v.id !== '1.8.9') return false
      if (!showSnapshots && v.type === 'snapshot') return false
      if (!showOld && (v.type === 'old_alpha' || v.type === 'old_beta')) return false
      if (!showAlphaBeta && (v.type === 'alpha' || v.type === 'beta')) return false
      return true
    })
    
    switch (sortBy) {
      case 'newest_first':
        versions.sort((a, b) => new Date(b.release_time).getTime() - new Date(a.release_time).getTime())
        break
      case 'oldest_first':
        versions.sort((a, b) => new Date(a.release_time).getTime() - new Date(b.release_time).getTime())
        break
      case 'alphabetical':
        versions.sort((a, b) => a.id.localeCompare(b.id))
        break
      case 'release_type':
        versions.sort((a, b) => {
          const typeOrder = { release: 0, beta: 1, alpha: 2, snapshot: 3, old_beta: 4, old_alpha: 5 }
          return (typeOrder[a.type as keyof typeof typeOrder] || 99) - (typeOrder[b.type as keyof typeof typeOrder] || 99)
        })
        break
    }
    
    set({ filteredVersions: versions })
  },
  
  getModLoaderVersions: async (modLoader, mcVersion) => {
    return modLoaderApi.getVersions(modLoader as any, mcVersion)
  },
  
  installModLoader: async (version, gameDir) => {
    return modLoaderApi.install(version, gameDir)
  },
  
  getInstalledModLoaders: async () => {
    return modLoaderApi.getInstalled()
  },
  
  detectJava: async () => {
    return javaApi.detect()
  },
  
  getJavaInstallations: async () => {
    return javaApi.getInstallations()
  },
  
  downloadJava: async (version, vendor, architecture) => {
    return javaApi.download(version, vendor, architecture)
  },
}))