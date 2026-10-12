#!/usr/bin/env node
/**
 * Writes SHA256SUMS.txt (readable by `shasum -c`) and release-body.md (notes plus a checksum
 * section) for the files a release attaches. Every failure exits 1 before any output is written.
 */
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import { sha256OfFile } from './lib/dict-fetch.mjs'

const USAGE = 'usage: release-checksums.mjs --notes <md> --out-dir <dir> <file...>'

/** @param {string[]} args @returns {{ notes: string, outDir: string, files: string[] }} */
export function parseArgs(args) {
  /** @type {string | undefined} */
  let notes
  /** @type {string | undefined} */
  let outDir
  /** @type {string[]} */
  const files = []
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--notes') notes = args[++i]
    else if (args[i] === '--out-dir') outDir = args[++i]
    else files.push(/** @type {string} */ (args[i]))
  }
  if (!notes || !outDir) throw new Error(USAGE)
  if (files.length === 0) throw new Error(`no files to hash. ${USAGE}`)
  return { notes, outDir, files }
}

/**
 * @param {string} notesText
 * @param {Array<{ name: string, hash: string }>} entries
 * @returns {string}
 */
export function releaseBody(notesText, entries) {
  const named = (/** @type {string} */ ext) => (entries.find((e) => e.name.endsWith(ext)) ?? entries[0])?.name ?? ''
  const mac = named('.dmg')
  const win = named('.msi')
  const list = entries.map((e) => `- \`${e.name}\`: \`${e.hash}\``).join('\n')
  return [
    notesText.replace(/\s+$/, ''),
    '',
    '### Kiểm tra tệp tải về (SHA-256)',
    '',
    list,
    '',
    'Tải `SHA256SUMS.txt` về cùng thư mục với bản cài. Hoa thường trong hash không quan trọng.',
    '',
    '**macOS** (Terminal):',
    '',
    '```sh',
    `shasum -a 256 ${mac}`,
    'shasum -a 256 -c --ignore-missing SHA256SUMS.txt',
    '```',
    '',
    '**Windows** (PowerShell):',
    '',
    '```powershell',
    `Get-FileHash .\\${win} -Algorithm SHA256`,
    '```',
    '',
  ].join('\n')
}

/**
 * @param {{ notes: string, outDir: string, files: string[] }} opts
 * @returns {Promise<void>}
 */
export async function writeReleaseChecksums({ notes, outDir, files }) {
  if (!existsSync(notes) || !statSync(notes).isFile()) throw new Error(`notes file not found: ${notes}`)
  const seen = new Set()
  for (const file of files) {
    if (!existsSync(file) || !statSync(file).isFile()) throw new Error(`file not found: ${file}`)
    const name = basename(file)
    if (seen.has(name)) throw new Error(`duplicate file name: ${name}`)
    seen.add(name)
  }
  const entries = []
  for (const file of files) entries.push({ name: basename(file), hash: await sha256OfFile(file) })
  const notesText = readFileSync(notes, 'utf8')
  mkdirSync(outDir, { recursive: true })
  writeFileSync(join(outDir, 'SHA256SUMS.txt'), entries.map((e) => `${e.hash}  ${e.name}\n`).join(''))
  writeFileSync(join(outDir, 'release-body.md'), releaseBody(notesText, entries))
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try {
    const opts = parseArgs(process.argv.slice(2))
    await writeReleaseChecksums({ ...opts, outDir: resolve(opts.outDir) })
    console.log(`release-checksums: wrote SHA256SUMS.txt and release-body.md for ${opts.files.length} file(s)`)
  } catch (err) {
    console.error(`release-checksums: ${err instanceof Error ? err.message : err}`)
    process.exit(1)
  }
}
