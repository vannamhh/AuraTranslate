import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

const SCRIPT = join(__dirname, '../../scripts/release-checksums.mjs')
const NOTES = '## AuraTranslate\n\n### Bản cài này không được ký số\n'
let dir: string

function run(args: string[]) {
  return spawnSync(process.execPath, [SCRIPT, ...args], { encoding: 'utf8' })
}
function file(name: string, content: string) {
  const path = join(dir, name)
  writeFileSync(path, content)
  return path
}

beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), 'release-checksums-'))
  file('notes.md', NOTES)
})
afterEach(() => {
  rmSync(dir, { recursive: true, force: true })
})

describe('release-checksums', () => {
  it('writes SHA256SUMS.txt and a body with both hashes and both verify commands', () => {
    const dmg = file('x.dmg', 'abc')
    const msi = file('y.msi', 'hello')
    const out = join(dir, 'out')
    const r = run(['--notes', join(dir, 'notes.md'), '--out-dir', out, dmg, msi])
    expect(r.status).toBe(0)
    const sums = readFileSync(join(out, 'SHA256SUMS.txt'), 'utf8').trimEnd().split('\n')
    expect(sums).toHaveLength(2)
    for (const line of sums) expect(line).toMatch(/^[0-9a-f]{64}  [^/\\]+$/)
    const abc = createHash('sha256').update('abc').digest('hex')
    expect(abc.startsWith('ba7816bf')).toBe(true)
    expect(sums[0]).toBe(`${abc}  x.dmg`)
    expect(sums[1]).toBe(`${createHash('sha256').update('hello').digest('hex')}  y.msi`)
    const body = readFileSync(join(out, 'release-body.md'), 'utf8')
    expect(body.startsWith(NOTES)).toBe(true)
    expect(body).toContain('### Kiểm tra tệp tải về (SHA-256)')
    for (const line of sums) {
      const [hash, name] = line.split('  ')
      expect(body).toContain(hash)
      expect(body).toContain(name)
    }
    expect(body).toContain('shasum -a 256 x.dmg')
    expect(body).toContain('shasum -a 256 -c --ignore-missing SHA256SUMS.txt')
    expect(body).toContain('Get-FileHash .\\y.msi -Algorithm SHA256')
  })

  it('fails on a missing file and writes nothing', () => {
    const out = join(dir, 'out')
    const r = run(['--notes', join(dir, 'notes.md'), '--out-dir', out, join(dir, 'nope.dmg')])
    expect(r.status).not.toBe(0)
    expect(r.stderr).toContain('nope.dmg')
    expect(existsSync(out)).toBe(false)
  })

  it('fails with usage when no file is given', () => {
    const out = join(dir, 'out')
    const r = run(['--notes', join(dir, 'notes.md'), '--out-dir', out])
    expect(r.status).not.toBe(0)
    expect(r.stderr).toContain('usage')
    expect(existsSync(out)).toBe(false)
  })

  it('fails on two paths with the same basename', () => {
    mkdirSync(join(dir, 'one'))
    mkdirSync(join(dir, 'two'))
    const a = file('one/a.dmg', '1')
    const b = file('two/a.dmg', '2')
    const out = join(dir, 'out')
    const r = run(['--notes', join(dir, 'notes.md'), '--out-dir', out, a, b])
    expect(r.status).not.toBe(0)
    expect(r.stderr).toContain('a.dmg')
    expect(existsSync(out)).toBe(false)
  })

  it('fails on a missing notes file and writes nothing', () => {
    const a = file('a.dmg', '1')
    const out = join(dir, 'out')
    const r = run(['--notes', join(dir, 'missing.md'), '--out-dir', out, a])
    expect(r.status).not.toBe(0)
    expect(r.stderr).toContain('missing.md')
    expect(existsSync(out)).toBe(false)
  })
})
