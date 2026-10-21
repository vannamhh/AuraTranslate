import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { copyFileSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { decodeMsiStreamName, msiStreamSizes } from '../../scripts/lib/msi-streams.mjs'

const REPO = join(__dirname, '..', '..')
const WEBVIEW2 = 'Binary.MicrosoftEdgeWebView2RuntimeInstaller.exe'

function msiChars(name: string) {
  const v = (ch: string) => '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz._'.indexOf(ch)
  let out = ''
  for (let i = 0; i < name.length; i += 2) {
    out += i + 1 < name.length
      ? String.fromCharCode(0x3800 + v(name[i]) + (v(name[i + 1]) << 6))
      : String.fromCharCode(0x4800 + v(name[i]))
  }
  return out
}

const FREE = 0xffffffff
const END = 0xfffffffe

/**
 * A Compound File whose FAT sectors sit at 0..fatSectors-1, an extra DIFAT sector right after them
 * when there are more than 109, and the directory chain at `dirSectors`; stream data is never read.
 */
function compoundFile(
  streams: Array<{ raw: string, size: number }>,
  { major = 3, fatSectors = 1, dirSectors = [1] }: { major?: 3 | 4, fatSectors?: number, dirSectors?: number[] } = {},
): Buffer {
  const ssz = major === 4 ? 4096 : 512
  const per = ssz / 4
  const difatSector = fatSectors > 109 ? fatSectors : null
  const last = Math.max(...dirSectors, difatSector ?? 0, fatSectors - 1)
  const file = Buffer.alloc((last + 2) * ssz, 0)
  const at = (n: number) => (n + 1) * ssz
  file.write('d0cf11e0a1b11ae1', 0, 'hex')
  file.writeUInt16LE(0x3e, 0x18)
  file.writeUInt16LE(major, 0x1a)
  file.writeUInt16LE(0xfffe, 0x1c)
  file.writeUInt16LE(major === 4 ? 12 : 9, 0x1e)
  file.writeUInt16LE(6, 0x20)
  file.writeUInt32LE(fatSectors, 0x2c)
  file.writeUInt32LE(dirSectors[0], 0x30)
  file.writeUInt32LE(4096, 0x38)
  file.writeUInt32LE(END, 0x3c)
  file.writeUInt32LE(difatSector ?? END, 0x44)
  file.writeUInt32LE(difatSector === null ? 0 : 1, 0x48)
  for (let i = 0; i < 109; i++) file.writeUInt32LE(i < fatSectors ? i : FREE, 0x4c + i * 4)
  if (difatSector !== null) {
    for (let i = 0; i < per; i++) file.writeUInt32LE(FREE, at(difatSector) + i * 4)
    for (let i = 109; i < fatSectors; i++) file.writeUInt32LE(i, at(difatSector) + (i - 109) * 4)
    file.writeUInt32LE(END, at(difatSector) + (per - 1) * 4)
  }
  const fat = new Array<number>(fatSectors * per).fill(FREE)
  for (let i = 0; i < fatSectors; i++) fat[i] = 0xfffffffd
  if (difatSector !== null) fat[difatSector] = 0xfffffffc
  dirSectors.forEach((s, i) => { fat[s] = dirSectors[i + 1] ?? END })
  fat.forEach((v, i) => file.writeUInt32LE(v, at(Math.floor(i / per)) + (i % per) * 4))
  const entries = [{ raw: 'Root Entry', size: 0, type: 5 }, ...streams.map((s) => ({ ...s, type: 2 }))]
  const slots = ssz / 128
  entries.forEach((e, i) => {
    const off = at(dirSectors[Math.floor(i / slots)]) + (i % slots) * 128
    file.write(e.raw, off, 'utf16le')
    file.writeUInt16LE((e.raw.length + 1) * 2, off + 64)
    file[off + 66] = e.type
    file.writeUInt32LE(e.size % 2 ** 32, off + 120)
    file.writeUInt32LE(Math.floor(e.size / 2 ** 32), off + 124)
  })
  return file
}

/** The directory name of the WebView2 stream as stored in the release `.msi` of run 39576340751. */
const REAL_WEBVIEW2_RAW = '䌋䄱䜵㶾䆬䒵䒶䗩䇎䈪䈠㿥䈬㢺䘛䗱䐬㲨䖱䄷䏯䕨䈾䈻'

let dir: string
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), 'msi-streams-'))
})
afterEach(() => {
  rmSync(dir, { recursive: true, force: true })
})

