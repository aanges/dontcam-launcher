#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod common;
mod dontcam;
mod java;
mod launch;
mod modloaders;
mod profiles;
mod settings;
mod versions;
mod window;

use std::sync::Arc;

use auth::AuthManager;
use java::JavaManager;
use launch::LaunchManager;
use modloaders::ModLoaderManager;
use profiles::ProfileManager;
use settings::SettingsManager;
use versions::VersionManager;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthManager>,
    pub versions: Arc<VersionManager>,
    pub profiles: Arc<ProfileManager>,
    pub launch: Arc<LaunchManager>,
    pub settings: Arc<SettingsManager>,
    pub modloaders: Arc<ModLoaderManager>,
    pub java: Arc<JavaManager>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let app_state = tauri::async_runtime::block_on(async {
        AppState {
            auth: Arc::new(AuthManager::new().await),
            versions: Arc::new(VersionManager::new().await),
            profiles: Arc::new(ProfileManager::new().await),
            launch: Arc::new(LaunchManager::new().await),
            settings: Arc::new(SettingsManager::new().await),
            modloaders: Arc::new(ModLoaderManager::new().await),
            java: Arc::new(JavaManager::new().await),
        }
    });

    let init_state = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            auth::login_microsoft,
            auth::login_offline,
            auth::logout,
            auth::get_accounts,
            auth::get_current_account,
            auth::set_current_account,
            auth::refresh_token,
            auth::validate_account,
            versions::fetch_version_manifest,
            versions::get_installed_versions,
            versions::install_version,
            versions::uninstall_version,
            versions::get_version_details,
            versions::ensure_version_ready,
            profiles::create_profile,
            profiles::get_profiles,
            profiles::get_profile,
            profiles::update_profile,
            profiles::delete_profile,
            profiles::duplicate_profile,
            launch::launch_game,
            launch::get_launch_status,
            launch::kill_game,
            settings::get_settings,
            settings::update_settings,
            settings::reset_settings,
            modloaders::get_modloader_versions,
            modloaders::install_modloader,
            modloaders::get_installed_modloaders,
            modloaders::resolve_loader,
            modloaders::install_modded,
            java::detect_java,
            java::get_java_installations,
            java::download_java,
            java::resolve_java_for_version,
            dontcam::check_dontcam_update,
            window::minimize_window,
            window::toggle_maximize,
            window::close_window,
            utils::open_folder,
            utils::get_app_data_dir,
            utils::get_game_dir,
        ])
        .setup(move |_app| {
            tauri::async_runtime::spawn(async move {
                if let Err(e) = init_state.settings.initialize().await {
                    tracing::error!("Failed to initialize settings: {}", e);
                }
                if let Err(e) = init_state.auth.load_accounts().await {
                    tracing::error!("Failed to load accounts: {}", e);
                }
                if let Err(e) = init_state.profiles.initialize().await {
                    tracing::error!("Failed to initialize profiles: {}", e);
                }
                if let Err(e) = init_state.java.detect_java().await {
                    tracing::error!("Failed to detect Java: {}", e);
                }
                // Fetch manifest in background, don't block startup
                if let Err(e) = init_state.versions.fetch_version_manifest().await {
                    tracing::warn!("Failed to fetch version manifest at startup: {}", e);
                }
                if let Err(e) = init_state.versions.scan_installed_versions().await {
                    tracing::warn!("Failed to scan installed versions: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod utils {
    use tauri::AppHandle;

    #[tauri::command]
    pub async fn open_folder(path: String) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    #[tauri::command]
    pub async fn get_app_data_dir(_app: AppHandle) -> Result<String, String> {
        // Point at the real game data dir (used by the Settings page),
        // not the Tauri identifier dir.
        Ok(crate::common::base_dir().to_string_lossy().to_string())
    }

    #[tauri::command]
    pub async fn get_game_dir(
        state: tauri::State<'_, crate::AppState>,
        profile_id: Option<String>,
    ) -> Result<String, String> {
        Ok(crate::common::game_dir_for_profile(&state.settings, profile_id.as_deref())
            .await
            .to_string_lossy()
            .to_string())
    }
}
