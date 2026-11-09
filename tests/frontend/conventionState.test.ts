import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { ConventionScanWire, ConventionSpanWire } from '../../src/config/convention'

const scanMock = vi.fn()
const acceptMock = vi.fn()
const revertMock = vi.fn()
const flushMock = vi.fn()
const moveMock = vi.fn()
const replaceMock = vi.fn()
const segments = ref<{ id: number; target_text: string; status: string }[]>([])
const caret = ref<number | null>(null)

vi.mock('../../src/config/convention', () => ({
  scanConventionChapter: (...args: unknown[]) => scanMock(...args),
  acceptConventionSpan: (...args: unknown[]) => acceptMock(...args),
  revertConventionSpan: (...args: unknown[]) => revertMock(...args),
}))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorCaretSegmentId: caret,
  editorSegments: segments,
  flushEditorBeforeDiscreteWrite: () => flushMock(),
  moveEditorCaretTo: (id: number) => {
    moveMock(id)
    caret.value = id
    return true
  },
  replaceEditorSegment: (...args: unknown[]) => replaceMock(...args),
}))
vi.mock('../../src/proofreadState', () => ({
  focusProofreadFinding: (segmentId: number, index: number) => {
    proofreadCursor = { segmentId, index }
  },
  proofreadCurrent: {
    get value() {
      return proofreadCursor
    },
  },
  proofreadMarkPositions: () => proofreadMarks,
  suppressProofreadStripFor: vi.fn(),
}))

let proofreadMarks: { segmentId: number; index: number; start: number; end: number }[] = []
let proofreadCursor: { segmentId: number; index: number } | null = null

const state = await import('../../src/conventionState')

function span(start: number, end: number, replacement: string | null = 'bảo đảm'): ConventionSpanWire {
  return { start, end, rule_key: 'đảm bảo', avoid: 'đảm bảo', replacement_cased: replacement, tier: 'global' }
}

function scanOf(generation: number, entries: Record<number, [string, ConventionSpanWire[]]>): ConventionScanWire {
  return {
    generation,
    chapter_id: 1,
    segments: Object.entries(entries).map(([id, [text, spans]]) => ({ segment_id: Number(id), scanned_text: text, spans })),
    ignored: 0,
    glossary_suppressed: 0,
    overlapped: 0,
    not_scanned: 0,
  }
}

async function loadScan(entries: Record<number, [string, ConventionSpanWire[]]>): Promise<void> {
  scanMock.mockImplementation((generation: number) => Promise.resolve({ scan: scanOf(generation, entries), error: null }))
  await state.scanConventionNow()
}

beforeEach(() => {
  state.resetConvention()
  scanMock.mockReset()
  acceptMock.mockReset()
  revertMock.mockReset()
  flushMock.mockReset().mockResolvedValue('clean')
  moveMock.mockReset()
  replaceMock.mockReset().mockImplementation((id: number, patch: { target_text?: string }) => {
    segments.value = segments.value.map((s) => (s.id === id ? { ...s, ...patch } : s))
  })
  proofreadMarks = []
  proofreadCursor = null
  segments.value = [
    { id: 1, target_text: 'Đảm bảo an toàn.', status: 'draft' },
    { id: 2, target_text: 'không có gì', status: 'draft' },
    { id: 3, target_text: 'Cần đảm bảo. Đảm bảo nữa.', status: 'confirmed' },
  ]
  caret.value = null
})

describe('scan results', () => {
  it('drops a result older than the newest request', async () => {
    let release: (value: unknown) => void = () => undefined
    scanMock.mockImplementationOnce(
      (generation: number) =>
        new Promise((resolve) => {
          release = () => resolve({ scan: scanOf(generation, { 1: ['Đảm bảo an toàn.', [span(0, 7)]] }), error: null })
        }),
    )
    const stale = state.scanConventionNow()
    state.resetConvention()
    release(null)
    await stale
    expect(state.conventionResults.value.size).toBe(0)
  })

  it('keeps marks and raises the failure notice when a scan fails', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]] })
    scanMock.mockResolvedValueOnce({ scan: null, error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: true } })
    await state.scanConventionNow()
    expect(state.conventionResults.value.size).toBe(1)
    expect(state.conventionNotice.value?.key).toBe('convention.status.scan_failed')
  })

  it('a typed edit kills the marks of that sentence only', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]], 3: ['Cần đảm bảo. Đảm bảo nữa.', [span(4, 11), span(13, 20)]] })
    state.clearConventionFor(1)
    expect([...state.conventionResults.value.keys()]).toEqual([3])
  })

  it('a sentence whose text changed since the scan has no current spot', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]] })
    caret.value = 1
    state.openConventionAt(1, 0)
    expect(state.conventionCurrent.value?.span.start).toBe(0)
    segments.value = [{ id: 1, target_text: 'Đảm bảo an toàn!', status: 'draft' }, ...segments.value.slice(1)]
    expect(state.conventionCurrent.value).toBeNull()
  })
})

