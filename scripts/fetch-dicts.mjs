#!/usr/bin/env node
/**
 * Downloads the dictionary `.db` files named in `dict-manifest.toml` (AD-25) into
 * `src-tauri/resources/dict/` and verifies each SHA-256 before the file takes its final name.
 * Exit 0 only when every entry is present and matches; any mismatch or failed download exits 1.
 *
 * Usage: node scripts/fetch-dicts.mjs [--manifest <path>] [--dest <dir>]
 */
import { readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { fetchDicts } from './lib/dict-fetch.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')

/** @param {string[]} argv */
function parseArgs(argv) {
  const options = {
    manifest: join(REPO_ROOT, 'dict-manifest.toml'),
    dest: join(REPO_ROOT, 'src-tauri', 'resources', 'dict'),
  }
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i]
    const value = argv[i + 1]
    if ((flag !== '--manifest' && flag !== '--dest') || value === undefined) {
      throw new Error(`unknown or incomplete argument '${flag}'. Usage: fetch-dicts.mjs [--manifest <path>] [--dest <dir>]`)
    }
    options[flag === '--manifest' ? 'manifest' : 'dest'] = resolve(value)
  }
  return options
}

try {
  const { manifest, dest } = parseArgs(process.argv.slice(2))
  const results = await fetchDicts({
    manifestText: readFileSync(manifest, 'utf8'),
    dest,
    log: (line) => console.log(line),
  })
  const bad = results.filter((r) => r.status === 'sha-mismatch' || r.status === 'download-failed')
  if (bad.length > 0) {
    console.error(`\n${bad.length} of ${results.length} dictionary files failed: ${bad.map((r) => r.fileName).join(', ')}`)
    process.exit(1)
  }
  console.log(`\nall ${results.length} dictionary files match ${manifest}`)
} catch (err) {
  console.error(`fetch-dicts: ${err instanceof Error ? err.message : err}`)
  process.exit(1)
}
