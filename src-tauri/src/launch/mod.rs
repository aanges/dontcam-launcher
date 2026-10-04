//! Game launch: version prep, loader profile discovery/merge, DontCam mod
//! staging, Java resolution, natives, classpath, args, spawn + watchdog.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use anyhow::{Result, Context};
use std::path::PathBuf;
use tauri::Emitter;
use crate::common::{is_natives_library, native_classifier_matches};

pub struct LaunchManager {
    current_process: Arc<Mutex<Option<tokio::process::Child>>>,
    launch_status: Arc<RwLock<LaunchStatus>>,
}

impl LaunchManager {
    pub async fn new() -> Self {
        Self {
            current_process: Arc::new(Mutex::new(None)),
            launch_status: Arc::new(RwLock::new(LaunchStatus::Idle)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LaunchStatus {
    Idle,
    Preparing,
    DownloadingAssets,
    DownloadingLibraries,
    Launching,
    Running,
    Error(String),
    Crashed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub profile_id: String,
    pub version_id: String,
    pub account: crate::auth::Account,
    #[serde(default)]
    pub java_path: String,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub game_args: Vec<String>,
    #[serde(default)]
    pub game_dir: String,
    pub resolution: Option<crate::profiles::Resolution>,
    #[serde(default)]
    pub server: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
}

#[tauri::command]
pub async fn launch_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    options: LaunchOptions,
) -> Result<String, String> {
    // prevent double launch
    {
        let status = state.launch.launch_status.read().await;
        if *status == LaunchStatus::Running || *status == LaunchStatus::Launching {
            return Err("Game is already running".to_string());
        }
    }
    set_status(&state, LaunchStatus::Preparing).await;
    let _ = app.emit("launch-status", "preparing");

    inner_launch(app.clone(), (*state).clone(), options)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            let _ = app.emit("launch-status", serde_json::json!({ "error": msg }));
            msg
        })
}

#[allow(clippy::too_many_lines)]
async fn inner_launch(
    app: tauri::AppHandle,
    state: crate::AppState,
    options: LaunchOptions,
) -> Result<String> {
    // 1. Ensure version files present
    set_status(&state, LaunchStatus::DownloadingLibraries).await;
    let version = state
        .versions
        .install_version(Some(app.clone()), &options.version_id, false)
        .await
        .context("Failed to prepare version")?;

    // raw details needed for legacy args + natives
    let base_details = load_raw_details(&options.version_id).await?;

    // 1b. Which modded profile to target (merged over the vanilla base)?
    // The loader is driven by the MC VERSION being launched — not blindly by
    // the profile. Rule (see effective_loader): the profile's loader wins
    // only when the profile targets this exact version; otherwise the
    // canonical loader for the launched version wins (1.8–1.12 Forge,
    // 1.16–1.21 Fabric + Fabric API) — never vanilla.
    // Missing loader profiles are installed on the fly so Play is self-healing.
    let (profile_loader_raw, profile_version) = if !options.profile_id.is_empty() {
        match state.profiles.get_cloned(&options.profile_id).await {
            Some(p) => (p.mod_loader, p.version_id),
            None => (crate::profiles::ModLoaderType::None, String::new()),
        }
    } else {
        (crate::profiles::ModLoaderType::None, String::new())
    };
    let profile_loader = effective_loader(&profile_loader_raw, &profile_version, &options.version_id);
    if profile_loader != profile_loader_raw {
        let _ = app.emit(
            "game-log",
            serde_json::json!({
                "stream": "stdout",
                "line": format!(
                    "Using {:?} for {} (profile asked {:?}) — every version launches modded.",
                    profile_loader, options.version_id, profile_loader_raw
                ),
            }),
        );
    }
    // Profile loaders (Fabric/Quilt) install under a deterministic id, so we
    // can detect a stale profile and upgrade it before launch. Jar installers
    // (Forge/NeoForge) generate their own ids — install only when nothing
    // is discovered.
    let is_profile_loader = matches!(
        profile_loader,
        crate::profiles::ModLoaderType::Fabric | crate::profiles::ModLoaderType::Quilt
    );
    if is_profile_loader {
        let preferred: Option<crate::modloaders::ModLoaderVersion> = {
            use crate::modloaders::ModLoaderType as ML;
            use crate::profiles::ModLoaderType as PL;
            let backend = match profile_loader {
                PL::Fabric => ML::Fabric,
                PL::Quilt => ML::Quilt,
                _ => ML::None,
            };
            match state.modloaders.get_modloader_versions(backend, &options.version_id).await {
                Ok(v) => v.into_iter().next(),
                Err(_) => None,
            }
        };
        if let Some(pref) = preferred {
            let json = crate::common::versions_dir()
                .join(&pref.id)
                .join(format!("{}.json", pref.id));
            if !json.exists() {
                set_status(&state, LaunchStatus::DownloadingLibraries).await;
                let _ = app.emit(
                    "game-log",
                    serde_json::json!({
                        "stream": "stdout",
                        "line": format!(
                            "Installing preferred {:?} build {} for {}…",
                            profile_loader, pref.version, options.version_id
                        ),
                    }),
                );
                match state.modloaders.install_modloader(&pref, &crate::common::base_dir()).await {
                    Ok(installed) => {
                        let _ = app.emit(
                            "game-log",
                            serde_json::json!({
                                "stream": "stdout",
                                "line": format!("Installed modded profile {}", installed.id),
                            }),
                        );
                        state.versions.scan_installed_versions().await.ok();
                    }
                    Err(e) => {
                        let _ = app.emit(
                            "game-log",
                            serde_json::json!({
                                "stream": "stderr",
                                "line": format!("Loader install failed (using installed profile if any): {}", e),
                            }),
                        );
                    }
                }
            }
        }
    }
    if !is_profile_loader
        && profile_loader != crate::profiles::ModLoaderType::None
        && discover_modded_version(&options.version_id, &profile_loader).await.is_none()
    {
        set_status(&state, LaunchStatus::DownloadingLibraries).await;
        let _ = app.emit(
            "game-log",
            serde_json::json!({
                "stream": "stdout",
                "line": format!(
                    "Modded profile missing — installing {:?} for {} now…",
                    profile_loader, options.version_id
                ),
            }),
        );
        match install_best_loader(&state, &options.version_id, &profile_loader).await {
            Ok(installed_id) => {
                let _ = app.emit(
                    "game-log",
                    serde_json::json!({
                        "stream": "stdout",
                        "line": format!("Installed modded profile {}", installed_id),
                    }),
                );
                state.versions.scan_installed_versions().await.ok();
            }
            Err(e) => {
                let _ = app.emit(
                    "game-log",
                    serde_json::json!({
                        "stream": "stderr",
                        "line": format!("Loader install failed (continuing vanilla): {}", e),
                    }),
                );
            }
        }
    }
    let (effective_id, raw_details) =
        match discover_modded_version(&options.version_id, &profile_loader).await {
            Some(mid) => {
                tracing::info!("Using modded version {}", mid);
                match load_raw_details(&mid).await {
                    Ok(modded) => {
                        let mut merged = merge_inherited(modded, &base_details);
                        merged.id = mid.clone();
                        let _ = app.emit(
                            "game-log",
                            serde_json::json!({
                                "stream": "stdout",
                                "line": format!(
                                    "Launching modded profile {} (base {}) — entrypoint {}.",
                                    mid, options.version_id, merged.main_class
                                ),
                            }),
                        );
                        (mid, merged)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load modded profile, vanilla fallback: {}", e);
                        let _ = app.emit(
                            "game-log",
                            serde_json::json!({
                                "stream": "stderr",
                                "line": format!(
                                    "Modded profile {} unreadable ({}) — launching vanilla {}.",
                                    mid, e, options.version_id
                                ),
                            }),
                        );
                        (options.version_id.clone(), base_details)
                    }
                }
            }
            None => {
                if profile_loader != crate::profiles::ModLoaderType::None {
                    let _ = app.emit(
                        "game-log",
                        serde_json::json!({
                            "stream": "stderr",
                            "line": format!(
                                "No installed {:?} profile for {} — launching vanilla. Use Install & Play to set it up.",
                                profile_loader, options.version_id
                            ),
                        }),
                    );
                }
                (options.version_id.clone(), base_details)
            }
        };

    // 2. Resolve game dir
    let game_dir = if options.game_dir.trim().is_empty() {
        crate::common::game_dir_for_profile(
            &state.settings,
            if options.profile_id.is_empty() {
                None
            } else {
                Some(options.profile_id.as_str())
            },
        )
        .await
    } else {
        PathBuf::from(&options.game_dir)
    };
    tokio::fs::create_dir_all(&game_dir).await.ok();

    // 2b. DontCam client mod — ALWAYS for lines with a bundled port
    // (1.20.1 -> 1.20 jar, 1.8.9 -> 1.8 jar, ...).
    // Fabric profiles also get Fabric API (hard dependency of our mod).
    if let Some(bundled) = bundled_mod_for(&options.version_id) {
        ensure_dontcam_mod(&game_dir, &options.version_id).await?;
        let _ = app.emit(
            "game-log",
            serde_json::json!({
                "stream": "stdout",
                "line": format!("Staged DontCam mod {} in mods/.", bundled.file_name),
            }),
        );
        if profile_loader == crate::profiles::ModLoaderType::Fabric {
            match state
                .modloaders
                .ensure_fabric_api(&game_dir.join("mods"), &options.version_id)
                .await
            {
                Ok(true) => {
                    let _ = app.emit(
                        "game-log",
                        serde_json::json!({
                            "stream": "stdout",
                            "line": format!("Fabric API for {} downloaded.", options.version_id),
                        }),
                    );
                }
                Ok(false) => {
                    let _ = app.emit(
                        "game-log",
                        serde_json::json!({
                            "stream": "stdout",
                            "line": "Fabric API already present.",
                        }),
                    );
                }
                Err(e) => {
                    let _ = app.emit(
                        "game-log",
                        serde_json::json!({
                            "stream": "stderr",
                            "line": format!("Fabric API auto-install failed (the game may show a missing-dependency screen): {}", e),
                        }),
                    );
                }
            }
        }
    }

    // 3. Resolve java
    let java_path = resolve_java(&app, &state, &options, &version).await?;

    // 4. Natives
    set_status(&state, LaunchStatus::Launching).await;
    let natives_dir = crate::common::natives_dir(&effective_id);
    tokio::fs::create_dir_all(&natives_dir).await.ok();
    extract_natives(&raw_details, &natives_dir).await?;

    // 5. Classpath (client jar always comes from the vanilla base version)
    let classpath = build_classpath(&raw_details, &options.version_id)?;

    // 6. Args
    let settings = state.settings.get().await;
    let jvm_args = build_jvm_args(&raw_details, &options, &settings, &game_dir, &natives_dir, &classpath)?;
    let game_args = build_game_args(&raw_details, &options, &settings, &version, &game_dir)?;

    tracing::info!("Launching {} with java {}", options.version_id, java_path.display());

    // pre-launch command
    if let Some(cmd) = &settings.advanced.pre_launch_command {
        if !cmd.trim().is_empty() {
            let _ = run_shell(cmd).await;
        }
    }

    let mut cmd = tokio::process::Command::new(&java_path);
    cmd.args(&jvm_args);
    cmd.arg(&raw_details.main_class);
    cmd.args(&game_args);
    cmd.current_dir(&game_dir);
    cmd.env("MC_VERSION", &options.version_id);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    // Show console window? keep hidden by default on Windows
    #[cfg(target_os = "windows")]
    {
        if !settings.advanced.console_enabled {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
    }

    let mut child = cmd.spawn().context("Failed to spawn java. Is Java installed?")?;

    // Forward stdout/stderr to the frontend log event
    if let Some(stdout) = child.stdout.take() {
        let app_c = app.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_c.emit("game-log", serde_json::json!({ "stream": "stdout", "line": line }));
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let app_c = app.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_c.emit("game-log", serde_json::json!({ "stream": "stderr", "line": line }));
            }
        });
    }

    let pid = child.id();
    {
        let mut lock = state.launch.current_process.lock().await;
        *lock = Some(child);
    }
    set_status(&state, LaunchStatus::Running).await;
    let _ = app.emit("launch-status", "running");

    // update profile last_played
    if !options.profile_id.is_empty() {
        state.profiles.mark_played(&options.profile_id).await.ok();
    }
    // touch account last_used
    state.auth.touch_last_used(&options.account.id).await.ok();

    // watchdog: wait for exit in background
    let state_c = state.clone();
    let app_c = app.clone();
    let post_cmd = settings.advanced.post_exit_command.clone();
    tokio::spawn(async move {
        let exit_code = {
            let mut lock = state_c.launch.current_process.lock().await;
            if let Some(child) = lock.as_mut() {
                match child.wait().await {
                    Ok(status) => Some(status.code().unwrap_or(-1)),
                    Err(_) => None,
                }
            } else {
                None
            }
        };
        {
            let mut lock = state_c.launch.current_process.lock().await;
            *lock = None;
        }
        match exit_code {
            Some(0) => {
                set_status(&state_c, LaunchStatus::Idle).await;
                let _ = app_c.emit("launch-status", "idle");
            }
            Some(code) => {
                let msg = format!("Game exited with code {}", code);
                set_status(&state_c, LaunchStatus::Crashed(msg.clone())).await;
                let _ = app_c.emit("launch-status", serde_json::json!({ "crashed": msg }));
            }
            None => {
                set_status(&state_c, LaunchStatus::Idle).await;
                let _ = app_c.emit("launch-status", "idle");
            }
        }
        if let Some(cmd) = post_cmd {
            if !cmd.trim().is_empty() {
                let _ = run_shell(&cmd).await;
            }
        }
    });

    Ok(format!("Game launched (pid {})", pid.unwrap_or(0)))
}

