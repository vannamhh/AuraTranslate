import { describe, expect, it } from 'vitest'
import { pinIsStale } from '../../src/panels/lookupHistoryState'

const sources = [{ code: 'tvc', source_version: '2' }]

describe('pinIsStale', () => {
  it('khác phiên bản hiện tại ⇒ cũ', () => {
    expect(pinIsStale({ source_code: 'tvc', source_version: '1' }, sources)).toBe(true)
  })
  it('cùng phiên bản ⇒ không cũ', () => {
    expect(pinIsStale({ source_code: 'tvc', source_version: '2' }, sources)).toBe(false)
  })
  it('ghim cũ NULL ⇒ không cảnh báo', () => {
    expect(pinIsStale({ source_code: 'tvc', source_version: null }, sources)).toBe(false)
  })
  it('nguồn chưa nạp ⇒ không cảnh báo', () => {
    expect(pinIsStale({ source_code: 'zzz', source_version: '1' }, sources)).toBe(false)
  })
})
