/**
 * Unit tests for `scripts/lib/dict-fetch.mjs`. Every case injects a fake `fetch`, so nothing
 * touches the network; files live in a per-test temp directory.
 */
import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { fetchDicts, inspectInstalledFonts, inspectInstalledLicenses, inspectInstalledTree, manifestEntries } from '../../scripts/lib/dict-fetch.mjs'

const sha = (text: string) => createHash('sha256').update(text).digest('hex')

const BODIES: Record<string, string> = {
  'dict-core.db': 'core bytes',
  'dict-thieu-chuu.db': 'thieu chuu bytes',
}

function manifest(overrides: Record<string, string> = {}): string {
  const core = overrides['dict-core.db'] ?? sha(BODIES['dict-core.db'])
  const thieu = overrides['dict-thieu-chuu.db'] ?? sha(BODIES['dict-thieu-chuu.db'])
  return `[base]
url            = "https://example.test/dict-v2/dict-core.db"
sha256         = "${core}"
source_version = "x"

[[detachable]]
name           = "thieu-chuu"
url            = "https://example.test/dict-v2/dict-thieu-chuu.db"
sha256         = "${thieu}"
source_version = "y"
`
}

function fakeFetch(bodies: Record<string, string>, failing: string[] = []) {
  const calls: string[] = []
  const impl = async (input: string | URL | Request): Promise<Response> => {
    const url = String(input)
    calls.push(url)
    const name = url.slice(url.lastIndexOf('/') + 1)
    if (failing.includes(name)) return new Response('missing', { status: 404, statusText: 'Not Found' })
    return new Response(bodies[name])
  }
  return { impl: impl as typeof fetch, calls }
}

let dest: string

beforeEach(() => {
  dest = mkdtempSync(join(tmpdir(), 'fetch-dicts-'))
})
afterEach(() => {
  rmSync(dest, { recursive: true, force: true })
})

describe('manifestEntries', () => {
  it('lists the base entry first and takes the file name from the url', () => {
    expect(manifestEntries(manifest()).map((e) => e.fileName)).toEqual(['dict-core.db', 'dict-thieu-chuu.db'])
  })

  it('rejects an entry whose sha256 is not 64 lowercase hex characters', () => {
    expect(() => manifestEntries(manifest({ 'dict-core.db': 'abc' }))).toThrow(/valid url\/sha256/)
  })
})

describe('fetchDicts', () => {
  it('downloads every file and keeps them under their final names when the hashes match', async () => {
    const { impl } = fakeFetch(BODIES)
    const results = await fetchDicts({ manifestText: manifest(), dest, fetchImpl: impl })
    expect(results.map((r) => r.status)).toEqual(['downloaded', 'downloaded'])
    expect(readFileSync(join(dest, 'dict-core.db'), 'utf8')).toBe('core bytes')
    expect(readdirSync(dest).sort()).toEqual(['dict-core.db', 'dict-thieu-chuu.db'])
  })

  it('reports expected and actual hash and leaves no file when the hash differs', async () => {
    const wrong = sha('not the real bytes')
    const { impl } = fakeFetch(BODIES)
    const lines: string[] = []
    const results = await fetchDicts({
      manifestText: manifest({ 'dict-core.db': wrong }),
      dest,
      fetchImpl: impl,
      log: (l) => lines.push(l),
    })
    const core = results.find((r) => r.fileName === 'dict-core.db')
    expect(core).toMatchObject({ status: 'sha-mismatch', expected: wrong, actual: sha('core bytes') })
    expect(existsSync(join(dest, 'dict-core.db'))).toBe(false)
    expect(readdirSync(dest)).toEqual(['dict-thieu-chuu.db'])
    expect(lines.join('\n')).toContain('dict-core.db')
    expect(lines.join('\n')).toContain(wrong)
  })

  it('leaves no partial file when the server answers 404', async () => {
    const { impl } = fakeFetch(BODIES, ['dict-thieu-chuu.db'])
    const results = await fetchDicts({ manifestText: manifest(), dest, fetchImpl: impl })
    expect(results.find((r) => r.fileName === 'dict-thieu-chuu.db')).toMatchObject({
      status: 'download-failed',
      detail: expect.stringContaining('404'),
    })
    expect(readdirSync(dest)).toEqual(['dict-core.db'])
  })

  it('leaves no partial file when the connection fails', async () => {
    const impl = (async () => {
      throw new TypeError('fetch failed')
    }) as unknown as typeof fetch
    const results = await fetchDicts({ manifestText: manifest(), dest, fetchImpl: impl })
    expect(results.map((r) => r.status)).toEqual(['download-failed', 'download-failed'])
    expect(readdirSync(dest)).toEqual([])
  })

  it('skips a file whose hash already matches and does not request it', async () => {
    writeFileSync(join(dest, 'dict-core.db'), BODIES['dict-core.db'])
    const { impl, calls } = fakeFetch(BODIES)
    const results = await fetchDicts({ manifestText: manifest(), dest, fetchImpl: impl })
    expect(results.map((r) => r.status)).toEqual(['kept', 'downloaded'])
    expect(calls).toEqual(['https://example.test/dict-v2/dict-thieu-chuu.db'])
  })

  it('leaves no file under the final name when a wrong-hash file is present and the replacement download fails', async () => {
    writeFileSync(join(dest, 'dict-core.db'), 'stale')
    const { impl } = fakeFetch(BODIES, ['dict-core.db'])
    const results = await fetchDicts({ manifestText: manifest(), dest, fetchImpl: impl })
    expect(results[0].status).toBe('download-failed')
    expect(existsSync(join(dest, 'dict-core.db'))).toBe(false)
  })

  it('replaces a present file whose hash does not match the manifest', async () => {
    writeFileSync(join(dest, 'dict-core.db'), 'stale')
    const { impl } = fakeFetch(BODIES)
    const results = await fetchDicts({ manifestText: manifest(), dest, fetchImpl: impl })
    expect(results[0].status).toBe('downloaded')
    expect(readFileSync(join(dest, 'dict-core.db'), 'utf8')).toBe('core bytes')
  })
})

