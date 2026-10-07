/** IPC adapters and the bootstrap field of the fuzzy TM strip, through the real `invoke` boundary. */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

beforeEach(() => {
  vi.resetModules()
  mockInvoke.mockReset()
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
})

const WIRE_MATCH = {
  tier: 'global',
  unit_id: 4,
  percent: 71,
  source_text: 'a b',
  target_text: 'x',
  diff: [{ kind: 'equal', text: 'a b' }],
  side: 'others',
}

describe('tmFuzzyMatches', () => {
  it('sends segmentId in camelCase and returns the typed outcome', async () => {
    mockInvoke.mockResolvedValueOnce({ segment_id: 9, matches: [WIRE_MATCH], exact: [] })
    const { tmFuzzyMatches } = await import('../../src/config/segment')
    const result = await tmFuzzyMatches(9)
    expect(mockInvoke).toHaveBeenCalledWith('tm_fuzzy_matches', { segmentId: 9 })
    expect(result.error).toBeNull()
    expect(result.outcome?.matches[0]?.unit_id).toBe(4)
  })

  it('a malformed payload is an error, never an empty list', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    mockInvoke.mockResolvedValueOnce({ segment_id: 9, matches: [{ ...WIRE_MATCH, side: 'both' }], exact: [] })
    const { tmFuzzyMatches } = await import('../../src/config/segment')
    const result = await tmFuzzyMatches(9)
    expect(result.outcome).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('an IpcError is passed through', async () => {
    const err = { code: 'segment.not_found', message_key: 'err.unknown', params: {}, retryable: false }
    mockInvoke.mockRejectedValueOnce(err)
    const { tmFuzzyMatches } = await import('../../src/config/segment')
    expect(await tmFuzzyMatches(9)).toEqual({ outcome: null, error: err })
  })
})

describe('acceptTmFuzzy', () => {
  it('sends the pair identity, the text shown and force', async () => {
    mockInvoke.mockResolvedValueOnce({
      segment_id: 9,
      target_text: 'x',
      translation_origin: 'other',
      status: 'draft',
      needs_confirmation: false,
      unsigned_draft: null,
    })
    const { acceptTmFuzzy } = await import('../../src/config/segment')
    const result = await acceptTmFuzzy(9, 'global', 4, 'Hắn đẩy cửa ra.', true)
    expect(mockInvoke).toHaveBeenCalledWith('accept_tm_fuzzy', { segmentId: 9, tier: 'global', unitId: 4, expectedTarget: 'Hắn đẩy cửa ra.', force: true })
    expect(result.outcome?.status).toBe('draft')
  })
})

describe('bootstrap tm_fuzzy_threshold', () => {
  function payload(value: unknown): Record<string, unknown> {
    const base: Record<string, unknown> = { theme: 'light', mode: 'library', shortcuts: {}, layout_presets: {} }
    if (value !== undefined) base.tm_fuzzy_threshold = value
    return base
  }

  it.each([
    [80, 80],
    [undefined, 65],
    ['80', 65],
    [49, 65],
    [100, 65],
    [70.5, 65],
  ])('%j ⇒ %j', async (wire, expected) => {
    mockInvoke.mockResolvedValueOnce(payload(wire))
    const { loadBootstrapConfig, bootstrapTmFuzzyThreshold } = await import('../../src/config/bootstrap')
    await loadBootstrapConfig()
    expect(bootstrapTmFuzzyThreshold.value).toBe(expected)
  })
})

describe('Story 7.8 wire', () => {
  const WIRE_EXACT = { tier: 'work', unit_id: 3, target_text: 'x', side: 'mine', created_at: '2026-08-03T00:00:00.000Z' }

  it('an old shape without exact is a wire mismatch, not an empty list', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    mockInvoke.mockResolvedValueOnce({ segment_id: 9, matches: [] })
    const { tmFuzzyMatches } = await import('../../src/config/segment')
    const result = await tmFuzzyMatches(9)
    expect(result.outcome).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('exact targets are typed and an exact row without created_at is rejected', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const { tmFuzzyMatches } = await import('../../src/config/segment')
    mockInvoke.mockResolvedValueOnce({ segment_id: 9, matches: [], exact: [WIRE_EXACT] })
    expect((await tmFuzzyMatches(9)).outcome?.exact[0]?.unit_id).toBe(3)
    mockInvoke.mockResolvedValueOnce({ segment_id: 9, matches: [], exact: [{ ...WIRE_EXACT, created_at: undefined }] })
    expect((await tmFuzzyMatches(9)).outcome).toBeNull()
  })

  it('acceptTmExact sends the pair identity and force to accept_tm_exact', async () => {
    mockInvoke.mockResolvedValueOnce({
      segment_id: 9, target_text: 'x', translation_origin: 'self', status: 'draft', needs_confirmation: false, unsigned_draft: null,
    })
    const { acceptTmExact } = await import('../../src/config/segment')
    const result = await acceptTmExact(9, 'work', 3, 'Bản mới', false)
    expect(mockInvoke).toHaveBeenCalledWith('accept_tm_exact', { segmentId: 9, tier: 'work', unitId: 3, expectedTarget: 'Bản mới', force: false })
    expect(result.outcome?.translation_origin).toBe('self')
  })

  it('a concordance hit without created_at is rejected', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const hit = { tier: 'work', unit_id: 1, source_text: 'a', target_text: 'b', side: 'mine' }
    mockInvoke.mockResolvedValueOnce({ query: 'a', tm_empty: false, total: 1, hits: [hit] })
    const { tmConcordance } = await import('../../src/config/segment')
    expect((await tmConcordance('a')).outcome).toBeNull()
  })
})
