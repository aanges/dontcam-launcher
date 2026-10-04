//! Game profiles (instances): one json file per profile, game files under
//! `instances/<profile_id>` (see `common::game_dir_for_profile`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

pub struct ProfileManager {
    profiles: Arc<RwLock<HashMap<String, Profile>>>,
    profiles_dir: std::path::PathBuf,
}

impl ProfileManager {
    pub async fn new() -> Self {
        let profiles_dir = crate::common::base_dir().join("profiles");
        tokio::fs::create_dir_all(&profiles_dir).await.ok();
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            profiles_dir,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.load_profiles().await?;
        // NOTE: no auto-created profile — first launch shows the onboarding
        // popup instead (user adds an account, then picks a version).
        Ok(())
    }

    async fn load_profiles(&self) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        profiles.clear();
        let mut entries = match tokio::fs::read_dir(&self.profiles_dir).await {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file()
                && entry.path().extension().map_or(false, |e| e == "json")
            {
                if let Ok(content) = tokio::fs::read_to_string(entry.path()).await {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        profiles.insert(profile.id.clone(), profile);
                    }
                }
            }
        }
        Ok(())
    }

    async fn save_profile(&self, profile: &Profile) -> Result<()> {
        let path = self.profiles_dir.join(format!("{}.json", profile.id));
        let content = serde_json::to_string_pretty(profile)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn mark_played(&self, profile_id: &str) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        if let Some(p) = profiles.get_mut(profile_id) {
            p.last_played = Some(Utc::now());
            p.updated_at = Utc::now();
            let cloned = p.clone();
            drop(profiles);
            self.save_profile(&cloned).await?;
        }
        Ok(())
    }

    pub async fn get_cloned(&self, profile_id: &str) -> Option<Profile> {
        self.profiles.read().await.get(profile_id).cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub version_id: String,
    #[serde(default)]
    pub mod_loader: ModLoaderType,
    #[serde(default)]
    pub mod_loader_version: Option<String>,
    pub account_id: Option<String>,
    /// Install the DontCam client mod into the instance on launch.
    #[serde(default = "default_true")]
    pub dontcam_mod: bool,
    #[serde(default)]
    pub java_args: String,
    #[serde(default)]
    pub game_args: Vec<String>,
    pub resolution: Option<Resolution>,
    #[serde(default)]
    pub mods: Vec<ModEntry>,
    #[serde(default)]
    pub resource_packs: Vec<ResourcePackEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    #[serde(default)]
    pub play_time: u64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub file_path: String,
    pub enabled: bool,
    #[serde(default)]
    pub mod_loader: ModLoaderType,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderType {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
    #[serde(alias = "liteloader")]
    LiteLoader,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackEntry {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub enabled: bool,
    pub priority: u32,
}

fn loader_for(mc_version: &str) -> ModLoaderType {
    match crate::modloaders::default_loader_for(mc_version) {
        crate::modloaders::ModLoaderType::Forge => ModLoaderType::Forge,
        crate::modloaders::ModLoaderType::Fabric => ModLoaderType::Fabric,
        crate::modloaders::ModLoaderType::Quilt => ModLoaderType::Quilt,
        crate::modloaders::ModLoaderType::NeoForge => ModLoaderType::NeoForge,
        _ => ModLoaderType::None,
    }
}

#[tauri::command]
pub async fn create_profile(
    state: tauri::State<'_, crate::AppState>,
    name: String,
    version_id: String,
    account_id: Option<String>,
) -> Result<Profile, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    let profile = Profile {
        id: Uuid::new_v4().to_string(),
        name,
        version_id: version_id.clone(),
        mod_loader: loader_for(&version_id),
        mod_loader_version: None,
        account_id,
        // Bundled DontCam mod installs automatically wherever a port exists;
        // toggle off to play pure vanilla.
        dontcam_mod: true,
        java_args: String::new(),
        game_args: vec![],
        resolution: None,
        mods: vec![],
        resource_packs: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_played: None,
        play_time: 0,
    };
    state
        .profiles
        .save_profile(&profile)
        .await
        .map_err(|e| e.to_string())?;
    {
        let mut profiles = state.profiles.profiles.write().await;
        profiles.insert(profile.id.clone(), profile.clone());
    }
    // ensure game dir exists
    crate::common::game_dir_for_profile(&state.settings, Some(profile.id.as_str())).await;
    Ok(profile)
}

#[tauri::command]
pub async fn get_profiles(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Profile>, String> {
    let profiles = state.profiles.profiles.read().await;
    let mut list: Vec<Profile> = profiles.values().cloned().collect();
    list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(list)
}

#[tauri::command]
pub async fn get_profile(
    state: tauri::State<'_, crate::AppState>,
    profile_id: String,
) -> Result<Option<Profile>, String> {
    let profiles = state.profiles.profiles.read().await;
    Ok(profiles.get(&profile_id).cloned())
}

#[tauri::command]
pub async fn update_profile(
    state: tauri::State<'_, crate::AppState>,
    profile: Profile,
) -> Result<Profile, String> {
    let mut updated = profile.clone();
    updated.updated_at = Utc::now();
    state
        .profiles
        .save_profile(&updated)
        .await
        .map_err(|e| e.to_string())?;
    {
        let mut profiles = state.profiles.profiles.write().await;
        profiles.insert(updated.id.clone(), updated.clone());
    }
    Ok(updated)
}

#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, crate::AppState>,
    profile_id: String,
) -> Result<(), String> {
    let path = state
        .profiles
        .profiles_dir
        .join(format!("{}.json", profile_id));
    if path.exists() {
        tokio::fs::remove_file(path).await.map_err(|e| e.to_string())?;
    }
    {
        let mut profiles = state.profiles.profiles.write().await;
        profiles.remove(&profile_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn duplicate_profile(
    state: tauri::State<'_, crate::AppState>,
    profile_id: String,
    new_name: String,
) -> Result<Profile, String> {
    let original = {
        let profiles = state.profiles.profiles.read().await;
        profiles
            .get(&profile_id)
            .cloned()
            .ok_or_else(|| "Profile not found".to_string())?
    };
    let mut dupe = original;
    dupe.id = Uuid::new_v4().to_string();
    dupe.name = new_name;
    dupe.created_at = Utc::now();
    dupe.updated_at = Utc::now();
    dupe.last_played = None;
    dupe.play_time = 0;
    state
        .profiles
        .save_profile(&dupe)
        .await
        .map_err(|e| e.to_string())?;
    {
        let mut profiles = state.profiles.profiles.write().await;
        profiles.insert(dupe.id.clone(), dupe.clone());
    }
    Ok(dupe)
}