async fn set_status(state: &crate::AppState, status: LaunchStatus) {
    let mut lock = state.launch.launch_status.write().await;
    *lock = status;
}

/// Bundled DontCam client mods, one per supported Minecraft line.
/// AUTO-GENERATED by build.rs from `resources/dontcam/dontcam-*.jar` —
/// drop new jars in that folder (see build.rs for naming) and rebuild.
pub(crate) struct BundledMod {
    /// MC version line this jar supports, e.g. "1.8".
    pub(crate) mc_prefix: &'static str,
    pub(crate) file_name: &'static str,
    pub(crate) bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/bundled_mods_generated.rs"));

/// True when a bundled mod line applies to a game version:
/// exact match, or prefix followed by '.'/'-' (so "1.2" never matches "1.21").
/// Keep in sync with `mc_line_of_jar` in build.rs (same boundary rules).
pub(crate) fn mod_applies(mc_prefix: &str, version_id: &str) -> bool {
    let v = version_id.trim();
    v == mc_prefix
        || v.starts_with(&format!("{}.", mc_prefix))
        || v.starts_with(&format!("{}-", mc_prefix))
}

pub(crate) fn bundled_mod_for(version_id: &str) -> Option<&'static BundledMod> {
    // Table is pre-sorted longest-prefix-first by build.rs.
    BUNDLED_MODS_GENERATED
        .iter()
        .find(|m| mod_applies(m.mc_prefix, version_id))
}

