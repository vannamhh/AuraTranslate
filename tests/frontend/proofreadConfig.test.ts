import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {
    onmessage: (v: unknown) => void = () => {}
  },
}))

const {
  addProofreadIgnore,
  cancelProofreadCall,
  clearProofreadIgnores,
  isProofreadEventWire,
  isProofreadOutcomeWire,
  listProofreadIgnores,
  recordProofreadScan,
  removeProofreadIgnore,
  runProofreadSegments,
} = await import('../../src/config/proofread')

const FINDING = { kind: 'grammar', start: 0, end: 3, explanation: 'e', suggestion: 's' }

beforeEach(() => {
  invokeMock.mockReset()
})

const SEGMENT = { segment_id: 3, scanned_text: 'a', findings: [FINDING], unlocated: 0, ambiguous: 0 }

describe('isProofreadOutcomeWire', () => {
  it('accepts the three outcomes', () => {
    expect(isProofreadOutcomeWire({ state: 'not_configured' })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'cancelled', not_scanned: 2 })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: 0 })).toBe(true)
  })

  it('rejects a done or cancelled outcome without not_scanned, and unknown states', () => {
    expect(isProofreadOutcomeWire({ state: 'done' })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0 })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: -1 })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: 1.5 })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0, filtered: '2' })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'cancelled' })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'generating' })).toBe(false)
  })
})

describe('isProofreadEventWire', () => {
  it('accepts planned and batch events, with or without usage', () => {
    expect(isProofreadEventWire({ kind: 'planned', segments: 4, batches: 1 })).toBe(true)
    expect(isProofreadEventWire({ kind: 'batch', usage: null, segments: [SEGMENT] })).toBe(true)
    const usage = { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2, cost_usd: null }
    expect(isProofreadEventWire({ kind: 'batch', usage, segments: [] })).toBe(true)
  })

  it('accepts every closed kind including the three meaning kinds', () => {
    for (const kind of ['spelling', 'grammar', 'mistranslation', 'too_loose', 'obscure']) {
      const event = { kind: 'batch', usage: null, segments: [{ ...SEGMENT, findings: [{ ...FINDING, kind }] }] }
      expect(isProofreadEventWire(event)).toBe(true)
    }
  })

  it('rejects a malformed finding, a segment without an id, and unknown kinds', () => {
    const base = { kind: 'batch', usage: null, segments: [SEGMENT] }
    expect(isProofreadEventWire({ ...base, segments: [{ ...SEGMENT, findings: [{ ...FINDING, kind: 'style' }] }] })).toBe(false)
    expect(isProofreadEventWire({ ...base, segments: [{ ...SEGMENT, segment_id: undefined }] })).toBe(false)
    expect(isProofreadEventWire({ kind: 'planned', segments: '4', batches: 1 })).toBe(false)
    expect(isProofreadEventWire({ kind: 'token' })).toBe(false)
  })
})

