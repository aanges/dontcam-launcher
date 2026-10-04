use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use reqwest::Client;
use std::path::PathBuf;

pub struct ModLoaderManager {
    client: Client,
    installed: Arc<RwLock<HashMap<String, InstalledModLoader>>>,
}

impl ModLoaderManager {
    pub async fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("DontCam-Client/0.1")
            .build()
            .unwrap();
        Self {
            client,
            installed: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_modloader_versions(
        &self,
        mod_loader: ModLoaderType,
        mc_version: &str,
    ) -> Result<Vec<ModLoaderVersion>> {
        match mod_loader {
            ModLoaderType::Forge => self.get_forge_versions(mc_version).await,
            ModLoaderType::Fabric => self.get_fabric_versions(mc_version).await,
            ModLoaderType::Quilt => self.get_quilt_versions(mc_version).await,
            ModLoaderType::NeoForge => self.get_neoforge_versions(mc_version).await,
            _ => Ok(vec![]),
        }
    }

    /// NeoForge version prefix for an MC version: "1.21.1" -> "21.1".
    /// Takes the first two numeric components after stripping a leading "1.".
    fn neoforge_prefix(mc_version: &str) -> String {
        let base = mc_version.split('-').next().unwrap_or(mc_version);
        let stripped = base.strip_prefix("1.").unwrap_or(base);
        let mut parts = stripped.split('.');
        match (parts.next(), parts.next()) {
            (Some(a), Some(b)) => format!("{}.{}", a, b),
            (Some(a), None) => a.to_string(),
            _ => stripped.to_string(),
        }
    }

    async fn get_neoforge_versions(&self, mc_version: &str) -> Result<Vec<ModLoaderVersion>> {
        let prefix = Self::neoforge_prefix(mc_version);
        let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
        let text = self.client.get(url).send().await?.text().await?;
        let mut versions: Vec<String> = Vec::new();
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with("<version>") && t.ends_with("</version>") {
                let v = t
                    .trim_start_matches("<version>")
                    .trim_end_matches("</version>")
                    .to_string();
                // NeoForge 21.1.x targets MC 1.21.1; match "21.1." or exact "21.1"
                if v == prefix || v.starts_with(&format!("{}.", prefix)) {
                    versions.push(v);
                }
            }
        }
        sort_versions_desc(&mut versions);
        Ok(versions
            .into_iter()
            .take(20)
            .map(|v| ModLoaderVersion {
                id: format!("neoforge-{}", v),
                version: v.clone(),
                mc_version: mc_version.to_string(),
                mod_loader: ModLoaderType::NeoForge,
                download_url: format!(
                    "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
                    v, v
                ),
                file_size: 0,
                sha1: None,
            })
            .collect())
    }

