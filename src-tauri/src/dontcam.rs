//! DontCam mod provisioning from the PUBLIC `aanges/MODY` GitHub repo.
//! Downloads use plain public HTTPS — NO tokens, NO passwords, NO auth.
//!
//! Folder mapping (repo folder -> Minecraft + loader). MC/loader versions
//! are FIXED — never change them here:
//!   1.8  -> MC 1.8.9  Forge    1.12 -> MC 1.12.2 Forge
//!   1.16 -> MC 1.16.5 Fabric   1.17 -> MC 1.17.1 Fabric
//!   1.18 -> MC 1.18.2 Fabric   1.19 -> MC 1.19.4 Fabric
//!   1.20 -> MC 1.20.1 Fabric   1.21 -> MC 1.21.1 Fabric
//!
//! Download order per MC version:
//!   1. `dontcam-<mc>-*.jar` asset from the latest MODY GitHub Release,
//!   2. `.jar` found in the version folder on branch `main`
//!      (raw.githubusercontent.com),
//!   3. embedded offline jar from `src-tauri/resources/dontcam/`
//!      (same mod, shipped with the launcher — the mod itself is untouched).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use tauri::Emitter;

pub(crate) const MODY_REPO: &str = "aanges/MODY";
pub(crate) const MODY_BRANCH: &str = "main";

/// (repo folder, MC version, loader id)
pub(crate) const FOLDER_MAP: &[(&str, &str, &str)] = &[
    ("1.8", "1.8.9", "forge"),
    ("1.12", "1.12.2", "forge"),
    ("1.16", "1.16.5", "fabric"),
    ("1.17", "1.17.1", "fabric"),
    ("1.18", "1.18.2", "fabric"),
    ("1.19", "1.19.4", "fabric"),
    ("1.20", "1.20.1", "fabric"),
    ("1.21", "1.21.1", "fabric"),
];

pub(crate) fn mapping_for_mc(mc_version: &str) -> Option<(&'static str, &'static str, &'static str)> {
    let base = mc_version.split('-').next().unwrap_or(mc_version);
    FOLDER_MAP
        .iter()
        .find(|(_, mc, _)| *mc == base)
        .copied()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DontcamRelease {
    pub mc_version: String,
    pub folder: String,
    pub loader: String,
    pub assets: Vec<String>,
    pub source: String,
}

fn client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("DontCam-Client/0.1")
        .build()?)
}

fn public(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    // Public repo: Accept header only, never any Authorization.
    builder
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
}

pub(crate) fn friendly_github_error(status: reqwest::StatusCode, what: &str) -> String {
    match status.as_u16() {
        403 => format!(
            "GitHub odrzucił żądanie (403) przy {}. To zwykle chwilowy limit API \
             dla niezalogowanych — odczekaj minutę i spróbuj ponownie.",
            what
        ),
        404 => format!(
            "Nie znaleziono zasobu (404) przy {}. Sprawdź czy w publicznym repo {} \
             na branchu {} istnieje dany plik/release.",
            what, MODY_REPO, MODY_BRANCH
        ),
        _ => format!("GitHub API error {} przy {}", status, what),
    }
}

async fn get_json(url: &str, what: &str) -> anyhow::Result<serde_json::Value> {
    let resp = public(client()?.get(url)).send().await?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("{}", friendly_github_error(status, what));
    }
    Ok(resp.json().await?)
}

async fn download_bytes(url: &str) -> anyhow::Result<bytes::Bytes> {
    let resp = public(client()?.get(url)).send().await?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("{}", friendly_github_error(status, "pobieraniu pliku"));
    }
    Ok(resp.bytes().await?)
}

fn bytes_are_jar(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04
}