/// Public MODY repo with DontCam mod jars (one per MC line). The launcher
/// takes the jar from the LATEST release there when present; embedded
/// `resources/dontcam` jars are the offline fallback. No auth, no tokens.
pub(crate) const MODS_REPO: &str = "aanges/MODY";

/// Expected remote asset (mc line prefix + jar name) for a game version.
pub(crate) fn mod_asset_for(version_id: &str) -> Option<(&'static str, &'static str)> {
    BUNDLED_MODS_GENERATED
        .iter()
        .find(|m| mod_applies(m.mc_prefix, version_id))
        .map(|m| (m.mc_prefix, m.file_name))
}

pub(crate) async fn ensure_dontcam_mod(game_dir: &PathBuf, version_id: &str) -> Result<()> {
    let mods = game_dir.join("mods");
    tokio::fs::create_dir_all(&mods).await.ok();
    let wanted = mod_asset_for(version_id).map(|(_, f)| f);
    // Remove stale/foreign bundled jars (e.g. a 1.21 jar left behind after
    // switching the profile to 1.20) — they crash the game on load.
    if let Ok(mut entries) = tokio::fs::read_dir(&mods).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("dontcam-")
                && name.ends_with(".jar")
                && Some(name.as_str()) != wanted
            {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
    }
    let Some((_, file_name)) = mod_asset_for(version_id) else {
        return Ok(());
    };
    let dest = mods.join(file_name);
    // 1. Remote first: latest GitHub release of MODS_REPO.
    match fetch_remote_mod(file_name, &dest).await {
        Ok(true) => return Ok(()),
        Ok(false) => {}
        Err(e) => tracing::warn!("Remote mod fetch failed ({}), using embedded fallback", e),
    }
    // 2. Embedded offline fallback.
    let Some(bundled) = bundled_mod_for(version_id) else {
        return Ok(());
    };
    let needs_write = match tokio::fs::metadata(&dest).await {
        Ok(meta) => meta.len() != bundled.bytes.len() as u64,
        Err(_) => true,
    };
    if needs_write {
        tokio::fs::write(&dest, bundled.bytes).await?;
    }
    Ok(())
}

/// Download `file_name` from the latest MODS_REPO release into `dest`.
/// Returns Ok(true) when `dest` is correct afterwards (downloaded now or
/// already matching the release size), Ok(false) when there is nothing
/// usable remotely (no releases / no such asset yet — MODY ships sources).
async fn fetch_remote_mod(file_name: &str, dest: &PathBuf) -> Result<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("DontCam-Client/0.1")
        .build()?;
    let api: serde_json::Value = client
        .get(format!("https://api.github.com/repos/{}/releases/latest", MODS_REPO))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "DontCam-Client/0.1")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let assets = api
        .get("assets")
        .and_then(|a| a.as_array())
        .ok_or_else(|| anyhow::anyhow!("no assets in latest mods release"))?;
    let asset = assets
        .iter()
        .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(file_name))
        .ok_or_else(|| anyhow::anyhow!("no {} in latest mods release", file_name))?;
    let size = asset.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
    let url = asset
        .get("browser_download_url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| anyhow::anyhow!("asset has no download url"))?;
    if size > 0 {
        if let Ok(meta) = tokio::fs::metadata(dest).await {
            if meta.len() == size {
                return Ok(true);
            }
        }
    } else if dest.exists() {
        return Ok(true);
    }
    let bytes = client
        .get(url)
        .header("User-Agent", "DontCam-Client/0.1")
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    if bytes.is_empty() || !bytes_start_with_zip(&bytes) {
        anyhow::bail!("downloaded mod is not a jar (bad response?)");
    }
    if size > 0 && bytes.len() as u64 != size {
        anyhow::bail!("downloaded mod has unexpected size");
    }
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    tokio::fs::write(dest, &bytes).await?;
    tracing::info!("Downloaded DontCam mod {} from GitHub ({} bytes)", file_name, bytes.len());
    Ok(true)
}

