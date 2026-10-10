import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {
    onmessage: (v: unknown) => void = () => {}
  },
}))

const { cancelProofreadCall, isProofreadOutcomeWire, runProofreadSegment } = await import('../../src/config/proofread')

const FINDING = { kind: 'grammar', start: 0, end: 3, explanation: 'e', suggestion: 's' }

beforeEach(() => {
  invokeMock.mockReset()
})

describe('isProofreadOutcomeWire', () => {
  it('accepts the three outcomes', () => {
    expect(isProofreadOutcomeWire({ state: 'not_configured' })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'cancelled' })).toBe(true)
    expect(isProofreadOutcomeWire({ state: 'done', usage: null, scanned_text: 'a', findings: [FINDING], unlocated: 0 })).toBe(true)
  })

  it('rejects a done outcome with a malformed finding or a missing field', () => {
    const base = { state: 'done', usage: null, scanned_text: 'a', findings: [FINDING], unlocated: 0 }
    expect(isProofreadOutcomeWire({ ...base, findings: [{ ...FINDING, kind: 'style' }] })).toBe(false)
    expect(isProofreadOutcomeWire({ ...base, findings: [{ ...FINDING, start: '0' }] })).toBe(false)
    expect(isProofreadOutcomeWire({ ...base, unlocated: undefined })).toBe(false)
    expect(isProofreadOutcomeWire({ state: 'generating' })).toBe(false)
  })
})

describe('runProofreadSegment', () => {
  it('invokes ai_proofread_segment with a camelCase segmentId and a channel', async () => {
    invokeMock.mockResolvedValue({ state: 'cancelled' })
    const result = await runProofreadSegment(5, () => {})
    expect(invokeMock.mock.calls[0]?.[0]).toBe('ai_proofread_segment')
    const args = invokeMock.mock.calls[0]?.[1] as { segmentId: number; channel: unknown }
    expect(args.segmentId).toBe(5)
    expect(args.channel).toBeDefined()
    expect(result).toEqual({ value: { state: 'cancelled' }, error: null })
  })

  it('returns the IpcError a rejected call carries, never throws', async () => {
    const err = { code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }
    invokeMock.mockRejectedValue(err)
    expect(await runProofreadSegment(5, () => {})).toEqual({ value: null, error: err })
  })

  it('turns a wrong-shaped reply into the unknown error', async () => {
    invokeMock.mockResolvedValue({ state: 'done' })
    const result = await runProofreadSegment(5, () => {})
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
