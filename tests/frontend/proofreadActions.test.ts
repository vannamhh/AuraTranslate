import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ProofreadFindingWire } from '../../src/config/proofread'
import type { IpcError } from '../../src/i18n'

const acceptMock = vi.fn()
const revertMock = vi.fn()
const flushMock = vi.fn()
const moveMock = vi.fn()
const invokeMock = vi.fn()
const skipMock = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invokeMock(...args), Channel: class {} }))
vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: () => Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0 }, error: null }),
  cancelProofreadCall: () => Promise.resolve(),
  acceptProofreadSuggestion: (...args: unknown[]) => acceptMock(...args),
  revertProofreadSuggestion: (...args: unknown[]) => revertMock(...args),
  skipProofreadDecision: (...args: unknown[]) => skipMock(...args),
  getProofreadDecisionCounts: () => Promise.resolve({ counts: { accepted: 0, not_error: 0, skipped: 0, by_kind: [] }, error: null }),
}))
vi.mock('../../src/panels/editorPanelState', async () => {
  const { ref } = await import('vue')
  const editorSegments = ref<{ id: number; target_text: string; translation_origin: string; status: string }[]>([])
  return {
    editorCaretSegmentId: ref<number | null>(null),
    editorSegments,
    flushEditorBeforeDiscreteWrite: () => flushMock(),
    moveEditorCaretTo: (id: number) => moveMock(id),
    replaceEditorSegment: (id: number, patch: Record<string, unknown>) => {
      editorSegments.value = editorSegments.value.map((s) => (s.id === id ? { ...s, ...patch } : s))
    },
  }
})

const state = await import('../../src/proofreadState')
const editor = await import('../../src/panels/editorPanelState')
const { t, tError } = await import('../../src/i18n')
const caretRef = editor.editorCaretSegmentId as unknown as { value: number | null }
const segmentsRef = editor.editorSegments as unknown as { value: unknown[] }

const TEXT = 'aa bbb cc ddd'
const spelling = (start: number, end: number, suggestion: string): ProofreadFindingWire => ({
  kind: 'spelling',
  start,
  end,
  explanation: 'e',
  suggestion,
})

function seed(id: number, text: string): void {
  const segments = editor.editorSegments as unknown as { value: unknown[] }
  segments.value = [...segments.value.filter((s) => (s as { id: number }).id !== id), { id, target_text: text, translation_origin: 'human', status: 'draft' }].sort(
    (a, b) => (a as { id: number }).id - (b as { id: number }).id,
  )
  state.resetProofread()
}

async function scan(id: number, text: string, findings: ProofreadFindingWire[]): Promise<void> {
  await runWith(id, text, findings, [])
}

async function runWith(id: number, text: string, findings: ProofreadFindingWire[], more: number[]): Promise<void> {
  const cfg = await import('../../src/config/proofread')
  const spy = vi.spyOn(cfg, 'runProofreadSegments').mockImplementation(((_ids: number[], onEvent: (e: unknown) => void) => {
    onEvent({ kind: 'planned', segments: 1 + more.length, batches: 1 })
    onEvent({
      kind: 'batch',
      usage: null,
      segments: [{ segment_id: id, scanned_text: text, findings, unlocated: 0, ambiguous: 0 }],
    })
    return Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0 }, error: null })
  }) as never)
  await state.runProofread([id, ...more])
  spy.mockRestore()
}

function reply(id: number, text: string, origin = 'ai_proofread') {
  return { outcome: { segment_id: id, target_text: text, translation_origin: origin, status: 'draft', needs_confirmation: false, unsigned_draft: null }, decisionId: 7, error: null }
}

function textOf(id: number): string {
  return (editor.editorSegments.value as unknown as { id: number; target_text: string }[]).find((s) => s.id === id)?.target_text ?? ''
}