#[tauri::command]
pub async fn get_dontcam_releases() -> Result<Vec<DontcamRelease>, String> {
    // Live asset names from the latest public MODY release (may be missing —
    // the repo currently ships sources only, no Releases).
    let mut release_assets: Vec<String> = Vec::new();
    match get_json(
        &format!("https://api.github.com/repos/{}/releases/latest", MODY_REPO),
        "liście release",
    )
    .await
    {
        Ok(api) => {
            if let Some(arr) = api.get("assets").and_then(|a| a.as_array()) {
                for a in arr {
                    if let Some(n) = a.get("name").and_then(|n| n.as_str()) {
                        release_assets.push(n.to_string());
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("MODY releases lookup failed ({}), using repo tree", e);
        }
    }

    // Per-folder jars from the public tree (also empty today — sources only).
    // Failures here are non-fatal: the static mapping below always stands,
    // and install falls back to the embedded offline jars.
    let mut out = Vec::with_capacity(FOLDER_MAP.len());
    for (folder, mc, loader) in FOLDER_MAP.iter() {
        let prefix = format!("dontcam-{}", mc);
        let mut assets: Vec<String> = release_assets
            .iter()
            .filter(|n| n.starts_with(&prefix) && n.ends_with(".jar"))
            .cloned()
            .collect();
        let mut source = if assets.is_empty() { "embedded" } else { "release" }.to_string();
        if assets.is_empty() {
            match tree_jars(folder).await {
                Ok(names) => {
                    assets = names;
                    if !assets.is_empty() {
                        source = "repo-tree".to_string();
                    }
                }
                Err(e) => {
                    tracing::warn!("MODY tree lookup for folder {} failed ({})", folder, e);
                }
            }
        }
        out.push(DontcamRelease {
            mc_version: mc.to_string(),
            folder: folder.to_string(),
            loader: loader.to_string(),
            assets,
            source,
        });
    }
    Ok(out)
}

/// `.jar` file names directly inside a version folder on branch main.
/// Empty when the folder holds sources only (current MODY state).
async fn tree_jars(folder: &str) -> anyhow::Result<Vec<String>> {
    let tree: serde_json::Value = get_json(
        &format!(
            "https://api.github.com/repos/{}/contents/{}?ref={}",
            MODY_REPO, folder, MODY_BRANCH
        ),
        &format!("folderowi {}", folder),
    )
    .await?;
    let mut names = Vec::new();
    if let Some(items) = tree.as_array() {
        for i in items {
            let is_file = i.get("type").and_then(|t| t.as_str()) == Some("file");
            if let Some(n) = i.get("name").and_then(|n| n.as_str()) {
                if is_file && n.ends_with(".jar") {
                    names.push(n.to_string());
                }
            }
        }
    }
    Ok(names)
}

fn emit_progress(app: &tauri::AppHandle, version_id: &str, stage: &str, current: u64, total: u64) {
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

/// True when a loader profile for this MC line already exists on disk
/// (anything besides the vanilla dir that mentions the MC version).
async fn loader_installed_for(mc_version: &str) -> bool {
    let base = mc_version.split('-').next().unwrap_or(mc_version);
    if let Ok(mut entries) = tokio::fs::read_dir(crate::common::versions_dir()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == base {
                continue;
            }
            if name.contains(base)
                && (name.contains("fabric")
                    || name.contains("forge")
                    || name.contains("quilt")
                    || name.contains("neoforge"))
            {
                return true;
            }
        }
    }
    false
}

#[tauri::command]
pub async fn install_dontcam_mod(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    mc_version: String,
    profile_id: Option<String>,
) -> Result<String, String> {
    let (folder, mc, loader) =
        mapping_for_mc(&mc_version).ok_or_else(|| format!("Nieobsługiwana wersja MC: {}", mc_version))?;
    let tag = format!("dontcam-{}", mc);
    emit_progress(&app, &tag, "start", 0, 100);

    // 1. Loader first: Forge for 1.8.9/1.12.2, Fabric for 1.16–1.21.
    if loader_installed_for(mc).await {
        emit_progress(&app, &tag, "loader-ok", 10, 100);
    } else {
        emit_progress(&app, &tag, "loader-install", 10, 100);
        let loader_ty = crate::modloaders::default_loader_for(mc);
        let builds = state
            .modloaders
            .get_modloader_versions(loader_ty, mc)
            .await
            .map_err(|e| format!("Nie udało się pobrać listy buildów {} dla {}: {}", loader, mc, e))?;
        let preferred = builds.into_iter().next().ok_or_else(|| {
            format!("Brak buildów {} dla Minecraft {} — sprawdź połączenie.", loader, mc)
        })?;
        state
            .modloaders
            .install_modloader(&preferred, &crate::common::base_dir())
            .await
            .map_err(|e| format!("Instalacja loadera {} nie powiodła się: {}", loader, e))?;
        state.versions.scan_installed_versions().await.ok();
        emit_progress(&app, &tag, "loader-done", 30, 100);
    }

    // 2. Target mods dir: instances/<profile>/mods/.
    let game_dir = match profile_id.as_deref() {
        Some(pid) if !pid.is_empty() => {
            crate::common::game_dir_for_profile(&state.settings, Some(pid)).await
        }
        _ => crate::common::game_dir_for_profile(&state.settings, None).await,
    };
    let mods_dir = game_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Remove the old DontCam jar so it gets overwritten cleanly.
    if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("dontcam-") && name.ends_with(".jar") {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
    }
    emit_progress(&app, &tag, "mod-download", 40, 100);

    // 3. Download: latest public MODY release asset first, then the public
    // repo tree file. No releases / no jars there today (sources only),
    // so normally this lands on the embedded offline jar — same mod.
    let (file_name, bytes) = match fetch_mody_jar(folder, mc).await {
        Ok(hit) => hit,
        Err(e) => {
            tracing::info!("Remote DontCam fetch unavailable ({}), using embedded jar", e);
            emit_progress(&app, &tag, "mod-embedded", 60, 100);
            embedded_mody_jar(mc).ok_or_else(|| {
                format!("Brak jara DontCam dla {} ani zdalnie, ani lokalnie: {}", mc, e)
            })?
        }
    };
    if !bytes_are_jar(&bytes) {
        return Err("Pobrany plik nie jest jarem (zła odpowiedź serwera?)".to_string());
    }
    emit_progress(&app, &tag, "mod-save", 90, 100);
    let dest = mods_dir.join(&file_name);
    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| format!("Zapis moda nie powiódł się: {}", e))?;

    // 4. Fabric lines also need Fabric API (shared helper).
    if loader == "fabric" {
        if let Err(e) = state.modloaders.ensure_fabric_api(&mods_dir, mc).await {
            tracing::warn!("Fabric API staging failed: {}", e);
        }
    }

    emit_progress(&app, &tag, "done", 100, 100);
    Ok(dest.to_string_lossy().to_string())
}

/// Release asset `dontcam-<mc>-*.jar` first, else first `.jar` under the
/// version folder on branch main — all over plain public HTTPS, no auth.
async fn fetch_mody_jar(folder: &str, mc: &str) -> anyhow::Result<(String, bytes::Bytes)> {
    // A. Latest release assets.
    if let Ok(api) = get_json(
        &format!("https://api.github.com/repos/{}/releases/latest", MODY_REPO),
        "liście release",
    )
    .await
    {
        if let Some(arr) = api.get("assets").and_then(|a| a.as_array()) {
            let prefix = format!("dontcam-{}", mc);
            let hit = arr.iter().find(|a| {
                a.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.starts_with(&prefix) && n.ends_with(".jar"))
                    .unwrap_or(false)
            });
            if let Some(asset) = hit {
                let name = asset
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let url = asset
                    .get("browser_download_url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                if !url.is_empty() && !name.is_empty() {
                    let bytes = download_bytes(url).await?;
                    tracing::info!("Downloaded DontCam {} from MODY release", name);
                    return Ok((name, bytes));
                }
            }
        }
    }

    // B. Repo tree: list the version folder, take the matching jar.
    let tree: serde_json::Value = get_json(
        &format!(
            "https://api.github.com/repos/{}/contents/{}?ref={}",
            MODY_REPO, folder, MODY_BRANCH
        ),
        &format!("folderowi {}", folder),
    )
    .await?;
    let items = tree
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Folder {} nie jest listą plików", folder))?;
    let jars: Vec<&serde_json::Value> = items
        .iter()
        .filter(|i| {
            i.get("type").and_then(|t| t.as_str()) == Some("file")
                && i.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.ends_with(".jar"))
                    .unwrap_or(false)
        })
        .collect();
    if jars.is_empty() {
        anyhow::bail!("Brak plików .jar w folderze {} repo {}", folder, MODY_REPO);
    }
    let prefix = format!("dontcam-{}", mc);
    let pick = jars
        .iter()
        .find(|i| {
            i.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.contains(mc))
                .unwrap_or(false)
        })
        .or(jars.first())
        .copied()
        .unwrap();
    let name = pick
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(&format!("{}.jar", prefix))
        .to_string();
    // Public tree: the API download_url, else raw.githubusercontent.com.
    // Both work without any token on a public repo.
    let url = pick
        .get("download_url")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                MODY_REPO, MODY_BRANCH, folder, name
            )
        });
    let bytes = download_bytes(&url).await?;
    tracing::info!("Downloaded DontCam {} from MODY tree ({})", name, folder);
    Ok((name, bytes))
}

