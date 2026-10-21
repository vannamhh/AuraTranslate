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

/** A version-3 Compound File with one FAT sector and one directory sector; stream data is never read. */
function compoundFile(streams: Array<{ raw: string, size: number }>): Buffer {
  const header = Buffer.alloc(512, 0)
  Buffer.from('d0cf11e0a1b11ae1', 'hex').copy(header, 0)
  header.writeUInt16LE(0x3e, 0x18)
  header.writeUInt16LE(3, 0x1a)
  header.writeUInt16LE(0xfffe, 0x1c)
  header.writeUInt16LE(9, 0x1e)
  header.writeUInt16LE(6, 0x20)
  header.writeUInt32LE(1, 0x2c)
  header.writeUInt32LE(1, 0x30)
  header.writeUInt32LE(4096, 0x38)
  header.writeUInt32LE(0xfffffffe, 0x3c)
  header.writeUInt32LE(0xfffffffe, 0x44)
  for (let off = 0x4c; off < 512; off += 4) header.writeUInt32LE(0xffffffff, off)
  header.writeUInt32LE(0, 0x4c)
  const fat = Buffer.alloc(512, 0xff)
  fat.writeUInt32LE(0xfffffffd, 0)
  fat.writeUInt32LE(0xfffffffe, 4)
  const dir = Buffer.alloc(512, 0)
  const entries = [{ raw: 'Root Entry', size: 0, type: 5 }, ...streams.map((s) => ({ ...s, type: 2 }))]
  entries.forEach((e, i) => {
    const off = i * 128
    dir.write(e.raw, off, 'utf16le')
    dir.writeUInt16LE((e.raw.length + 1) * 2, off + 64)
    dir[off + 66] = e.type
    dir.writeUInt32LE(e.size, off + 120)
  })
  return Buffer.concat([header, fat, dir])
}

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