describe('F8 order across both families', () => {
  it('visits Proofreader findings and convention spots in text order and stops at the end', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]], 3: ['Cần đảm bảo. Đảm bảo nữa.', [span(4, 11), span(13, 20)]] })
    proofreadMarks = [{ segmentId: 1, index: 0, start: 8, end: 9 }, { segmentId: 3, index: 0, start: 0, end: 3 }]
    caret.value = 1
    const visited: string[] = []
    for (let i = 0; i < 6; i += 1) {
      state.stepMarks(1)
      const spot = state.conventionCurrent.value
      visited.push(spot === null ? `${String(caret.value)}:other` : `${String(spot.segmentId)}:${String(spot.span.start)}`)
      if (spot !== null) proofreadCursor = null
      if (state.conventionNotice.value?.key === 'convention.status.no_next') break
    }
    expect(visited).toEqual(['1:0', '1:other', '3:other', '3:4', '3:13', '3:13'])
    expect(state.conventionNotice.value?.key).toBe('convention.status.no_next')
  })

  it('reports no_prev before the first mark and no_marks on an empty Chapter', async () => {
    state.stepMarks(1)
    expect(state.conventionNotice.value?.key).toBe('convention.status.no_marks')
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]] })
    caret.value = 1
    state.stepMarks(1)
    state.stepMarks(-1)
    expect(state.conventionNotice.value?.key).toBe('convention.status.no_prev')
  })
})

describe('F8 after the strip was hidden', () => {
  it('continues from the caret offset instead of returning to the first mark of the sentence', async () => {
    await loadScan({ 3: ['Cần đảm bảo. Đảm bảo nữa.', [span(4, 11), span(13, 20)]] })
    caret.value = 3
    const cell = document.createElement('div')
    cell.setAttribute('data-col', 'tgt')
    cell.setAttribute('data-segment-id', '3')
    cell.textContent = 'Cần đảm bảo. Đảm bảo nữa.'
    document.body.appendChild(cell)
    try {
      state.openConventionAt(3, 13)
      state.hideConventionStrip()
      const text = cell.firstChild as Text
      document.getSelection()?.setBaseAndExtent(text, 15, text, 15)
      state.stepMarks(1)
      expect(state.conventionNotice.value?.key).toBe('convention.status.no_next')
      expect(state.conventionCurrent.value).toBeNull()
      state.stepMarks(-1)
      expect(state.conventionCurrent.value?.span.start).toBe(4)
    } finally {
      cell.remove()
    }
  })
})

