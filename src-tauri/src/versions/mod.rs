//! Mojang version manifest, vanilla installs (jar + libraries + assets),
//! and the installed-versions scan. Progress goes out as
//! `download-progress` events: { version_id, stage, current, total }.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use reqwest::Client;
use std::path::PathBuf;
use tauri::Emitter;

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

pub struct VersionManager {
    client: Client,
    manifest: Arc<RwLock<Option<VersionManifest>>>,
    installed_versions: Arc<RwLock<HashMap<String, GameVersion>>>,
}

impl VersionManager {
    pub async fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("DontCam-Client/0.1")
            .build()
            .unwrap();
        let _ = tokio::fs::create_dir_all(crate::common::versions_dir()).await;
        let _ = tokio::fs::create_dir_all(crate::common::libraries_dir()).await;
        let _ = tokio::fs::create_dir_all(crate::common::assets_dir()).await;
        Self {
            client,
            manifest: Arc::new(RwLock::new(None)),
            installed_versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn fetch_version_manifest(&self) -> Result<VersionManifest> {
        let response = self.client.get(MANIFEST_URL).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Manifest fetch failed: {}", response.status());
        }
        let manifest: VersionManifest = response.json().await?;
        *self.manifest.write().await = Some(manifest.clone());
        Ok(manifest)
    }

    pub async fn get_version_manifest(&self) -> Result<VersionManifest> {
        {
            let lock = self.manifest.read().await;
            if let Some(m) = lock.as_ref() {
                return Ok(m.clone());
            }
        }
        self.fetch_version_manifest().await
    }