/// True when the bytes start with the ZIP magic (jars are zips).
fn bytes_start_with_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04
}

async fn run_shell(cmd_str: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.args(["/C", cmd_str]);
        crate::common::hide_console_tokio(&mut cmd);
        cmd.status().await?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.args(["-c", cmd_str]);
        crate::common::hide_console_tokio(&mut cmd);
        cmd.status().await?;
    }
    Ok(())
}

async fn load_raw_details(version_id: &str) -> Result<crate::versions::VersionDetails> {
    let path = crate::common::versions_dir()
        .join(version_id)
        .join(format!("{}.json", version_id));
    let content = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&content)?)
}

/// Install the newest loader build for (mc, loader choice). Used by Play
/// so a profile is never stuck on vanilla when it asked for modded.
async fn install_best_loader(
    state: &crate::AppState,
    mc_version: &str,
    loader: &crate::profiles::ModLoaderType,
) -> Result<String> {
    use crate::modloaders::ModLoaderType as ML;
    use crate::profiles::ModLoaderType as PL;
    let backend_loader = match loader {
        PL::Forge => ML::Forge,
        PL::Fabric => ML::Fabric,
        PL::Quilt => ML::Quilt,
        PL::NeoForge => ML::NeoForge,
        _ => anyhow::bail!("no loader selected"),
    };
    let versions = state
        .modloaders
        .get_modloader_versions(backend_loader, mc_version)
        .await?;
    let preferred = versions
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No loader builds for {}", mc_version))?;
    let installed = state
        .modloaders
        .install_modloader(&preferred, &crate::common::base_dir())
        .await?;
    Ok(installed.id)
}

/// Version-driven loader choice for a launch.
///
/// - The profile's loader wins when the profile targets THIS mc version
///   (an explicit/manual choice such as NeoForge is respected).
/// - Otherwise the canonical loader for the launched version wins
///   (1.8–1.12 Forge, 1.16–1.21 Fabric), so a profile created for another
///   MC line can never drag its loader along. Unknown lines fall back to
///   the canonical default as well (never None when a default exists).
pub(crate) fn effective_loader(
    profile_loader: &crate::profiles::ModLoaderType,
    profile_version: &str,
    mc_version: &str,
) -> crate::profiles::ModLoaderType {
    use crate::modloaders::ModLoaderType as ML;
    use crate::profiles::ModLoaderType as PL;
    if *profile_loader != PL::None && profile_version == mc_version {
        return profile_loader.clone();
    }
    match crate::modloaders::default_loader_for(mc_version) {
        ML::Forge => PL::Forge,
        ML::Fabric => PL::Fabric,
        ML::Quilt => PL::Quilt,
        ML::NeoForge => PL::NeoForge,
        _ => PL::None,
    }
}

/// Find an installed modded profile for a vanilla version + loader choice
/// (e.g. `1.20.1-forge-47.2.0` for 1.20.1 + Forge). Newest match wins.
pub(crate) async fn discover_modded_version(
    mc_id: &str,
    loader: &crate::profiles::ModLoaderType,
) -> Option<String> {
    use crate::profiles::ModLoaderType as PL;
    let marker = match loader {
        PL::Forge => "forge",
        PL::Fabric => "fabric",
        PL::Quilt => "quilt",
        PL::NeoForge => "neoforge",
        _ => return None,
    };
    let mc_lower = mc_id.to_lowercase();
    let mut best: Option<(String, std::time::SystemTime)> = None;
    if let Ok(mut entries) = tokio::fs::read_dir(crate::common::versions_dir()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == mc_id {
                continue;
            }
            let lower = name.to_lowercase();
            if !lower.contains(marker) || !lower.contains(&mc_lower) {
                continue;
            }
            if !entry.path().join(format!("{}.json", name)).exists() {
                continue;
            }
            let mtime = entry
                .metadata()
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::UNIX_EPOCH);
            if best.as_ref().map_or(true, |(_, t)| mtime > *t) {
                best = Some((name, mtime));
            }
        }
    }
    best.map(|(name, _)| name)
}

/// Merge a loader profile (with `inheritsFrom`) over its vanilla base:
/// base libraries + profile additions, base args + profile additions,
/// entrypoint from the profile, assets/downloads/java from the base.
fn merge_inherited(
    modded: crate::versions::VersionDetails,
    base: &crate::versions::VersionDetails,
) -> crate::versions::VersionDetails {
    let mut out = modded;
    let mut libraries = base.libraries.clone();
    libraries.extend(out.libraries.clone());
    out.libraries = libraries;

    let base_args = base.arguments.clone().unwrap_or_default();
    let mut args = out.arguments.clone().unwrap_or_default();
    let mut game = base_args.game;
    game.extend(args.game);
    args.game = game;
    let mut jvm = base_args.jvm;
    jvm.extend(args.jvm);
    args.jvm = jvm;
    out.arguments = Some(args);

    if out.minecraft_arguments.is_none() {
        out.minecraft_arguments = base.minecraft_arguments.clone();
    }
    if out.asset_index.is_none() {
        out.asset_index = base.asset_index.clone();
    }
    if out.downloads.client.is_none() {
        out.downloads.client = base.downloads.client.clone();
    }
    if out.downloads.server.is_none() {
        out.downloads.server = base.downloads.server.clone();
    }
    if out.java_version.is_none() {
        out.java_version = base.java_version.clone();
    }
    out
}

