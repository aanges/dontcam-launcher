<#
  verify-mody-mods.ps1 — verification of the DontCam mod builds from the
  PUBLIC aanges/MODY repo. RUN ON A MACHINE WITH THE TOOLCHAIN, not on a
  bare gaming PC (this repo's launcher side needs no toolchain).

  Prereqs (set as env vars before running):
    JDK8_HOME   -> JDK 8   (ForgeGradle 2.x: folders 1.8, 1.12)
    JDK17_HOME  -> JDK 17  (fabric-loom 1.7.4: folders 1.16-1.20)
    JDK21_HOME  -> JDK 21  (fabric-loom 1.7.4: folder 1.21)
    GRADLE_FG18  -> Gradle 2.14   (default 2.14, on PATH as gradle-2.14 or GRADLE_FG18\bin\gradle)
    GRADLE_FG112 -> Gradle 4.10.3 (default 4.10.3)
    GRADLE_LOOM  -> Gradle 8.10   (default 8.10)
    MODY_DIR     -> local clone of https://github.com/aanges/MODY (default .\MODY)

  Flow per folder:
    1.8, 1.12            -> build      (ForgeGradle)
    1.16 - 1.21          -> remapJar  (fabric-loom)
  Then every produced jar is checked: exists, non-empty, ZIP-valid,
  contains the mod metadata (mcmod.info for Forge, fabric.mod.json for
  Fabric), SHA-256 printed for the record.

  NOTE: MODY has NO gradle wrapper (verified: 0 wrapper files in the repo),
  so system Gradle installs above are mandatory until wrappers are added
  on the MODY side (`gradle wrapper --gradle-version X` per folder).

  Usage:
    powershell -ExecutionPolicy Bypass -File .\tools\verify-mody-mods.ps1
#>
param()

$ErrorActionPreference = 'Stop'

$GradleFg18  = $env:GRADLE_FG18;  if (-not $GradleFg18)  { $GradleFg18  = '2.14' }
$GradleFg112 = $env:GRADLE_FG112; if (-not $GradleFg112) { $GradleFg112 = '4.10.3' }
$GradleLoom  = $env:GRADLE_LOOM;  if (-not $GradleLoom)  { $GradleLoom  = '8.10' }
$ModyDir     = $env:MODY_DIR;     if (-not $ModyDir)     { $ModyDir     = (Join-Path $PSScriptRoot '..\MODY') }

$failures = 0

function Find-Gradle([string]$version) {
  foreach ($c in @("gradle-$version", 'gradle')) {
    $g = Get-Command $c -ErrorAction SilentlyContinue
    if ($g) { return $g.Source }
  }
  throw "Gradle $version not found on PATH (need gradle-$version or gradle). Install it first."
}

function Find-Jdk([string]$envName, [string]$label) {
  $jdkHome = [Environment]::GetEnvironmentVariable($envName)
  if (-not $jdkHome -or -not (Test-Path (Join-Path $jdkHome 'bin\java.exe'))) {
    throw "$label not found: set $envName to a JDK install (need bin\java.exe)."
  }
  return $jdkHome
}

function Test-Jar([string]$jar, [string]$meta) {
  if (-not (Test-Path $jar)) { throw "missing jar: $jar" }
  $len = (Get-Item $jar).Length
  if ($len -eq 0) { throw "empty jar: $jar" }
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $zip = [System.IO.Compression.ZipFile]::OpenRead($jar)
  try {
    $names = @($zip.Entries | ForEach-Object { $_.FullName })
    if (-not ($names -contains $meta)) { throw "jar lacks ${meta}: $jar" }
  } finally { $zip.Dispose() }
  $hash = (Get-FileHash $jar -Algorithm SHA256).Hash
  Write-Output "  OK $(Split-Path $jar -Leaf) ($len bytes, sha256 $hash)"
}

# ---- prereqs ----
$jdk8  = Find-Jdk 'JDK8_HOME'  'JDK 8'
$jdk17 = Find-Jdk 'JDK17_HOME' 'JDK 17'
$jdk21 = Find-Jdk 'JDK21_HOME' 'JDK 21'
$gFg18  = Find-Gradle $GradleFg18
$gFg112 = Find-Gradle $GradleFg112
$gLoom  = Find-Gradle $GradleLoom
if (-not (Test-Path (Join-Path $ModyDir '1.20\build.gradle'))) {
  throw "MODY clone not found at $ModyDir (set MODY_DIR or clone https://github.com/aanges/MODY there)."
}
Write-Output "[verify] toolchain OK: jdk8=$jdk8 jdk17=$jdk17 jdk21=$jdk21"

$plan = @(
  @{ folder = '1.8';  task = 'build';    gradle = $gFg18;  jdk = $jdk8;  meta = 'mcmod.info' },
  @{ folder = '1.12'; task = 'build';    gradle = $gFg112; jdk = $jdk8;  meta = 'mcmod.info' },
  @{ folder = '1.16'; task = 'remapJar'; gradle = $gLoom;  jdk = $jdk17; meta = 'fabric.mod.json' },
  @{ folder = '1.17'; task = 'remapJar'; gradle = $gLoom;  jdk = $jdk17; meta = 'fabric.mod.json' },
  @{ folder = '1.18'; task = 'remapJar'; gradle = $gLoom;  jdk = $jdk17; meta = 'fabric.mod.json' },
  @{ folder = '1.19'; task = 'remapJar'; gradle = $gLoom;  jdk = $jdk17; meta = 'fabric.mod.json' },
  @{ folder = '1.20'; task = 'remapJar'; gradle = $gLoom;  jdk = $jdk17; meta = 'fabric.mod.json' },
  @{ folder = '1.21'; task = 'remapJar'; gradle = $gLoom;  jdk = $jdk21; meta = 'fabric.mod.json' }
)

foreach ($p in $plan) {
  $dir = Join-Path $ModyDir $p.folder
  Write-Output "[verify] === $($p.folder): $($p.task) ==="
  try {
    $env:JAVA_HOME = $p.jdk
    $env:Path = "$($p.jdk)\bin;" + $env:Path
    Push-Location $dir
    & $p.gradle $p.task --console=plain
    if ($LASTEXITCODE -ne 0) { throw "$($p.gradle) $($p.task) exited with $LASTEXITCODE" }
    Pop-Location
    $jars = Get-ChildItem (Join-Path $dir 'build\libs\*.jar') -ErrorAction SilentlyContinue |
      Where-Object { $_.Name -notmatch '-sources\.jar$|-dev\.jar$' }
    if (-not $jars) { throw "no remapped jar in build\libs" }
    foreach ($j in $jars) { Test-Jar $j.FullName $p.meta }
  } catch {
    Write-Output "  FAIL $($p.folder): $_"
    $failures++
    try { Pop-Location } catch {}
  }
}

if ($failures -gt 0) {
  Write-Output "[verify] RESULT: FAIL ($failures/8 folders)"
  exit 1
}
Write-Output '[verify] RESULT: BUILD SUCCESSFUL (8/8 folders)'
