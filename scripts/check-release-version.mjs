#!/usr/bin/env node
/**
 * Release gate: the pushed tag `v<semver>` must equal the version in `package.json`,
 * `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. Exit 1 names each file that differs.
 *
 * Usage: node scripts/check-release-version.mjs <tag> [--root <dir>]
 */
import { readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const TAG_RE = /^v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)$/
const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')

/** @param {string} text */
function cargoPackageVersion(text) {
  const section = /^\[package\]\s*$([\s\S]*?)(?=^\[)/m.exec(text)
  const match = section && /^version\s*=\s*"([^"]+)"\s*$/m.exec(section[1])
  if (!match) throw new Error("no version line under [package]")
  return match[1]
}

/**
 * @param {string} tag
 * @param {string} root
 * @returns {string[]} one problem per mismatch
 */
export function releaseVersionProblems(tag, root) {
  const match = TAG_RE.exec(tag)
  if (!match) return [`tag '${tag}' is not of the form v<major>.<minor>.<patch>`]
  const expected = match[1]
  const sources = [
    ['package.json', (t) => JSON.parse(t).version],
    ['src-tauri/Cargo.toml', cargoPackageVersion],
    ['src-tauri/tauri.conf.json', (t) => JSON.parse(t).version],
  ]
  const problems = []
  for (const [file, read] of sources) {
    const seen = read(readFileSync(join(root, file), 'utf8'))
    if (seen !== expected) problems.push(`${file}: version is '${seen}', the tag ${tag} expects '${expected}'`)
  }
  return problems
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try {
    const args = process.argv.slice(2)
    const rootAt = args.indexOf('--root')
    const root = rootAt >= 0 ? resolve(args[rootAt + 1] ?? '') : REPO_ROOT
    const tag = rootAt >= 0 ? args.filter((_, i) => i !== rootAt && i !== rootAt + 1)[0] : args[0]
    if (!tag) throw new Error('usage: check-release-version.mjs <tag> [--root <dir>]')
    const problems = releaseVersionProblems(tag, root)
    if (problems.length > 0) {
      for (const p of problems) console.error(`check-release-version: ${p}`)
      process.exit(1)
    }
    console.log(`check-release-version: ${tag} matches package.json, Cargo.toml and tauri.conf.json`)
  } catch (err) {
    console.error(`check-release-version: ${err instanceof Error ? err.message : err} (infrastructure failure, not a pass)`)
    process.exit(1)
  }
}