    pub async fn scan_installed_versions(&self) -> Result<()> {
        let mut versions = self.installed_versions.write().await;
        versions.clear();
        let dir = crate::common::versions_dir();
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let version_json = entry.path().join(format!("{}.json", name));
            if !version_json.exists() {
                continue;
            }
            if let Ok(content) = tokio::fs::read_to_string(&version_json).await {
                if let Ok(details) = serde_json::from_str::<VersionDetails>(&content) {
                    versions.insert(name.clone(), GameVersion::from(details));
                    continue;
                }
                // Loader profile json (fabric/quilt/forge) — still counts as installed.
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if v.get("mainClass").is_some() || v.get("main_class").is_some() {
                        versions.insert(
                            name.clone(),
                            GameVersion {
                                id: name.clone(),
                                version_type: v
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("modded")
                                    .to_string(),
                                main_class: v
                                    .get("mainClass")
                                    .or_else(|| v.get("main_class"))
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                arguments: GameArguments { game: vec![], jvm: vec![] },
                                libraries: vec![],
                                asset_index: AssetIndex {
                                    id: String::new(),
                                    sha1: String::new(),
                                    size: 0,
                                    total_size: 0,
                                    url: String::new(),
                                },
                                downloads: Downloads { client: None, server: None },
                                java_version: None,
                            },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn get_installed_versions(&self) -> Result<Vec<GameVersion>> {
        let versions = self.installed_versions.read().await;
        let mut list: Vec<_> = versions.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(list)
    }

    /// Full install: json + client jar + libraries + assets.
    pub async fn install_version(
        &self,
        app: Option<tauri::AppHandle>,
        version_id: &str,
        force: bool,
    ) -> Result<GameVersion> {
        let manifest = self.get_version_manifest().await?;
        let version_info = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version not found: {}", version_id))?;

        if force {
            let version_dir = crate::common::versions_dir().join(version_id);
            if version_dir.exists() {
                tokio::fs::remove_dir_all(&version_dir).await?;
            }
        }

        emit_progress(&app, version_id, "metadata", 0, 3);
        let details: VersionDetails = self
            .client
            .get(&version_info.url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse version json")?;

        let version_dir = crate::common::versions_dir().join(version_id);
        tokio::fs::create_dir_all(&version_dir).await?;
        tokio::fs::write(
            version_dir.join(format!("{}.json", version_id)),
            serde_json::to_string_pretty(&details)?,
        )
        .await?;

        // Client jar
        emit_progress(&app, version_id, "client", 1, 4);
        if let Some(client) = details.downloads.client.as_ref() {
            let jar_path = version_dir.join(format!("{}.jar", version_id));
            self.download_verified(&client.url, &jar_path, Some(&client.sha1), client.size)
                .await?;
        } else if let Some(url) = details.downloads.client_url_fallback() {
            // very old versions: client url field differs
            let jar_path = version_dir.join(format!("{}.jar", version_id));
            self.download_file(&url, &jar_path).await?;
        }

        // Libraries
        emit_progress(&app, version_id, "libraries", 2, 4);
        self.download_libraries(&details, &app, version_id).await?;

        // Assets
        emit_progress(&app, version_id, "assets", 3, 4);
        self.download_assets(&details).await?;

        emit_progress(&app, version_id, "done", 4, 4);

        let game_version = GameVersion::from(details);
        self.installed_versions
            .write()
            .await
            .insert(version_id.to_string(), game_version.clone());
        Ok(game_version)
    }

    async fn download_libraries(
        &self,
        details: &VersionDetails,
        app: &Option<tauri::AppHandle>,
        version_id: &str,
    ) -> Result<()> {
        let libs_dir = crate::common::libraries_dir();
        let total = details.libraries.len();
        for (i, lib) in details.libraries.iter().enumerate() {
            if !library_allowed_on_current_os(lib) {
                continue;
            }
            // Modern natives entries share the same OS rules across
            // architectures — keep only this platform's jars.
            if let Some(name) = lib.name.as_deref() {
                if crate::common::is_natives_library(name) {
                    let classifier = name.split(':').nth(3).unwrap_or("");
                    if !crate::common::native_classifier_matches(classifier) {
                        continue;
                    }
                }
            }
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let dest = libs_dir.join(artifact.local_path());
                    self.download_verified(&artifact.url, &dest, Some(&artifact.sha1), artifact.size)
                        .await?;
                }
                // natives classifier for current os
                if let Some(classifiers) = &downloads.classifiers {
                    if let Some(native) = pick_native(classifiers) {
                        let dest = libs_dir.join(native.local_path());
                        self.download_verified(&native.url, &dest, Some(&native.sha1), native.size)
                            .await?;
                    }
                }
            } else if let Some(name) = lib.name.as_ref() {
                // Legacy lib without downloads: build URL from maven coordinates.
                if let Some((url, path)) = legacy_lib_url(name) {
                    let dest = libs_dir.join(&path);
                    if !dest.exists() {
                        self.download_file(&url, &dest).await?;
                    }
                }
            }
            if i % 10 == 0 {
                emit_progress(app, version_id, "libraries", i, total.max(1));
            }
        }
        Ok(())
    }

    async fn download_assets(&self, details: &VersionDetails) -> Result<()> {
        let Some(asset_index) = details.asset_index.as_ref() else {
            return Ok(());
        };
        let assets = crate::common::assets_dir();
        let indexes_dir = assets.join("indexes");
        let objects_dir = assets.join("objects");
        tokio::fs::create_dir_all(&indexes_dir).await?;
        tokio::fs::create_dir_all(&objects_dir).await?;

        let index_path = indexes_dir.join(format!("{}.json", asset_index.id));
        self.download_verified(&asset_index.url, &index_path, Some(&asset_index.sha1), asset_index.size)
            .await?;

        let content = tokio::fs::read_to_string(&index_path).await?;
        let index: serde_json::Value = serde_json::from_str(&content)?;
        let objects = index["objects"].as_object().cloned().unwrap_or_default();

        // Download objects concurrently (bounded).
        let sem = Arc::new(tokio::sync::Semaphore::new(8));
        let mut handles = Vec::new();
        for (_name, obj) in objects {
            let (hash, size) = (
                obj["hash"].as_str().unwrap_or("").to_string(),
                obj["size"].as_u64().unwrap_or(0),
            );
            if hash.len() < 2 {
                continue;
            }
            let dest = objects_dir.join(&hash[0..2]).join(&hash);
            if dest.exists() {
                if dest.metadata().map(|m| m.len()).unwrap_or(0) == size && size > 0 {
                    continue;
                }
            }
            let url = format!("{}/{}/{}", RESOURCES_URL, &hash[0..2], hash);
            let client = self.client.clone();
            let permit_slot = sem.clone();
            handles.push(tokio::spawn(async move {
                let _permit = permit_slot.acquire_owned().await;
                if let Some(parent) = dest.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                if dest.exists() {
                    return Ok::<(), anyhow::Error>(());
                }
                let bytes = client.get(&url).send().await?.bytes().await?;
                tokio::fs::write(&dest, bytes).await?;
                Ok::<(), anyhow::Error>(())
            }));
            if handles.len() >= 64 {
                for h in handles.drain(..) {
                    let _ = h.await;
                }
            }
        }
        for h in handles {
            let _ = h.await;
        }
        Ok(())
    }

    async fn download_file(&self, url: &str, path: &PathBuf) -> Result<()> {
        if path.exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Download failed {}: {}", url, response.status());
        }
        let bytes = response.bytes().await?;
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    async fn download_verified(
        &self,
        url: &str,
        path: &PathBuf,
        sha1: Option<&str>,
        expected_size: u64,
    ) -> Result<()> {
        if path.exists() {
            if expected_size > 0 {
                if let Ok(meta) = tokio::fs::metadata(path).await {
                    if meta.len() == expected_size {
                        return Ok(());
                    }
                }
            } else if sha1.map_or(true, |h| h.is_empty()) {
                return Ok(());
            }
            // verify hash if we have it
            if let Some(hash) = sha1 {
                if !hash.is_empty() {
                    if let Ok(data) = tokio::fs::read(path).await {
                        if verify_sha1(&data, hash) {
                            return Ok(());
                        }
                    }
                }
            } else {
                return Ok(());
            }
        }
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Download failed {}: {}", url, response.status());
        }
        let bytes = response.bytes().await?;
        if let Some(hash) = sha1 {
            if !hash.is_empty() && !verify_sha1(&bytes, hash) {
                anyhow::bail!("SHA1 mismatch for {}", url);
            }
        }
        tokio::fs::write(path, &bytes).await?;
        Ok(())
    }

    pub async fn uninstall_version(&self, version_id: &str) -> Result<()> {
        let version_dir = crate::common::versions_dir().join(version_id);
        if version_dir.exists() {
            tokio::fs::remove_dir_all(version_dir).await?;
        }
        self.installed_versions.write().await.remove(version_id);
        Ok(())
    }

    pub async fn get_version_details(&self, version_id: &str) -> Result<GameVersion> {
        {
            let installed = self.installed_versions.read().await;
            if let Some(version) = installed.get(version_id) {
                return Ok(version.clone());
            }
        }
        // try loading json from disk
        let json_path = crate::common::versions_dir()
            .join(version_id)
            .join(format!("{}.json", version_id));
        if json_path.exists() {
            let content = tokio::fs::read_to_string(&json_path).await?;
            if let Ok(details) = serde_json::from_str::<VersionDetails>(&content) {
                let gv = GameVersion::from(details);
                self.installed_versions
                    .write()
                    .await
                    .insert(version_id.to_string(), gv.clone());
                return Ok(gv);
            }
        }
        self.install_version(None, version_id, false).await
    }
}

// ---------- data types ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(alias = "releaseTime")]
    pub release_time: DateTime<Utc>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default, alias = "complianceLevel")]
    pub compliance_level: Option<u32>,
}

