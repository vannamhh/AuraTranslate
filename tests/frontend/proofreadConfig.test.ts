import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {
    onmessage: (v: unknown) => void = () => {}
  },
}))

const { cancelProofreadCall, isProofreadEventWire, isProofreadOutcomeWire, runProofreadSegments } = await import('../../src/config/proofread')

const FINDING = { kind: 'grammar', start: 0, end: 3, explanation: 'e', suggestion: 's' }

beforeEach(() => {
  invokeMock.mockReset()
})

const SEGMENT = { segment_id: 3, scanned_text: 'a', findings: [FINDING], unlocated: 0 }

describe('isProofreadOutcomeWire', () => {
  it('accepts the three outcomes', () => {
    expect(isProofreadOutcomeWire({ state: 'not_configured' })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'cancelled', not_scanned: 2 })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'done', not_scanned: 0 })).toBe(true)
  })

  it('rejects a done or cancelled outcome without not_scanned, and unknown states', () => {
    expect(isProofreadOutcomeWire({ state: 'done' })).toBe(false)
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
      return Promise.resolve({ state: 'done', not_scanned: 0 })
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
    invokeMock.mockResolvedValue({ state: 'done' })
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