async fn resolve_java(
    app: &tauri::AppHandle,
    state: &crate::AppState,
    options: &LaunchOptions,
    version: &crate::versions::GameVersion,
) -> Result<PathBuf> {
    // required major: version json > recommended map
    let required = version
        .java_version
        .as_ref()
        .map(|j| j.major_version)
        .unwrap_or_else(|| crate::common::recommended_java_major(&options.version_id));

    // Explicit paths (frontend / settings) win — but only when they point at
    // a compatible runtime. A stale/incompatible path is rejected with a log
    // line instead of launching silently into an ASM/Mixin crash.
    let mut explicit: Option<PathBuf> = None;
    if !options.java_path.trim().is_empty() && options.java_path != "java" {
        let p = PathBuf::from(&options.java_path);
        if p.exists() {
            explicit = Some(p);
        }
    }
    if explicit.is_none() {
        let settings = state.settings.get().await;
        if let Some(custom) = &settings.java.custom_java_path {
            if !custom.trim().is_empty() {
                let p = PathBuf::from(custom);
                if p.exists() {
                    explicit = Some(p);
                }
            }
        }
    }
    if let Some(p) = explicit {
        match state.java.probe_major(&p).await {
            Some(major) if crate::java::java_acceptable(required, major) => {
                return Ok(crate::common::prefer_javaw(&p));
            }
            Some(major) => {
                let _ = app.emit(
                    "game-log",
                    serde_json::json!({
                        "stream": "stderr",
                        "line": format!(
                            "Configured Java {} ({}) is wrong for {} (needs Java {}) — resolving the right one instead.",
                            major, p.display(), options.version_id, required
                        ),
                    }),
                );
            }
            None => {
                tracing::warn!("Configured java path unreadable: {}", p.display());
            }
        }
    }

    match state.java.ensure_java(required).await {
        Some(found) => {
            if !crate::java::java_acceptable(required, found.version.major) {
                // Offline last resort: clearly warn, this runtime may crash.
                let _ = app.emit(
                    "game-log",
                    serde_json::json!({
                        "stream": "stderr",
                        "line": format!(
                            "WARNING: no Java {} found, launching {} with Java {} ({}) — may crash; connect online to auto-download Java {}.",
                            required, options.version_id, found.version.major, found.path.display(), required
                        ),
                    }),
                );
            } else if found.version.major != required {
                let _ = app.emit(
                    "game-log",
                    serde_json::json!({
                        "stream": "stdout",
                        "line": format!(
                            "Using Java {} for {} (preferred {}).",
                            found.version.major, options.version_id, required
                        ),
                    }),
                );
            }
            Ok(crate::common::prefer_javaw(&found.path))
        }
        None => {
            let _ = app.emit(
                "game-log",
                serde_json::json!({
                    "stream": "stderr",
                    "line": format!(
                        "No Java found for {} (needs Java {}); trying PATH.",
                        options.version_id, required
                    ),
                }),
            );
            // fallback: java on PATH
            Ok(crate::common::prefer_javaw(&PathBuf::from("java")))
        }
    }
}

fn build_classpath(details: &crate::versions::VersionDetails, version_id: &str) -> Result<String> {
    let mut cp = Vec::new();
    let client_jar = crate::common::versions_dir()
        .join(version_id)
        .join(format!("{}.jar", version_id));
    cp.push(client_jar.to_string_lossy().to_string());

    for lib in &details.libraries {
        if !crate::versions::library_allowed_on_current_os(lib) {
            continue;
        }
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                // Main jars always go on the classpath. Modern natives entries
                // (4-part coords like `...:natives-windows`) go on it too so the
                // LWJGL3 SharedLibraryLoader finds their nested resources —
                // but only the ones matching this platform.
                let mut include = true;
                if let Some(name) = lib.name.as_deref() {
                    if is_natives_library(name) {
                        let classifier = name.split(':').nth(3).unwrap_or("");
                        include = native_classifier_matches(classifier);
                    }
                }
                if include {
                    let p = crate::common::libraries_dir().join(artifact.local_path());
                    cp.push(p.to_string_lossy().to_string());
                }
            }
        } else if let Some(name) = &lib.name {
            // natives jars are extracted to the natives dir, never on the classpath
            if is_natives_library(name) {
                continue;
            }
            if let Some((_, path)) = legacy_lib_coords_to_path(name) {
                let p = crate::common::libraries_dir().join(&path);
                cp.push(p.to_string_lossy().to_string());
            }
        }
    }
    Ok(cp.join(if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }))
}

fn legacy_lib_coords_to_path(coords: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = coords.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let group_path = group.replace('.', "/");
    // 4th coordinate is a classifier (e.g. natives-windows) — part of the file name
    let filename = match parts.get(3) {
        Some(classifier) => format!("{}-{}-{}.jar", artifact, version, classifier),
        None => format!("{}-{}.jar", artifact, version),
    };
    let path = format!("{}/{}/{}/{}", group_path, artifact, version, filename);
    Some((String::new(), path))
}

async fn extract_natives(
    details: &crate::versions::VersionDetails,
    natives_dir: &PathBuf,
) -> Result<()> {
    for lib in &details.libraries {
        if !crate::versions::library_allowed_on_current_os(lib) {
            continue;
        }

        let excludes = lib
            .extract
            .as_ref()
            .map(|e| e.exclude.clone())
            .unwrap_or_default();

        // Legacy format: natives chosen via downloads.classifiers (e.g. 1.8.9–1.16).
        if let Some(classifiers) = lib.downloads.as_ref().and_then(|d| d.classifiers.as_ref()) {
            if let Some(native) = pick_native_for_extract(classifiers) {
                let jar_path = crate::common::libraries_dir().join(native.local_path());
                if jar_path.exists() {
                    extract_zip(&jar_path, natives_dir, &excludes)?;
                }
            }
            continue;
        }

        // Modern format: natives are separate library entries with a classifier
        // coordinate. Extract the matching ones.
        let name = lib.name.as_deref().unwrap_or("");
        let classifier = name.split(':').nth(3).unwrap_or("");
        let is_native_entry = is_natives_library(name)
            || (!classifier.is_empty()
                && (classifier.starts_with("linux-")
                    || classifier.starts_with("osx-")
                    || classifier.starts_with("macos-")));
        if !is_native_entry || !native_classifier_matches(classifier) {
            continue;
        }

        let jar_path = if let Some(artifact) =
            lib.downloads.as_ref().and_then(|d| d.artifact.as_ref())
        {
            crate::common::libraries_dir().join(artifact.local_path())
        } else if let Some((_, path)) = legacy_lib_coords_to_path(name) {
            crate::common::libraries_dir().join(&path)
        } else {
            continue;
        };
        if !jar_path.exists() {
            continue;
        }
        extract_zip(&jar_path, natives_dir, &excludes)?;
    }
    // Modern LWJGL3 jars nest DLLs (e.g. `windows/x64/org/lwjgl/lwjgl.dll`).
    // Copy every native library to the natives root so `-Djava.library.path`
    // lookups by bare filename succeed.
    flatten_natives(natives_dir)?;
    Ok(())
}

