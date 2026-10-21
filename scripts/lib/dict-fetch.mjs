import { createHash } from 'node:crypto'
import { createReadStream, createWriteStream } from 'node:fs'
import { mkdir, readdir, rename, rm, stat } from 'node:fs/promises'
import { join } from 'node:path'
import { Readable, Transform } from 'node:stream'
import { pipeline } from 'node:stream/promises'
import { parseStrictToml } from './strict-toml.mjs'

const SHA256_RE = /^[0-9a-f]{64}$/

/**
 * @typedef {{ fileName: string, url: string, sha256: string }} ManifestEntry
 * @typedef {{ fileName: string, status: 'kept' | 'downloaded' | 'sha-mismatch' | 'download-failed', expected: string, actual?: string, detail?: string }} EntryResult
 */

/**
 * Entries of `dict-manifest.toml`: `[base]` first, then each `[[detachable]]`, in file order.
 * `[[removed]]` rows are tombstones and never become entries.
 * @param {string} text
 * @returns {ManifestEntry[]}
 */
export function manifestEntries(text) {
  const { tables } = parseStrictToml(text)
  const rows = [...(tables.get('base') ?? []), ...(tables.get('detachable') ?? [])]
  if (rows.length === 0) throw new Error('dict-manifest.toml has no [base] or [[detachable]] entry')
  return rows.map((row) => {
    const url = row.url
    const sha256 = row.sha256
    if (typeof url !== 'string' || typeof sha256 !== 'string' || !SHA256_RE.test(sha256)) {
      throw new Error(`manifest entry without a valid url/sha256: ${JSON.stringify(row)}`)
    }
    const fileName = url.slice(url.lastIndexOf('/') + 1)
    if (!/^[A-Za-z0-9._-]+\.db$/.test(fileName)) {
      throw new Error(`manifest url does not end in a plain .db file name: ${url}`)
    }
    return { fileName, url, sha256 }
  })
}

/** @param {string} path */
export async function sha256OfFile(path) {
  const hash = createHash('sha256')
  for await (const chunk of createReadStream(path)) hash.update(chunk)
  return hash.digest('hex')
}

/** @param {string} path */
async function exists(path) {
  try {
    await stat(path)
    return true
  } catch {
    return false
  }
}

/**
 * Downloads every manifest entry into `dest`, renaming a temporary file into place only after
 * its SHA-256 equals the manifest's. A file already in `dest` whose hash matches is kept.
 * Nothing unverified is ever left under its final name.
 *
 * @param {{ manifestText: string, dest: string, fetchImpl?: typeof fetch, log?: (line: string) => void }} options
 * @returns {Promise<EntryResult[]>}
 */
export async function fetchDicts({ manifestText, dest, fetchImpl = fetch, log = () => {} }) {
  const entries = manifestEntries(manifestText)
  await mkdir(dest, { recursive: true })
  /** @type {EntryResult[]} */
  const results = []

  for (const entry of entries) {
    const target = join(dest, entry.fileName)
    const partial = join(dest, `.${entry.fileName}.part`)

    if (await exists(target)) {
      const present = await sha256OfFile(target)
      if (present === entry.sha256) {
        log(`kept       ${entry.fileName} (sha256 already matches)`)
        results.push({ fileName: entry.fileName, status: 'kept', expected: entry.sha256 })
        continue
      }
      log(`replacing  ${entry.fileName} (sha256 ${present} does not match the manifest)`)
      await rm(target, { force: true })
    }

    try {
      const response = await fetchImpl(entry.url)
      if (!response.ok || !response.body) {
        throw new Error(`HTTP ${response.status} ${response.statusText}`.trim())
      }
      const hash = createHash('sha256')
      const tap = new Transform({
        transform(chunk, _encoding, done) {
          hash.update(chunk)
          done(null, chunk)
        },
      })
      await pipeline(Readable.fromWeb(/** @type {any} */ (response.body)), tap, createWriteStream(partial))
      const actual = hash.digest('hex')
      if (actual !== entry.sha256) {
        await rm(partial, { force: true })
        log(`SHA MISMATCH ${entry.fileName}\n  expected ${entry.sha256}\n  actual   ${actual}`)
        results.push({ fileName: entry.fileName, status: 'sha-mismatch', expected: entry.sha256, actual })
        continue
      }
      await rename(partial, target)
      log(`downloaded ${entry.fileName} (sha256 ${actual})`)
      results.push({ fileName: entry.fileName, status: 'downloaded', expected: entry.sha256, actual })
    } catch (err) {
      await rm(partial, { force: true })
      const detail = String(err instanceof Error ? err.message : err)
      log(`DOWNLOAD FAILED ${entry.fileName} from ${entry.url}: ${detail}`)
      results.push({ fileName: entry.fileName, status: 'download-failed', expected: entry.sha256, detail })
    }
  }
  return results
}