describe('runProofreadSegments', () => {
  it('invokes ai_proofread_segments with camelCase segmentIds and a channel', async () => {
    invokeMock.mockResolvedValue({ state: 'cancelled', not_scanned: 1 })
    const result = await runProofreadSegments([5, 6], () => {})
    expect(invokeMock.mock.calls[0]?.[0]).toBe('ai_proofread_segments')
    const args = invokeMock.mock.calls[0]?.[1] as { segmentIds: number[]; channel: unknown }
    expect(args.segmentIds).toEqual([5, 6])
    expect(args.channel).toBeDefined()
    expect(result).toEqual({ value: { state: 'cancelled', not_scanned: 1 }, error: null })
  })

  it('forwards well-shaped channel events and drops malformed ones', async () => {
    const seen: unknown[] = []
    invokeMock.mockImplementation((_cmd: string, args: { channel: { onmessage: (v: unknown) => void } }) => {
      args.channel.onmessage({ kind: 'planned', segments: 2, batches: 1 })
      args.channel.onmessage({ kind: 'batch', segments: 'x' })
      return Promise.resolve({ state: 'done', not_scanned: 0, filtered: 0 })
    })
    vi.spyOn(console, 'error').mockImplementation(() => {})
    await runProofreadSegments([1], (e) => seen.push(e))
    expect(seen).toEqual([{ kind: 'planned', segments: 2, batches: 1 }])
  })

  it('returns the IpcError a rejected call carries, never throws', async () => {
    const err = { code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }
    invokeMock.mockRejectedValue(err)
    expect(await runProofreadSegments([5], () => {})).toEqual({ value: null, error: err })
  })

  it('turns a wrong-shaped reply into the unknown error', async () => {
    invokeMock.mockResolvedValue({ state: 'done', not_scanned: 0 })
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const result = await runProofreadSegments([5], () => {})
    expect(result.value).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})

describe('cancelProofreadCall', () => {
  it('invokes ai_proofread_cancel without arguments and swallows a failure', async () => {
    invokeMock.mockRejectedValue(new Error('x'))
    await expect(cancelProofreadCall()).resolves.toBeUndefined()
    expect(invokeMock).toHaveBeenCalledWith('ai_proofread_cancel')
  })
})

describe('the remembered-ignore adapters', () => {
  const ROW = { id: 7, kind: 'grammar', phrase: 'p', context_before: 'a', context_after: 'b', created_at: '2026-10-12T00:00:00Z' }
  const err = { code: 'ai_proofread.text_changed', message_key: 'err.ai_proofread.text_changed', params: {}, retryable: false }

  it('ignore_add sends camelCase keys without a replacement', async () => {
    invokeMock.mockResolvedValue(null)
    const result = await addProofreadIgnore({ segmentId: 3, expectedText: 'abc', start: 0, end: 2, expectedPhrase: 'ab', kind: 'spelling' })
    expect(result).toEqual({ ok: true, error: null })
    expect(invokeMock).toHaveBeenCalledWith('ai_proofread_ignore_add', {
      segmentId: 3,
      expectedText: 'abc',
      start: 0,
      end: 2,
      expectedPhrase: 'ab',
      kind: 'spelling',
    })
  })

  it('ignore_add returns the IpcError a rejected call carries', async () => {
    invokeMock.mockRejectedValue(err)
    const result = await addProofreadIgnore({ segmentId: 3, expectedText: 'abc', start: 0, end: 2, expectedPhrase: 'ab', kind: 'spelling' })
    expect(result).toEqual({ ok: false, error: err })
  })

  it('ignore_list accepts a list of well-shaped rows and rejects any other shape', async () => {
    invokeMock.mockResolvedValue([ROW])
    expect(await listProofreadIgnores()).toEqual({ rows: [ROW], error: null })
    vi.spyOn(console, 'error').mockImplementation(() => {})
    for (const bad of [{}, [{ ...ROW, kind: 'style' }], [{ ...ROW, id: '7' }], [{ ...ROW, context_after: null }], null]) {
      invokeMock.mockResolvedValue(bad)
      const result = await listProofreadIgnores()
      expect(result.rows).toBeNull()
      expect(result.error?.code).toBe('ipc.unknown')
    }
  })

  it('ignore_remove sends the id and requires a boolean', async () => {
    invokeMock.mockResolvedValue(false)
    expect(await removeProofreadIgnore(7)).toEqual({ removed: false, error: null })
    expect(invokeMock).toHaveBeenCalledWith('ai_proofread_ignore_remove', { id: 7 })
    vi.spyOn(console, 'error').mockImplementation(() => {})
    invokeMock.mockResolvedValue(1)
    expect((await removeProofreadIgnore(7)).error?.code).toBe('ipc.unknown')
  })

  it('ignore_clear sends no arguments and requires a non-negative integer count', async () => {
    invokeMock.mockResolvedValue(4)
    expect(await clearProofreadIgnores()).toEqual({ cleared: 4, error: null })
    expect(invokeMock).toHaveBeenCalledWith('ai_proofread_ignore_clear')
    vi.spyOn(console, 'error').mockImplementation(() => {})
    for (const bad of [-1, 'x', null, true]) {
      invokeMock.mockResolvedValue(bad)
      expect((await clearProofreadIgnores()).cleared).toBeNull()
    }
  })

  it('scan_record sends workId, scope, reported and filtered, and never throws', async () => {
    invokeMock.mockResolvedValue(null)
    expect(await recordProofreadScan('w1', 'chapter', 3, 2)).toEqual({ ok: true, error: null })
    expect(invokeMock).toHaveBeenCalledWith('ai_proofread_scan_record', { workId: 'w1', scope: 'chapter', reported: 3, filtered: 2 })
    invokeMock.mockRejectedValue(err)
    expect(await recordProofreadScan('w1', 'chapter', 3, 2)).toEqual({ ok: false, error: err })
  })
})