/// Copy *.dll / *.so / *.dylib found in subdirectories of `dir`
/// directly into `dir` (skip-if-identical).
fn flatten_natives(dir: &PathBuf) -> Result<()> {
    fn is_native_file(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        lower.ends_with(".dll")
            || lower.ends_with(".so")
            || lower.ends_with(".dylib")
            || lower.ends_with(".jnilib")
    }

    let mut stack = vec![dir.clone()];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path != *dir {
                    stack.push(path);
                }
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if !is_native_file(&name) {
                continue;
            }
            let dest = dir.join(&name);
            if dest == path {
                continue;
            }
            let same = std::fs::metadata(&dest)
                .ok()
                .and_then(|m| Some(m.len() == path.metadata().ok()?.len()))
                .unwrap_or(false);
            if !same {
                std::fs::copy(&path, &dest)?;
            }
        }
    }
    Ok(())
}

fn pick_native_for_extract(
    classifiers: &std::collections::HashMap<String, crate::versions::Artifact>,
) -> Option<crate::versions::Artifact> {
    let os = crate::common::current_os_name();
    let keys: Vec<&str> = match os {
        "windows" => vec!["natives-windows-64", "natives-windows-32", "natives-windows"],
        "osx" => vec!["natives-macos-arm64", "natives-macos", "natives-osx", "natives-osx-arm64"],
        _ => vec!["natives-linux"],
    };
    for k in keys {
        if let Some(a) = classifiers.get(k) {
            return Some(a.clone());
        }
    }
    // fallback: any natives-*
    for (k, v) in classifiers {
        if k.starts_with("natives") {
            return Some(v.clone());
        }
    }
    None
}

fn extract_zip(jar: &PathBuf, dest: &PathBuf, excludes: &[String]) -> Result<()> {
    let file = std::fs::File::open(jar)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if entry.is_dir() {
            continue;
        }
        if excludes.iter().any(|ex| {
            // META-INF exclusion is most common
            if ex.ends_with('/') {
                name.starts_with(ex)
            } else {
                name == *ex || name.starts_with(ex)
            }
        }) {
            continue;
        }
        // zip slip protection
        let out = dest.join(&name);
        if !out.starts_with(dest) {
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // skip if exists and same size
        if out.exists() {
            if out.metadata().map(|m| m.len()).unwrap_or(0) == entry.size() {
                continue;
            }
        }
        let mut out_file = std::fs::File::create(&out)?;
        std::io::copy(&mut entry, &mut out_file)?;
    }
    Ok(())
}

fn build_jvm_args(
    details: &crate::versions::VersionDetails,
    options: &LaunchOptions,
    settings: &crate::settings::Settings,
    game_dir: &PathBuf,
    natives_dir: &PathBuf,
    classpath: &str,
) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();

    // memory
    let (min_mb, max_mb) = match settings.java.memory_allocation.unit {
        crate::settings::MemoryUnit::GB => (
            settings.java.memory_allocation.min * 1024,
            settings.java.memory_allocation.max * 1024,
        ),
        crate::settings::MemoryUnit::MB => (
            settings.java.memory_allocation.min,
            settings.java.memory_allocation.max,
        ),
    };
    out.push(format!("-Xms{}M", min_mb.max(256)));
    out.push(format!("-Xmx{}M", max_mb.max(min_mb)));

    // base defines
    out.push(format!("-Djava.library.path={}", natives_dir.to_string_lossy()));
    out.push("-Dminecraft.launcher.brand=DontCam Client".to_string());
    out.push("-Dminecraft.launcher.version=0.1.1".to_string());

    // DontCam mod identity: auth source (msa = Microsoft, legacy = offline)
    // and the player's UUID.
    let dontcam_auth = match options.account.account_type {
        crate::auth::AccountType::Microsoft => "msa",
        crate::auth::AccountType::Offline => "legacy",
    };
    out.push(format!("-Ddontcam.auth={}", dontcam_auth));
    out.push(format!("-Ddontcam.uuid={}", options.account.uuid));

    // version-provided jvm args (modern format)
    if let Some(args) = &details.arguments {
        let ctx = jvm_context(options, game_dir, natives_dir, classpath);
        for arg in &args.jvm {
            for expanded in expand_argument(arg, &ctx) {
                // skip classpath placeholder — we add -cp ourselves
                if expanded.contains("${classpath}") {
                    continue;
                }
                out.push(expanded);
            }
        }
    }

    // user jvm args from settings + profile
    for a in split_args(&settings.java.jvm_args) {
        if a.starts_with("-Xms") || a.starts_with("-Xmx") {
            continue; // memory handled above
        }
        out.push(a);
    }
    for a in &options.jvm_args {
        out.push(a.clone());
    }

    // classpath
    out.push("-cp".to_string());
    out.push(classpath.to_string());

    Ok(out)
}