describe('msiStreamSizes', () => {
  it('decodes packed MSI names and reads each stream size from the directory', () => {
    const path = join(dir, 'a.msi')
    writeFileSync(path, compoundFile([{ raw: msiChars(WEBVIEW2), size: 214_156_024 }, { raw: msiChars('app.cab'), size: 7 }]))
    expect(msiStreamSizes(path)).toEqual([
      { name: WEBVIEW2, size: 214_156_024 },
      { name: 'app.cab', size: 7 },
    ])
  })

  it('decodes the stream name exactly as a real release .msi stores it', () => {
    expect(decodeMsiStreamName(REAL_WEBVIEW2_RAW)).toBe(WEBVIEW2)
  })

  it('reads a version-4 file with 4096-byte sectors, a two-sector directory and a size above 4 GiB', () => {
    const path = join(dir, 'v4.msi')
    const filler = Array.from({ length: 40 }, (_, i) => ({ raw: msiChars(`s${i}`), size: i }))
    writeFileSync(path, compoundFile([...filler, { raw: REAL_WEBVIEW2_RAW, size: 2 ** 32 + 5 }], { major: 4, dirSectors: [1, 2] }))
    const streams = msiStreamSizes(path)
    expect(streams).toHaveLength(41)
    expect(streams.at(-1)).toEqual({ name: WEBVIEW2, size: 2 ** 32 + 5 })
  })

  it('follows the DIFAT chain when the FAT needs more than 109 sectors', () => {
    const path = join(dir, 'difat.msi')
    const deep = 109 * 128
    const filler = ['a', 'b', 'c', 'd'].map((n) => ({ raw: msiChars(n), size: 1 }))
    writeFileSync(path, compoundFile([...filler, { raw: REAL_WEBVIEW2_RAW, size: 214_156_024 }], { fatSectors: 110, dirSectors: [deep, deep + 1] }))
    expect(msiStreamSizes(path).at(-1)).toEqual({ name: WEBVIEW2, size: 214_156_024 })
  })

  it('decodes a table stream marker', () => {
    expect(decodeMsiStreamName(`䡀${msiChars('_Tables')}`)).toBe('!_Tables')
  })

  it('refuses a file that is not a Compound File', () => {
    const path = join(dir, 'b.msi')
    writeFileSync(path, Buffer.alloc(1024, 0x41))
    expect(() => msiStreamSizes(path)).toThrow(/not a Compound File/)
  })
})

describe('inspect-release on an msiexec /a target', () => {
  function stageTree(root: string) {
    const fonts = join(REPO, 'src-tauri', 'resources', 'fonts')
    mkdirSync(join(root, 'fonts'), { recursive: true })
    for (const name of readdirSync(fonts).filter((n) => /\.(otf|ttf)$/.test(n))) copyFileSync(join(fonts, name), join(root, 'fonts', name))
    mkdirSync(join(root, 'license'), { recursive: true })
    const licenseSources = [join(REPO, 'src-tauri', 'resources', 'license'), join(REPO, 'tools', 'dict-build', 'assets', 'licenses')]
    for (const src of licenseSources) {
      for (const name of readdirSync(src).filter((n) => n.endsWith('.txt'))) copyFileSync(join(src, name), join(root, 'license', name))
    }
    copyFileSync(join(REPO, 'NOTICE.txt'), join(root, 'license', 'NOTICE.txt'))
    mkdirSync(join(root, 'dict'), { recursive: true })
    writeFileSync(join(root, 'dict', 'dict-core.db'), 'core bytes')
  }

  it('reports the Binary-table WebView2 and leaves the administrative package copy out of the payload', () => {
    const root = join(dir, 'target')
    stageTree(root)
    const before = readdirSync(root, { recursive: true, withFileTypes: true })
      .filter((e) => e.isFile())
      .reduce((sum, e) => sum + readFileSync(join(e.parentPath, e.name)).length, 0)
    const installer = join(dir, 'AuraTranslate_0.1.0_x64_en-US.msi')
    writeFileSync(installer, compoundFile([{ raw: msiChars(WEBVIEW2), size: 214_156_024 }, { raw: msiChars('app.cab'), size: 9 }]))
    writeFileSync(join(root, 'AuraTranslate_0.1.0_x64_en-US.msi'), Buffer.alloc(5000, 1))
    const manifest = join(dir, 'dict-manifest.toml')
    const sha = createHash('sha256').update('core bytes').digest('hex')
    writeFileSync(manifest, `[base]\nurl            = "https://example.test/dict-core.db"\nsha256         = "${sha}"\nsource_version = "x"\n`)
    const json = join(dir, 'sizes.json')
    const r = spawnSync(process.execPath, [join(REPO, 'scripts', 'inspect-release.mjs'), '--root', root, '--installer', installer, '--manifest', manifest, '--json', json], { encoding: 'utf8' })
    expect(r.stderr).not.toContain('problem')
    expect(r.status).toBe(0)
    expect(JSON.parse(readFileSync(json, 'utf8'))).toEqual({
      os: 'Windows',
      installerBytes: readFileSync(installer).length,
      payloadBytes: before,
      webview2Bytes: 214_156_024,
    })
    expect(r.stdout).toContain('| `.msi` copy left by `msiexec /a`, excluded | 5,000 |')
  })
})