/// Embedded offline fallback: jar shipped in `src-tauri/resources/dontcam/`.
/// Same DontCam mod, only used when MODY has no downloadable jar.
fn embedded_mody_jar(mc: &str) -> Option<(String, bytes::Bytes)> {
    let bundled = crate::launch::bundled_mod_for(mc)?;
    Some((bundled.file_name.to_string(), bytes::Bytes::from_static(bundled.bytes)))
}

// Re-exported so AppState wiring stays one line per command.
#[allow(dead_code)]
pub(crate) fn repo() -> &'static str {
    MODY_REPO
}

/// Base MC version hidden inside any version id (vanilla `1.20.1`,
/// `1.20.1-forge-47.2.0`, `fabric-loader-0.16.14-1.20.1`, ...).
/// Longest match first so `1.21.1` never collapses into a shorter line.
pub(crate) fn mc_of_version(version_id: &str) -> Option<&'static str> {
    let mut lines: Vec<&'static str> = FOLDER_MAP.iter().map(|(_, mc, _)| *mc).collect();
    lines.sort_by(|a, b| b.len().cmp(&a.len()));
    lines.into_iter().find(|mc| {
        version_id == *mc
            || version_id.contains(&format!("{}-", mc))
            || version_id.contains(&format!("{}.", mc))
            || version_id.ends_with(*mc)
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

fn find_local_dontcam(mods_dir: &std::path::PathBuf) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(mods_dir).ok()?;
    let mut hits: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("DontCam") || n.starts_with("dontcam-"))
                .unwrap_or(false)
                && p.extension().and_then(|e| e.to_str()) == Some("jar")
        })
        .collect();
    hits.sort();
    hits.into_iter().next()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DontcamCheck {
    pub updated: bool,
    pub version: String,
    pub path: String,
    pub offline: bool,
    pub message: String,
}

