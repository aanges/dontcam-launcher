@echo off
REM DontCam Client - alpha dev starter (Windows)
REM Sets up Rust + MSVC env, then runs the app in dev mode.
setlocal
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where link.exe >nul 2>nul
if errorlevel 1 (
  echo [alpha] link.exe not in PATH - loading MSVC env via vcvars64...
  call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
)
cd /d "%~dp0"
npx tauri dev
