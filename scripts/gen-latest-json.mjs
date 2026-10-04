// gen-latest-json.mjs — builds the Tauri v2 updater `latest.json` from the
// signed bundles produced by `npm run tauri:build` (createUpdaterArtifacts=true).
//
// Scans:
//   src-tauri/target/release/bundle/nsis/*-setup.exe (+ .sig)
//   src-tauri/target/release/bundle/msi/*.msi (+ .sig)
//
// Writes: src-tauri/target/release/bundle/latest.json
// Upload that file + the installers to a GitHub Release of
// aanges/dontcam-launcher tagged v<package.json version>, so the updater
// endpoint (.../releases/latest/download/latest.json) serves it.
//
// Usage: npm run updater:json
// Env: UPDATER_BASE_URL (default https://github.com/aanges/dontcam-launcher/releases/download)
//      UPDATER_NOTES (release notes text)

import { readdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
const version = pkg.version
const baseUrl =
  process.env.UPDATER_BASE_URL ??
  'https://github.com/aanges/dontcam-launcher/releases/download'
const tag = `v${version}`
const notes = process.env.UPDATER_NOTES ?? `DontCam Client ${tag}`

const bundle = join(root, 'src-tauri', 'target', 'release', 'bundle')
const platforms = {}

function pick(dir, ext) {
  const full = join(bundle, dir)
  if (!existsSync(full)) return null
  const files = readdirSync(full).filter((f) => f.endsWith(ext))
  // prefer setup.exe over plain exe for nsis
  files.sort((a, b) => {
    const score = (n) => (n.includes('setup') ? 0 : 1)
    return score(a) - score(b) || a.localeCompare(b)
  })
  return files.length ? join(full, files[0]) : null
}

const nsis = pick('nsis', '.exe')
const msi = pick('msi', '.msi')
// Tauri updater key for 64-bit Windows NSIS/MSI:
for (const [key, file] of Object.entries({ 'windows-x86_64': nsis ?? msi })) {
  if (!file) continue
  const sigFile = `${file}.sig`
  if (!existsSync(sigFile)) {
    console.error(`[updater:json] MISSING signature: ${sigFile} — run the signed build (build-release.ps1) first.`)
    process.exit(1)
  }
  const signature = readFileSync(sigFile, 'utf8').trim()
  const name = file.split(/[/\\]/).pop()
  platforms[key] = {
    signature,
    url: `${baseUrl}/${tag}/${name}`,
  }
}

if (Object.keys(platforms).length === 0) {
  console.error('[updater:json] No bundles found — run `npm run tauri:build` first.')
  process.exit(1)
}

const latest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms,
}

const out = join(bundle, 'latest.json')
writeFileSync(out, JSON.stringify(latest, null, 2))
console.log(`[updater:json] wrote ${out} for v${version}`)
console.log(`[updater:json] upload it + installers to GitHub Release ${tag} on aanges/dontcam-launcher`)
