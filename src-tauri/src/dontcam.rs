//! DontCam mod provisioning from the private `aanges/MODY` GitHub repo.
//!
//! Folder mapping (repo folder -> Minecraft + loader). MC/loader versions
//! are FIXED — never change them here:
//!   1.8  -> MC 1.8.9  Forge    1.12 -> MC 1.12.2 Forge
//!   1.16 -> MC 1.16.5 Fabric   1.17 -> MC 1.17.1 Fabric
//!   1.18 -> MC 1.18.2 Fabric   1.19 -> MC 1.19.4 Fabric
//!   1.20 -> MC 1.20.1 Fabric   1.21 -> MC 1.21.1 Fabric
//!
//! Auth: token comes ONLY from the `GITHUB_TOKEN` env var, or — as a
//! fallback — from the `gh` CLI credential store (`gh auth token`).
//! The token is used in-memory for the `Authorization` header and is NEVER
//! written to code, json, logs or git history.

use serde::{Deserialize, Serialize};
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

/// Read-only token lookup: `GITHUB_TOKEN` env, else `gh auth token`.
/// Returns None when neither is available. Never logs the value.
fn github_token() -> Option<String> {
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        let t = t.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    // `gh` keeps its token in the OS credential store — reuse it without
    // ever persisting anything ourselves.
    let mut cmd = std::process::Command::new("gh");
    cmd.args(["auth", "token"]);
    crate::common::hide_console_std(&mut cmd);
    match cmd.output() {
        Ok(out) if out.status.success() => {
            let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        }
        _ => None,
    }
}

fn client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("DontCam-Client/0.1")
        .build()?)
}

fn authed(
    builder: reqwest::RequestBuilder,
    token: Option<&str>,
) -> reqwest::RequestBuilder {
    let b = builder.header("Accept", "application/vnd.github+json");
    match token {
        Some(t) => b
            .header("Authorization", format!("Bearer {}", t))
            .header("X-GitHub-Api-Version", "2022-11-28"),
        None => b,
    }
}

pub(crate) fn friendly_github_error(status: reqwest::StatusCode, what: &str) -> String {
    match status.as_u16() {
        401 => format!(
            "Brak dostępu do prywatnego repo {} (401 Unauthorized) przy {}. \
             Ustaw zmienną środowiskową GITHUB_TOKEN albo zaloguj się `gh auth login`, \
             potem zrestartuj launcher.",
            MODY_REPO, what
        ),
        403 => format!(
            "GitHub odrzucił żądanie (403) przy {}. Możliwe wyczerpanie limitu API \
             albo brak scope `repo` na tokenie.",
            what
        ),
        404 => format!(
            "Nie znaleziono zasobu (404) przy {}. Sprawdź czy repo {}, branch {} \
             i dany plik/release istnieją oraz czy token ma dostęp.",
            what, MODY_REPO, MODY_BRANCH
        ),
        _ => format!("GitHub API error {} przy {}", status, what),
    }
}

async fn get_json(
    url: &str,
    token: Option<&str>,
    what: &str,
) -> anyhow::Result<serde_json::Value> {
    let resp = authed(client()?.get(url), token).send().await?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("{}", friendly_github_error(status, what));
    }
    Ok(resp.json().await?)
}

async fn download_bytes(url: &str, token: Option<&str>) -> anyhow::Result<bytes::Bytes> {
    // browser_download_url / raw URLs: auth header is harmless when public.
    let resp = authed(client()?.get(url), token).send().await?;
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
    let token = github_token();
    let tok = token.as_deref();

    // Live asset names from the latest MODY release (may be empty / missing).
    let mut release_assets: Vec<String> = Vec::new();
    let mut source = "repo-tree";
    match get_json(
        &format!("https://api.github.com/repos/{}/releases/latest", MODY_REPO),
        tok,
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
            if !release_assets.is_empty() {
                source = "release";
            }
        }
        Err(e) => {
            // Private repo without token (401) or no releases yet (404):
            // still return the static mapping so the UI can offer versions,
            // unless it is an auth problem — that one must be loud.
            let msg = e.to_string();
            if msg.contains("(401") {
                return Err(msg);
            }
            tracing::warn!("MODY releases lookup failed ({}), using repo tree mapping", msg);
        }
    }

    let out: Vec<DontcamRelease> = FOLDER_MAP
        .iter()
        .map(|(folder, mc, loader)| {
            let prefix = format!("dontcam-{}", mc);
            let assets: Vec<String> = release_assets
                .iter()
                .filter(|n| n.starts_with(&prefix) && n.ends_with(".jar"))
                .cloned()
                .collect();
            DontcamRelease {
                mc_version: mc.to_string(),
                folder: folder.to_string(),
                loader: loader.to_string(),
                assets,
                source: source.to_string(),
            }
        })
        .collect();
    Ok(out)
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

    // 3. Download: latest MODY release asset first, then repo tree file.
    let token = github_token();
    let tok = token.as_deref();
    let (file_name, bytes) = fetch_mody_jar(folder, mc, tok).await.map_err(|e| e.to_string())?;
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
/// version folder on branch main. Falls back to Err (caller may use the
/// embedded offline jar via `ensure_dontcam_mod`).
async fn fetch_mody_jar(
    folder: &str,
    mc: &str,
    token: Option<&str>,
) -> anyhow::Result<(String, bytes::Bytes)> {
    // A. Latest release assets.
    if let Ok(api) = get_json(
        &format!("https://api.github.com/repos/{}/releases/latest", MODY_REPO),
        token,
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
                    let bytes = download_bytes(url, token).await?;
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
        token,
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
    // Prefer the API download_url (works with the same token for private
    // repos); fall back to raw.githubusercontent.com.
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
    let bytes = download_bytes(&url, token).await?;
    tracing::info!("Downloaded DontCam {} from MODY tree ({})", name, folder);
    Ok((name, bytes))
}

// Re-exported so AppState wiring stays one line per command.
#[allow(dead_code)]
pub(crate) fn repo() -> &'static str {
    MODY_REPO
}
