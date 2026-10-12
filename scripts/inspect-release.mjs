#!/usr/bin/env node
/**
 * Checks an unpacked release build (the `.app`, or the target of `msiexec /a`) against
 * `dict-manifest.toml` and the bundled fonts under `src-tauri/resources/fonts`, and prints a
 * Markdown size report for the CI step summary (NFR6). Exit 1 when any dictionary or font file
 * is missing, duplicated or has the wrong SHA-256. The sizes are reported, never judged.
 *
 * Usage: node scripts/inspect-release.mjs --root <dir> --installer <file> [--manifest <path>]
 */
import { readFileSync, statSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { inspectInstalledFonts, inspectInstalledTree } from './lib/dict-fetch.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const FONT_SOURCE_DIR = join(REPO_ROOT, 'src-tauri', 'resources', 'fonts')

/** @param {string[]} argv */
function parseArgs(argv) {
  /** @type {Record<string, string>} */
  const values = { '--manifest': join(REPO_ROOT, 'dict-manifest.toml') }
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i]
    const value = argv[i + 1]
    if (!['--root', '--installer', '--manifest'].includes(flag) || value === undefined) {
      throw new Error(`unknown or incomplete argument '${flag}'. Usage: inspect-release.mjs --root <dir> --installer <file> [--manifest <path>]`)
    }
    values[flag] = resolve(value)
  }
  if (!values['--root'] || !values['--installer']) throw new Error('--root and --installer are required')
  return { root: values['--root'], installer: values['--installer'], manifest: values['--manifest'] }
}

/** @param {number} n */
const fmt = (n) => n.toLocaleString('en-US')

try {
  const { root, installer, manifest } = parseArgs(process.argv.slice(2))
  const { problems, installedBytes, webview2Bytes } = await inspectInstalledTree({
    manifestText: readFileSync(manifest, 'utf8'),
    root,
  })
  const fonts = await inspectInstalledFonts({ sourceDir: FONT_SOURCE_DIR, root })
  problems.push(...fonts.problems)
  const installerBytes = statSync(installer).size
  console.log(`### NFR6 size report (${installer.split(/[\\/]/).pop()})`)
  console.log('')
  console.log('| Measure | bytes |')
  console.log('|---|---:|')
  console.log(`| installer file | ${fmt(installerBytes)} |`)
  console.log(`| installed tree, WebView2 excluded | ${fmt(installedBytes)} |`)
  console.log(`| WebView2 files excluded from the line above | ${fmt(webview2Bytes)} |`)
  console.log('')
  console.log('Reference ceiling 400,000,000 bytes. Measured only; the verdict is the PRD tier (Ice).')
  if (problems.length > 0) {
    console.error(`\n${problems.length} problem(s) in ${root}:`)
    for (const p of problems) console.error(`  ${p}`)
    process.exit(1)
  }
  console.error(`all dictionary files in ${root} match ${manifest} and all ${fonts.fontCount} font files match ${FONT_SOURCE_DIR}`)
} catch (err) {
  console.error(`inspect-release: ${err instanceof Error ? err.message : err}`)
  process.exit(1)
}
