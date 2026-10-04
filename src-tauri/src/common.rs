//! Shared filesystem layout + OS helpers for all backend modules.
//!
//! Layout (next to vanilla `.minecraft`):
//! `.../DontCamCl/{versions,libraries,assets,instances,profiles,java,...}`

use std::path::PathBuf;

/// Vanilla `.minecraft` location, per OS.
pub fn minecraft_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".minecraft")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("minecraft")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".minecraft")
    }
}

/// Base data dir, right next to vanilla `.minecraft`.
pub fn base_dir() -> PathBuf {
    let parent = minecraft_dir()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    parent.join("DontCamCl")
}

/// Hide the console window of a child process (Windows only).
#[cfg(target_os = "windows")]
pub fn hide_console_std(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
}

#[cfg(not(target_os = "windows"))]
pub fn hide_console_std(_cmd: &mut std::process::Command) {}

/// Hide the console window of a child process (Windows only).
#[cfg(target_os = "windows")]
pub fn hide_console_tokio(cmd: &mut tokio::process::Command) {
    // tokio::process::Command has an inherent creation_flags on Windows.
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
}

#[cfg(not(target_os = "windows"))]
pub fn hide_console_tokio(_cmd: &mut tokio::process::Command) {}

/// Prefer `javaw.exe` over `java.exe` on Windows: same JVM, no console window.
pub fn prefer_javaw(java: &std::path::Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if java.as_os_str() == "java" {
            return PathBuf::from("javaw");
        }
        let file_name = java.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.eq_ignore_ascii_case("java.exe") {
            let javaw = java.with_file_name("javaw.exe");
            if javaw.exists() {
                return javaw;
            }
        }
    }
    java.to_path_buf()
}

/// Game dir resolution:
/// - settings.game.default_game_dir if set
/// - otherwise base_dir/instances/<profile_id or "global">
pub async fn game_dir_for_profile(
    settings: &crate::settings::SettingsManager,
    profile_id: Option<&str>,
) -> PathBuf {
    let s = settings.get().await;
    if let Some(custom) = &s.game.default_game_dir {
        if !custom.trim().is_empty() {
            let p = PathBuf::from(custom);
            let full = if let Some(pid) = profile_id {
                p.join("instances").join(pid)
            } else {
                p
            };
            let _ = tokio::fs::create_dir_all(&full).await;
            return full;
        }
    }
    let mut dir = base_dir().join("instances");
    dir = dir.join(profile_id.unwrap_or("global"));
    let _ = tokio::fs::create_dir_all(&dir).await;
    for sub in ["mods", "resourcepacks", "saves", "screenshots", "logs", "config"] {
        let _ = tokio::fs::create_dir_all(dir.join(sub)).await;
    }
    dir
}

pub fn libraries_dir() -> PathBuf {
    base_dir().join("libraries")
}

pub fn versions_dir() -> PathBuf {
    base_dir().join("versions")
}

pub fn assets_dir() -> PathBuf {
    base_dir().join("assets")
}

pub fn natives_dir(version_id: &str) -> PathBuf {
    versions_dir().join(version_id).join("natives")
}

/// Recommended Java major for a given MC version:
/// 1.8–1.15 -> 8, 1.16–1.20.4 -> 17, 1.20.5+ -> 21.
pub fn recommended_java_major(mc_version: &str) -> u32 {
    let v = mc_version.trim();
    // strip loader suffixes like "1.20.1-forge-..." -> "1.20.1"
    let base = v.split('-').next().unwrap_or(v);
    let parts: Vec<u32> = base
        .split('.')
        .filter_map(|p| p.parse::<u32>().ok())
        .collect();
    if parts.is_empty() {
        return 21;
    }
    let minor = *parts.get(1).unwrap_or(&0);
    let patch = *parts.get(2).unwrap_or(&0);
    if parts[0] != 1 {
        return 21; // future-proof: 2.x etc.
    }
    match minor {
        0..=15 => 8,
        16 | 17 | 18 | 19 => 17,
        20 => {
            if patch >= 5 {
                21
            } else {
                17
            }
        }
        _ => 21, // 1.21+
    }
}

pub fn current_os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

pub fn current_arch_name() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86"
    }
}

fn norm_arch(t: &str) -> &str {
    match t {
        "x64" | "x86_64" | "x86-64" => "x64",
        "arm64" | "aarch64" | "aarch_64" => "arm64",
        "x86" | "i386" | "i686" => "x86",
        _ => t,
    }
}

pub fn arch_matches(token: &str, current: &str) -> bool {
    norm_arch(token) == norm_arch(current)
}

fn is_known_arch_token(suffix: &str) -> bool {
    matches!(
        suffix,
        "x86" | "x64" | "x86_64" | "x86-64" | "arm64" | "aarch64" | "aarch_64" | "i386" | "i686"
    )
}

/// True when maven coordinates carry a natives classifier
/// (4th part starting with `natives-`).
pub fn is_natives_library(coords: &str) -> bool {
    coords
        .split(':')
        .nth(3)
        .map_or(false, |c| c.starts_with("natives-"))
}

/// Does a natives classifier target the current OS + architecture?
/// Non-arch suffixes (like `-patch`) match on OS alone — over-extracting is
/// harmless, under-extracting crashes the game.
pub fn native_classifier_matches(classifier: &str) -> bool {
    let os = current_os_name();
    let arch = current_arch_name();
    let rest = classifier.strip_prefix("natives-").unwrap_or(classifier);

    let (os_name, remainder) = if let Some(r) = rest.strip_prefix("windows") {
        ("windows", r)
    } else if let Some(r) = rest.strip_prefix("macos") {
        ("macos", r)
    } else if let Some(r) = rest.strip_prefix("osx") {
        ("osx", r)
    } else if let Some(r) = rest.strip_prefix("linux") {
        ("linux", r)
    } else {
        return false;
    };

    let os_ok = match os_name {
        "windows" => os == "windows",
        "macos" | "osx" => os == "osx",
        "linux" => os == "linux",
        _ => false,
    };
    if !os_ok {
        return false;
    }

    let suffix = remainder.trim_start_matches('-');
    if suffix.is_empty() {
        // No arch suffix: historically x64 artifacts; linux ships one jar for all archs.
        return arch == "x64" || os == "linux";
    }
    if !is_known_arch_token(suffix) {
        return true;
    }
    arch_matches(suffix, arch)
}
