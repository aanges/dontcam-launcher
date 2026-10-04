export interface Account {
  id: string
  username: string
  uuid: string
  access_token?: string
  refresh_token?: string
  token_expires_at?: string
  account_type: 'microsoft' | 'offline'
  skin_url?: string
  cape_url?: string
  created_at: string
  last_used: string
}

export interface VersionManifest {
  latest: {
    release: string
    snapshot: string
  }
  versions: VersionInfo[]
}

export interface VersionInfo {
  id: string
  type: string
  url: string
  time: string
  release_time: string
  sha1?: string
  compliance_level?: number
}

export interface GameVersion {
  id: string
  version_type: string
  main_class: string
  arguments: GameArguments
  libraries: Library[]
  asset_index: AssetIndex
  downloads: Downloads
  java_version?: JavaVersion
}

export interface GameArguments {
  game: Argument[]
  jvm: Argument[]
}

export type Argument = string | { rules: Rule[]; value: string[] }

export interface Rule {
  action: string
  os?: OsRule
  features?: Record<string, boolean>
}

export interface OsRule {
  name: string
  version?: string
  arch?: string
}

export interface Library {
  name: string
  downloads?: LibraryDownloads
  rules?: Rule[]
  extract?: ExtractRule
  natives?: Record<string, string>
}

export interface LibraryDownloads {
  artifact?: Artifact
  classifiers?: Record<string, Artifact>
}

export interface Artifact {
  path: string
  url: string
  sha1: string
  size: number
}

export interface ExtractRule {
  exclude: string[]
}

export interface AssetIndex {
  id: string
  sha1: string
  size: number
  total_size: number
  url: string
}

export interface Downloads {
  client: Artifact
  server?: Artifact
}

export interface JavaVersion {
  component: string
  major_version: number
}

export interface Profile {
  id: string
  name: string
  icon?: string
  version_id: string
  mod_loader: ModLoaderType
  mod_loader_version?: string
  account_id?: string
  /** Install the bundled DontCam client mod (1.8.x Forge) into the instance. */
  dontcam_mod: boolean
  java_args: string
  game_args: string[]
  resolution?: Resolution
  mods: ModEntry[]
  resource_packs: ResourcePackEntry[]
  created_at: string
  updated_at: string
  last_played?: string
  play_time: number
}

export interface Resolution {
  width: number
  height: number
  fullscreen: boolean
}

export interface ModEntry {
  id: string
  name: string
  version: string
  file_path: string
  enabled: boolean
  mod_loader: ModLoaderType
  dependencies: string[]
}

export type ModLoaderType = 'forge' | 'fabric' | 'quilt' | 'neoforge' | 'liteloader' | 'none'

export interface ResourcePackEntry {
  id: string
  name: string
  file_path: string
  enabled: boolean
  priority: number
}

export interface Settings {
  theme: Theme
  language: string
  java: JavaSettings
  game: GameSettings
  network: NetworkSettings
  ui: UISettings
  advanced: AdvancedSettings
}

export type Theme = 'light' | 'dark' | 'system'

export interface JavaSettings {
  auto_detect: boolean
  preferred_version?: string
  custom_java_path?: string
  jvm_args: string
  memory_allocation: MemoryAllocation
}

export interface MemoryAllocation {
  min: number
  max: number
  unit: 'MB' | 'GB'
}

export interface GameSettings {
  default_game_dir?: string
  auto_close_launcher: boolean
  keep_launcher_open: boolean
  custom_resolution?: Resolution
  fullscreen: boolean
  vsync: boolean
  fov: number
  render_distance: number
  max_fps: number
  enable_mods: boolean
  enable_resource_packs: boolean
}

export interface NetworkSettings {
  proxy_enabled: boolean
  proxy_host: string
  proxy_port: number
  proxy_username?: string
  proxy_password?: string
  download_threads: number
  bandwidth_limit?: number
}

export interface UISettings {
  show_snapshots: boolean
  show_old_versions: boolean
  show_alpha_beta: boolean
  sort_versions_by: VersionSort
  compact_mode: boolean
  animations: boolean
  background_blur: boolean
  news_enabled: boolean
}

export type VersionSort = 'newest_first' | 'oldest_first' | 'alphabetical' | 'release_type'

export interface AdvancedSettings {
  debug_logging: boolean
  console_enabled: boolean
  custom_game_args: string[]
  environment_variables: Record<string, string>
  pre_launch_command?: string
  post_exit_command?: string
  verify_downloads: boolean
  parallel_downloads: number
}

export interface ModLoaderVersion {
  id: string
  version: string
  mc_version: string
  mod_loader: ModLoaderType
  download_url: string
  file_size: number
  sha1?: string
}

export interface InstalledModLoader {
  id: string
  version: string
  mc_version: string
  mod_loader: ModLoaderType
  installed_at: string
}

export interface InstallModdedResult {
  version_id: string
  loader: ModLoaderType
  modded_version_id?: string | null
  loader_error?: string | null
}

export interface JavaInstallation {
  path: string
  version: JavaVersionInfo
  vendor: JavaVendor
  architecture: JavaArchitecture
  source: JavaSource
}

export interface JavaVersionInfo {
  major: number
  full: string
}

export type JavaVendor = 
  | 'eclipse_adoptium'
  | 'microsoft'
  | 'amazon_corretto'
  | 'azul_zulu'
  | 'bellsoft_liberica'
  | 'oracle'
  | 'openjdk'
  | 'unknown'

export type JavaArchitecture = 'x64' | 'x86' | 'arm64'

export type JavaSource = 'system' | 'path' | 'detected' | 'managed' | 'custom'

export type LaunchStatus = 
  | 'idle'
  | 'preparing'
  | 'downloading_assets'
  | 'downloading_libraries'
  | 'launching'
  | 'running'
  | { error: string }
  | { crashed: string }

export interface LaunchOptions {
  profile_id: string
  version_id: string
  account: Account
  java_path: string
  jvm_args: string[]
  game_args: string[]
  game_dir: string
  resolution?: Resolution
  server?: ServerInfo
}

export interface ServerInfo {
  host: string
  port: number
}