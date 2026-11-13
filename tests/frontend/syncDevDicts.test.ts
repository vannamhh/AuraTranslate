import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, utimesSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { devDictDir, syncDevDicts } from '../../scripts/sync-dev-dicts.mjs'

let root: string
let source: string
let dest: string

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), 'sync-dev-dicts-'))
  source = join(root, 'resources', 'dict')
  dest = join(root, 'target', 'debug', 'dict')
  mkdirSync(source, { recursive: true })
})

afterEach(() => {
  rmSync(root, { recursive: true, force: true })
})

describe('syncDevDicts', () => {
  it('copies every .db into a missing destination and ignores other files', () => {
    writeFileSync(join(source, 'dict-core.db'), 'core')
    writeFileSync(join(source, 'dict-vietphrase.db'), 'vp')
    writeFileSync(join(source, '.gitkeep'), '')
    writeFileSync(join(source, 'README.md'), 'readme')

    const results = syncDevDicts(source, dest)

    expect(results).toEqual([
      { name: 'dict-core.db', status: 'copied' },
      { name: 'dict-vietphrase.db', status: 'copied' },
    ])
    expect(readdirSync(dest).sort()).toEqual(['dict-core.db', 'dict-vietphrase.db'])
    expect(readFileSync(join(dest, 'dict-core.db'), 'utf8')).toBe('core')
  })

  it('leaves a destination file alone when size and mtime already match', () => {
    writeFileSync(join(source, 'dict-core.db'), 'core')
    syncDevDicts(source, dest)
    const marker = new Date('2020-01-01T00:00:00Z')
    utimesSync(join(dest, 'dict-core.db'), marker, statSync(join(dest, 'dict-core.db')).mtime)

    const results = syncDevDicts(source, dest)

    expect(results).toEqual([{ name: 'dict-core.db', status: 'up-to-date' }])
    expect(statSync(join(dest, 'dict-core.db')).atime.getTime()).toBe(marker.getTime())
  })

  it('re-copies a dictionary rebuilt in the source', () => {
    writeFileSync(join(source, 'dict-core.db'), 'old!')
    syncDevDicts(source, dest)
    writeFileSync(join(source, 'dict-core.db'), 'new!')
    const later = new Date(statSync(join(source, 'dict-core.db')).mtimeMs + 5000)
    utimesSync(join(source, 'dict-core.db'), later, later)

    const results = syncDevDicts(source, dest)

    expect(results).toEqual([{ name: 'dict-core.db', status: 'copied' }])
    expect(readFileSync(join(dest, 'dict-core.db'), 'utf8')).toBe('new!')
    expect(existsSync(join(dest, 'dict-core.db.partial'))).toBe(false)
  })

  it('does nothing when the source holds no .db, keeping a fixture already in the destination', () => {
    writeFileSync(join(source, '.gitkeep'), '')
    mkdirSync(dest, { recursive: true })
    writeFileSync(join(dest, 'dict-core.db'), 'ci fixture')

    const results = syncDevDicts(source, dest)

    expect(results).toEqual([])
    expect(readFileSync(join(dest, 'dict-core.db'), 'utf8')).toBe('ci fixture')
  })

  it('creates no destination directory when the source holds no .db', () => {
    syncDevDicts(source, dest)

    expect(existsSync(dest)).toBe(false)
  })

  it('keeps destination files that have no counterpart in the source', () => {
    writeFileSync(join(source, 'dict-core.db'), 'core')
    mkdirSync(dest, { recursive: true })
    writeFileSync(join(dest, 'dict-fixture-only.db'), 'fixture')

    syncDevDicts(source, dest)

    expect(readdirSync(dest).sort()).toEqual(['dict-core.db', 'dict-fixture-only.db'])
  })
})

describe('devDictDir', () => {
  it('points at src-tauri/target/debug/dict by default', () => {
    expect(devDictDir({})).toMatch(/src-tauri[\\/]target[\\/]debug[\\/]dict$/)
  })

  it('follows an absolute CARGO_TARGET_DIR', () => {
    expect(devDictDir({ CARGO_TARGET_DIR: join(root, 'shared-target') })).toBe(join(root, 'shared-target', 'debug', 'dict'))
  })
})
