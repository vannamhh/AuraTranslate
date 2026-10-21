/**
 * Unit tests for `scripts/lib/detachable-all.mjs`: the layer partition behind
 * `check:dict-manifest` and Kiểm D of `check:dict`.
 */
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { checkLayerPartition, readDetachableAllCodes } from '../../scripts/lib/detachable-all.mjs'

const CODES = ['thieu-chuu', 'vietphrase', 'tran-van-chanh']
const det = (name: string) => ({ name, url: `https://example.test/dict-${name}.db`, sha256: 'x', source_version: 'v' })
const tomb = (name: string, extra: Record<string, string> = {}) => ({ name, reason: 'r', date: '2026-10-21', ...extra })

describe('readDetachableAllCodes', () => {
  it('reads the three layer codes from the real sources_meta.rs', () => {
    const text = readFileSync(join(__dirname, '..', '..', 'tools', 'dict-build', 'src', 'sources_meta.rs'), 'utf8')
    const { codes, problems } = readDetachableAllCodes(text)
    expect(problems).toEqual([])
    expect(codes).toEqual(CODES)
  })

  it('reports a source without DETACHABLE_ALL', () => {
    expect(readDetachableAllCodes('fn main() {}').problems).toHaveLength(1)
  })
})

describe('checkLayerPartition', () => {
  it('accepts every layer detachable', () => {
    expect(checkLayerPartition({ codes: CODES, detachable: CODES.map(det), removed: [] })).toEqual([])
  })

  it('accepts one layer moved to [[removed]]', () => {
    const detachable = ['thieu-chuu', 'vietphrase'].map(det)
    expect(checkLayerPartition({ codes: CODES, detachable, removed: [tomb('tran-van-chanh')] })).toEqual([])
  })

  it('names a layer that is in neither array', () => {
    const problems = checkLayerPartition({ codes: CODES, detachable: ['thieu-chuu', 'vietphrase'].map(det), removed: [] })
    expect(problems).toHaveLength(1)
    expect(problems[0]).toContain('tran-van-chanh')
  })

  it('names a layer written in both arrays', () => {
    const problems = checkLayerPartition({ codes: CODES, detachable: CODES.map(det), removed: [tomb('vietphrase')] })
    expect(problems).toHaveLength(1)
    expect(problems[0]).toContain('vietphrase')
  })

  it.each(['reason', 'date'])('names the row and field when [[removed]] lacks %s', (field) => {
    const row: Record<string, string> = tomb('tran-van-chanh')
    delete row[field]
    const problems = checkLayerPartition({ codes: CODES, detachable: ['thieu-chuu', 'vietphrase'].map(det), removed: [row] })
    expect(problems).toEqual([`[[removed]] #1: thiếu hoặc rỗng '${field}'`])
  })

  it.each(['url', 'sha256'])('names the row and field when [[removed]] carries %s', (field) => {
    const problems = checkLayerPartition({
      codes: CODES,
      detachable: ['thieu-chuu', 'vietphrase'].map(det),
      removed: [tomb('tran-van-chanh', { [field]: 'x' })],
    })
    expect(problems).toHaveLength(1)
    expect(problems[0]).toContain('[[removed]] #1')
    expect(problems[0]).toContain(`'${field}'`)
  })

  it('names a [[removed]] row whose name is not in DETACHABLE_ALL', () => {
    const problems = checkLayerPartition({ codes: CODES, detachable: CODES.map(det), removed: [tomb('zzz')] })
    expect(problems).toHaveLength(1)
    expect(problems[0]).toContain('[[removed]] #1')
    expect(problems[0]).toContain('zzz')
  })
})
