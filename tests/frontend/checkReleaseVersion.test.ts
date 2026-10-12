import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { releaseVersionProblems } from '../../scripts/check-release-version.mjs'

let root: string

function write(version: { pkg: string; cargo: string; tauri: string }) {
  mkdirSync(join(root, 'src-tauri'), { recursive: true })
  writeFileSync(join(root, 'package.json'), JSON.stringify({ name: 'x', version: version.pkg }))
  writeFileSync(join(root, 'src-tauri/tauri.conf.json'), JSON.stringify({ version: version.tauri }))
  writeFileSync(join(root, 'src-tauri/Cargo.toml'), `[package]\nname = "x"\nversion = "${version.cargo}"\n\n[dependencies]\nserde = { version = "9.9.9" }\n`)
}

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), 'release-version-'))
})
afterEach(() => {
  rmSync(root, { recursive: true, force: true })
})

describe('releaseVersionProblems', () => {
  it('accepts a tag equal to all three versions', () => {
    write({ pkg: '0.1.0', cargo: '0.1.0', tauri: '0.1.0' })
    expect(releaseVersionProblems('v0.1.0', root)).toEqual([])
  })

  it('names each file whose version differs, with both values', () => {
    write({ pkg: '0.1.0', cargo: '0.2.0', tauri: '0.1.0' })
    const problems = releaseVersionProblems('v0.2.0', root)
    expect(problems).toHaveLength(2)
    expect(problems[0]).toContain("package.json: version is '0.1.0'")
    expect(problems[0]).toContain("expects '0.2.0'")
    expect(problems[1]).toContain('src-tauri/tauri.conf.json')
  })

  it.each(['0.1.0', 'v0.1', 'v1.2.3.4', 'dict-v2', 'vlatest', 'v0.2.0-rc1', 'v1.0.0+b', ''])('rejects the tag %j', (tag) => {
    write({ pkg: '0.1.0', cargo: '0.1.0', tauri: '0.1.0' })
    expect(releaseVersionProblems(tag, root)).toHaveLength(1)
  })

  it('exits non-zero from the command line on a mismatch', () => {
    write({ pkg: '0.1.0', cargo: '0.1.0', tauri: '0.1.1' })
    const bad = spawnSync(process.execPath, ['scripts/check-release-version.mjs', 'v0.1.0', '--root', root], { encoding: 'utf8' })
    expect(bad.status).toBe(1)
    expect(bad.stderr).toContain('src-tauri/tauri.conf.json')
    const good = spawnSync(process.execPath, ['scripts/check-release-version.mjs', 'v0.1.0'], { encoding: 'utf8' })
    expect(good.status).toBe(0)
  })
})
