//! Java runtimes: auto-detect (JAVA_HOME, PATH, well-known dirs, managed),
//! Temurin auto-download, and per-MC-version resolution.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct JavaManager {
    client: Client,
    installations: Arc<RwLock<Vec<JavaInstallation>>>,
    java_dir: PathBuf,
}

impl JavaManager {
    pub async fn new() -> Self {
        let java_dir = crate::common::base_dir().join("java");
        tokio::fs::create_dir_all(&java_dir).await.ok();

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
            installations: Arc::new(RwLock::new(Vec::new())),
            java_dir,
        }
    }

    pub async fn detect_java(&self) -> Result<()> {
        let mut installations = Vec::new();

        // JAVA_HOME
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            let java_path = PathBuf::from(java_home).join("bin").join(java_bin());
            if java_path.exists() {
                if let Ok(version) = self.get_java_version(&java_path).await {
                    installations.push(JavaInstallation {
                        path: java_path.clone(),
                        version,
                        vendor: self.detect_vendor(&java_path).await,
                        architecture: self.detect_architecture(&java_path).await,
                        source: JavaSource::System,
                    });
                }
            }
        }

        // PATH
        if let Ok(path) = std::env::var("PATH") {
            let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
            for dir in path.split(sep) {
                let java_path = PathBuf::from(dir).join(java_bin());
                if java_path.exists() {
                    if let Ok(version) = self.get_java_version(&java_path).await {
                        if !installations.iter().any(|i| i.path == java_path) {
                            installations.push(JavaInstallation {
                                path: java_path.clone(),
                                version,
                                vendor: self.detect_vendor(&java_path).await,
                                architecture: self.detect_architecture(&java_path).await,
                                source: JavaSource::Path,
                            });
                        }
                    }
                }
            }
        }

        // Well-known installation directories
        for path in self.get_common_java_paths() {
            if path.exists() {
                if let Ok(version) = self.get_java_version(&path).await {
                    if !installations.iter().any(|i| i.path == path) {
                        installations.push(JavaInstallation {
                            path: path.clone(),
                            version,
                            vendor: self.detect_vendor(&path).await,
                            architecture: self.detect_architecture(&path).await,
                            source: JavaSource::Detected,
                        });
                    }
                }
            }
        }

        // Managed installations
        let managed_dir = self.java_dir.join("managed");
        if managed_dir.exists() {
            let mut entries = tokio::fs::read_dir(&managed_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let java_path = entry.path().join("bin").join(java_bin());
                if java_path.exists() {
                    if let Ok(version) = self.get_java_version(&java_path).await {
                        installations.push(JavaInstallation {
                            path: java_path.clone(),
                            version,
                            vendor: self.detect_vendor(&java_path).await,
                            architecture: self.detect_architecture(&java_path).await,
                            source: JavaSource::Managed,
                        });
                    }
                }
            }
        }

        *self.installations.write().await = installations;
        Ok(())
    }

    fn get_common_java_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if cfg!(target_os = "windows") {
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                let pf = PathBuf::from(&program_files);
                paths.push(pf.join("Java"));
                paths.push(pf.join("Eclipse Adoptium"));
                paths.push(pf.join("Microsoft").join("jdk"));
                paths.push(pf.join("Amazon Corretto"));
                paths.push(pf.join("Zulu"));
                paths.push(pf.join("BellSoft").join("LibericaJDK"));
            }
            if let Ok(pfx86) = std::env::var("ProgramFiles(x86)") {
                paths.push(PathBuf::from(pfx86).join("Java"));
            }
        } else if cfg!(target_os = "macos") {
            paths.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
            paths.push(PathBuf::from("/usr/lib/jvm"));
            paths.push(dirs::home_dir().unwrap_or_default().join(".sdkman").join("candidates").join("java"));
        } else {
            paths.push(PathBuf::from("/usr/lib/jvm"));
            paths.push(PathBuf::from("/usr/java"));
            paths.push(dirs::home_dir().unwrap_or_default().join(".sdkman").join("candidates").join("java"));
        }

        paths
    }

    async fn get_java_version(&self, java_path: &Path) -> Result<JavaVersion> {
        let mut cmd = Command::new(java_path);
        cmd.arg("-version");
        crate::common::hide_console_std(&mut cmd);
        let output = cmd.output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let version_line = stderr.lines().next().unwrap_or("");

        // Parse version like: openjdk version "21.0.2" 2024-01-16
        let version_str = version_line.split('"').nth(1).unwrap_or("").to_string();
        let major = version_str.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);

        Ok(JavaVersion {
            major,
            full: version_str,
        })
    }

    async fn detect_vendor(&self, java_path: &Path) -> JavaVendor {
        let mut cmd = Command::new(java_path);
        cmd.arg("-version");
        crate::common::hide_console_std(&mut cmd);
        let Ok(output) = cmd.output() else {
            return JavaVendor::Unknown;
        };
        let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();

        if stderr.contains("eclipse") || stderr.contains("adoptium") || stderr.contains("temurin") {
            return JavaVendor::EclipseAdoptium;
        }
        if stderr.contains("microsoft") {
            return JavaVendor::Microsoft;
        }
        if stderr.contains("amazon") || stderr.contains("corretto") {
            return JavaVendor::AmazonCorretto;
        }
        if stderr.contains("azul") || stderr.contains("zulu") {
            return JavaVendor::AzulZulu;
        }
        if stderr.contains("bellsoft") || stderr.contains("liberica") {
            return JavaVendor::BellSoftLiberica;
        }
        if stderr.contains("oracle") {
            return JavaVendor::Oracle;
        }
        if stderr.contains("openjdk") {
            return JavaVendor::OpenJDK;
        }
        JavaVendor::Unknown
    }

    async fn detect_architecture(&self, java_path: &Path) -> JavaArchitecture {
        let mut cmd = Command::new(java_path);
        cmd.arg("-version");
        crate::common::hide_console_std(&mut cmd);
        if let Ok(output) = cmd.output() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
            if stderr.contains("64-bit") || stderr.contains("x86_64") || stderr.contains("aarch64") {
                return JavaArchitecture::X64;
            }
            if stderr.contains("32-bit") || stderr.contains("x86") {
                return JavaArchitecture::X86;
            }
            if stderr.contains("arm") {
                return JavaArchitecture::ARM64;
            }
        }

        if cfg!(target_arch = "x86_64") {
            JavaArchitecture::X64
        } else if cfg!(target_arch = "aarch64") {
            JavaArchitecture::ARM64
        } else {
            JavaArchitecture::X86
        }
    }

    /// Probe the major version of a java binary without registering it.
    pub async fn probe_major(&self, java_path: &Path) -> Option<u32> {
        self.get_java_version(java_path).await.map(|v| v.major).ok()
    }

    pub async fn download_java(
        &self,
        version: u32,
        vendor: JavaVendor,
        architecture: JavaArchitecture,
    ) -> Result<JavaInstallation> {
        let download_url = self.get_download_url(version, &vendor, &architecture)?;

        let install_dir = self
            .java_dir
            .join("managed")
            .join(format!("{}-{}", vendor.dir_name(), version));
        tokio::fs::create_dir_all(&install_dir).await?;

        // NOTE: the archive MUST keep a proper extension — Windows
        // Expand-Archive refuses extensionless files.
        let archive_name = if cfg!(target_os = "windows") {
            "java_archive.zip"
        } else {
            "java_archive.tar.gz"
        };
        let archive_path = install_dir.join(archive_name);
        let response = self.client.get(&download_url).send().await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(&archive_path, bytes).await?;

        if let Err(e) = self.extract_java(&archive_path, &install_dir).await {
            // Don't leave a dead archive behind.
            tokio::fs::remove_file(&archive_path).await.ok();
            return Err(e);
        }
        tokio::fs::remove_file(&archive_path).await.ok();

        let java_path = install_dir.join("bin").join(java_bin());
        let java_version = self.get_java_version(&java_path).await?;

        let installation = JavaInstallation {
            path: java_path,
            version: java_version,
            vendor,
            architecture,
            source: JavaSource::Managed,
        };

        self.installations.write().await.push(installation.clone());
        Ok(installation)
    }

    fn get_download_url(
        &self,
        version: u32,
        vendor: &JavaVendor,
        architecture: &JavaArchitecture,
    ) -> Result<String> {
        let arch_str = match architecture {
            JavaArchitecture::X64 => "x64",
            JavaArchitecture::ARM64 => "aarch64",
            JavaArchitecture::X86 => "x86",
        };

        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };

        let url = match vendor {
            JavaVendor::EclipseAdoptium => {
                format!("https://api.adoptium.net/v3/binary/latest/{}/{}/{}/{}/{}/{}/{}", version, "ga", os, arch_str, "jdk", "hotspot", "normal/eclipse")
            }
            JavaVendor::Microsoft => {
                format!("https://aka.ms/download-jdk/microsoft-jdk-{}-{}-{}.{}", version, os, arch_str, ext)
            }
            JavaVendor::AmazonCorretto => {
                format!("https://corretto.aws/downloads/latest/amazon-corretto-{}-{}-{}-jdk.{}", version, arch_str, os, ext)
            }
            JavaVendor::AzulZulu => {
                format!("https://cdn.azul.com/zulu/bin/zulu{}-ca-jdk{}-{}-{}.{}", version, version, os, arch_str, ext)
            }
            JavaVendor::BellSoftLiberica => {
                format!("https://download.bell-sw.com/java/{}/bellsoft-jdk{}-{}-{}.{}", version, version, os, arch_str, ext)
            }
            _ => {
                // Default to Adoptium
                format!("https://api.adoptium.net/v3/binary/latest/{}/{}/{}/{}/{}/{}/{}", version, "ga", os, arch_str, "jdk", "hotspot", "normal/eclipse")
            }
        };

        Ok(url)
    }

    async fn extract_java(&self, archive_path: &Path, dest_dir: &Path) -> Result<()> {
        if cfg!(target_os = "windows") {
            // Use PowerShell to extract ZIP
            let mut cmd = Command::new("powershell");
            cmd.args([
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    dest_dir.display()
                ),
            ]);
            crate::common::hide_console_std(&mut cmd);
            let output = cmd.output()?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to extract: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            // Vendor zips contain a single top-level folder (e.g. jdk-17.0.x/)
            // holding bin/. Flatten it so <dest>/bin/java.exe exists.
            flatten_single_top_level(dest_dir).await?;
        } else {
            // Use tar for tar.gz
            let output = Command::new("tar")
                .args([
                    "-xzf",
                    archive_path.to_str().unwrap(),
                    "-C",
                    dest_dir.to_str().unwrap(),
                    "--strip-components=1",
                ])
                .output()?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to extract: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Ok(())
    }

    /// Find best matching installation for required major version.
    /// Exact major wins; otherwise a slightly newer runtime is acceptable for
    /// modern MC (>=16). Never a wildly newer runtime (old ASM/Mixin stacks
    /// crash on their class files). Use ensure_java() for auto-download.
    pub async fn find_best(&self, required_major: u32) -> Option<JavaInstallation> {
        let installs = self.installations.read().await;
        if installs.is_empty() {
            return None;
        }
        // exact match first (managed installs preferred)
        let mut exact: Vec<&JavaInstallation> =
            installs.iter().filter(|i| i.version.major == required_major).collect();
        if !exact.is_empty() {
            exact.sort_by_key(|i| source_rank(&i.source));
            return exact.into_iter().next().cloned();
        }
        // acceptable newer runtime (lowest), managed preferred
        let mut newer: Vec<&JavaInstallation> = installs
            .iter()
            .filter(|i| i.version.major > required_major && java_acceptable(required_major, i.version.major))
            .collect();
        newer.sort_by_key(|i| (i.version.major, source_rank(&i.source)));
        newer.into_iter().next().cloned()
    }

    /// Legacy last-resort pick (closest newer for modern MC, else highest).
    /// Only for offline fallback — may return an incompatible runtime.
    async fn find_closest_legacy(&self, required_major: u32) -> Option<JavaInstallation> {
        let installs = self.installations.read().await;
        if installs.is_empty() {
            return None;
        }
        let mut newer: Vec<&JavaInstallation> = installs
            .iter()
            .filter(|i| i.version.major > required_major)
            .collect();
        newer.sort_by_key(|i| (i.version.major, source_rank(&i.source)));
        if required_major >= 16 {
            if let Some(best) = newer.into_iter().next() {
                return Some(best.clone());
            }
        }
        let mut all: Vec<&JavaInstallation> = installs.iter().collect();
        all.sort_by_key(|i| (i.version.major, source_rank(&i.source)));
        all.into_iter().last().cloned()
    }

    /// Resolve a runnable Java for the required major: exact/acceptable
    /// installed runtime first, otherwise auto-download managed Temurin,
    /// otherwise (offline) closest available as last resort.
    pub async fn ensure_java(&self, required_major: u32) -> Option<JavaInstallation> {
        if let Some(best) = self.find_best(required_major).await {
            return Some(best);
        }
        let arch = if cfg!(target_arch = "x86_64") {
            JavaArchitecture::X64
        } else if cfg!(target_arch = "aarch64") {
            JavaArchitecture::ARM64
        } else {
            JavaArchitecture::X86
        };
        match self
            .download_java(required_major, JavaVendor::EclipseAdoptium, arch)
            .await
        {
            Ok(inst) => {
                tracing::info!(
                    "Auto-provisioned Java {} for MC (needs {})",
                    inst.version.full,
                    required_major
                );
                return Some(inst);
            }
            Err(e) => {
                tracing::warn!(
                    "Auto-download of Java {} failed ({}); falling back to closest installed",
                    required_major,
                    e
                );
            }
        }
        self.find_closest_legacy(required_major).await
    }
}

