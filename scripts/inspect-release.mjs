#!/usr/bin/env node
/**
 * Checks an unpacked release build (the `.app`, or the target of `msiexec /a`) against
 * `dict-manifest.toml`, the bundled fonts under `src-tauri/resources/fonts` and the license files
 * under `license/`, and prints a Markdown size report for the CI step summary (NFR6). Exit 1 when
 * any dictionary, font or license file is missing, duplicated or has the wrong SHA-256. The sizes
 * are reported, never judged.
 *
 * With `--json <path>` it also writes the sizes as `{os, installerBytes, payloadBytes, webview2Bytes}`
 * (only when the build is clean), which `release-checksums.mjs --sizes` appends to the release notes.
 *
 * Usage: node scripts/inspect-release.mjs --root <dir> --installer <file> [--manifest <path>] [--json <path>]
 */
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { inspectInstalledFonts, inspectInstalledLicenses, inspectInstalledTree } from './lib/dict-fetch.mjs'
import { msiStreamSizes } from './lib/msi-streams.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const FONT_SOURCE_DIR = join(REPO_ROOT, 'src-tauri', 'resources', 'fonts')
const LICENSE_DIRS = [
  join(REPO_ROOT, 'src-tauri', 'resources', 'license'),
  join(REPO_ROOT, 'tools', 'dict-build', 'assets', 'licenses'),
]

function expectedLicenses() {
  const listed = LICENSE_DIRS.flatMap((dir) =>
    readdirSync(dir)
      .filter((name) => name.endsWith('.txt'))
      .map((name) => ({ name, sourcePath: join(dir, name) })),
  )
  return [...listed, { name: 'NOTICE.txt', sourcePath: join(REPO_ROOT, 'NOTICE.txt') }]
}

/** @param {string[]} argv */
function parseArgs(argv) {
  /** @type {Record<string, string>} */
  const values = { '--manifest': join(REPO_ROOT, 'dict-manifest.toml') }
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i]
    const value = argv[i + 1]
    if (!['--root', '--installer', '--manifest', '--json'].includes(flag) || value === undefined) {
      throw new Error(`unknown or incomplete argument '${flag}'. Usage: inspect-release.mjs --root <dir> --installer <file> [--manifest <path>] [--json <path>]`)
    }
    values[flag] = resolve(value)
  }
  if (!values['--root'] || !values['--installer']) throw new Error('--root and --installer are required')
  return { root: values['--root'], installer: values['--installer'], manifest: values['--manifest'], json: values['--json'] }
}

/** @param {number} n */
const fmt = (n) => n.toLocaleString('en-US')

try {
  const { root, installer, manifest, json } = parseArgs(process.argv.slice(2))
  const tree = await inspectInstalledTree({ manifestText: readFileSync(manifest, 'utf8'), root })
  const { problems, installedBytes, adminPackageBytes } = tree
  const isMsi = installer.toLowerCase().endsWith('.msi')
  const binaryWebview2Bytes = isMsi
    ? msiStreamSizes(installer)
        .filter((s) => /^Binary\..*webview2/i.test(s.name))
        .reduce((sum, s) => sum + s.size, 0)
    : 0
  const webview2Bytes = tree.webview2Bytes + binaryWebview2Bytes
  const fonts = await inspectInstalledFonts({ sourceDir: FONT_SOURCE_DIR, root })
  problems.push(...fonts.problems)
  const licenses = await inspectInstalledLicenses({ expected: expectedLicenses(), root })
  problems.push(...licenses.problems)
  const installerBytes = statSync(installer).size
  console.log(`### NFR6 size report (${installer.split(/[\\/]/).pop()})`)
  console.log('')
  console.log('| Measure | bytes |')
  console.log('|---|---:|')
  console.log(`| installer file | ${fmt(installerBytes)} |`)
  console.log(`| installed tree, WebView2 and administrative-image package excluded | ${fmt(installedBytes)} |`)
  console.log(`| WebView2 files excluded from the line above | ${fmt(tree.webview2Bytes)} |`)
  console.log(`| WebView2 in the installer's Binary table | ${fmt(binaryWebview2Bytes)} |`)
  console.log(`| \`.msi\` copy left by \`msiexec /a\`, excluded | ${fmt(adminPackageBytes)} |`)
  console.log('')
  console.log('Reference ceiling 500,000,000 bytes. Measured only; the verdict is the PRD tier (Ice).')
  if (problems.length > 0) {
    console.error(`\n${problems.length} problem(s) in ${root}:`)
    for (const p of problems) console.error(`  ${p}`)
    process.exit(1)
  }
  if (json) {
    const os = isMsi ? 'Windows' : 'macOS'
    writeFileSync(json, `${JSON.stringify({ os, installerBytes, payloadBytes: installedBytes, webview2Bytes }, null, 2)}\n`)
  }
  console.error(`all dictionary files in ${root} match ${manifest}, all ${fonts.fontCount} font files match ${FONT_SOURCE_DIR} and all ${licenses.licenseCount} license files match their sources`)
} catch (err) {
  console.error(`inspect-release: ${err instanceof Error ? err.message : err}`)
  process.exit(1)
}
