#!/usr/bin/env node
/**
 * Writes SHA256SUMS.txt (readable by `shasum -c`) and release-body.md (notes plus a checksum
 * section) for the files a release attaches. Every failure exits 1 before any output is written.
 */
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import { sha256OfFile } from './lib/dict-fetch.mjs'

const USAGE = 'usage: release-checksums.mjs --notes <md> --out-dir <dir> [--sizes <json>]... <file...>'

/** @param {string[]} args @returns {{ notes: string, outDir: string, sizes: string[], files: string[] }} */
export function parseArgs(args) {
  /** @type {string | undefined} */
  let notes
  /** @type {string | undefined} */
  let outDir
  /** @type {string[]} */
  const files = []
  /** @type {string[]} */
  const sizes = []
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--notes') notes = args[++i]
    else if (args[i] === '--out-dir') outDir = args[++i]
    else if (args[i] === '--sizes') sizes.push(/** @type {string} */ (args[++i]))
    else files.push(/** @type {string} */ (args[i]))
  }
  if (!notes || !outDir) throw new Error(USAGE)
  if (files.length === 0) throw new Error(`no files to hash. ${USAGE}`)
  return { notes, outDir, sizes, files }
}

const SIZE_OSES = ['macOS', 'Windows']

/** @param {unknown} n */
const positive = (n) => typeof n === 'number' && Number.isInteger(n) && n > 0

/**
 * Validates the per-OS size reports written by `inspect-release.mjs --json`. Every OS needs a
 * positive payload; Windows also needs a positive WebView2 (macOS embeds none).
 *
 * @param {Array<{ source: string, data: any }>} reports
 * @returns {Array<{ os: string, payloadBytes: number, webview2Bytes: number }>}
 */
export function checkSizes(reports) {
  const byOs = new Map()
  for (const { source, data } of reports) {
    if (!SIZE_OSES.includes(data?.os)) throw new Error(`${source}: unknown or missing 'os'`)
    if (byOs.has(data.os)) throw new Error(`${source}: second size report for ${data.os}`)
    if (!positive(data.payloadBytes)) throw new Error(`${source}: missing payloadBytes for ${data.os}`)
    if (data.os === 'Windows' && !positive(data.webview2Bytes)) {
      throw new Error(`${source}: missing webview2Bytes for Windows`)
    }
    byOs.set(data.os, { os: data.os, payloadBytes: data.payloadBytes, webview2Bytes: data.webview2Bytes ?? 0 })
  }
  for (const os of SIZE_OSES) if (!byOs.has(os)) throw new Error(`no size report for ${os}`)
  return SIZE_OSES.map((os) => byOs.get(os))
}

/** @param {number} n */
const bytes = (n) => n.toLocaleString('en-US')

/** @param {Array<{ os: string, payloadBytes: number, webview2Bytes: number }>} sizes */
function sizeSection(sizes) {
  const lines = sizes.flatMap((s) => [
    `- ${s.os}, payload sản phẩm: ${bytes(s.payloadBytes)} byte`,
    s.os === 'Windows'
      ? `- Windows, WebView2 Runtime nhúng (không tính vào trần): ${bytes(s.webview2Bytes)} byte`
      : '- macOS, WebView2: không nhúng, dùng WKWebView của hệ điều hành',
  ])
  return ['### Dung lượng (NFR6)', '', ...lines, '']
}

/**
 * @param {string} notesText
 * @param {Array<{ name: string, hash: string }>} entries
 * @param {Array<{ os: string, payloadBytes: number, webview2Bytes: number }>} [sizes]
 * @returns {string}
 */
export function releaseBody(notesText, entries, sizes = []) {
  const named = (/** @type {string} */ ext) => (entries.find((e) => e.name.endsWith(ext)) ?? entries[0])?.name ?? ''
  const mac = named('.dmg')
  const win = named('.msi')
  const list = entries.map((e) => `- \`${e.name}\`: \`${e.hash}\``).join('\n')
  return [
    notesText.replace(/\s+$/, ''),
    '',
    ...(sizes.length > 0 ? sizeSection(sizes) : []),
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
 * @param {{ notes: string, outDir: string, sizes?: string[], files: string[] }} opts
 * @returns {Promise<void>}
 */
export async function writeReleaseChecksums({ notes, outDir, sizes = [], files }) {
  const sizeReports = sizes.map((source) => {
    if (!existsSync(source) || !statSync(source).isFile()) throw new Error(`size report not found: ${source}`)
    try {
      return { source, data: JSON.parse(readFileSync(source, 'utf8')) }
    } catch {
      throw new Error(`${source}: not valid JSON`)
    }
  })
  const checkedSizes = sizes.length > 0 ? checkSizes(sizeReports) : []
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
  writeFileSync(join(outDir, 'release-body.md'), releaseBody(notesText, entries, checkedSizes))
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