fn build_game_args(
    details: &crate::versions::VersionDetails,
    options: &LaunchOptions,
    settings: &crate::settings::Settings,
    version: &crate::versions::GameVersion,
    game_dir: &PathBuf,
) -> Result<Vec<String>> {
    let ctx = game_context(options, settings, version, game_dir);
    let mut out: Vec<String> = Vec::new();

    if let Some(args) = &details.arguments {
        for arg in &args.game {
            out.extend(expand_argument(arg, &ctx));
        }
    } else if let Some(legacy) = &details.minecraft_arguments {
        out.extend(split_args(&replace_placeholders(legacy, &ctx)));
    } else {
        // minimal fallback (very old / modloader json without args)
        out.extend(
            [
                "--username",
                &ctx["auth_player_name"],
                "--version",
                &ctx["version_name"],
                "--gameDir",
                &ctx["game_directory"],
                "--assetsDir",
                &ctx["assets_root"],
                "--assetIndex",
                &ctx["assets_index_name"],
                "--uuid",
                &ctx["auth_uuid"],
                "--accessToken",
                &ctx["auth_access_token"],
                "--userType",
                &ctx["user_type"],
                "--versionType",
                &ctx["version_type"],
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    }

    // resolution
    let res = options.resolution.clone().or_else(|| {
        settings.game.custom_resolution.clone().map(|r| crate::profiles::Resolution {
            width: r.width,
            height: r.height,
            fullscreen: settings.game.fullscreen,
        })
    });
    if let Some(r) = res {
        if !out.contains(&"--width".to_string()) {
            out.push("--width".to_string());
            out.push(r.width.to_string());
            out.push("--height".to_string());
            out.push(r.height.to_string());
        }
        if r.fullscreen && !out.contains(&"--fullscreen".to_string()) {
            out.push("--fullscreen".to_string());
        }
    }

    // quick-play server
    if let Some(server) = &options.server {
        out.push("--server".to_string());
        out.push(server.host.clone());
        out.push("--port".to_string());
        out.push(server.port.to_string());
    }

    // profile + global custom args
    out.extend(options.game_args.clone());
    out.extend(settings.advanced.custom_game_args.clone());

    // filter empty
    Ok(out.into_iter().filter(|s| !s.is_empty()).collect())
}

fn jvm_context(
    options: &LaunchOptions,
    game_dir: &PathBuf,
    natives_dir: &PathBuf,
    classpath: &str,
) -> std::collections::HashMap<String, String> {
    let mut m = std::collections::HashMap::new();
    m.insert("natives_directory".to_string(), natives_dir.to_string_lossy().to_string());
    m.insert("launcher_name".to_string(), "DontCam Client".to_string());
    m.insert("launcher_version".to_string(), "0.1.1".to_string());
    m.insert("classpath".to_string(), classpath.to_string());
    m.insert("game_directory".to_string(), game_dir.to_string_lossy().to_string());
    m.insert("auth_player_name".to_string(), options.account.username.clone());
    m
}

fn game_context(
    options: &LaunchOptions,
    settings: &crate::settings::Settings,
    version: &crate::versions::GameVersion,
    game_dir: &PathBuf,
) -> std::collections::HashMap<String, String> {
    let mut m = std::collections::HashMap::new();
    m.insert("auth_player_name".to_string(), options.account.username.clone());
    m.insert("version_name".to_string(), options.version_id.clone());
    m.insert("game_directory".to_string(), game_dir.to_string_lossy().to_string());
    m.insert("assets_root".to_string(), crate::common::assets_dir().to_string_lossy().to_string());
    m.insert("assets_index_name".to_string(), version.asset_index.id.clone());
    m.insert("auth_uuid".to_string(), options.account.uuid.replace('-', ""));
    m.insert(
        "auth_access_token".to_string(),
        options.account.access_token.clone().unwrap_or_else(|| "0".to_string()),
    );
    let user_type = match options.account.account_type {
        crate::auth::AccountType::Microsoft => "msa",
        crate::auth::AccountType::Offline => "legacy",
    };
    m.insert("user_type".to_string(), user_type.to_string());
    m.insert("version_type".to_string(), "DontCam Client".to_string());
    m.insert("game_assets".to_string(), crate::common::assets_dir().to_string_lossy().to_string());
    m.insert("auth_session".to_string(), "0".to_string());
    // features
    m.insert("has_custom_resolution".to_string(), "false".to_string());
    m.insert("is_demo_user".to_string(), "false".to_string());
    let _ = settings;
    m
}

fn expand_argument(
    arg: &crate::versions::Argument,
    ctx: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    match arg {
        crate::versions::Argument::Plain(s) => vec![replace_placeholders(s, ctx)],
        crate::versions::Argument::Ruled { rules, value } => {
            // evaluate rules: all must allow (simplified: any allow wins unless disallow matches)
            let mut allowed = false;
            let mut has_allow_rule = false;
            for rule in rules {
                let matches = rule_matches(rule, ctx);
                if rule.action == "allow" {
                    has_allow_rule = true;
                    if matches {
                        allowed = true;
                    }
                } else if rule.action == "disallow" && matches {
                    return vec![];
                }
            }
            if !has_allow_rule {
                allowed = true;
            }
            if !allowed {
                return vec![];
            }
            match value {
                serde_json::Value::String(s) => vec![replace_placeholders(s, ctx)],
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| replace_placeholders(s, ctx))
                    .collect(),
                _ => vec![],
            }
        }
    }
}

fn rule_matches(
    rule: &crate::versions::Rule,
    ctx: &std::collections::HashMap<String, String>,
) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            let current = crate::common::current_os_name();
            let ok = match name.as_str() {
                "windows" => current == "windows",
                "osx" | "macos" => current == "osx",
                "linux" => current == "linux",
                _ => false,
            };
            if !ok {
                return false;
            }
        }
        // arch + version ignored (rarely blocking)
    }
    if let Some(features) = &rule.features {
        for (k, expected) in features {
            let actual = ctx.get(k.as_str()).map(|v| v == "true").unwrap_or(false);
            if actual != *expected {
                return false;
            }
        }
    }
    true
}

fn replace_placeholders(s: &str, ctx: &std::collections::HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in ctx {
        out = out.replace(&format!("${{{}}}", k), v);
    }
    out
}