beforeEach(() => {
  state.resetProofread()
  acceptMock.mockReset()
  revertMock.mockReset()
  skipMock.mockReset()
  skipMock.mockResolvedValue({ ok: true, error: null })
  flushMock.mockReset()
  moveMock.mockReset()
  invokeMock.mockReset()
  flushMock.mockResolvedValue('clean')
  moveMock.mockReturnValue(true)
  ;segmentsRef.value = []
  caretRef.value = null
})

describe('current finding at the caret', () => {
  it('shows only while the segment text still equals the scanned text', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'BBB')])
    expect(state.proofreadCurrent.value?.finding.suggestion).toBe('BBB')
    editor.replaceEditorSegment(1, { target_text: `${TEXT}!` })
    expect(state.proofreadCurrent.value).toBeNull()
  })
})

describe('accept', () => {
  it('flushes, replaces only the range with the verbatim suggestion, then shifts the later findings', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'BBBBB'), spelling(10, 13, 'D')])
    const order: string[] = []
    flushMock.mockImplementation(() => {
      order.push('flush')
      return Promise.resolve('clean')
    })
    acceptMock.mockImplementation(() => {
      order.push('accept')
      return Promise.resolve(reply(1, 'aa BBBBB cc ddd'))
    })
    await state.acceptProofreadFinding()

    expect(order).toEqual(['flush', 'accept'])
    expect(acceptMock).toHaveBeenCalledWith({
      segmentId: 1,
      expectedText: TEXT,
      start: 3,
      end: 6,
      expectedPhrase: 'bbb',
      replacement: 'BBBBB',
      kind: 'spelling',
    })
    expect(textOf(1)).toBe('aa BBBBB cc ddd')
    const entry = state.proofreadResults.value.get(1)
    expect(entry?.scannedText).toBe('aa BBBBB cc ddd')
    expect(entry?.findings).toEqual([spelling(12, 15, 'D')])
    expect(state.proofreadCurrent.value?.count).toBe(1)
    expect('aa BBBBB cc ddd'.slice(12, 15)).toBe('ddd')
  })

  it('a refused write changes nothing and surfaces the error at the caret segment', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'B')])
    const err: IpcError = { code: 'ai_proofread.text_changed', message_key: 'err.ai_proofread.text_changed', params: { segment_id: '1' }, retryable: false }
    acceptMock.mockResolvedValue({ outcome: null, error: err })
    await state.acceptProofreadFinding()
    expect(textOf(1)).toBe(TEXT)
    expect(state.proofreadResults.value.get(1)?.findings).toHaveLength(1)
    expect(state.proofreadActionError.value).toEqual(err)
    caretRef.value = 2
    expect(state.proofreadActionError.value).toBeNull()
  })

  it.each([
    ['still-dirty', 'proofread.accept_flush_still_dirty'],
    ['failed', 'proofread.accept_flush_failed'],
  ])('flush %s: no write, and the message is not the scan one', async (flushed, key) => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'B')])
    flushMock.mockResolvedValue(flushed)
    await state.acceptProofreadFinding()
    expect(acceptMock).not.toHaveBeenCalled()
    const failure = state.proofreadActionError.value
    expect(failure?.message_key).toBe(key)
    expect(tError(failure as IpcError)).not.toBe(t('proofread.flush_failed'))
    expect(tError(failure as IpcError)).not.toBe(t('proofread.flush_still_dirty'))
    expect(tError(failure as IpcError).toLowerCase()).not.toContain('chưa quét')
  })

  it('flush failure on revert uses the same own message', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'B')])
    acceptMock.mockResolvedValue(reply(1, 'aa B cc ddd'))
    await state.acceptProofreadFinding()
    flushMock.mockResolvedValue('failed')
    await state.revertProofreadFinding()
    expect(revertMock).not.toHaveBeenCalled()
    expect(state.proofreadActionError.value?.message_key).toBe('proofread.accept_flush_failed')
  })
})

