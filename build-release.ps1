<#
  build-release.ps1 — signed release build of DontCam Client (with updater artifacts).

  Reads the Tauri signing key from %USERPROFILE%\.tauri\dontcam.key
  (+ password from dontcam.key.pass) and runs `tauri build`.
  On success, updater artifacts land in:
    src-tauri\target\release\bundle\nsis\  (*-setup.exe + *-setup.exe.sig)
    src-tauri\target\release\bundle\msi\   (*.msi + *.msi.sig)

  NEVER commit the .key / .key.pass files. Back them up offline —
  losing the private key means installed users can never auto-update again.

  Usage:
    powershell -ExecutionPolicy Bypass -File .\build-release.ps1
    powershell -ExecutionPolicy Bypass -File .\build-release.ps1 -CopyToRoot
#>
param(
  [switch]$CopyToRoot
)

$ErrorActionPreference = 'Stop'

$keyPath = Join-Path $env:USERPROFILE '.tauri\dontcam.key'
$passPath = Join-Path $env:USERPROFILE '.tauri\dontcam.key.pass'

if (-not (Test-Path $keyPath)) {
  throw "Signing key not found: $keyPath. Generate one with: npx tauri signer generate -w `"$keyPath`""
}

$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $keyPath -Raw
if (Test-Path $passPath) {
  $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = Get-Content $passPath -Raw
}

Write-Output '[release] building signed bundles (updater artifacts included)…'
npm run tauri:build

if ($CopyToRoot -and $LASTEXITCODE -eq 0) {
  $ver = (node -p "require('./package.json').version")
  Copy-Item 'src-tauri\target\release\dontcam-client.exe' '.\DontCamClient.exe' -Force
  $setup = Get-ChildItem 'src-tauri\target\release\bundle\nsis\*-setup.exe' | Select-Object -First 1
  if ($setup) {
    Copy-Item $setup.FullName ".\DontCamClient-Setup-$ver.exe" -Force
  }
  Write-Output "[release] copied exe + setup (v$ver) to project root."
}

# Don't leave the password in the environment longer than needed.
Remove-Item Env:\TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
Remove-Item Env:\TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
