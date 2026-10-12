import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ProofreadEventWire, ProofreadOutcomeWire } from '../../src/config/proofread'
import type { AiTranslateUsageWire } from '../../src/config/aitranslate'
import type { IpcError } from '../../src/i18n'

const runMock = vi.fn()
const cancelMock = vi.fn()
const flushMock = vi.fn()

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: (...args: unknown[]) => cancelMock(...args),
}))
vi.mock('../../src/panels/editorPanelState', async () => {
  const { ref } = await import('vue')
  return {
    editorCaretSegmentId: ref<number | null>(null),
    editorSegments: ref([]),
    flushEditorBeforeDiscreteWrite: () => flushMock(),
  }
})

const state = await import('../../src/proofreadState')

type OnEvent = (event: ProofreadEventWire) => void
type RunResult = { value: ProofreadOutcomeWire | null; error: IpcError | null }

const FINDING = { kind: 'spelling' as const, start: 4, end: 9, explanation: 'e', suggestion: 's' }
const DONE: ProofreadOutcomeWire = { state: 'done', not_scanned: 0, filtered: 0 }

function batch(ids: number[], usage: AiTranslateUsageWire | null = null): ProofreadEventWire {
  return {
    kind: 'batch',
    usage,
    segments: ids.map((id) => ({ segment_id: id, scanned_text: `xin chaof ${String(id)}`, findings: [FINDING], unlocated: 0, ambiguous: 0 })),
  }
}

function usage(total: number, cost: number | null) {
  return { prompt_tokens: total - 1, completion_tokens: 1, total_tokens: total, cost_usd: cost }
}

function pendingRun(): { emit: OnEvent; settle: (r: RunResult) => void } {
  let onEvent: OnEvent = () => {
    throw new Error('runMock not called yet')
  }
  let resolve: (r: RunResult) => void = () => {
    throw new Error('runMock not called yet')
  }
  runMock.mockImplementation((_ids: number[], cb: OnEvent) => {
    onEvent = cb
    return new Promise<RunResult>((r) => {
      resolve = r
    })
  })
  return { emit: (e) => onEvent(e), settle: (r) => resolve(r) }
}

async function started(ids: number[]): Promise<{ run: Promise<void>; emit: OnEvent; settle: (r: RunResult) => void }> {
  const pending = pendingRun()
  const run = state.runProofread(ids)
  await vi.waitFor(() => {
    expect(runMock).toHaveBeenCalled()
  })
  return { run, ...pending }
}

beforeEach(() => {
  state.resetProofread()
  runMock.mockReset()
  cancelMock.mockReset()
  flushMock.mockReset()
  flushMock.mockResolvedValue('clean')
})