describe('inspectInstalledTree', () => {
  function install(files: Record<string, string>) {
    for (const [rel, body] of Object.entries(files)) {
      const full = join(dest, rel)
      mkdirSync(dirname(full), { recursive: true })
      writeFileSync(full, body)
    }
  }

  it('accepts a tree that carries every manifest file in a dict directory with the right hash', async () => {
    install({ 'App/Resources/dict/dict-core.db': BODIES['dict-core.db'], 'App/Resources/dict/dict-thieu-chuu.db': BODIES['dict-thieu-chuu.db'] })
    const report = await inspectInstalledTree({ manifestText: manifest(), root: dest })
    expect(report.problems).toEqual([])
    expect(report.installedBytes).toBe('core bytes'.length + 'thieu chuu bytes'.length)
  })

  it('reports a missing file, a wrong hash and a copy outside a dict directory', async () => {
    install({ 'App/dict/dict-core.db': 'tampered', 'App/other/dict-thieu-chuu.db': BODIES['dict-thieu-chuu.db'] })
    const report = await inspectInstalledTree({ manifestText: manifest(), root: dest })
    expect(report.problems).toHaveLength(2)
    expect(report.problems[0]).toContain('dict-core.db: sha256')
    expect(report.problems[1]).toContain('dict-thieu-chuu.db: found 0 copies')
  })

  it('reports a .db under a dict directory that the manifest does not list', async () => {
    install({
      'App/dict/dict-core.db': BODIES['dict-core.db'],
      'App/dict/dict-thieu-chuu.db': BODIES['dict-thieu-chuu.db'],
      'App/dict/dict-tran-van-chanh.db': 'stale',
    })
    const report = await inspectInstalledTree({ manifestText: manifest(), root: dest })
    expect(report.problems).toEqual(["dict-tran-van-chanh.db: .db under a 'dict' directory but not in dict-manifest.toml"])
  })

  it('treats a [[removed]] tombstone as no entry, so its stale .db is reported', async () => {
    install({
      'App/dict/dict-core.db': BODIES['dict-core.db'],
      'App/dict/dict-thieu-chuu.db': BODIES['dict-thieu-chuu.db'],
      'App/dict/dict-gone.db': 'stale',
    })
    const text = `${manifest()}\n[[removed]]\nname   = "gone"\nreason = "r"\ndate   = "2026-10-21"\n`
    expect(manifestEntries(text).map((e) => e.fileName)).toEqual(['dict-core.db', 'dict-thieu-chuu.db'])
    const report = await inspectInstalledTree({ manifestText: text, root: dest })
    expect(report.problems).toHaveLength(1)
    expect(report.problems[0]).toContain('dict-gone.db')
  })

  it('leaves WebView2 files out of the installed total', async () => {
    install({ 'App/dict/dict-core.db': BODIES['dict-core.db'], 'App/MicrosoftEdgeWebView2Setup.exe': 'x'.repeat(1000) })
    const report = await inspectInstalledTree({ manifestText: manifest(), root: dest })
    expect(report.webview2Bytes).toBe(1000)
    expect(report.installedBytes).toBe('core bytes'.length)
  })

  it('leaves the root .msi copy of an administrative image out of the installed total, but not a nested one', async () => {
    install({ 'dict/dict-core.db': BODIES['dict-core.db'], 'AuraTranslate.msi': 'x'.repeat(700), 'App/inner.msi': 'y'.repeat(30) })
    const report = await inspectInstalledTree({ manifestText: manifest(), root: dest })
    expect(report.adminPackageBytes).toBe(700)
    expect(report.installedBytes).toBe('core bytes'.length + 30)
  })
})