/// Dedup back-to-back checks (doLaunch + launchGame hit this twice per Play):
/// same mods content re-checked within `CHECK_TTL` returns the cached row.
static CHECK_CACHE: LazyLock<Mutex<HashMap<String, (Instant, String, DontcamCheck)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
const CHECK_TTL_SECS: u64 = 120;

/// Pre-launch DontCam freshness check, called before EVERY game start.
/// Never blocks the game: network trouble returns `offline: true` and the
/// game plays on the local jar. Only hard-fails when no jar exists anywhere.
#[tauri::command]
pub async fn check_dontcam_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    mc_version: String,
    profile_id: Option<String>,
) -> Result<DontcamCheck, String> {
    let mc = mc_of_version(&mc_version)
        .ok_or_else(|| format!("Nieobsługiwana wersja MC: {}", mc_version))?;
    let (folder, _, _) = mapping_for_mc(mc).expect("mc comes from the same table");
    let tag = format!("dontcam-{}", mc);

    let game_dir = match profile_id.as_deref() {
        Some(pid) if !pid.is_empty() => {
            crate::common::game_dir_for_profile(&state.settings, Some(pid)).await
        }
        _ => crate::common::game_dir_for_profile(&state.settings, None).await,
    };
    let mods_dir = game_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir)
        .await
        .map_err(|e| e.to_string())?;

    let local = find_local_dontcam(&mods_dir);
    let local_hash = match &local {
        Some(p) => tokio::fs::read(p)
            .await
            .map(|b| sha256_hex(&b))
            .unwrap_or_default(),
        None => String::new(),
    };
    let cache_key = format!("{}|{}", mc, mods_dir.to_string_lossy());

    // Fresh cache + unchanged local jar -> no network at all.
    if let Ok(cache) = CHECK_CACHE.lock() {
        if let Some((at, hash, row)) = cache.get(&cache_key) {
            if *hash == local_hash && at.elapsed().as_secs() < CHECK_TTL_SECS {
                return Ok(row.clone());
            }
        }
    }

    emit_progress(&app, &tag, "dontcam-check", 5, 100);
    let done = |row: DontcamCheck| {
        if let Ok(mut cache) = CHECK_CACHE.lock() {
            cache.insert(cache_key.clone(), (Instant::now(), local_hash.clone(), row.clone()));
        }
        row
    };

    match fetch_mody_jar(folder, mc).await {
        Ok((name, bytes)) => {
            let remote_hash = sha256_hex(&bytes);
            if !local_hash.is_empty() && local_hash == remote_hash {
                return Ok(done(DontcamCheck {
                    updated: false,
                    version: name,
                    path: local
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    offline: false,
                    message: format!("DontCam dla {} aktualny.", mc),
                }));
            }
            emit_progress(&app, &tag, "dontcam-download", 50, 100);
            remove_old_dontcam(&mods_dir).await;
            let dest = mods_dir.join(&name);
            tokio::fs::write(&dest, &bytes)
                .await
                .map_err(|e| format!("Zapis moda nie powiódł się: {}", e))?;
            emit_progress(&app, &tag, "done", 100, 100);
            Ok(done(DontcamCheck {
                updated: true,
                version: name.clone(),
                path: dest.to_string_lossy().to_string(),
                offline: false,
                message: format!("DontCam dla {} zaktualizowany.", mc),
            }))
        }
        Err(e) => {
            if local.is_some() {
                // Network trouble (or no remote jar yet) -> play on the old mod.
                return Ok(done(DontcamCheck {
                    updated: false,
                    version: local
                        .as_ref()
                        .and_then(|p| p.file_name().and_then(|n| n.to_str()))
                        .unwrap_or("")
                        .to_string(),
                    path: local
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    offline: true,
                    message: format!(
                        "Check sieciowy DontCam nie powiódł się ({}). Gram na lokalnym modzie.",
                        short_err(&e.to_string())
                    ),
                }));
            }
            // First run without any jar -> embedded offline copy so the
            // game still starts with the mod.
            emit_progress(&app, &tag, "mod-embedded", 60, 100);
            let (name, bytes) = embedded_mody_jar(mc).ok_or_else(|| {
                format!("Brak jara DontCam dla {} ani zdalnie, ani lokalnie: {}", mc, e)
            })?;
            let dest = mods_dir.join(&name);
            tokio::fs::write(&dest, &bytes)
                .await
                .map_err(|e| format!("Zapis moda nie powiódł się: {}", e))?;
            emit_progress(&app, &tag, "done", 100, 100);
            Ok(done(DontcamCheck {
                updated: true,
                version: name.clone(),
                path: dest.to_string_lossy().to_string(),
                offline: true,
                message: format!("Brak sieci — użyto wbudowanego moda DontCam dla {}.", mc),
            }))
        }
    }
}