describe('runProofread', () => {
  it('flushes the Editor before calling the adapter, and passes the ids', async () => {
    const order: string[] = []
    flushMock.mockImplementation(() => {
      order.push('flush')
      return Promise.resolve('clean')
    })
    runMock.mockImplementation(() => {
      order.push('run')
      return Promise.resolve({ value: DONE, error: null })
    })
    await state.runProofread([7, 8])
    expect(order).toEqual(['flush', 'run'])
    expect(runMock.mock.calls[0]?.[0]).toEqual([7, 8])
  })

  it('keeps one entry per segment from each batch event', async () => {
    const { run, emit, settle } = await started([1, 2, 3])
    emit({ kind: 'planned', segments: 3, batches: 2 })
    emit(batch([1, 2]))
    expect(state.proofreadProgress.value).toEqual({ done: 2, total: 3 })
    emit(batch([3]))
    settle({ value: DONE, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('done')
    expect([...state.proofreadResults.value.keys()]).toEqual([1, 2, 3])
    expect(state.proofreadResults.value.get(2)?.scannedText).toBe('xin chaof 2')
    expect(state.proofreadFindingCount.value).toBe(3)
  })

  it('does nothing for an empty scope', async () => {
    await state.runProofread([])
    expect(flushMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('idle')
  })

  it.each([
    ['still-dirty', 'proofread.flush_still_dirty'],
    ['failed', 'proofread.flush_failed'],
  ])('flush %s: no adapter call, state error with its own key', async (flushed, key) => {
    flushMock.mockResolvedValue(flushed)
    await state.runProofread([7])
    expect(runMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('error')
    expect(state.proofreadError.value?.message_key).toBe(key)
  })

  it('maps not_configured', async () => {
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    await state.runProofread([1])
    expect(state.proofreadStateValue.value).toBe('not_configured')
  })

  it('an IpcError keeps the earlier batches and reads not_scanned from its params', async () => {
    const { run, emit, settle } = await started([1, 2, 3])
    emit(batch([1]))
    const err: IpcError = { code: 'ai.network', message_key: 'err.ai.network', params: { not_scanned: '2' }, retryable: true }
    settle({ value: null, error: err })
    await run
    expect(state.proofreadStateValue.value).toBe('error')
    expect(state.proofreadNotScanned.value).toBe(2)
    expect([...state.proofreadResults.value.keys()]).toEqual([1])
  })

  it('refuses a second scan while one is running', async () => {
    const { run, settle } = await started([1])
    await state.runProofread([2])
    expect(state.proofreadRunSegmentIds.value).toEqual([1])
    settle({ value: DONE, error: null })
    await run
  })

  it('cancelled keeps the findings that arrived and reports not_scanned', async () => {
    const { run, emit, settle } = await started([1, 2, 3, 4])
    emit({ kind: 'planned', segments: 4, batches: 4 })
    emit(batch([1]))
    emit(batch([2]))
    state.cancelProofread()
    settle({ value: { state: 'cancelled', not_scanned: 2 }, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('cancelled')
    expect(state.proofreadNotScanned.value).toBe(2)
    expect([...state.proofreadResults.value.keys()]).toEqual([1, 2])
  })
})

describe('usage totals', () => {
  async function finish(events: ProofreadEventWire[], ids: number[]): Promise<void> {
    const { run, emit, settle } = await started(ids)
    events.forEach(emit)
    settle({ value: DONE, error: null })
    await run
  }

  it('sums a full run with a price', async () => {
    await finish([batch([1, 2], usage(100, 0.01)), batch([3], usage(50, 0.005))], [1, 2, 3])
    expect(state.proofreadUsageSummary.value).toEqual({ tokenCount: 150, reportedCount: 3, sentenceCount: 3, costUsd: 0.015 })
  })

  it('has no cost when a reported batch has no price', async () => {
    await finish([batch([1], usage(100, 0.01)), batch([2], usage(50, null))], [1, 2])
    expect(state.proofreadUsageSummary.value.costUsd).toBeNull()
    expect(state.proofreadUsageSummary.value.tokenCount).toBe(150)
  })

  it('counts a batch without usage as arrived but not reported', async () => {
    await finish([batch([1, 2], usage(100, 0.01)), batch([3], null)], [1, 2, 3])
    expect(state.proofreadUsageSummary.value).toMatchObject({ reportedCount: 2, sentenceCount: 3 })
  })

  it('totals only the batches that arrived when cancelled', async () => {
    const { run, emit, settle } = await started([1, 2, 3])
    emit(batch([1], usage(100, 0.01)))
    settle({ value: { state: 'cancelled', not_scanned: 2 }, error: null })
    await run
    expect(state.proofreadUsageSummary.value).toEqual({ tokenCount: 100, reportedCount: 1, sentenceCount: 1, costUsd: 0.01 })
  })
})

describe('cancel and stale results', () => {
  it('sends the Rust cancel only while scanning', async () => {
    state.cancelProofread()
    expect(cancelMock).not.toHaveBeenCalled()
    const { run, settle } = await started([1])
    state.cancelProofread()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    settle({ value: { state: 'cancelled', not_scanned: 1 }, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('cancelled')
  })

  it('a cancel during the flush stops the scan without a Rust call', async () => {
    let release!: (v: string) => void
    flushMock.mockReturnValue(new Promise<string>((r) => (release = r)))
    const run = state.runProofread([1])
    state.cancelProofread()
    release('clean')
    await run
    expect(cancelMock).not.toHaveBeenCalled()
    expect(runMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('cancelled')
  })

  it('a reset in flight cancels Rust and drops a batch that arrives late', async () => {
    const { run, emit, settle } = await started([1, 2])
    emit(batch([1]))
    state.resetProofread()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    emit(batch([2]))
    settle({ value: DONE, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('idle')
    expect(state.proofreadResults.value.size).toBe(0)
    expect(state.proofreadUsageSummary.value.sentenceCount).toBe(0)
  })

  it('a late batch of an old scan does not leak into a new scan', async () => {
    const first = await started([1, 2])
    state.resetProofread()
    runMock.mockReset()
    const second = await started([5])
    first.emit(batch([1]))
    second.emit(batch([5]))
    first.settle({ value: DONE, error: null })
    second.settle({ value: DONE, error: null })
    await Promise.all([first.run, second.run])
    expect([...state.proofreadResults.value.keys()]).toEqual([5])
  })
})

describe('clearProofreadFor', () => {
  it('a typed edit in a single-segment scan cancels it and drops the late result', async () => {
    const { run, emit, settle } = await started([7])
    state.clearProofreadFor(7)
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.proofreadStateValue.value).toBe('idle')
    emit(batch([7]))
    settle({ value: DONE, error: null })
    await run
    expect(state.proofreadStateValue.value).toBe('idle')
    expect(state.proofreadResults.value.size).toBe(0)
  })

  it('clears one cell of a finished chapter scan and keeps the others', async () => {
    runMock.mockImplementation((_ids: number[], cb: OnEvent) => {
      cb(batch([1, 2, 3]))
      return Promise.resolve({ value: DONE, error: null })
    })
    await state.runProofread([1, 2, 3])
    state.clearProofreadFor(2)
    expect(state.proofreadStateValue.value).toBe('done')
    expect([...state.proofreadResults.value.keys()]).toEqual([1, 3])
  })

  it('an edit during a chapter scan clears that cell, keeps the scan, and drops its later result', async () => {
    const { run, emit, settle } = await started([1, 2, 3])
    emit(batch([1]))
    state.clearProofreadFor(1)
    state.clearProofreadFor(3)
    expect(cancelMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('scanning')
    emit(batch([2, 3]))
    settle({ value: DONE, error: null })
    await run
    expect([...state.proofreadResults.value.keys()]).toEqual([2])
  })

  it('ignores a segment outside the scope', async () => {
    runMock.mockImplementation((_ids: number[], cb: OnEvent) => {
      cb(batch([1, 2]))
      return Promise.resolve({ value: DONE, error: null })
    })
    await state.runProofread([1, 2])
    state.clearProofreadFor(99)
    expect(state.proofreadResults.value.size).toBe(2)
  })
})

describe('scope chooser state', () => {
  it('opens, closes and is closed by a reset', () => {
    state.openProofreadScope()
    expect(state.proofreadScopeIsOpen.value).toBe(true)
    state.closeProofreadScope()
    expect(state.proofreadScopeIsOpen.value).toBe(false)
    state.openProofreadScope()
    state.resetProofread()
    expect(state.proofreadScopeIsOpen.value).toBe(false)
  })
})