describe('inspectInstalledFonts', () => {
  const FONTS: Record<string, string> = { 'A.otf': 'a', 'B[wght].ttf': 'b', 'C.ttf': 'c', 'D.ttf': 'd' }
  let sourceDir: string

  function put(base: string, files: Record<string, string>) {
    for (const [rel, body] of Object.entries(files)) {
      const full = join(base, rel)
      mkdirSync(dirname(full), { recursive: true })
      writeFileSync(full, body)
    }
  }
  const installed = (files: Record<string, string>) => Object.fromEntries(Object.entries(files).map(([n, b]) => [`App/Resources/fonts/${n}`, b]))

  beforeEach(() => {
    sourceDir = mkdtempSync(join(tmpdir(), 'font-src-'))
    put(sourceDir, { ...FONTS, 'OFL.txt': 'licence' })
  })
  afterEach(() => {
    rmSync(sourceDir, { recursive: true, force: true })
  })

  it('accepts a tree carrying every font with the source hash', async () => {
    put(dest, installed(FONTS))
    expect(await inspectInstalledFonts({ sourceDir, root: dest })).toEqual({ problems: [], fontCount: 4 })
  })

  it('names a missing font and a font with different bytes', async () => {
    const { 'C.ttf': _gone, 'D.ttf': _changed, ...rest } = FONTS
    put(dest, installed({ ...rest, 'D.ttf': 'tampered' }))
    const { problems } = await inspectInstalledFonts({ sourceDir, root: dest })
    expect(problems).toHaveLength(2)
    expect(problems.find((p) => p.startsWith('C.ttf'))).toContain('found 0 copies')
    expect(problems.find((p) => p.startsWith('D.ttf'))).toContain('does not match the source')
  })

  it('does not count a font outside a fonts directory', async () => {
    const { 'A.otf': _moved, ...rest } = FONTS
    put(dest, { ...installed(rest), 'App/Resources/other/A.otf': 'a' })
    const { problems } = await inspectInstalledFonts({ sourceDir, root: dest })
    expect(problems).toEqual([expect.stringContaining('A.otf: found 0 copies')])
  })

  it('aborts instead of passing when the source holds fewer than four fonts', async () => {
    rmSync(join(sourceDir, 'D.ttf'))
    await expect(inspectInstalledFonts({ sourceDir, root: dest })).rejects.toThrow(/infrastructure failure/)
  })
})

describe('inspectInstalledLicenses', () => {
  const LICENSES: Record<string, string> = Object.fromEntries(Array.from({ length: 9 }, (_, i) => [`L${i}.txt`, `text ${i}`]))
  let sourceDir: string

  function put(base: string, files: Record<string, string>) {
    for (const [rel, body] of Object.entries(files)) {
      const full = join(base, rel)
      mkdirSync(dirname(full), { recursive: true })
      writeFileSync(full, body)
    }
  }
  const installed = (files: Record<string, string>) => Object.fromEntries(Object.entries(files).map(([n, b]) => [`App/Resources/license/${n}`, b]))
  const expectedOf = (files: Record<string, string>) => Object.keys(files).map((name) => ({ name, sourcePath: join(sourceDir, name) }))

  beforeEach(() => {
    sourceDir = mkdtempSync(join(tmpdir(), 'license-src-'))
    put(sourceDir, LICENSES)
  })
  afterEach(() => {
    rmSync(sourceDir, { recursive: true, force: true })
  })

  it('accepts a tree carrying every license file with the source hash', async () => {
    put(dest, installed(LICENSES))
    expect(await inspectInstalledLicenses({ expected: expectedOf(LICENSES), root: dest })).toEqual({ problems: [], licenseCount: 9 })
  })

  it('names the missing license file', async () => {
    const { 'L3.txt': _gone, ...rest } = LICENSES
    put(dest, installed(rest))
    const { problems } = await inspectInstalledLicenses({ expected: expectedOf(LICENSES), root: dest })
    expect(problems).toEqual([expect.stringContaining('L3.txt: found 0 copies')])
  })

  it('names a license file whose bytes differ from the source', async () => {
    put(dest, installed({ ...LICENSES, 'L5.txt': 'text 5!' }))
    const { problems } = await inspectInstalledLicenses({ expected: expectedOf(LICENSES), root: dest })
    expect(problems).toEqual([expect.stringContaining('L5.txt: sha256')])
    expect(problems[0]).toContain('does not match the source')
  })

  it('names a license file that appears twice under license directories', async () => {
    put(dest, { ...installed(LICENSES), 'App/Resources/extra/license/L2.txt': 'text 2' })
    const { problems } = await inspectInstalledLicenses({ expected: expectedOf(LICENSES), root: dest })
    expect(problems).toEqual([expect.stringContaining('L2.txt: found 2 copies')])
  })

  it('does not count a license file outside a license directory', async () => {
    const { 'L1.txt': _moved, ...rest } = LICENSES
    put(dest, { ...installed(rest), 'App/Resources/other/L1.txt': 'text 1' })
    const { problems } = await inspectInstalledLicenses({ expected: expectedOf(LICENSES), root: dest })
    expect(problems).toEqual([expect.stringContaining('L1.txt: found 0 copies')])
  })

  it('aborts instead of passing when fewer than nine sources are expected', async () => {
    const { 'L8.txt': _dropped, ...rest } = LICENSES
    await expect(inspectInstalledLicenses({ expected: expectedOf(rest), root: dest })).rejects.toThrow(/infrastructure failure/)
  })
})