async fn remove_old_dontcam(mods_dir: &std::path::PathBuf) {
    if let Ok(mut entries) = tokio::fs::read_dir(mods_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if (name.starts_with("dontcam-") || name.starts_with("DontCam"))
                && name.ends_with(".jar")
            {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
    }
}

/// First sentence only — keeps console/UI messages readable.
fn short_err(msg: &str) -> String {
    msg.split(['.', '\n'])
        .next()
        .unwrap_or(msg)
        .trim()
        .chars()
        .take(160)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mc_of_version_covers_all_forms() {
        assert_eq!(mc_of_version("1.20.1"), Some("1.20.1"));
        assert_eq!(mc_of_version("1.8.9"), Some("1.8.9"));
        assert_eq!(
            mc_of_version("1.20.1-forge-47.2.0"),
            Some("1.20.1")
        );
        assert_eq!(
            mc_of_version("fabric-loader-0.16.14-1.20.1"),
            Some("1.20.1")
        );
        assert_eq!(mc_of_version("1.21.1"), Some("1.21.1"));
        assert_eq!(mc_of_version("9.9.9"), None);
    }

    #[test]
    fn mapping_matches_task_table() {
        // MC -> folder, Forge for 1.8.9/1.12.2, Fabric for the rest.
        let table = [
            ("1.8.9", "1.8", "forge"),
            ("1.12.2", "1.12", "forge"),
            ("1.16.5", "1.16", "fabric"),
            ("1.17.1", "1.17", "fabric"),
            ("1.18.2", "1.18", "fabric"),
            ("1.19.4", "1.19", "fabric"),
            ("1.20.1", "1.20", "fabric"),
            ("1.21.1", "1.21", "fabric"),
        ];
        for (mc, folder, loader) in table {
            assert_eq!(mapping_for_mc(mc), Some((folder, mc, loader)));
        }
    }
}