describe('revert', () => {
  it('swaps the roles, restores the finding and the original text', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'BB')])
    acceptMock.mockResolvedValue(reply(1, 'aa BB cc ddd'))
    await state.acceptProofreadFinding()
    expect(state.proofreadRevertible.value).toBe(true)
    expect(state.proofreadCurrent.value).toBeNull()

    revertMock.mockResolvedValue(reply(1, TEXT, 'human'))
    await state.revertProofreadFinding()
    expect(revertMock).toHaveBeenCalledWith({
      segmentId: 1,
      expectedText: 'aa BB cc ddd',
      start: 0,
      end: 12,
      expectedPhrase: 'aa BB cc ddd',
      replacement: TEXT,
      kind: 'spelling',
      decisionId: 7,
    })
    expect(textOf(1)).toBe(TEXT)
    expect(state.proofreadRevertible.value).toBe(false)
    expect(state.proofreadCurrent.value?.finding).toEqual(spelling(3, 6, 'BB'))
  })

  it('reverts an accepted deletion to the exact old text over the whole text', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(2, 6, '')])
    acceptMock.mockResolvedValue(reply(1, 'aa cc ddd'))
    await state.acceptProofreadFinding()
    expect(state.proofreadRevertible.value).toBe(true)

    revertMock.mockResolvedValue(reply(1, TEXT, 'human'))
    await state.revertProofreadFinding()
    expect(revertMock).toHaveBeenCalledWith({
      segmentId: 1,
      expectedText: 'aa cc ddd',
      start: 0,
      end: 9,
      expectedPhrase: 'aa cc ddd',
      replacement: TEXT,
      kind: 'spelling',
      decisionId: 7,
    })
    expect(textOf(1)).toBe(TEXT)
    expect(state.proofreadCurrent.value?.finding).toEqual(spelling(2, 6, ''))
  })

  it('is last-in-first-out per segment', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(0, 2, 'A'), spelling(3, 6, 'B')])
    acceptMock.mockResolvedValueOnce(reply(1, 'A bbb cc ddd'))
    await state.acceptProofreadFinding()
    acceptMock.mockResolvedValueOnce(reply(1, 'A B cc ddd'))
    await state.acceptProofreadFinding()
    revertMock.mockResolvedValueOnce(reply(1, 'A bbb cc ddd'))
    await state.revertProofreadFinding()
    expect(revertMock.mock.calls[0]?.[0]).toMatchObject({ expectedPhrase: 'A B cc ddd', replacement: 'A bbb cc ddd', start: 0, end: 10 })
    expect(state.proofreadRevertible.value).toBe(true)
    revertMock.mockResolvedValueOnce(reply(1, TEXT, 'human'))
    await state.revertProofreadFinding()
    expect(revertMock.mock.calls[1]?.[0]).toMatchObject({ expectedPhrase: 'A bbb cc ddd', replacement: TEXT, start: 0, end: 12 })
    expect(textOf(1)).toBe(TEXT)
    expect(state.proofreadRevertible.value).toBe(false)
  })

  it('is no longer offered once the text moved away from the accepted text', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'B')])
    acceptMock.mockResolvedValue(reply(1, 'aa B cc ddd'))
    await state.acceptProofreadFinding()
    editor.replaceEditorSegment(1, { target_text: 'aa B cc ddd!' })
    expect(state.proofreadRevertible.value).toBe(false)
    await state.revertProofreadFinding()
    expect(revertMock).not.toHaveBeenCalled()
  })

  it('typing in the segment drops its revert stack', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(3, 6, 'B')])
    acceptMock.mockResolvedValue(reply(1, 'aa B cc ddd'))
    await state.acceptProofreadFinding()
    state.clearProofreadFor(1)
    expect(state.proofreadRevertible.value).toBe(false)
  })
})

