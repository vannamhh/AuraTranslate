#!/usr/bin/env node
/**
 * Copies `src-tauri/resources/dict/*.db` into the dev resource directory
 * (`<cargo target>/debug/dict/`), which `tauri dev` reads and only `tauri.release.conf.json`
 * fills. Runs as `predev`. A file whose size and mtime already match is left alone; files
 * present only in the destination (CI's e2e fixture) are never touched.
 */
import { constants, copyFileSync, existsSync, mkdirSync, readdirSync, renameSync, statSync, utimesSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const TAURI_DIR = join(REPO_ROOT, 'src-tauri')

/** @param {NodeJS.ProcessEnv} env */
export function devDictDir(env = process.env) {
  const target = env.CARGO_TARGET_DIR ? resolve(TAURI_DIR, env.CARGO_TARGET_DIR) : join(TAURI_DIR, 'target')
  return join(target, 'debug', 'dict')
}

/**
 * @param {string} source
 * @param {string} dest
 * @returns {{ name: string, status: 'copied' | 'up-to-date' }[]}
 */
export function syncDevDicts(source, dest) {
  const names = readdirSync(source).filter((name) => name.endsWith('.db')).sort()
  if (names.length > 0) mkdirSync(dest, { recursive: true })
  return names.map((name) => {
    const from = join(source, name)
    const to = join(dest, name)
    const src = statSync(from)
    if (existsSync(to)) {
      const dst = statSync(to)
      // utimes round-trips through floating-point seconds, so an exact mtime match can miss by a hair.
      if (dst.size === src.size && Math.abs(dst.mtimeMs - src.mtimeMs) < 1) {
        return { name, status: 'up-to-date' }
      }
    }
    // Rename over the old file so a still-running app keeps reading the inode it opened.
    const partial = `${to}.partial`
    copyFileSync(from, partial, constants.COPYFILE_FICLONE)
    utimesSync(partial, src.atimeMs / 1000, src.mtimeMs / 1000)
    renameSync(partial, to)
    return { name, status: 'copied' }
  })
}

const isMain = process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href
if (isMain) {
  const source = join(TAURI_DIR, 'resources', 'dict')
  const dest = devDictDir()
  try {
    const results = syncDevDicts(source, dest)
    if (results.length === 0) {
      console.log(`sync-dev-dicts: no .db in ${source}; tauri dev will run without dictionaries`)
    } else {
      const copied = results.filter((r) => r.status === 'copied').length
      console.log(`sync-dev-dicts: ${copied} copied, ${results.length - copied} up to date -> ${dest}`)
    }
  } catch (err) {
    console.error(`sync-dev-dicts: ${err instanceof Error ? err.message : String(err)}`)
    process.exit(1)
  }
}
