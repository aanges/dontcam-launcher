# DontCam Client

![DontCam Client logo](public/logo.svg)

Minecraft launcher built with **Tauri 2 + Rust + React + TypeScript**.

## Features

- **Accounts**: offline (correct Java UUID v3) + Microsoft (OAuth2 via system browser + localhost callback, Xbox Live → XSTS → Minecraft Services, ownership check, token refresh, local `accounts.json` persistence).
- **Versions**: full Mojang manifest, install of any release/snapshot/beta/alpha including **1.8.9, 1.12.2, 1.20.1, 1.21.1+**:
  - client jar (SHA1 verified), libraries with OS-rule evaluation, natives extraction, assets (indexes + objects, parallel download),
  - legacy version JSON support (`minecraftArguments`, maven-only libraries),
  - `download-progress` events for the UI.
- **Launch**: real game start via managed `tokio::process`:
  - JVM args from version JSON + memory settings + custom args,
  - game args with all placeholders (`auth_player_name`, `auth_uuid`, `assets_root`, ...), legacy arg strings, resolution, quick-play server,
  - per-profile game dirs (`instances/<profile>` with mods/resourcepacks/saves), stdout/stderr forwarded as `game-log` events, crash detection, pre/post commands, kill support.
- **Java**: auto-detect (`JAVA_HOME`, `PATH`, common install locations, managed runtimes), best-match selection per version (8 for ≤1.16, 16 for 1.17, 17 for 1.18–1.20.4, 21 for 1.20.5+/1.21+), managed download from Adoptium/Microsoft/Corretto/Zulu/Liberica.
- **Profiles**: create/duplicate/edit/delete, per-profile version, mod loader tag, account override, JVM/game args, resolution, JSON persistence.
- **Mod loaders**: Fabric + Quilt via official meta APIs (profile JSON install), Forge via promotions + maven-metadata with headless installer run.
- **Settings & theme**: light/dark/system theme, language, memory, resolution, network, advanced args/env/commands, persisted to `settings.json`.
- **UI pages**: Home (featured versions, quick play), Versions (filters, search, install), Profiles, Mods (browse/install/howto), Accounts, Settings.

## Data layout (`DontCamCl` next to vanilla `.minecraft`)

- Windows: `%APPDATA%\DontCamCl`
- macOS: `~/Library/Application Support/DontCamCl`
- Linux: `~/.minecraft/../DontCamCl` (sibling of `~/.minecraft`)

```
DontCamCl/
  versions/<id>/<id>.json + <id>.jar (+ natives/)
  libraries/**   assets/indexes + assets/objects/**
  instances/<profile>/{mods,resourcepacks,saves,...}
  profiles/*.json  accounts.json  settings.json  java/managed/**
```

## Requirements

- Node 18+, npm
- **Rust toolchain** (cargo + rustc) for `tauri dev` / `tauri build` — install from https://rustup.rs
- Java runtimes for the versions you play: 8 (1.8.9/1.12.2), 17 (1.20.1), 21 (1.21.1+). The launcher auto-picks from installed ones.

## Run

```sh
npm install
npm run dev        # frontend only (works in browser, backend calls fail gracefully)
npm run tauri:dev  # full app (needs Rust)
npm run build      # typecheck + production frontend build
npm run tauri:build
```

## Microsoft login notes

Uses the public Minecraft client id (`00000000402b5328`) + `XboxLive.signin offline_access` scope.
The launcher opens the system browser and listens on `http://localhost:<random>/callback` for 5 minutes.
Tokens stay local; refresh is automatic. An account must own Minecraft (entitlements check).

## Events (backend → frontend)

- `download-progress` — `{ version_id, stage, current, total }`
- `launch-status` — `preparing | running | idle | { error } | { crashed }`
- `game-log` — `{ stream: "stdout" | "stderr", line }`
- `ms-login-url` — fallback login URL if the browser didn't open

## Tauri commands

Auth: `login_microsoft, login_offline, logout, get_accounts, get_current_account, set_current_account, refresh_token, validate_account`
Versions: `fetch_version_manifest, get_installed_versions, install_version, ensure_version_ready, uninstall_version, get_version_details`
Profiles: `create_profile, get_profiles, get_profile, update_profile, delete_profile, duplicate_profile`
Launch: `launch_game, get_launch_status, kill_game`
Settings / loaders / java / window / utils: see `src-tauri/src/main.rs`.
