import { spawnSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

const BENCH = join(__dirname, '..', '..', '_bmad-output', 'implementation-artifacts', '6-18-ban-do')
const RAW = ['nfr3-raw.tsv', 'startup-raw.tsv', 'memory-raw.tsv']

let dir: string
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), 'nfr-bench-'))
  copyFileSync(join(BENCH, 'summarize.mjs'), join(dir, 'summarize.mjs'))
  for (const name of RAW) copyFileSync(join(BENCH, name), join(dir, name))
})
afterEach(() => {
  rmSync(dir, { recursive: true, force: true })
})

const summarize = () => spawnSync(process.execPath, [join(dir, 'summarize.mjs')], { encoding: 'utf8' })

describe('6-18-ban-do summarize.mjs', () => {
  it('writes REPORT.md from the full recorded matrix', () => {
    const r = summarize()
    expect(r.status).toBe(0)
    expect(readFileSync(join(dir, 'REPORT.md'), 'utf8')).toContain('| NFR4 |')
  })

  it('refuses and writes no REPORT.md when one of the 30 launches is missing', () => {
    const lines = readFileSync(join(dir, 'startup-raw.tsv'), 'utf8').trimEnd().split('\n')
    writeFileSync(join(dir, 'startup-raw.tsv'), `${lines.slice(0, -1).join('\n')}\n`)
    const r = summarize()
    expect(r.status).not.toBe(0)
    expect(r.stderr).toContain('NFR4 cần đúng 30 lần launch ok')
    expect(existsSync(join(dir, 'REPORT.md'))).toBe(false)
  })
})