fn split_args(s: &str) -> Vec<String> {
    // split respecting quotes
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    for c in s.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' | '\n' if !in_quotes => {
                if !cur.is_empty() {
                    out.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[tauri::command]
pub async fn get_launch_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<LaunchStatus, String> {
    Ok(state.launch.launch_status.read().await.clone())
}

#[tauri::command]
pub async fn kill_game(state: tauri::State<'_, crate::AppState>) -> Result<(), String> {
    let mut process = state.launch.current_process.lock().await;
    if let Some(child) = process.as_mut() {
        child.kill().await.map_err(|e| e.to_string())?;
    }
    *process = None;
    drop(process);
    set_status(&state, LaunchStatus::Idle).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: these expectations assume Windows x64 (CI/dev machine).
    #[test]
    fn natives_matcher_windows_x64() {
        assert!(native_classifier_matches("natives-windows"));
        assert!(!native_classifier_matches("natives-windows-arm64"));
        assert!(!native_classifier_matches("natives-windows-x86"));
        assert!(!native_classifier_matches("natives-linux"));
        assert!(!native_classifier_matches("natives-macos"));
        assert!(!native_classifier_matches("natives-macos-arm64"));
        assert!(!native_classifier_matches("linux-x86_64"));
        assert!(is_natives_library("org.lwjgl:lwjgl-glfw:3.3.3:natives-windows"));
        assert!(!is_natives_library("org.lwjgl:lwjgl-glfw:3.3.3"));
        assert!(!is_natives_library(
            "io.netty:netty-transport-native-epoll:4.2.7.Final:linux-x86_64"
        ));
    }

    #[test]
    fn legacy_coords_with_classifier() {
        let (_, path) =
            legacy_lib_coords_to_path("org.lwjgl:lwjgl-glfw:3.3.3:natives-windows").expect("coords");
        assert_eq!(
            path,
            "org/lwjgl/lwjgl-glfw/3.3.3/lwjgl-glfw-3.3.3-natives-windows.jar"
        );
    }

    #[test]
    fn mod_line_matching() {
        assert!(mod_applies("1.20", "1.20.1"));
        assert!(mod_applies("1.20", "1.20.1-forge-47.2.0"));
        assert!(!mod_applies("1.2", "1.21.1"));
        assert!(!mod_applies("1.20", "1.21.1"));
    }

    fn test_details(
        id: &str,
        main_class: &str,
        game_args: Vec<&str>,
        asset_id: Option<&str>,
    ) -> crate::versions::VersionDetails {
        use crate::versions::{Argument, AssetIndex, DownloadsOpt, GameArguments};
        crate::versions::VersionDetails {
            id: id.to_string(),
            type_: "release".to_string(),
            main_class: main_class.to_string(),
            arguments: Some(GameArguments {
                game: game_args.into_iter().map(|s| Argument::Plain(s.to_string())).collect(),
                jvm: vec![],
            }),
            minecraft_arguments: None,
            libraries: vec![],
            asset_index: asset_id.map(|a| AssetIndex {
                id: a.to_string(),
                sha1: String::new(),
                size: 0,
                total_size: 0,
                url: String::new(),
            }),
            assets: None,
            downloads: DownloadsOpt::default(),
            java_version: None,
            minimum_launcher_version: None,
        }
    }

    #[test]
    fn merges_loader_profile_over_vanilla() {
        let base = test_details(
            "1.20.1",
            "net.minecraft.client.main.Main",
            vec!["--username", "${auth_player_name}"],
            Some("5"),
        );
        // loader profiles omit assetIndex -> inherited from the base
        let profile = test_details(
            "1.20.1-forge-47.2.0",
            "cpw.mods.bootstraplauncher.BootstrapLauncher",
            vec!["--launchTarget", "forgeclient"],
            None,
        );
        let merged = merge_inherited(profile, &base);
        assert_eq!(merged.main_class, "cpw.mods.bootstraplauncher.BootstrapLauncher");
        let game = merged.arguments.expect("args").game;
        assert_eq!(game.len(), 4);
        assert_eq!(merged.asset_index.expect("assets").id, "5");
        assert_eq!(merged.id, "1.20.1-forge-47.2.0");
    }

    #[tokio::test]
    async fn injects_bundled_mod() {
        let dir = std::env::temp_dir().join(format!(
            "dontcam-inject-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let mods = dir.join("mods");
        tokio::fs::create_dir_all(&mods).await.unwrap();
        // Unknown lines: nothing bundled, but stale foreign jars are cleaned.
        tokio::fs::write(mods.join("dontcam-9.9-0.0.1.jar"), b"stale").await.unwrap();
        ensure_dontcam_mod(&dir, "9.9.9").await.unwrap();
        // Lines with a bundled port: the right jar lands, others go away.
        tokio::fs::write(mods.join("dontcam-9.9-0.0.1.jar"), b"stale").await.unwrap();
        ensure_dontcam_mod(&dir, "1.20.1").await.unwrap();
        let mut entries = tokio::fs::read_dir(&mods).await.unwrap();
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        assert_eq!(names, vec!["dontcam-1.20.1-0.1.0.jar".to_string()]);
        tokio::fs::remove_dir_all(&dir).await.ok();
    }

    #[test]
    fn effective_loader_prefers_matching_profile_but_canonical_otherwise() {
        use crate::profiles::ModLoaderType as PL;
        // Profile made for THIS version: its choice wins (manual NeoForge kept).
        assert_eq!(effective_loader(&PL::NeoForge, "1.21.1", "1.21.1"), PL::NeoForge);
        assert_eq!(effective_loader(&PL::Fabric, "1.20.1", "1.20.1"), PL::Fabric);
        assert_eq!(effective_loader(&PL::Forge, "1.8.9", "1.8.9"), PL::Forge);
        // Profile made for ANOTHER version: canonical loader for the launched
        // version wins (a 1.8.9 Forge profile must not force Forge on 1.20.1).
        assert_eq!(effective_loader(&PL::Forge, "1.8.9", "1.20.1"), PL::Fabric);
        assert_eq!(effective_loader(&PL::Fabric, "1.20.1", "1.8.9"), PL::Forge);
        assert_eq!(effective_loader(&PL::Fabric, "1.21.1", "1.20.1"), PL::Fabric);
        // No profile / loader unset: canonical default, never vanilla.
        assert_eq!(effective_loader(&PL::None, "", "1.20.1"), PL::Fabric);
        assert_eq!(effective_loader(&PL::None, "", "1.12.2"), PL::Forge);
        assert_eq!(effective_loader(&PL::None, "1.20.1", "1.20.1"), PL::Fabric);
    }

    #[test]
    fn mod_asset_mapping() {
        assert_eq!(mod_asset_for("1.20.1"), Some(("1.20", "dontcam-1.20.1-0.1.0.jar")));
        assert_eq!(mod_asset_for("1.8.9"), Some(("1.8", "dontcam-1.8.9-0.1.0.jar")));
        assert_eq!(mod_asset_for("1.16.5"), Some(("1.16", "dontcam-1.16.5-0.1.0.jar")));
    }
}