/** @param {string} dir @returns {Promise<{ path: string, size: number }[]>} */
async function walkFiles(dir) {
  /** @type {{ path: string, size: number }[]} */
  const files = []
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name)
    if (entry.isDirectory()) files.push(...(await walkFiles(full)))
    else if (entry.isFile()) files.push({ path: full, size: (await stat(full)).size })
  }
  return files
}

/**
 * Looks for every manifest file under `root` (an unpacked `.app` or a `msiexec /a` target),
 * requiring exactly one copy per name, inside a directory called `dict`, with the manifest's
 * SHA-256, and no other `.db` under a `dict` directory. Also totals the bytes of the whole tree, minus files named like WebView2.
 *
 * @param {{ manifestText: string, root: string }} options
 * @returns {Promise<{ problems: string[], installedBytes: number, webview2Bytes: number }>}
 */
export async function inspectInstalledTree({ manifestText, root }) {
  const entries = manifestEntries(manifestText)
  const files = await walkFiles(root)
  const problems = []
  const known = new Set(entries.map((e) => e.fileName))
  for (const f of files) {
    const posix = f.path.replaceAll('\\', '/')
    const m = posix.match(/\/dict\/([^/]+\.db)$/)
    if (m && !known.has(m[1])) problems.push(`${m[1]}: .db under a 'dict' directory but not in dict-manifest.toml`)
  }
  for (const entry of entries) {
    const hits = files.filter((f) => f.path.replaceAll('\\', '/').endsWith(`/dict/${entry.fileName}`))
    if (hits.length !== 1) {
      problems.push(`${entry.fileName}: found ${hits.length} copies under a 'dict' directory, need exactly 1`)
      continue
    }
    const actual = await sha256OfFile(hits[0].path)
    if (actual !== entry.sha256) {
      problems.push(`${entry.fileName}: sha256 ${actual} does not match the manifest ${entry.sha256}`)
    }
  }
  const webview2 = files.filter((f) => /webview2/i.test(f.path.slice(root.length)))
  const webview2Bytes = webview2.reduce((sum, f) => sum + f.size, 0)
  const installedBytes = files.reduce((sum, f) => sum + f.size, 0) - webview2Bytes
  return { problems, installedBytes, webview2Bytes }
}

const MIN_BUNDLED_FONTS = 4

/**
 * Requires every `.otf`/`.ttf` in `sourceDir` to appear exactly once under a `fonts` directory
 * of `root` with the same SHA-256.
 *
 * @param {{ sourceDir: string, root: string }} options
 * @returns {Promise<{ problems: string[], fontCount: number }>}
 */
export async function inspectInstalledFonts({ sourceDir, root }) {
  const sources = (await readdir(sourceDir)).filter((name) => /\.(otf|ttf)$/.test(name)).sort()
  if (sources.length < MIN_BUNDLED_FONTS) {
    throw new Error(`found ${sources.length} font files in ${sourceDir}, need at least ${MIN_BUNDLED_FONTS}; this is an infrastructure failure, not a pass`)
  }
  const files = await walkFiles(root)
  const problems = []
  for (const name of sources) {
    const hits = files.filter((f) => f.path.replaceAll('\\', '/').endsWith(`/fonts/${name}`))
    if (hits.length !== 1) {
      problems.push(`${name}: found ${hits.length} copies under a 'fonts' directory, need exactly 1`)
      continue
    }
    const [expected, actual] = await Promise.all([sha256OfFile(join(sourceDir, name)), sha256OfFile(hits[0].path)])
    if (actual !== expected) problems.push(`${name}: sha256 ${actual} does not match the source ${expected}`)
  }
  return { problems, fontCount: sources.length }
}

const MIN_BUNDLED_LICENSES = 9

/**
 * Requires every expected license file to appear exactly once under a `license` directory of
 * `root` with the same SHA-256 as its source.
 *
 * @param {{ expected: { name: string, sourcePath: string }[], root: string }} options
 * @returns {Promise<{ problems: string[], licenseCount: number }>}
 */
export async function inspectInstalledLicenses({ expected, root }) {
  if (expected.length < MIN_BUNDLED_LICENSES) {
    throw new Error(`found ${expected.length} license sources, need at least ${MIN_BUNDLED_LICENSES}; this is an infrastructure failure, not a pass`)
  }
  const files = await walkFiles(root)
  const problems = []
  for (const { name, sourcePath } of expected) {
    const hits = files.filter((f) => f.path.replaceAll('\\', '/').endsWith(`/license/${name}`))
    if (hits.length !== 1) {
      problems.push(`${name}: found ${hits.length} copies under a 'license' directory, need exactly 1`)
      continue
    }
    const [want, actual] = await Promise.all([sha256OfFile(sourcePath), sha256OfFile(hits[0].path)])
    if (actual !== want) problems.push(`${name}: sha256 ${actual} does not match the source ${want}`)
  }
  return { problems, licenseCount: expected.length }
}
