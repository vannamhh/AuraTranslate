import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ProofreadOutcomeWire } from '../../src/config/proofread'
import type { IpcError } from '../../src/i18n'

const runMock = vi.fn()
const cancelMock = vi.fn()
const flushMock = vi.fn()

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegment: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: (...args: unknown[]) => cancelMock(...args),
}))
vi.mock('../../src/panels/editorPanelState', () => ({
  flushEditorBeforeDiscreteWrite: () => flushMock(),
}))

const state = await import('../../src/proofreadState')

const DONE: ProofreadOutcomeWire = {
  state: 'done',
  usage: null,
  scanned_text: 'xin chaof',
  findings: [{ kind: 'spelling', start: 4, end: 9, explanation: 'e', suggestion: 's' }],
  unlocated: 1,
}

function deferred<T>(): { promise: Promise<T>; resolve: (v: T) => void } {
  let resolve!: (v: T) => void
  const promise = new Promise<T>((r) => {
    resolve = r
  })
  return { promise, resolve }
}

beforeEach(() => {
  state.resetProofread()
  runMock.mockReset()
  cancelMock.mockReset()
  flushMock.mockReset()
  flushMock.mockResolvedValue('clean')
})

describe('runProofread', () => {
  it('flushes the Editor before calling the adapter', async () => {
    const order: string[] = []
    flushMock.mockImplementation(() => {
      order.push('flush')
      return Promise.resolve('clean')
    })
    runMock.mockImplementation(() => {
      order.push('run')
      return Promise.resolve({ value: DONE, error: null })
    })
    await state.runProofread(7)
    expect(order).toEqual(['flush', 'run'])
    expect(runMock.mock.calls[0]?.[0]).toBe(7)
  })

  it('keeps the findings, scanned text and unlocated count of a done outcome', async () => {
    runMock.mockResolvedValue({ value: DONE, error: null })
    await state.runProofread(7)
    expect(state.proofreadStateValue.value).toBe('done')
    expect(state.proofreadRunSegmentId.value).toBe(7)
    expect(state.proofreadScannedText.value).toBe('xin chaof')
    expect(state.proofreadFindings.value).toHaveLength(1)
    expect(state.proofreadUnlocated.value).toBe(1)
  })

  it.each([
    ['still-dirty', 'proofread.flush_still_dirty'],
    ['failed', 'proofread.flush_failed'],
  ])('flush %s: no adapter call, state error with its own key', async (flushed, key) => {
    flushMock.mockResolvedValue(flushed)
    await state.runProofread(7)
    expect(runMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('error')
    expect(state.proofreadError.value?.message_key).toBe(key)
  })

  it('does nothing without a caret segment', async () => {
    await state.runProofread(null)
    expect(flushMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('idle')
  })

  it('maps not_configured, cancelled and an IpcError', async () => {
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    await state.runProofread(1)
    expect(state.proofreadStateValue.value).toBe('not_configured')

    runMock.mockResolvedValue({ value: { state: 'cancelled' }, error: null })
    await state.runProofread(1)
    expect(state.proofreadStateValue.value).toBe('cancelled')
    expect(state.proofreadFindings.value).toEqual([])

    const err: IpcError = { code: 'ai_proofread.reply_malformed', message_key: 'err.ai_proofread.reply_malformed', params: {}, retryable: true }
    runMock.mockResolvedValue({ value: null, error: err })
    await state.runProofread(1)
    expect(state.proofreadStateValue.value).toBe('error')
    expect(state.proofreadError.value?.retryable).toBe(true)
    expect(state.proofreadFindings.value).toEqual([])
  })

  it('refuses a second scan while one is running', async () => {
    const d = deferred<unknown>()
    runMock.mockReturnValue(d.promise)
    const first = state.runProofread(1)
    await Promise.resolve()
    await state.runProofread(2)
    expect(state.proofreadRunSegmentId.value).toBe(1)
    d.resolve({ value: DONE, error: null })
    await first
  })
})

describe('cancel and stale results', () => {
  it('sends the Rust cancel only while scanning', async () => {
    state.cancelProofread()
    expect(cancelMock).not.toHaveBeenCalled()

    const d = deferred<unknown>()
    runMock.mockReturnValue(d.promise)
    const run = state.runProofread(1)
    await new Promise((r) => setTimeout(r, 0))
    state.cancelProofread()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    d.resolve({ value: { state: 'cancelled' }, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('cancelled')
  })

  it('a cancel during the flush stops the scan without a Rust call', async () => {
    const d = deferred<string>()
    flushMock.mockReturnValue(d.promise)
    const run = state.runProofread(1)
    state.cancelProofread()
    d.resolve('clean')
    await run
    expect(cancelMock).not.toHaveBeenCalled()
    expect(runMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('cancelled')
  })

  it('a reset in flight cancels Rust and drops the late result', async () => {
    const d = deferred<unknown>()
    runMock.mockReturnValue(d.promise)
    const run = state.runProofread(1)
    await new Promise((r) => setTimeout(r, 0))
    state.resetProofread()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    d.resolve({ value: DONE, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('idle')
    expect(state.proofreadFindings.value).toEqual([])
  })
})

describe('clearProofreadFor', () => {
  it('a typed edit during the scan cancels it and drops the late result', async () => {
    const d = deferred<unknown>()
    runMock.mockReturnValue(d.promise)
    const run = state.runProofread(7)
    await new Promise((r) => setTimeout(r, 0))
    state.clearProofreadFor(7)
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.proofreadStateValue.value).toBe('idle')
    d.resolve({ value: DONE, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('idle')
    expect(state.proofreadFindings.value).toEqual([])
  })

  it('clears the findings of the scanned segment only', async () => {
    runMock.mockResolvedValue({ value: DONE, error: null })
    await state.runProofread(7)
    state.clearProofreadFor(8)
    expect(state.proofreadStateValue.value).toBe('done')
    state.clearProofreadFor(7)
    expect(state.proofreadStateValue.value).toBe('idle')
    expect(state.proofreadFindings.value).toEqual([])
  })
})