describe('accept and revert', () => {
  it('shifts the later spots of the sentence and opens a revert tray', async () => {
    await loadScan({ 3: ['Cần đảm bảo. Đảm bảo nữa.', [span(4, 11), span(13, 20)]] })
    caret.value = 3
    state.openConventionAt(3, 4)
    acceptMock.mockResolvedValue({
      outcome: {
        segment_id: 3,
        target_text: 'Cần bảo đảm. Đảm bảo nữa.',
        translation_origin: 'other',
        status: 'confirmed',
        chapter_started: false,
        start: 4,
        end: 11,
        phrase: 'bảo đảm',
        version_created: true,
      },
      error: null,
    })
    await state.acceptConventionSpot()
    expect(acceptMock).toHaveBeenCalledWith(
      expect.objectContaining({ segmentId: 3, start: 4, end: 11, expectedPhrase: 'đảm bảo', ruleKey: 'đảm bảo' }),
    )
    expect(state.conventionResults.value.get(3)?.spans.map((s) => [s.start, s.end])).toEqual([[13, 20]])
    expect(state.conventionRevertible.value).toBe(true)
    expect(state.conventionRevertConfirmed.value).toBe(true)
  })

  it('a rule without a replacement offers no accept and sends nothing', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7, null)]] })
    caret.value = 1
    state.openConventionAt(1, 0)
    await state.acceptConventionSpot()
    expect(acceptMock).not.toHaveBeenCalled()
  })

  it('keeps the sentence unchanged and shows the error when Rust refuses', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]] })
    caret.value = 1
    state.openConventionAt(1, 0)
    acceptMock.mockResolvedValue({
      outcome: null,
      error: { code: 'convention.text_changed', message_key: 'convention.text_changed', params: { segment_id: '1' }, retryable: false },
    })
    await state.acceptConventionSpot()
    expect(replaceMock).not.toHaveBeenCalled()
    expect(state.conventionActionError.value?.code).toBe('convention.text_changed')
    expect(state.conventionResults.value.get(1)?.spans).toHaveLength(1)
  })

  it('revert restores the span and drops the tray', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]] })
    caret.value = 1
    state.openConventionAt(1, 0)
    acceptMock.mockResolvedValue({
      outcome: { segment_id: 1, target_text: 'Bảo đảm an toàn.', translation_origin: 'other', status: 'draft', chapter_started: false, start: 0, end: 7, phrase: 'Bảo đảm', version_created: false },
      error: null,
    })
    await state.acceptConventionSpot()
    revertMock.mockResolvedValue({
      outcome: { segment_id: 1, target_text: 'Đảm bảo an toàn.', translation_origin: 'other', status: 'draft', chapter_started: false, start: 0, end: 7, phrase: 'Đảm bảo', version_created: false },
      error: null,
    })
    await state.revertConventionSpot()
    expect(revertMock).toHaveBeenCalledWith(
      expect.objectContaining({ expectedText: 'Bảo đảm an toàn.', expectedPhrase: 'Bảo đảm', originalPhrase: 'Đảm bảo' }),
    )
    expect(state.conventionResults.value.get(1)?.spans.map((s) => s.start)).toEqual([0])
    expect(state.conventionRevertible.value).toBe(false)
  })
})

describe('Chapter replace hooks', () => {
  it('the viewed rule count is the Chapter-wide count of spots with a replacement, and null without one', async () => {
    await loadScan({
      1: ['Đảm bảo an toàn.', [span(0, 7)]],
      3: ['Cần đảm bảo. Đảm bảo nữa.', [span(4, 11), span(13, 20), span(21, 24, null)]],
    })
    expect(state.conventionViewedRuleCount.value).toBeNull()
    caret.value = 1
    state.openConventionAt(1, 0)
    expect(state.conventionViewedRuleCount.value).toBe(3)
    expect(state.viewedConventionRuleKey()).toBe('đảm bảo')
  })

  it('a committed replace mirrors each sentence, drops its marks and tray, keeps other marks, and says what happened', async () => {
    await loadScan({ 1: ['Đảm bảo an toàn.', [span(0, 7)]], 3: ['Cần đảm bảo. Đảm bảo nữa.', [span(4, 11), span(13, 20)]] })
    scanMock.mockClear()
    state.applyConventionReplaced({
      outcomes: [
        { segment_id: 1, target_text: 'Bảo đảm an toàn.', translation_origin: 'other', status: 'draft', version_created: false },
      ],
      replaced: 1,
      sentences: 1,
      confirmed: 0,
      drafts: 1,
      chapter_started: false,
    })
    expect(replaceMock).toHaveBeenCalledWith(1, { target_text: 'Bảo đảm an toàn.', translation_origin: 'other', status: 'draft' })
    expect(state.conventionResults.value.has(1)).toBe(false)
    expect(state.conventionResults.value.has(3)).toBe(true)
    expect(state.conventionNotice.value).toEqual({
      key: 'convention.status.replaced',
      params: { replaced_count: '1', sentence_count: '1', confirmed_count: '0', draft_count: '1' },
    })
    expect(scanMock).toHaveBeenCalledTimes(1)
  })
})