fn java_bin() -> &'static str {
    if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    }
}

fn source_rank(source: &JavaSource) -> u8 {
    match source {
        JavaSource::Managed => 0,
        JavaSource::System => 1,
        JavaSource::Detected => 2,
        JavaSource::Path => 3,
        JavaSource::Custom => 4,
    }
}

/// Tolerance for newer runtimes on modern MC (>=16): exact major preferred,
/// but a slightly newer one still runs the game. Too-new runtimes break old
/// ASM/Mixin stacks. Legacy MC (<16) requires the exact major.
const NEWER_JAVA_TOLERANCE: u32 = 4;

pub fn java_acceptable(required_major: u32, actual_major: u32) -> bool {
    if actual_major == required_major {
        return true;
    }
    if required_major >= 16 {
        return actual_major > required_major && actual_major - required_major <= NEWER_JAVA_TOLERANCE;
    }
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub version: JavaVersion,
    pub vendor: JavaVendor,
    pub architecture: JavaArchitecture,
    pub source: JavaSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub major: u32,
    pub full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JavaVendor {
    EclipseAdoptium,
    Microsoft,
    AmazonCorretto,
    AzulZulu,
    BellSoftLiberica,
    Oracle,
    OpenJDK,
    Unknown,
}

impl JavaVendor {
    fn dir_name(&self) -> &'static str {
        match self {
            JavaVendor::EclipseAdoptium => "eclipse_adoptium",
            JavaVendor::Microsoft => "microsoft",
            JavaVendor::AmazonCorretto => "amazon_corretto",
            JavaVendor::AzulZulu => "azul_zulu",
            JavaVendor::BellSoftLiberica => "bellsoft_liberica",
            JavaVendor::Oracle => "oracle",
            JavaVendor::OpenJDK => "openjdk",
            JavaVendor::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JavaArchitecture {
    X64,
    X86,
    ARM64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JavaSource {
    System,
    Path,
    Detected,
    Managed,
    Custom,
}

/// Move the contents of a single top-level folder up one level.
/// Turns `<dest>/jdk-17.0.x/bin/java.exe` into `<dest>/bin/java.exe`.
async fn flatten_single_top_level(dest_dir: &Path) -> Result<()> {
    let bin_name = java_bin();
    if dest_dir.join("bin").join(bin_name).exists() {
        return Ok(());
    }
    let mut subdirs = Vec::new();
    let mut entries = tokio::fs::read_dir(dest_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            subdirs.push(entry.path());
        }
    }
    if subdirs.len() != 1 {
        anyhow::bail!(
            "extracted archive has no bin/{} and no single top-level folder",
            bin_name
        );
    }
    let mut inner = tokio::fs::read_dir(&subdirs[0]).await?;
    while let Some(entry) = inner.next_entry().await? {
        let dest = dest_dir.join(entry.file_name());
        tokio::fs::rename(entry.path(), &dest).await?;
    }
    tokio::fs::remove_dir(&subdirs[0]).await.ok();
    if !dest_dir.join("bin").join(bin_name).exists() {
        anyhow::bail!("extracted archive has no bin/{}", bin_name);
    }
    Ok(())
}

#[tauri::command]
pub async fn detect_java(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<JavaInstallation>, String> {
    state.java.detect_java().await.map_err(|e| e.to_string())?;
    Ok(state.java.installations.read().await.clone())
}

#[tauri::command]
pub async fn get_java_installations(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<JavaInstallation>, String> {
    Ok(state.java.installations.read().await.clone())
}

#[tauri::command]
pub async fn download_java(
    state: tauri::State<'_, crate::AppState>,
    version: u32,
    vendor: JavaVendor,
    architecture: JavaArchitecture,
) -> Result<JavaInstallation, String> {
    state
        .java
        .download_java(version, vendor, architecture)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_java_for_version(
    state: tauri::State<'_, crate::AppState>,
    version_id: String,
) -> Result<Option<JavaInstallation>, String> {
    // custom path wins
    let settings = state.settings.get().await;
    if let Some(custom) = settings.java.custom_java_path {
        if !custom.trim().is_empty() {
            let p = std::path::PathBuf::from(&custom);
            if p.exists() {
                return Ok(Some(JavaInstallation {
                    path: p,
                    version: JavaVersion { major: 0, full: "custom".to_string() },
                    vendor: JavaVendor::Unknown,
                    architecture: JavaArchitecture::X64,
                    source: JavaSource::Custom,
                }));
            }
        }
    }
    let required = crate::common::recommended_java_major(&version_id);
    Ok(state.java.ensure_java(required).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_window_policy() {
        // exact always wins
        assert!(java_acceptable(8, 8));
        assert!(java_acceptable(17, 17));
        assert!(java_acceptable(21, 21));
        // modern MC tolerates slightly newer runtimes (21 runs a 17 game)
        assert!(java_acceptable(17, 21));
        assert!(java_acceptable(17, 18));
        assert!(java_acceptable(21, 25));
        // too-new runtimes break old ASM/Mixin (Java 26 vs 1.20.1's Java 17)
        assert!(!java_acceptable(17, 26));
        assert!(!java_acceptable(17, 22));
        assert!(!java_acceptable(21, 26));
        // legacy MC needs the exact major — newer never acceptable
        assert!(!java_acceptable(8, 11));
        assert!(!java_acceptable(8, 17));
        assert!(!java_acceptable(8, 7));
        // older never acceptable
        assert!(!java_acceptable(17, 11));
        assert!(!java_acceptable(17, 8));
    }
}
