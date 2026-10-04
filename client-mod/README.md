# DontCam Client Mod (Forge 1.8.9)

Companion mod for DontCam Client: Right-Shift menu, HUD (FPS / CPS / keystrokes),
DC badges before nametags, owner crown (Microsoft sessions only).

## Build

Needs Java 8 and Gradle 2.14:

```bat
set JAVA_HOME=C:\path\to\temurin-8
tools\gradle\gradle-2.14\bin\gradle.bat -p client-mod build
```

Output: `client-mod/build/libs/dontcam-1.8.9-0.1.0.jar`
→ copy to `src-tauri/resources/dontcam/dontcam-1.8.9-0.1.0.jar`
(the launcher embeds it with `include_bytes!` and injects it into the
instance `mods/` on launch when the profile has `dontcam_mod` on).

Toolchain notes:
- `build.gradle` uses ForgeGradle **2.1.6** (`apply plugin: 'net.minecraftforge.gradle.forge'`)
  with `mappings = "stable_22"` and Forge `1.8.9-11.15.1.2318`.
  (FG 1.2 is too old for 11.15.x; its plugin id is the short `'forge'`.)
- Repos must be `mavenCentral()` + `https://maven.minecraftforge.net/`
  (the old `files.minecraftforge.net/maven` 301-redirects, `jcenter()` is dead).

## Runtime contract with the launcher

The launcher passes (JVM properties):
- `-Ddontcam.auth=msa|legacy` — Microsoft vs offline session
- `-Ddontcam.uuid=<dashed-uuid>` — local player UUID

Rules enforced client-side:
- DC badge renders for roster members + self.
- Crown renders only when UUID == `76b1ef92-f047-428a-a068-ccd4d1eb4863`
  **and** that session is Microsoft (`msa`). Offline never gets the crown.

## Multiplayer roster protocol (needs companion server plugin for full effect)

- Client → server on join: `REGISTER` (`DontCam|Hello\0DontCam|Roster`), then
  `DontCam|Hello` JSON: `{"uuid":"<undashed>","microsoft":bool,"mod":"0.1.0"}`.
- Server → client: `DontCam|Roster` JSON:
  `{"players":[{"uuid":"...","microsoft":true}]}`.
- Without a companion plugin, vanilla servers ignore the hello packet and only
  the local badge/crown render.

## Porting to all launcher versions

One module per era (this one is the reference):
- [x] 1.8.x — Forge (`client-mod/`), MCP stable_22, Java 8
- [ ] 1.12.x — Forge (FG 2.3 / MCP snapshot, Java 8)
- [ ] 1.16.x — Forge (FG 4/5, MojMaps, Java 8/16)
- [ ] 1.17.x — Forge (FG 5, MojMaps, Java 16)
- [ ] 1.18–1.19 — Forge 9 (Java 17)
- [ ] 1.20–1.21 — Fabric (Yarn, Java 17/21, one range build)
- [ ] 26.x — TBD (toolchain depends on Mojang's current mappings/loader state)

Launcher side is already multi-version ready: `BUNDLED_MODS` table in
`src-tauri/src/launch/mod.rs` maps MC prefix → embedded jar. New ports only
add a row + the built jar under `src-tauri/resources/dontcam/`.