/// Tolerant version json: supports modern (arguments) and legacy (minecraftArguments).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    #[serde(rename = "type", default = "default_release_type")]
    pub type_: String,
    #[serde(rename = "mainClass", alias = "main_class")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<GameArguments>,
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(rename = "assetIndex", default)]
    pub asset_index: Option<AssetIndex>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(default)]
    pub downloads: DownloadsOpt,
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub minimum_launcher_version: Option<u32>,
}

fn default_release_type() -> String {
    "release".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadsOpt {
    #[serde(default)]
    pub client: Option<Artifact>,
    #[serde(default)]
    pub server: Option<Artifact>,
    // very old format
    #[serde(default, rename = "client_url")]
    pub client_url: Option<String>,
}

impl DownloadsOpt {
    fn client_url_fallback(&self) -> Option<String> {
        self.client_url.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameArguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Ruled { rules: Vec<Rule>, value: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Library {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(default)]
    pub extract: Option<ExtractRule>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    // legacy: direct url
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    // NOTE: very old version JSONs (e.g. 1.8.9) omit `path` — use local_path().
    #[serde(default)]
    pub path: String,
    pub url: String,
    #[serde(default)]
    pub sha1: String,
    #[serde(default)]
    pub size: u64,
}

impl Artifact {
    /// Local relative path under libraries/. Modern JSONs carry `path`;
    /// legacy ones only have `url` (e.g. https://libraries.minecraft.net/<path>).
    pub fn local_path(&self) -> String {
        if !self.path.is_empty() {
            return self.path.clone();
        }
        if let Ok(url) = url::Url::parse(&self.url) {
            let p = url.path().trim_start_matches('/').to_string();
            if !p.is_empty() && !p.ends_with('/') {
                return p;
            }
        }
        self.url
            .rsplit('/')
            .next()
            .unwrap_or("unknown.jar")
            .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRule {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize", default)]
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

// Frontend-facing simplified version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub id: String,
    pub version_type: String,
    pub main_class: String,
    pub arguments: GameArguments,
    pub libraries: Vec<Library>,
    pub asset_index: AssetIndex,
    pub downloads: Downloads,
    pub java_version: Option<JavaVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Downloads {
    pub client: Option<Artifact>,
    pub server: Option<Artifact>,
}

impl From<VersionDetails> for GameVersion {
    fn from(d: VersionDetails) -> Self {
        let asset_index = d.asset_index.clone().unwrap_or(AssetIndex {
            id: d.assets.clone().unwrap_or_else(|| "legacy".to_string()),
            sha1: String::new(),
            size: 0,
            total_size: 0,
            url: String::new(),
        });
        Self {
            id: d.id.clone(),
            version_type: d.type_.clone(),
            main_class: d.main_class.clone(),
            arguments: d.arguments.clone().unwrap_or_default(),
            libraries: d.libraries.clone(),
            asset_index,
            downloads: Downloads {
                client: d.downloads.client.clone(),
                server: d.downloads.server.clone(),
            },
            java_version: d.java_version.clone(),
        }
    }
}

// ---------- helpers ----------

fn verify_sha1(data: &[u8], expected: &str) -> bool {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize()).eq_ignore_ascii_case(expected)
}

fn emit_progress(
    app: &Option<tauri::AppHandle>,
    version_id: &str,
    stage: &str,
    current: usize,
    total: usize,
) {
    if let Some(app) = app {
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "version_id": version_id,
                "stage": stage,
                "current": current,
                "total": total,
            }),
        );
    }
}

pub fn library_allowed_on_current_os(lib: &Library) -> bool {
    let Some(rules) = &lib.rules else {
        return true;
    };
    // vanilla rule evaluation: last matching rule wins; default allow
    let mut allowed = true;
    for rule in rules {
        if rule_matches_current_os(rule) {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn rule_matches_current_os(rule: &Rule) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            let current = crate::common::current_os_name();
            // Mojang uses "osx" for macos, "windows", "linux"
            let matches = match name.as_str() {
                "windows" => current == "windows",
                "osx" | "macos" => current == "osx",
                "linux" => current == "linux",
                _ => false,
            };
            if !matches {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            let current_arch = if cfg!(target_arch = "x86_64") {
                "x64"
            } else if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "x86"
            };
            // Mojang arch values: x86, x64...
            if arch != current_arch && !(arch == "x86_64" && current_arch == "x64") {
                return false;
            }
        }
        // os.version regex matching skipped (rarely used)
    }
    // features (is_demo_user, ...) — treat demo=false
    if let Some(features) = &rule.features {
        for (k, v) in features {
            if k == "is_demo_user" && *v {
                return false;
            }
        }
    }
    true
}

fn pick_native(classifiers: &HashMap<String, Artifact>) -> Option<Artifact> {
    let os = crate::common::current_os_name();
    // Mojang keys: natives-windows, natives-osx, natives-linux, natives-windows-64, etc.
    let candidates: Vec<String> = match os {
        "windows" => vec!["natives-windows-64".to_string(), "natives-windows".to_string()],
        "osx" => vec![
            "natives-macos-arm64".to_string(),
            "natives-macos".to_string(),
            "natives-osx".to_string(),
        ],
        _ => vec!["natives-linux".to_string()],
    };
    for key in candidates {
        if let Some(a) = classifiers.get(&key) {
            return Some(a.clone());
        }
        // fallback: any key starting with natives-<os>
        let prefix = format!("natives-{}", if os == "osx" { "osx" } else { os });
        for (k, v) in classifiers {
            if k.starts_with(&prefix) || (os == "osx" && k.starts_with("natives-macos")) {
                return Some(v.clone());
            }
        }
    }
    None
}

/// Build download URL for legacy libraries that only have maven coordinates.
fn legacy_lib_url(coords: &str) -> Option<(String, String)> {
    // coords: group:artifact:version[:classifier]
    let parts: Vec<&str> = coords.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let classifier = parts.get(3);
    let group_path = group.replace('.', "/");
    let filename = if let Some(c) = classifier {
        format!("{}-{}-{}.jar", artifact, version, c)
    } else {
        format!("{}-{}.jar", artifact, version)
    };
    let path = format!("{}/{}/{}/{}", group_path, artifact, version, filename);
    let url = format!("https://libraries.minecraft.net/{}", path);
    Some((url, path))
}

// ---------- commands ----------

#[tauri::command]
pub async fn fetch_version_manifest(
    state: tauri::State<'_, crate::AppState>,
) -> Result<VersionManifest, String> {
    state
        .versions
        .fetch_version_manifest()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_installed_versions(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<GameVersion>, String> {
    state
        .versions
        .get_installed_versions()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_version(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    version_id: String,
    force: Option<bool>,
) -> Result<GameVersion, String> {
    state
        .versions
        .install_version(Some(app), &version_id, force.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ensure_version_ready(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    version_id: String,
) -> Result<GameVersion, String> {
    // idempotent: returns installed or installs
    state
        .versions
        .get_version_details(&version_id)
        .await
        .map_err(|e| e.to_string())?;
    // make sure all files present (re-run install is cheap due to skip-if-exists)
    state
        .versions
        .install_version(Some(app), &version_id, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_version(
    state: tauri::State<'_, crate::AppState>,
    version_id: String,
) -> Result<(), String> {
    state
        .versions
        .uninstall_version(&version_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_version_details(
    state: tauri::State<'_, crate::AppState>,
    version_id: String,
) -> Result<GameVersion, String> {
    state
        .versions
        .get_version_details(&version_id)
        .await
        .map_err(|e| e.to_string())
}