describe('dismiss', () => {
  it('records one skipped decision in Rust and removes the finding: no text write, count down by one', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(0, 2, 'A'), spelling(3, 6, 'B')])
    expect(state.proofreadFindingCount.value).toBe(2)
    await state.dismissProofreadFinding()
    expect(skipMock).toHaveBeenCalledExactlyOnceWith('spelling')
    expect(state.proofreadFindingCount.value).toBe(1)
    expect(state.proofreadCurrent.value?.count).toBe(1)
    expect(state.proofreadCurrent.value?.finding.suggestion).toBe('B')
    expect(textOf(1)).toBe(TEXT)
    expect(acceptMock).not.toHaveBeenCalled()
    expect(revertMock).not.toHaveBeenCalled()
    expect(invokeMock).not.toHaveBeenCalled()
    expect(state.proofreadStateValue.value).toBe('done')
  })

  it('keeps the finding and reports the error when Rust refuses to record the skip', async () => {
    seed(1, TEXT)
    caretRef.value = 1
    await scan(1, TEXT, [spelling(0, 2, 'A')])
    const err = { code: 'store.write_failed', message_key: 'store.write_failed', params: {}, retryable: false } as unknown as IpcError
    skipMock.mockResolvedValue({ ok: false, error: err })
    await state.dismissProofreadFinding()
    expect(state.proofreadFindingCount.value).toBe(1)
    expect(state.proofreadActionError.value).toEqual(err)
  })
})

describe('next / previous', () => {
  async function twoSegments(): Promise<void> {
    seed(1, TEXT)
    seed(2, TEXT)
    ;(editor.editorSegments as unknown as { value: unknown[] }).value = [
      { id: 1, target_text: TEXT, translation_origin: 'human', status: 'draft' },
      { id: 2, target_text: TEXT, translation_origin: 'human', status: 'draft' },
    ]
    caretRef.value = 1
    const cfg = await import('../../src/config/proofread')
    const spy = vi.spyOn(cfg, 'runProofreadSegments').mockImplementation(((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 2, batches: 1 })
      onEvent({
        kind: 'batch',
        usage: null,
        segments: [
          { segment_id: 1, scanned_text: TEXT, findings: [spelling(0, 2, 'A'), spelling(3, 6, 'B')], unlocated: 0, ambiguous: 0 },
          { segment_id: 2, scanned_text: TEXT, findings: [spelling(10, 13, 'D')], unlocated: 0, ambiguous: 0 },
        ],
      })
      return Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0 }, error: null })
    }) as never)
    await state.runProofread([1, 2])
    spy.mockRestore()
  }

  it('steps within a segment, then across segments in chapter order, without wrapping', async () => {
    await twoSegments()
    expect(state.proofreadCurrent.value?.index).toBe(0)
    state.prevProofreadFinding()
    expect(state.proofreadCurrent.value?.index).toBe(0)
    state.nextProofreadFinding()
    expect(state.proofreadCurrent.value?.index).toBe(1)
    moveMock.mockImplementation((id: number) => {
      caretRef.value = id
      return true
    })
    state.nextProofreadFinding()
    expect(moveMock).toHaveBeenCalledWith(2)
    expect(state.proofreadCurrent.value).toMatchObject({ segmentId: 2, index: 0 })
    state.nextProofreadFinding()
    expect(moveMock).toHaveBeenCalledTimes(1)
    state.prevProofreadFinding()
    expect(state.proofreadCurrent.value).toMatchObject({ segmentId: 1, index: 1 })
  })

  it('a refused caret move leaves the position alone', async () => {
    await twoSegments()
    state.nextProofreadFinding()
    moveMock.mockReturnValue(false)
    state.nextProofreadFinding()
    expect(moveMock).toHaveBeenCalledWith(2)
    expect(state.proofreadCurrent.value).toMatchObject({ segmentId: 1, index: 1 })
    moveMock.mockImplementation((id: number) => {
      caretRef.value = id
      return true
    })
    state.nextProofreadFinding()
    expect(state.proofreadCurrent.value).toMatchObject({ segmentId: 2, index: 0 })
  })
})
