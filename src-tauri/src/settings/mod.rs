use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use std::path::PathBuf;

pub struct SettingsManager {
    settings: Arc<RwLock<Settings>>,
    settings_path: PathBuf,
}

impl SettingsManager {
    pub async fn new() -> Self {
        let settings_path = crate::common::base_dir().join("settings.json");
        if let Some(parent) = settings_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        let settings = if settings_path.exists() {
            let content = tokio::fs::read_to_string(&settings_path).await.unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Settings::default()
        };

        Self {
            settings: Arc::new(RwLock::new(settings)),
            settings_path,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.save().await?;
        Ok(())
    }

    pub async fn get(&self) -> Settings {
        self.settings.read().await.clone()
    }

    async fn save(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let content = serde_json::to_string_pretty(&*settings)?;
        tokio::fs::write(&self.settings_path, content).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub language: String,
    pub java: JavaSettings,
    pub game: GameSettings,
    pub network: NetworkSettings,
    pub ui: UISettings,
    pub advanced: AdvancedSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            language: "en-US".to_string(),
            java: JavaSettings::default(),
            game: GameSettings::default(),
            network: NetworkSettings::default(),
            ui: UISettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaSettings {
    pub auto_detect: bool,
    pub preferred_version: Option<String>,
    pub custom_java_path: Option<String>,
    pub jvm_args: String,
    pub memory_allocation: MemoryAllocation,
}

impl Default for JavaSettings {
    fn default() -> Self {
        Self {
            auto_detect: true,
            preferred_version: Some("21".to_string()),
            custom_java_path: None,
            jvm_args: "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1 -Dusing.aikars.flags=https://mcflags.emc.gs -Daikars.new.flags=true".to_string(),
            memory_allocation: MemoryAllocation::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    pub min: u32,
    pub max: u32,
    pub unit: MemoryUnit,
}

impl Default for MemoryAllocation {
    fn default() -> Self {
        Self {
            min: 1,
            max: 4,
            unit: MemoryUnit::GB,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryUnit {
    MB,
    GB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub default_game_dir: Option<String>,
    pub auto_close_launcher: bool,
    pub keep_launcher_open: bool,
    pub custom_resolution: Option<Resolution>,
    pub fullscreen: bool,
    pub vsync: bool,
    pub fov: f32,
    pub render_distance: u8,
    pub max_fps: u32,
    pub enable_mods: bool,
    pub enable_resource_packs: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            default_game_dir: None,
            auto_close_launcher: false,
            keep_launcher_open: true,
            custom_resolution: None,
            fullscreen: false,
            vsync: true,
            fov: 70.0,
            render_distance: 12,
            max_fps: 0,
            enable_mods: true,
            enable_resource_packs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub proxy_enabled: bool,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub download_threads: u8,
    pub bandwidth_limit: Option<u64>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            proxy_enabled: false,
            proxy_host: "".to_string(),
            proxy_port: 8080,
            proxy_username: None,
            proxy_password: None,
            download_threads: 4,
            bandwidth_limit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub show_snapshots: bool,
    pub show_old_versions: bool,
    pub show_alpha_beta: bool,
    pub sort_versions_by: VersionSort,
    pub compact_mode: bool,
    pub animations: bool,
    pub background_blur: bool,
    pub news_enabled: bool,
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            show_snapshots: false,
            show_old_versions: true,
            show_alpha_beta: false,
            sort_versions_by: VersionSort::NewestFirst,
            compact_mode: false,
            animations: true,
            background_blur: true,
            news_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VersionSort {
    NewestFirst,
    OldestFirst,
    Alphabetical,
    ReleaseType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub debug_logging: bool,
    pub console_enabled: bool,
    pub custom_game_args: Vec<String>,
    pub environment_variables: HashMap<String, String>,
    pub pre_launch_command: Option<String>,
    pub post_exit_command: Option<String>,
    pub verify_downloads: bool,
    pub parallel_downloads: u8,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            debug_logging: false,
            console_enabled: false,
            custom_game_args: vec![],
            environment_variables: HashMap::new(),
            pre_launch_command: None,
            post_exit_command: None,
            verify_downloads: true,
            parallel_downloads: 4,
        }
    }
}

#[tauri::command]
pub async fn get_settings(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Settings, String> {
    let settings = state.settings.settings.read().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn update_settings(
    state: tauri::State<'_, crate::AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let mut current = state.settings.settings.write().await;
    *current = settings.clone();
    drop(current);
    
    state.settings.save().await.map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn reset_settings(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Settings, String> {
    let default = Settings::default();
    let mut current = state.settings.settings.write().await;
    *current = default.clone();
    drop(current);
    
    state.settings.save().await.map_err(|e| e.to_string())?;
    Ok(default)
}