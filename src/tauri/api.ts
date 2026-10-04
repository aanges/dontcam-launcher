import { invoke } from '@tauri-apps/api/core'
import type {
  Account,
  VersionManifest,
  GameVersion,
  Profile,
  Settings,
  ModLoaderVersion,
  InstalledModLoader,
  InstallModdedResult,
  JavaInstallation,
  LaunchOptions,
  LaunchStatus,
  ModLoaderType,
} from '../types'

// NOTE: Tauri v2 commands default to `rename_all = "camelCase"`,
// so multi-word argument keys must be camelCase here.
// (Nested struct fields like LaunchOptions stay snake_case via serde.)

// Auth API
export const authApi = {
  loginMicrosoft: (): Promise<Account> =>
    invoke('login_microsoft'),

  loginOffline: (username: string): Promise<Account> =>
    invoke('login_offline', { username }),

  logout: (accountId: string): Promise<void> =>
    invoke('logout', { accountId }),

  getAccounts: (): Promise<Account[]> =>
    invoke('get_accounts'),

  getCurrentAccount: (): Promise<Account | null> =>
    invoke('get_current_account'),

  setCurrentAccount: (accountId: string): Promise<Account> =>
    invoke('set_current_account', { accountId }),

  refreshToken: (accountId: string): Promise<Account> =>
    invoke('refresh_token', { accountId }),

  validateAccount: (accountId: string): Promise<boolean> =>
    invoke('validate_account', { accountId }),
}

// Version API
export const versionApi = {
  fetchManifest: (): Promise<VersionManifest> =>
    invoke('fetch_version_manifest'),

  getInstalled: (): Promise<GameVersion[]> =>
    invoke('get_installed_versions'),

  install: (versionId: string, force = false): Promise<GameVersion> =>
    invoke('install_version', { versionId, force }),

  ensureReady: (versionId: string): Promise<GameVersion> =>
    invoke('ensure_version_ready', { versionId }),

  uninstall: (versionId: string): Promise<void> =>
    invoke('uninstall_version', { versionId }),

  getDetails: (versionId: string): Promise<GameVersion> =>
    invoke('get_version_details', { versionId }),
}

// Profile API
export const profileApi = {
  create: (name: string, versionId: string, accountId?: string): Promise<Profile> =>
    invoke('create_profile', { name, versionId, accountId: accountId ?? null }),

  getAll: (): Promise<Profile[]> =>
    invoke('get_profiles'),

  get: (profileId: string): Promise<Profile | null> =>
    invoke('get_profile', { profileId }),

  update: (profile: Profile): Promise<Profile> =>
    invoke('update_profile', { profile }),

  delete: (profileId: string): Promise<void> =>
    invoke('delete_profile', { profileId }),

  duplicate: (profileId: string, newName: string): Promise<Profile> =>
    invoke('duplicate_profile', { profileId, newName }),
}

// Launch API
export const launchApi = {
  launch: (options: LaunchOptions): Promise<string> =>
    invoke('launch_game', { options }),

  getStatus: (): Promise<LaunchStatus> =>
    invoke('get_launch_status'),

  kill: (): Promise<void> =>
    invoke('kill_game'),
}

// Settings API
export const settingsApi = {
  get: (): Promise<Settings> =>
    invoke('get_settings'),

  update: (settings: Settings): Promise<Settings> =>
    invoke('update_settings', { settings }),

  reset: (): Promise<Settings> =>
    invoke('reset_settings'),
}

// Mod Loader API
export const modLoaderApi = {
  getVersions: (modLoader: ModLoaderType, mcVersion: string): Promise<ModLoaderVersion[]> =>
    invoke('get_modloader_versions', { modLoader, mcVersion }),

  install: (version: ModLoaderVersion, gameDir: string): Promise<InstalledModLoader> =>
    invoke('install_modloader', { version, gameDir }),

  getInstalled: (): Promise<InstalledModLoader[]> =>
    invoke('get_installed_modloaders'),

  resolveLoader: (mcVersion: string): Promise<ModLoaderVersion> =>
    invoke('resolve_loader', { mcVersion }),

  installModded: (versionId: string, force = false, profileId?: string): Promise<InstallModdedResult> =>
    invoke('install_modded', { versionId, force, profileId: profileId ?? null }),
}

// Java API
export const javaApi = {
  detect: (): Promise<JavaInstallation[]> =>
    invoke('detect_java'),

  getInstallations: (): Promise<JavaInstallation[]> =>
    invoke('get_java_installations'),

  download: (version: number, vendor: string, architecture: string): Promise<JavaInstallation> =>
    invoke('download_java', { version, vendor, architecture }),

  resolveForVersion: (versionId: string): Promise<JavaInstallation | null> =>
    invoke('resolve_java_for_version', { versionId }),
}

// DontCam mod (public aanges/MODY repo) API.
// No tokens, no passwords anywhere: the backend uses plain public HTTPS,
// falling back to the jars embedded in the launcher when MODY has none.
export interface DontcamRelease {
  mc_version: string
  folder: string
  loader: string
  assets: string[]
  source: string
}

export interface DontcamCheckResult {
  updated: boolean
  version: string
  path: string
  offline: boolean
  message: string
}

export const dontcamApi = {
  list: (): Promise<DontcamRelease[]> =>
    invoke('get_dontcam_releases'),

  install: (mcVersion: string, profileId?: string): Promise<string> =>
    invoke('install_dontcam_mod', { mcVersion, profileId: profileId ?? null }),

  check: (mcVersion: string, profileId?: string): Promise<DontcamCheckResult> =>
    invoke('check_dontcam_update', { mcVersion, profileId: profileId ?? null }),
}

// Utility API
export const utilsApi = {
  openFolder: (path: string): Promise<void> =>
    invoke('open_folder', { path }),

  getAppDataDir: (): Promise<string> =>
    invoke('get_app_data_dir'),

  getGameDir: (profileId?: string): Promise<string> =>
    invoke('get_game_dir', { profileId: profileId ?? null }),
}
