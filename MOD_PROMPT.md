# PROMPT — DontCam Client companion mod

> Copy everything below the line into the AI.
> Fill in TARGET_MC + TARGET_LOADER first (pick one row from the matrix).

---

Write a Minecraft client-side companion mod called **DontCam Client Mod** for:

- **TARGET_MC = ___** (exact version, e.g. `1.19.4`)
- **TARGET_LOADER = ___** (`Forge` for 1.8–1.12, `Fabric` for 1.16+)
- Mod id: `dontcam`, version: `0.1.0`, environment: client only
- Output jar MUST be named exactly: `dontcam-<mc>-0.1.0.jar`
  (pattern the launcher requires, e.g. `dontcam-1.19.4-0.1.0.jar`)

## 0. Version matrix (all lines the launcher supports — one port per row)

| MC line | Exact versions | Loader | Java (run) | Toolchain to resolve |
|---|---|---|---|---|
| 1.8 | 1.8, 1.8.1–1.8.9 | Forge 11.15.1.2318 | 8 | FG 2.1.6, MCP stable_22, Gradle 2.14 — DONE, see `client-mod/` |
| 1.12 | 1.12, 1.12.1, 1.12.2 | Forge 14.23.5.2860 | 8 | FG 2.3.x, MCP snapshot/stable for 1.12.2, Gradle 4.x |
| 1.16 | 1.16–1.16.5 | Fabric | 8/16/17 | Loom + Yarn 1.16.5 + loader/API for 1.16.5 |
| 1.17 | 1.17, 1.17.1 | Fabric | 16 | Loom + Yarn 1.17.1 + loader/API for 1.17.1 |
| 1.18 | 1.18, 1.18.1, 1.18.2 | Fabric | 17 | Loom + Yarn 1.18.2 + loader/API for 1.18.2 |
| 1.19 | 1.19–1.19.4 | Fabric | 17 | Loom + Yarn 1.19.4 + loader/API for 1.19.4 |
| 1.20 | 1.20–1.20.6 | Fabric | 17 | DONE for 1.20.1 (Loom 1.7.4, Yarn build.10, loader 0.15.11, API 0.92.x) — copy `fabric-mod-1.20.1/`, translate deltas |
| 1.21 | 1.21–1.21.11 | Fabric | 21 | DONE (`fabric-mod/`: Loom 1.7.4, Yarn 1.21.1+build.3, loader 0.19.5, API 0.116.x) |
| 26 | 26.1, 26.1.1, 26.1.2, 26.2, 26.3 | Fabric (verify!) | 21+ | UNVERIFIED — first probe whether Fabric/Yarn support 26.x (endpoints below); if not, report back instead of guessing |

RULE: before writing any code, resolve the exact dependency versions from the
official endpoints and print them (no guessing from memory):
- Yarn builds: `https://meta.fabricmc.net/v2/versions/yarn/<mc>` (take newest `+build.N`)
- Loader: `https://meta.fabricmc.net/v2/versions/loader/<mc>` (take newest `stable`)
- Fabric API: list `https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/`, take newest `0.x+<mc>`
- Forge builds: `https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json` (`<mc>-recommended` else `<mc>-latest`) or maven-metadata fallback
- Mappings/loader APIs differ PER ERA — verify every hooked method against the real Yarn tiny mappings (`...-mergedv2.jar` → `mappings/mappings.tiny`) before relying on it.

## 1. Features (ALL required, identical on every line)

1. **Right-Shift menu** — keybind + per-tick edge check (open only when no screen is open): toggle buttons FPS / CPS / Keystrokes / DC Badges + Done. Persist to `config/dontcam.json` (Gson).
2. **FPS HUD** — top-left, semi-transparent black box. Hidden when F3 debug is open or HUD hidden.
3. **CPS HUD** — left|right clicks per second, rolling 1s window, edge-triggered (count the press, not the hold). Under the FPS box.
4. **Keystrokes HUD** — bottom-center above hotbar: W row, A/S/D row, wide SPACE bar, LMB/RMB row. Pressed = white bg + black text.
5. **DC badge** — blue-bold `D` + aqua-bold `C` + space rendered BEFORE the nick, in (a) nametag above head, (b) tab list. Toggleable.
6. **Owner crown** — player UUID `76b1ef92-f047-428a-a068-ccd4d1eb4863` additionally gets a gold bold `★` AFTER the nick (nametag + tab list) **iff** their session is Microsoft-authenticated (§2). Offline NEVER gets the crown, even on UUID match.
7. **Custom main menu** — replace vanilla buttons with 4 pretty ones (rounded rect, blue border, hover glow): Singleplayer / Multiplayer / DontCam Mods (opens the menu from (1)) / Quit. Paint a minimal banner over the logo zone: centered `DontCam Client` + subtitle + version line.

## 2. Identity, roster, server protocol (same on every line)

- The launcher passes JVM props: `-Ddontcam.auth=msa|legacy`, `-Ddontcam.uuid=<dashed-uuid>`. Read once at init.
- `isDontCam(uuid)` = equals local UUID **or** in roster set. `isOwner(uuid)` = equals OWNER_UUID **and** (local ? localMicrosoft : rosterMicrosoft flag).
- On server join send hello: channel `dontcam:hello`, JSON `{"uuid":"<undashed>","microsoft":bool,"mod":"0.1.0"}`.
  Use the ERA-CORRECT networking API:
  - 1.8.9 Forge: raw `C17PacketCustomPayload` (+ `REGISTER` packet first)
  - 1.20.1 Fabric: legacy `ClientPlayNetworking.send(Identifier, PacketByteBuf)` / `registerGlobalReceiver`
  - 1.21.x Fabric: `CustomPayload` records + `PacketCodec` + `PayloadTypeRegistry`
- Roster (server → client, channel `dontcam:roster`): JSON `{"players":[{"uuid":"...","microsoft":bool}]}` replaces the roster. Without a companion server plugin only the local badge/crown render — expected, not a bug.

## 3. Hard requirements / acceptance

- Build green on first verified pass; remapped production jar only.
- No console spam; no crash on vanilla singleplayer + vanilla server join.
- RShift opens menu; toggles persist across restarts.
- HUD renders in-game, hidden in F3/menu.
- Badge visible in F5 + tab list for self; crown ONLY with `-Ddontcam.auth=msa`.
- Main menu: banner + exactly the 4 buttons, all working; no vanilla buttons remain.
- Loader API (Forge / Fabric API) is a hard dependency entry; the mod must fail cleanly on its absence screen — never crash.
- Give me: full build files, every config json, every source file complete (no stubs/TODO), exact build command with JAVA_HOME.

## 4. Reference implementations in this repo (same feature set, working)

- `client-mod/` — Forge 1.8.9 (MCP stable_22, Java 8): menu, HUD, badges+crown, hello/roster via `C17PacketCustomPayload`, config JSON.
- `fabric-mod-1.20.1/` — Fabric 1.20.1 (legacy networking API variant).
- `fabric-mod/` — Fabric 1.21.x (modern payload records, mixins verified against Yarn tiny).
- For a new line: copy the CLOSEST reference (1.12←1.8 Forge; 1.16–1.20←1.20.1 Fabric; 26.x←1.21.x Fabric), translate only what the era's mappings require, and verify each hooked method in that line's Yarn/MCP mappings before trusting it.
