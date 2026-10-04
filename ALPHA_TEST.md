# DontCam Client — Alpha test plan (v0.1.0)

## Status (verified 2026-09-28)

- ✅ `npm run build` — frontend green
- ✅ `cargo check` / `cargo build` — backend green (only dead-code warnings)
- ✅ `tauri dev` smoke test — window process starts and stays alive, Vite serves UI (HTTP 200), `%APPDATA%/DontCam Client` created with `settings.json` + default profile
- ✅ First run — no auto-profile; onboarding popup (Microsoft green first, offline as text)
- ✅ Version install pipeline — fixed legacy-JSON bug (`Artifact.path` missing in ≤1.12 era JSONs, derived from URL now); e2e install of 1.8.9 verified into `DontCamCl/versions/1.8.9` (jar+json)
- ✅ Version picker — grouped by minor line (1.21, 1.20, …) with Installed section; install dialog with force-reinstall, Java detection and target path
- ✅ Home: hero banner, Install & Play (auto-installs missing version, then launches), live console (download progress, launch status, game output), install errors visible
- ✅ Game launch 1.21.11 — fixed modern natives: separate `*:natives-*` entries are extracted (legacy `classifiers` path kept), natives JARs added to classpath for LWJGL3, DLLs flattened to natives root, foreign-arch jars skipped at download
- ✅ Standalone exe — `DontCamClient.exe` (release, embedded UI, no console, no localhost) verified by screenshot with real grouped-picker UI
- ✅ DontCam mod 0.1.0 (Forge 1.8.9) — RShift menu, FPS/CPS/keystrokes HUD, DC badge + owner crown (Microsoft only), auto-injected from embedded jar; e2e: Mods → install Forge 1.8.9 → profile with DontCam Mod on → Play → RShift
- ✅ DontCam mod 0.1.0 (Fabric 1.21.x) — same HUD/menu/badges + custom main menu ("DontCam Client" banner, pretty buttons); auto-installed with every 1.21 Pick via Install & Play
- ✅ DontCam mods for ALL lines (1.8.9, 1.12.2, 1.16.5, 1.17.1, 1.18.2, 1.19.4, 1.20.1, 1.21.1) — wired from `resources/dontcam/`, auto-detected per MC line, stale jars cleaned, Fabric API version-matched per line, era-capped Fabric loaders (0.14.25 ≤1.19, 0.15.11 for 1.20)
- ✅ Fabric API auto-install (Modrinth, version-matched, stale jars cleaned) at install AND launch time
- ✅ Stale-mod cleanup: switching MC lines removes foreign `dontcam-*.jar` (fixes the 1.21-mod-in-1.20 crash)
- ✅ Self-healing Play + install-time mod/API staging into the profile instance
- ✅ Fabric API auto-install from Modrinth into instance mods (fixes the missing-dependency screen)
- ✅ Self-healing Play: missing loader profile installs on the fly, then launches modded (no more "fabric się nie pobrał")
- ✅ NSIS installer — `src-tauri/target/release/bundle/nsis/DontCam Client_0.1.0_x64-setup.exe` (6.36 MiB)
- Release command: `npx tauri build --features tauri/custom-protocol --bundles nsis` (plain `cargo build` binaries load the vite devUrl — never ship those; dev HMR untouched)
- ✅ Microsoft login — embedded in-app sign-in window (public client only allows `oauth20_desktop.srf`; localhost rejected by MS, device flow unsupported for this client)

## 0. Requirements for testers

## 0. Requirements for testers

- Windows 10/11 64-bit, internet access
- Java (the launcher auto-selects the best installed one):
  - 1.8.9 / 1.12.2 → Java 8
  - 1.20.1 → Java 17
  - 1.21.1 → Java 21
- Microsoft account **owning Minecraft** (for premium test) and/or willingness to use offline mode
- Fresh `DontCamCl` folder recommended for first run (it sits next to vanilla `.minecraft`: `%APPDATA%\DontCamCl` on Windows).
  If you tested earlier builds, delete or move the old `%APPDATA%\DontCam Client` folder — the launcher does not auto-migrate it.

## 1. Smoke test (5 min)

1. `npm run tauri:dev` starts, window 1200×720, custom titlebar works (min/max/close).
2. Theme toggle (titlebar + sidebar) switches light/dark, survives restart.
3. No error dialog on startup; Home page shows "Welcome back".

## 2. Accounts

| # | Steps | Expected |
|---|-------|----------|
| A1 | Accounts → Add offline `Testowy123` | Account appears, marked active |
| A2 | Add offline `zly nick!` | Rejected with validation message |
| A3 | Restart launcher | Accounts persist, active account remembered |
| A4 | Microsoft login → approve in browser | Browser opens (or fallback link shows), account `GamerTag (microsoft)` added; ownership-less accounts get a clear "does not own Minecraft" error |
| A5 | Remove account | Active switches to another account |

## 3. Versions (1.8.9 / 1.12.2 / 1.20.1 / 1.21.1)

| # | Steps | Expected |
|---|-------|----------|
| V1 | Versions tab loads list, latest release shown | List + counts, search finds `1.8.9` etc. |
| V2 | Versions → open e.g. `1.21` group → Pick `1.21.1` | Green notice, row shows "Picked"; nothing is downloaded yet |
| V2b | Home → Install & Play | Version downloads (watch console), then game launches |
| V2c | Installed card → folder icon | Opens `DontCamCl/versions/<id>` in Explorer |
| V3 | Install `1.8.9` (legacy format) | Installs despite old `minecraftArguments` JSON |
| V4 | Uninstall a version | Folder `versions/<id>` removed |
| V5 | Filters: snapshots on/off, sorting | List updates accordingly |

## 4. Launch (the critical path)

| # | Steps | Expected |
|---|-------|----------|
| L1 | Home → Play with offline account, `1.21.1` (+Java 21 installed) | Game window opens; status `running`; Stop button kills it |
| L2 | Play `1.8.9` with Java 8 | Launches with legacy args |
| L3 | No matching Java installed | Clear error "Failed to spawn java", not a silent crash |
| L4 | Custom resolution in profile | Game window opens at that size |
| L5 | Quick-play server on profile launch | Connects to given host:port |

## 5. Profiles / mods / settings

- P1: create → duplicate → edit (version, JVM args) → delete profile.
- M1: Mods → Fabric for 1.20.1 → Install; check `versions/fabric-...json` exists.
- M2: How-to tab shows game folder; Open-folder button works.
- S1: memory 2G→6G, custom Java path, pre/post commands; settings persist after restart.

## 6. What to report

For every failure: version tested, account type (offline/Microsoft), installed Javas (`Settings → Java & Memory`), launcher logs (console output of `tauri:dev`), and the `game-log` if the game started. Screenshots of error boxes welcome.

## Known alpha limitations

- No auto-update, no news feed, no skins preview beyond avatar dot.
- Forge install runs the official installer headless — first install can take minutes.
- `tauri:build` installer icons are empty placeholders (`bundle.icon: []`).
- Download progress is per-stage, not byte-precise.