    async fn get_forge_versions(&self, mc_version: &str) -> Result<Vec<ModLoaderVersion>> {
        // 1. promotions_slim.json for latest/recommended
        let mut out: Vec<ModLoaderVersion> = Vec::new();
        if let Ok(resp) = self
            .client
            .get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
            .send()
            .await
        {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(promos) = json.get("promos").and_then(|p| p.as_object()) {
                    for suffix in ["recommended", "latest"] {
                        let key = format!("{}-{}", mc_version, suffix);
                        if let Some(v) = promos.get(&key).and_then(|v| v.as_str()) {
                            let full = format!("{}-{}", mc_version, v);
                            out.push(ModLoaderVersion {
                                id: format!("forge-{}", full),
                                version: full.clone(),
                                mc_version: mc_version.to_string(),
                                mod_loader: ModLoaderType::Forge,
                                download_url: format!(
                                    "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
                                    full, full
                                ),
                                file_size: 0,
                                sha1: None,
                            });
                        }
                    }
                }
            }
        }
        // 2. maven-metadata.xml for fuller list (last 20)
        if let Ok(resp) = self
            .client
            .get("https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml")
            .send()
            .await
        {
            if let Ok(text) = resp.text().await {
                let mut versions: Vec<String> = Vec::new();
                for line in text.lines() {
                    let t = line.trim();
                    if t.starts_with("<version>") && t.ends_with("</version>") {
                        let v = t
                            .trim_start_matches("<version>")
                            .trim_end_matches("</version>")
                            .to_string();
                        if v.starts_with(&format!("{}-", mc_version)) {
                            versions.push(v);
                        }
                    }
                }
                versions.sort();
                for v in versions.iter().rev().take(20) {
                    if out.iter().any(|e| &e.version == v) {
                        continue;
                    }
                    out.push(ModLoaderVersion {
                        id: format!("forge-{}", v),
                        version: v.clone(),
                        mc_version: mc_version.to_string(),
                        mod_loader: ModLoaderType::Forge,
                        download_url: format!(
                            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
                            v, v
                        ),
                        file_size: 0,
                        sha1: None,
                    });
                }
            }
        }
        Ok(out)
    }

    async fn get_fabric_versions(&self, mc_version: &str) -> Result<Vec<ModLoaderVersion>> {
        // Per-game loader list (only loaders compatible with this MC version).
        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let data: Vec<serde_json::Value> = resp.json().await?;
        // NOTE: the endpoint does NOT filter by compatibility — newest loaders
        // can break old game versions. Cap per line to era-correct loaders.
        let cap = Self::max_fabric_loader(mc_version);
        let mut out = Vec::new();
        for item in data {
            if out.len() >= 10 {
                break;
            }
            let loader_ver = item["loader"]["version"].as_str().unwrap_or("").to_string();
            if loader_ver.is_empty() {
                continue;
            }
            if let Some(max) = cap {
                if version_key(&loader_ver) > version_key(max) {
                    continue;
                }
            }
            out.push(ModLoaderVersion {
                id: format!("fabric-loader-{}-{}", loader_ver, mc_version),
                version: loader_ver.clone(),
                mc_version: mc_version.to_string(),
                mod_loader: ModLoaderType::Fabric,
                download_url: format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                    mc_version, loader_ver
                ),
                file_size: 0,
                sha1: None,
            });
        }
        Ok(out)
    }

    /// Newest Fabric Loader line known-good for an MC line (endpoint is unfiltered).
    /// Floors come from the mods themselves: every bundled DontCam Fabric mod
    /// declares `fabricloader >= 0.15.11` (1.21 needs >= 0.19.5), and each
    /// line's current Fabric API declares its own minimum (1.20's 0.92.x
    /// needs >= 0.16.10 — capping 1.20 at 0.15.x installs a loader the API
    /// rejects with an "Incompatible mods" screen).
    fn max_fabric_loader(mc_version: &str) -> Option<&'static str> {
        let base = mc_version.split('-').next().unwrap_or(mc_version);
        if base.starts_with("1.16.") || base == "1.16" {
            Some("0.15.11")
        } else if base.starts_with("1.17.") || base == "1.17" {
            Some("0.15.11")
        } else if base.starts_with("1.18.") || base == "1.18" {
            Some("0.15.11")
        } else if base.starts_with("1.19.") || base == "1.19" {
            Some("0.15.11")
        } else if base.starts_with("1.20.") || base == "1.20" {
            Some("0.16.14")
        } else {
            None
        }
    }

    async fn get_quilt_versions(&self, mc_version: &str) -> Result<Vec<ModLoaderVersion>> {
        let url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let data: Vec<serde_json::Value> = resp.json().await?;
        let mut out = Vec::new();
        for item in data.into_iter().take(10) {
            // quilt meta v3 loader list for a game version: [{loader: {version}, ...}]
            let loader_ver = item
                .get("loader")
                .and_then(|l| l.get("version"))
                .and_then(|v| v.as_str())
                .or_else(|| item.get("version").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();
            if loader_ver.is_empty() {
                continue;
            }
            out.push(ModLoaderVersion {
                id: format!("quilt-loader-{}-{}", loader_ver, mc_version),
                version: loader_ver.clone(),
                mc_version: mc_version.to_string(),
                mod_loader: ModLoaderType::Quilt,
                download_url: format!(
                    "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
                    mc_version, loader_ver
                ),
                file_size: 0,
                sha1: None,
            });
        }
        Ok(out)
    }

    pub async fn install_modloader(
        &self,
        version: &ModLoaderVersion,
        game_dir: &PathBuf,
    ) -> Result<InstalledModLoader> {
        match version.mod_loader {
            ModLoaderType::Fabric | ModLoaderType::Quilt => {
                self.install_profile_loader(version, game_dir).await
            }
            ModLoaderType::Forge | ModLoaderType::NeoForge => {
                self.install_jar_installer(version, game_dir).await
            }
            _ => anyhow::bail!("Unsupported mod loader"),
        }
    }

    async fn install_profile_loader(
        &self,
        version: &ModLoaderVersion,
        game_dir: &PathBuf,
    ) -> Result<InstalledModLoader> {
        let profile: serde_json::Value = self
            .client
            .get(&version.download_url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to download loader profile")?;

        let version_id = profile["id"].as_str().unwrap_or(&version.id).to_string();
        let version_dir = crate::common::versions_dir().join(&version_id);
        tokio::fs::create_dir_all(&version_dir).await?;
        tokio::fs::write(
            version_dir.join(format!("{}.json", version_id)),
            serde_json::to_string_pretty(&profile)?,
        )
        .await?;

        // libraries from profile
        if let Some(libs) = profile.get("libraries").and_then(|l| l.as_array()) {
            for lib in libs {
                let name = lib["name"].as_str().unwrap_or("");
                // downloads.artifact
                if let Some(artifact) = lib.get("downloads").and_then(|d| d.get("artifact")) {
                    let (url, path) = (
                        artifact["url"].as_str().unwrap_or(""),
                        artifact["path"].as_str().unwrap_or(""),
                    );
                    if url.is_empty() || path.is_empty() {
                        continue;
                    }
                    // respect rules
                    if let Some(rules) = lib.get("rules") {
                        if !rules_allowed(rules) {
                            continue;
                        }
                    }
                    let dest = crate::common::libraries_dir().join(path);
                    if dest.exists() {
                        continue;
                    }
                    if let Some(parent) = dest.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    let bytes = self.client.get(url).send().await?.bytes().await?;
                    tokio::fs::write(&dest, bytes).await?;
                } else if !name.is_empty() {
                    // maven coords fallback (fabric uses maven urls)
                    if let Some(url) = lib.get("url").and_then(|u| u.as_str()) {
                        if let Some(path) = maven_coords_to_path(name) {
                            let dest = crate::common::libraries_dir().join(&path);
                            if dest.exists() {
                                continue;
                            }
                            if let Some(parent) = dest.parent() {
                                tokio::fs::create_dir_all(parent).await?;
                            }
                            let full = format!("{}{}", url.trim_end_matches('/'), format!("/{}", path));
                            if let Ok(resp) = self.client.get(&full).send().await {
                                if resp.status().is_success() {
                                    let bytes = resp.bytes().await?;
                                    tokio::fs::write(&dest, bytes).await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        // copy vanilla jar reference: fabric profile usually has inheritsFrom — nothing to copy.
        let _ = game_dir;

        let installed = InstalledModLoader {
            id: version_id.clone(),
            version: version.version.clone(),
            mc_version: version.mc_version.clone(),
            mod_loader: version.mod_loader.clone(),
            installed_at: chrono::Utc::now(),
        };
        let mut map = self.installed.write().await;
        map.insert(version_id.clone(), installed.clone());
        drop(map);
        // Remove superseded profiles of the same loader line
        // (e.g. fabric-loader-0.15.11-1.20.1 after 0.16.14 lands), so the
        // Installed list doesn't accumulate stale, possibly incompatible
        // loader entries. Anchored to `fabric-loader-*-<mc>` / quilt
        // equivalent — never touches vanilla or other loaders.
        prune_superseded_profiles(&version_id, &version.mc_version).await;
        Ok(installed)
    }

    /// Forge + NeoForge: download the official installer jar and run it headless.
    /// Returns the REAL installed version id discovered in versions/ afterwards
    /// (e.g. `1.20.1-forge-47.2.0`), so launch can target the merged profile.
    async fn install_jar_installer(
        &self,
        version: &ModLoaderVersion,
        game_dir: &PathBuf,
    ) -> Result<InstalledModLoader> {
        let loader_name = match version.mod_loader {
            ModLoaderType::NeoForge => "NeoForge",
            _ => "Forge",
        };
        // Download installer
        let installer_path =
            std::env::temp_dir().join(format!("{}-installer.jar", version.id));
        let bytes = self
            .client
            .get(&version.download_url)
            .send()
            .await?
            .bytes()
            .await
            .with_context(|| format!("Failed to download {} installer", loader_name))?;
        tokio::fs::write(&installer_path, bytes).await?;

        // Find java matching this MC version (legacy installers need old Java)
        let java = find_java_for_mc(&version.mc_version)
            .await
            .unwrap_or_else(|| PathBuf::from("java"));

        let before = list_version_dirs().await;
        // Old installers (<=1.12 era) use --installClient, modern ones --install-client.
        let legacy = mc_is_legacy(&version.mc_version);
        let mc_base = crate::common::base_dir();
        // Legacy installers refuse to run without a vanilla launcher profile
        // ("There is no minecraft launcher profile ... you need to run the
        // launcher first!"). Our base dir is not the vanilla launcher dir,
        // so plant a minimal profile file when it's missing.
        if legacy {
            ensure_launcher_profile(&mc_base).await;
        }
        let mut install_cmd = tokio::process::Command::new(&java);
        install_cmd.arg("-jar").arg(&installer_path);
        if legacy {
            install_cmd
                .arg("--installClient")
                .arg(mc_base.to_string_lossy().to_string());
        } else {
            install_cmd
                .arg("--install-client")
                .arg(mc_base.to_string_lossy().to_string());
        }
        crate::common::hide_console_tokio(&mut install_cmd);
        let output = install_cmd
            .output()
            .await
            .with_context(|| format!("Failed to run {} installer", loader_name))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            // trim giant logs, keep the tail where the actual error usually is
            let combined = format!("{} {}", out, err);
            let tail: String = combined.chars().rev().take(2000).collect::<String>().chars().rev().collect();
            anyhow::bail!("{} installer failed: {}", loader_name, tail.trim());
        }
        let _ = game_dir;
        let _ = tokio::fs::remove_file(&installer_path).await;

        let installed_id = discover_new_version(&before)
            .await
            .unwrap_or_else(|| version.id.clone());
        let installed = InstalledModLoader {
            id: installed_id.clone(),
            version: version.version.clone(),
            mc_version: version.mc_version.clone(),
            mod_loader: version.mod_loader.clone(),
            installed_at: chrono::Utc::now(),
        };
        let mut map = self.installed.write().await;
        map.insert(installed_id, installed.clone());
        Ok(installed)
    }

    pub async fn get_installed_modloaders(&self) -> Result<Vec<InstalledModLoader>> {
        let installed = self.installed.read().await;
        let mut list: Vec<_> = installed.values().cloned().collect();
        list.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
        Ok(list)
    }

    /// Ensure Fabric API for this MC version is present in the instance mods dir.
    /// Our bundled Fabric mod hard-depends on it. Jars built for a different
    /// MC line are removed (they crash the game), then the right one downloads.
    /// Corrupt files (not a ZIP, e.g. a saved error page or partial download)
    /// are also removed and re-downloaded — the loader silently ignores them
    /// and then reports the API as missing. Returns true when it downloaded now.
    pub async fn ensure_fabric_api(
        &self,
        mods_dir: &PathBuf,
        mc_version: &str,
    ) -> Result<bool> {
        tokio::fs::create_dir_all(mods_dir).await.ok();
        let tag = format!("+{}.jar", mc_version);
        let alt_tag = format!("-{}.jar", mc_version);
        // Some API builds are named per minor line (e.g. `+1.16.jar` instead
        // of `+1.16.5.jar`) — accept those too so we don't re-download API
        // on every launch.
        let base = mc_version.split('-').next().unwrap_or(mc_version);
        let mut parts = base.split('.');
        let minor_tag = match (parts.next(), parts.next()) {
            (Some(a), Some(b)) => format!("+{}.{}.jar", a, b),
            _ => String::new(),
        };
        let mut matching = false;
        if let Ok(mut entries) = tokio::fs::read_dir(mods_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("fabric-api-") && name.ends_with(".jar") {
                    let for_this_line = name.ends_with(&tag)
                        || name.ends_with(&alt_tag)
                        || (!minor_tag.is_empty() && name.ends_with(&minor_tag));
                    if for_this_line && file_is_zip(entry.path()).await {
                        matching = true;
                    } else {
                        // API built for another MC line — would crash on load —
                        // or a corrupt file the loader ignores and then reports
                        // as a missing dependency.
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
        if matching {
            return Ok(false);
        }

        // Latest release for this game version + loader, via Modrinth.
        let url = format!(
            "https://api.modrinth.com/v2/project/fabric-api/version?game_versions=%5B%22{}%22%5D&loaders=%5B%22fabric%22%5D&limit=5",
            mc_version
        );
        let list: Vec<serde_json::Value> = self
            .client
            .get(&url)
            .header("User-Agent", "DontCam-Client/0.1.0 (contact: local)")
            .send()
            .await?
            .json()
            .await?;
        let entry = list
            .iter()
            .find(|v| v["version_type"].as_str() == Some("release"))
            .or(list.first())
            .ok_or_else(|| anyhow::anyhow!("No Fabric API build for Minecraft {}", mc_version))?;
        let files = entry["files"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Fabric API entry has no files"))?;
        let file = files
            .iter()
            .find(|f| f["primary"].as_bool() == Some(true))
            .or(files.first())
            .ok_or_else(|| anyhow::anyhow!("Fabric API entry has no files"))?;
        let file_url = file["url"].as_str().unwrap_or("");
        let filename = file["filename"].as_str().unwrap_or("");
        let expected_size = file["size"].as_u64().unwrap_or(0);
        if file_url.is_empty() || filename.is_empty() {
            anyhow::bail!("Fabric API entry has no downloadable file");
        }
        let dest = mods_dir.join(filename);
        let bytes = self.client.get(file_url).send().await?.bytes().await?;
        if bytes.is_empty() || !bytes_start_with_zip(&bytes) {
            anyhow::bail!("Downloaded Fabric API is not a jar (bad response?)");
        }
        if expected_size > 0 && bytes.len() as u64 != expected_size {
            anyhow::bail!("Downloaded Fabric API has unexpected size");
        }
        tokio::fs::write(&dest, bytes).await?;
        Ok(true)
    }
}

/// True when the file starts with the ZIP magic (jars are zips).
async fn file_is_zip(path: PathBuf) -> bool {
    let Ok(mut file) = tokio::fs::File::open(&path).await else {
        return false;
    };
    use tokio::io::AsyncReadExt;
    let mut magic = [0u8; 4];
    match file.read_exact(&mut magic).await {
        Ok(_) => magic == [0x50, 0x4B, 0x03, 0x04],
        Err(_) => false,
    }
}

/// True when the bytes start with the ZIP magic.
fn bytes_start_with_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04
}

/// Remove stale same-loader profiles for an MC line after an upgrade.
/// Only `fabric-loader-*-<mc>` / `quilt-loader-*-<mc>` dirs are eligible and
/// only when they are NOT the just-installed id. Vanilla and other loaders
/// are never touched.
async fn prune_superseded_profiles(keep_id: &str, mc_version: &str) {
    let (prefix, suffix) = if keep_id.starts_with("fabric-loader-") {
        ("fabric-loader-", format!("-{}", mc_version))
    } else if keep_id.starts_with("quilt-loader-") {
        ("quilt-loader-", format!("-{}", mc_version))
    } else {
        return;
    };
    if let Ok(mut entries) = tokio::fs::read_dir(crate::common::versions_dir()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == keep_id {
                continue;
            }
            if name.starts_with(prefix) && name.ends_with(&suffix) {
                let dir = entry.path();
                if dir.join(format!("{}.json", name)).exists() {
                    tracing::info!("Removing superseded loader profile {}", name);
                    tokio::fs::remove_dir_all(&dir).await.ok();
                }
            }
        }
    }
}

/// Minimal vanilla `launcher_profiles.json`, just enough for legacy
/// (<=1.12 era) Forge installers which abort without one. Only created
/// when the file is missing; never overwritten.
async fn ensure_launcher_profile(mc_base: &PathBuf) {
    let path = mc_base.join("launcher_profiles.json");
    if path.exists() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let content = serde_json::json!({
        "profiles": {},
        "selectedProfile": "(Default)",
        "clientToken": format!("dontcam-{}", now),
        "authenticationDatabase": {},
        "launcherVersion": { "name": "1.6.93", "format": 21 },
        "settings": {
            "crashAssistance": true,
            "enableAdvanced": false,
            "enableAnalytics": true,
            "enableHistorical": false,
            "enableReleases": true,
            "enableSnapshots": false,
            "keepLauncherOpen": false,
            "profileSorting": "ByLastPlayed",
            "showGameLog": false,
            "showMenu": false,
            "soundOn": false
        }
    });
    if let Ok(text) = serde_json::to_string_pretty(&content) {
        tokio::fs::write(&path, text).await.ok();
    }
}

/// MC 1.12 and older use the legacy installer flag style.
fn mc_is_legacy(mc_version: &str) -> bool {
    let base = mc_version.split('-').next().unwrap_or(mc_version);
    let mut parts = base.split('.').filter_map(|p| p.parse::<u64>().ok());
    match (parts.next(), parts.next()) {
        (Some(1), Some(minor)) => minor <= 12,
        (Some(major), _) if major < 1 => true,
        _ => false,
    }
}

async fn list_version_dirs() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(crate::common::versions_dir()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                out.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }
    out
}

/// Find the version dir created by an installer run (newest dir not in `before`
/// that contains a matching `<id>.json`).
async fn discover_new_version(before: &[String]) -> Option<String> {
    let mut candidates: Vec<(String, std::time::SystemTime)> = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(crate::common::versions_dir()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if before.contains(&name) {
                continue;
            }
            if !entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let json = entry.path().join(format!("{}.json", name));
            if !json.exists() {
                continue;
            }
            let mtime = entry
                .metadata()
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::UNIX_EPOCH);
            candidates.push((name, mtime));
        }
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.into_iter().next().map(|(name, _)| name)
}

/// Numeric key for dotted versions ("47.2.10" > "47.2.9").
fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

/// Descending numeric-aware sort for dotted versions ("47.2.10" > "47.2.9").
fn sort_versions_desc(versions: &mut [String]) {
    versions.sort_by(|a, b| version_key(b).cmp(&version_key(a)));
}

fn rules_allowed(rules: &serde_json::Value) -> bool {    // simplified: evaluate os.name only
    if let Some(arr) = rules.as_array() {
        let mut allowed = true;
        for rule in arr {
            let action = rule["action"].as_str().unwrap_or("allow");
            let mut matches = true;
            if let Some(os) = rule.get("os").and_then(|o| o.get("name")).and_then(|n| n.as_str()) {
                let current = crate::common::current_os_name();
                matches = match os {
                    "windows" => current == "windows",
                    "osx" => current == "osx",
                    "linux" => current == "linux",
                    _ => false,
                };
            }
            if matches {
                allowed = action == "allow";
            }
        }
        allowed
    } else {
        true
    }
}

fn maven_coords_to_path(coords: &str) -> Option<String> {
    let parts: Vec<&str> = coords.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    Some(format!(
        "{}/{}/{}/{}-{}.jar",
        group.replace('.', "/"),
        artifact,
        version,
        artifact,
        version
    ))
}

async fn find_any_java() -> Option<PathBuf> {
    for candidate in ["javaw", "java", "javaw.exe", "java.exe"] {
        let mut probe = tokio::process::Command::new(candidate);
        probe.arg("-version");
        crate::common::hide_console_tokio(&mut probe);
        if probe.output().await.is_ok() {
            return Some(PathBuf::from(candidate));
        }
    }
    None
}

/// Managed Java matching the MC version first (legacy installers need old Java),
/// falling back to anything on PATH.
async fn find_java_for_mc(mc_version: &str) -> Option<PathBuf> {
    let required = crate::common::recommended_java_major(mc_version);
    if let Ok(entries) = std::fs::read_dir(crate::common::base_dir().join("java").join("managed")) {
        for entry in entries.flatten() {
            for bin in ["javaw.exe", "java"] {
                let java_path = entry.path().join("bin").join(bin);
                if !java_path.exists() {
                    continue;
                }
                if let Ok(output) = {
                    let mut probe = tokio::process::Command::new(&java_path);
                    probe.arg("-version");
                    crate::common::hide_console_tokio(&mut probe);
                    probe.output().await
                } {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // `"<major>...` — first quoted component, like the manager parses
                    let major = stderr
                        .lines()
                        .next()
                        .unwrap_or("")
                        .split('"')
                        .nth(1)
                        .unwrap_or("")
                        .split('.')
                        .next()
                        .unwrap_or("0")
                        .parse::<u32>()
                        .unwrap_or(0);
                    if major == required {
                        return Some(java_path);
                    }
                }
            }
        }
    }
    find_any_java().await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoaderVersion {
    pub id: String,
    pub version: String,
    pub mc_version: String,
    pub mod_loader: ModLoaderType,
    pub download_url: String,
    pub file_size: u64,
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModLoader {
    pub id: String,
    pub version: String,
    pub mc_version: String,
    pub mod_loader: ModLoaderType,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderType {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
    LiteLoader,
    None,
}

#[tauri::command]
pub async fn get_modloader_versions(
    state: tauri::State<'_, crate::AppState>,
    mod_loader: ModLoaderType,
    mc_version: String,
) -> Result<Vec<ModLoaderVersion>, String> {
    state
        .modloaders
        .get_modloader_versions(mod_loader, &mc_version)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_modloader(
    state: tauri::State<'_, crate::AppState>,
    version: ModLoaderVersion,
    game_dir: String,
) -> Result<InstalledModLoader, String> {
    let dir = if game_dir.trim().is_empty() {
        crate::common::base_dir()
    } else {
        PathBuf::from(game_dir)
    };
    state
        .modloaders
        .install_modloader(&version, &dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_installed_modloaders(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<InstalledModLoader>, String> {
    state
        .modloaders
        .get_installed_modloaders()
        .await
        .map_err(|e| e.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_resolution_per_line() {
        // 1.8–1.12 -> Forge, 1.16–1.21 -> Fabric (with Fabric API).
        assert_eq!(default_loader_for("1.8.9"), ModLoaderType::Forge);
        assert_eq!(default_loader_for("1.12.2"), ModLoaderType::Forge);
        assert_eq!(default_loader_for("1.16.5"), ModLoaderType::Fabric);
        assert_eq!(default_loader_for("1.17.1"), ModLoaderType::Fabric);
        assert_eq!(default_loader_for("1.18.2"), ModLoaderType::Fabric);
        assert_eq!(default_loader_for("1.19.4"), ModLoaderType::Fabric);
        assert_eq!(default_loader_for("1.20.1"), ModLoaderType::Fabric);
        assert_eq!(default_loader_for("1.21.1"), ModLoaderType::Fabric);
    }

    #[test]
    fn neoforge_prefixes() {
        assert_eq!(ModLoaderManager::neoforge_prefix("1.21.1"), "21.1");
        assert_eq!(ModLoaderManager::neoforge_prefix("1.20.1"), "20.1");
    }

    #[test]
    fn legacy_flag_eras() {
        assert!(mc_is_legacy("1.8.9"));
        assert!(mc_is_legacy("1.12.2"));
        assert!(!mc_is_legacy("1.13"));
        assert!(!mc_is_legacy("1.16.5"));
        assert!(!mc_is_legacy("1.21.1"));
    }

    #[test]
    fn fabric_loader_caps() {
        // floors: bundled mods need loader >= 0.15.11 (1.21: >= 0.19.5),
        // 1.20's API needs >= 0.16.10
        assert_eq!(ModLoaderManager::max_fabric_loader("1.16.5"), Some("0.15.11"));
        assert_eq!(ModLoaderManager::max_fabric_loader("1.19.4"), Some("0.15.11"));
        assert_eq!(ModLoaderManager::max_fabric_loader("1.20.1"), Some("0.16.14"));
        assert_eq!(ModLoaderManager::max_fabric_loader("1.21.1"), None);
        // capped-out loaders sort above the cap
        assert!(version_key("0.19.5") > version_key("0.14.25"));
        assert!(version_key("0.15.11") > version_key("0.14.25"));
        assert!(version_key("0.16.14") > version_key("0.15.11"));
    }
}

pub(crate) fn default_loader_for(mc_version: &str) -> ModLoaderType {
    // 1.8–1.12 -> Forge, 1.16–1.21 -> Fabric (+ Fabric API at install/launch).
    // (NeoForge remains available as a manual choice in Mods/Profiles.)
    let base = mc_version.split('-').next().unwrap_or(mc_version);
    let mut parts = base.split('.').filter_map(|p| p.parse::<u64>().ok());
    match (parts.next(), parts.next()) {
        (Some(1), Some(minor)) if minor <= 12 => ModLoaderType::Forge,
        (Some(1), Some(minor)) if (16..=21).contains(&minor) => ModLoaderType::Fabric,
        // Fallback for any other 1.x line: Fabric (it exists for 1.13+ too).
        (Some(1), Some(_)) => ModLoaderType::Fabric,
        _ => ModLoaderType::Fabric,
    }
}

#[tauri::command]
pub async fn resolve_loader(
    state: tauri::State<'_, crate::AppState>,
    mc_version: String,
) -> Result<ModLoaderVersion, String> {
    let loader = default_loader_for(&mc_version);
    let versions = state
        .modloaders
        .get_modloader_versions(loader.clone(), &mc_version)
        .await
        .map_err(|e| e.to_string())?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| format!("No {:?} build found for Minecraft {}", loader, mc_version))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallModdedResult {
    pub version_id: String,
    pub loader: ModLoaderType,
    pub modded_version_id: Option<String>,
    pub loader_error: Option<String>,
}

/// Full chain for one click: vanilla files + best loader for this MC version.
/// Never vanilla-only: 1.8–1.12 -> Forge, 1.16–1.21 -> Fabric (+ Fabric API).
/// Loader failures are reported (not fatal) so vanilla stays playable.
/// The bundled DontCam mod for the MC line is ALWAYS staged into the
/// instance right away (1.20.1 -> jar from the 1.20 line, 1.8.9 -> 1.8, etc.),
/// so the mods folder is correct even before the first launch.
#[tauri::command]
pub async fn install_modded(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    version_id: String,
    force: Option<bool>,
    profile_id: Option<String>,
) -> Result<InstallModdedResult, String> {
    let force = force.unwrap_or(false);
    state
        .versions
        .install_version(Some(app.clone()), &version_id, force)
        .await
        .map_err(|e| e.to_string())?;

    let loader = default_loader_for(&version_id);
    if loader == ModLoaderType::None {
        return Ok(InstallModdedResult {
            version_id: version_id.clone(),
            loader,
            modded_version_id: None,
            loader_error: None,
        });
    }

    // Provision the right Java BEFORE running a loader installer: legacy
    // Forge installers break on too-new runtimes, and find_java_for_mc picks
    // the managed runtime matching this MC version when it exists.
    let required_java = crate::common::recommended_java_major(&version_id);
    if let Err(e) = (|| async {
        state.java.ensure_java(required_java).await
            .ok_or_else(|| anyhow::anyhow!("no Java available"))?;
        Ok::<(), anyhow::Error>(())
    })()
    .await
    {
        tracing::warn!("Java provisioning for {} failed: {}", version_id, e);
    }

    let result = (|| async {
        let versions = state
            .modloaders
            .get_modloader_versions(loader.clone(), &version_id)
            .await?;
        let preferred = versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No {:?} build for {}", loader, version_id))?;
        let installed = state
            .modloaders
            .install_modloader(&preferred, &crate::common::base_dir())
            .await?;
        // refresh the installed-versions view so launch can discover the profile
        state.versions.scan_installed_versions().await.ok();
        Ok::<InstalledModLoader, anyhow::Error>(installed)
    })()
    .await;

    let (modded_version_id, loader_error) = match result {
        Ok(installed) => (Some(installed.id), None),
        Err(e) => (None, Some(e.to_string())),
    };

    // Stage bundled mod (+ API) into the instance immediately — ALWAYS,
    // not gated on any profile flag: every installed version gets its
    // line-specific DontCam jar (Fabric lines also get Fabric API).
    if crate::launch::bundled_mod_for(&version_id).is_some() {
        // Resolve target game dir: profile instance when known, else the
        // profile's dir / global instance so `mods/` is still correct.
        let game_dir = if let Some(pid) = profile_id.as_deref() {
            if pid.is_empty() {
                crate::common::game_dir_for_profile(&state.settings, None).await
            } else {
                crate::common::game_dir_for_profile(&state.settings, Some(pid)).await
            }
        } else {
            crate::common::game_dir_for_profile(&state.settings, None).await
        };
        if let Err(e) = crate::launch::ensure_dontcam_mod(&game_dir, &version_id).await {
            tracing::warn!("Mod staging failed: {}", e);
        }
        if loader == ModLoaderType::Fabric {
            if let Err(e) = state
                .modloaders
                .ensure_fabric_api(&game_dir.join("mods"), &version_id)
                .await
            {
                tracing::warn!("Fabric API staging failed: {}", e);
            }
        }
    }

    Ok(InstallModdedResult {
        version_id: version_id.clone(),
        loader,
        modded_version_id,
        loader_error,
    })
}
