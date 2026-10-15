/** Wire guards for the three strips' new fields: counts by side, block counters, TM suggestions. */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {
    onmessage: (v: unknown) => void = () => {}
  },
}))

const { tmCountPairs } = await import('../../src/config/tm')
const { isProofreadOutcomeWire, listProofreadIgnores, recordProofreadScan } = await import('../../src/config/proofread')
const { listGlossaryTmSuggestions } = await import('../../src/config/glossary')

beforeEach(() => invokeMock.mockReset())

describe('tmCountPairs', () => {
  it('reads total, mine and others', async () => {
    invokeMock.mockResolvedValue({ total: 12409, mine: 11882, others: 527 })
    expect(await tmCountPairs()).toEqual({ total: 12409, mine: 11882, others: 527 })
  })

  it('is unknown, not zero, for a bare number or a partial object', async () => {
    invokeMock.mockResolvedValue(5)
    expect(await tmCountPairs()).toBeNull()
    invokeMock.mockResolvedValue({ total: 5, mine: 5 })
    expect(await tmCountPairs()).toBeNull()
  })
})

describe('proofread wire', () => {
  it('a done outcome must carry the blocked list', () => {
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: 0, blocked: [{ ignore_id: 1, count: 2 }] })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: 0 })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: 0, blocked: [{ ignore_id: 1 }] })).toBe(false)
  })

  it('ignore rows accept a NULL counter and reject a missing one', async () => {
    const row = { id: 1, kind: 'spelling', phrase: 'a', context_before: '', context_after: '', created_at: 't' }
    invokeMock.mockResolvedValue([{ ...row, blocked_count: null }])
    expect((await listProofreadIgnores()).rows).toHaveLength(1)
    invokeMock.mockResolvedValue([row])
    expect((await listProofreadIgnores()).rows).toBeNull()
  })

  it('the scan record sends the blocked array', async () => {
    invokeMock.mockResolvedValue(null)
    await recordProofreadScan('w', 'chapter', 3, 2, [{ ignore_id: 9, count: 2 }])
    expect(invokeMock).toHaveBeenCalledWith('ai_proofread_scan_record', {
      workId: 'w',
      scope: 'chapter',
      reported: 3,
      filtered: 2,
      blocked: [{ ignore_id: 9, count: 2 }],
    })
  })
})

describe('listGlossaryTmSuggestions', () => {
  it('sends sourceTerm in camelCase and returns the grouped list', async () => {
    invokeMock.mockResolvedValue([{ target_text: 'X', count: 2 }])
    const result = await listGlossaryTmSuggestions('T')
    expect(invokeMock).toHaveBeenCalledWith('glossary_tm_suggestions', { sourceTerm: 'T' })
    expect(result).toEqual({ value: [{ target_text: 'X', count: 2 }], error: null })
  })

  it('rejects a malformed list instead of showing it', async () => {
    invokeMock.mockResolvedValue([{ target_text: 'X' }])
    expect((await listGlossaryTmSuggestions('T')).error).not.toBeNull()
  })
})
