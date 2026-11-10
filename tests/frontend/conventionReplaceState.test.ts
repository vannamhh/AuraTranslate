import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { ConventionScanWire, ConventionSpanWire } from '../../src/config/convention'

const scanMock = vi.fn()
const replaceMock = vi.fn()
const flushMock = vi.fn()
const appliedMock = vi.fn()
const segments = ref<{ id: number; ord: number; target_text: string; status: string }[]>([])

vi.mock('../../src/config/convention', () => ({
  scanConventionSegments: () => Promise.resolve({ scan: null, error: null }),
  scanConventionChapterWithProgress: (...args: unknown[]) => scanMock(...args),
  replaceConventionChapter: (...args: unknown[]) => replaceMock(...args),
}))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorSegments: segments,
  flushEditorBeforeDiscreteWrite: () => flushMock(),
}))
vi.mock('../../src/conventionState', () => ({
  applyConventionReplaced: (...args: unknown[]) => appliedMock(...args),
}))

const state = await import('../../src/conventionReplaceState')

function span(start: number, end: number, ruleKey = 'đảm bảo', replacement: string | null = 'bảo đảm'): ConventionSpanWire {
  return { start, end, rule_key: ruleKey, avoid: ruleKey, replacement_cased: replacement, tier: 'global' }
}

type Entries = Record<number, [string, ConventionSpanWire[]]>

function scanOf(generation: number, entries: Entries, hidden: Partial<ConventionScanWire> = {}): ConventionScanWire {
  return {
    generation,
    chapter_id: 1,
    segments: Object.entries(entries).map(([id, [text, spans]]) => ({ segment_id: Number(id), scanned_text: text, spans })),
    ignored: 0,
    glossary_suppressed: 0,
    overlapped: 0,
    not_scanned: 0,
    ...hidden,
  }
}

function stubScan(entries: () => Entries, hidden: Partial<ConventionScanWire> = {}): void {
  scanMock.mockImplementation((generation: number, onProgress: (p: { done: number; total: number }) => void) => {
    onProgress({ done: 3, total: 3 })
    return Promise.resolve({ scan: scanOf(generation, entries(), hidden), error: null })
  })
}

const TEXT_A = 'Đảm bảo rằng mọi thứ ổn.'
const TEXT_B = 'nhằm bảo đảm bảo vệ đoàn xe.'
const TEXT_C = 'Quân sư dặn phải đảm bảo tin tức.'
const TEXT_D = 'Một thập kỷ trôi qua.'

function entries(): Entries {
  return {
    1: [TEXT_A, [span(0, 7)]],
    2: [TEXT_B, [span(9, 16)]],
    3: [TEXT_C, [span(17, 24)]],
    4: [TEXT_D, [span(4, 11, 'thập kỷ', 'thập niên')]],
  }
}

beforeEach(() => {
  state.closeConventionReplace()
  scanMock.mockReset()
  replaceMock.mockReset()
  appliedMock.mockReset()
  flushMock.mockReset().mockResolvedValue('clean')
  segments.value = [
    { id: 1, ord: 1, target_text: TEXT_A, status: 'draft' },
    { id: 2, ord: 22, target_text: TEXT_B, status: 'confirmed' },
    { id: 3, ord: 30, target_text: TEXT_C, status: 'confirmed' },
    { id: 4, ord: 40, target_text: TEXT_D, status: 'confirmed' },
  ]
  stubScan(entries)
})

function visibleKeys(): string[] {
  return state.conventionReplaceVisible.value.flatMap((g) => g.spots.map((s) => s.key))
}

describe('open', () => {
  it('flushes before it scans and selects every spot', async () => {
    await state.openConventionReplace(null)
    expect(flushMock).toHaveBeenCalledTimes(1)
    expect(flushMock.mock.invocationCallOrder[0]).toBeLessThan(scanMock.mock.invocationCallOrder[0])
    expect(state.conventionReplaceTotals.value).toEqual({ matches: 4, sentences: 4, selected: 4 })
  })

  it.each(['failed', 'still-dirty'])('a %s flush shows the flush error and never scans', async (result) => {
    flushMock.mockResolvedValue(result)
    await state.openConventionReplace(null)
    expect(scanMock).not.toHaveBeenCalled()
    expect(state.conventionReplaceFailure.value?.message_key).toBe('err.convention.flush_failed')
    expect(state.conventionReplaceIsOpen.value).toBe(true)
  })

  it('a scan error shows the error and keeps the list empty', async () => {
    scanMock.mockResolvedValue({ scan: null, error: { code: 'x', message_key: 'err.x', params: {}, retryable: true } })
    await state.openConventionReplace(null)
    expect(state.conventionReplaceFailure.value?.code).toBe('x')
    expect(state.conventionReplaceTotals.value.matches).toBe(0)
  })

  it('counts the four hidden totals and drops spots of a rule without a replacement', async () => {
    stubScan(() => ({ ...entries(), 5: ['kỷ nguyên', [span(0, 3, 'kỷ', null)]] }), {
      ignored: 3,
      glossary_suppressed: 1,
      overlapped: 2,
      not_scanned: 4,
    })
    await state.openConventionReplace(null)
    expect(state.conventionReplaceHidden.value).toEqual({ ignored: 3, glossarySuppressed: 1, overlapped: 2, notScanned: 4 })
    expect(state.conventionReplaceTotals.value.matches).toBe(4)
    expect(state.conventionReplaceOptions.value.map((o) => o.ruleKey).sort()).toEqual(['thập kỷ', 'đảm bảo'].sort())
  })

  it('opens with the filter on the given rule', async () => {
    await state.openConventionReplace('thập kỷ')
    expect(state.conventionReplaceFilter.value).toBe('thập kỷ')
    expect(visibleKeys()).toEqual(['4:thập kỷ:4'])
  })
})

describe('filter and selection', () => {
  it('the filter keeps every selection and K counts the hidden rows', async () => {
    await state.openConventionReplace(null)
    state.toggleConventionReplaceSpot('2:đảm bảo:9')
    state.setConventionReplaceFilter('thập kỷ')
    expect(state.conventionReplaceTotals.value.selected).toBe(3)
    state.setConventionReplaceFilter(null)
    expect(state.isConventionReplaceSelected('2:đảm bảo:9')).toBe(false)
  })

  it('select none and select all act on the visible rows only', async () => {
    await state.openConventionReplace(null)
    state.setConventionReplaceFilter('đảm bảo')
    state.selectConventionReplaceVisible(false)
    expect(state.conventionReplaceTotals.value.selected).toBe(1)
    expect(state.isConventionReplaceSelected('4:thập kỷ:4')).toBe(true)
    state.selectConventionReplaceVisible(true)
    expect(state.conventionReplaceTotals.value.selected).toBe(4)
  })

  it('arrow keys move the cursor inside the visible rows and Space-style toggling follows it', async () => {
    await state.openConventionReplace(null)
    state.moveConventionReplaceCursor(1)
    expect(state.conventionReplaceCursorKey.value).toBe('2:đảm bảo:9')
    state.toggleConventionReplaceAtCursor()
    expect(state.isConventionReplaceSelected('2:đảm bảo:9')).toBe(false)
    state.moveConventionReplaceCursor(99)
    expect(state.conventionReplaceCursorKey.value).toBe('4:thập kỷ:4')
  })
})

describe('confirm', () => {
  function committed(): unknown {
    return {
      outcome: {
        outcomes: [{ segment_id: 2, target_text: 'x', translation_origin: 'other', status: 'confirmed', version_created: true }],
        replaced: 3,
        sentences: 3,
        confirmed: 2,
        drafts: 1,
        chapter_started: false,
      },
      error: null,
    }
  }

  it('a selected draft sentence needs a second press naming the draft sentences', async () => {
    replaceMock.mockResolvedValue(committed())
    await state.openConventionReplace(null)
    state.toggleConventionReplaceSpot('2:đảm bảo:9')
    await state.confirmConventionReplace()
    expect(replaceMock).not.toHaveBeenCalled()
    expect(state.conventionReplaceArmed.value).toBe(true)
    expect(state.conventionReplaceDraftList()).toBe('câu 1')
    await state.confirmConventionReplace()
    expect(replaceMock).toHaveBeenCalledTimes(1)
    expect(appliedMock).toHaveBeenCalledTimes(1)
    expect(state.conventionReplaceIsOpen.value).toBe(false)
  })

  it('any checkbox change disarms the second press', async () => {
    await state.openConventionReplace(null)
    await state.confirmConventionReplace()
    expect(state.conventionReplaceArmed.value).toBe(true)
    state.toggleConventionReplaceSpot('3:đảm bảo:17')
    expect(state.conventionReplaceArmed.value).toBe(false)
    await state.confirmConventionReplace()
    expect(replaceMock).not.toHaveBeenCalled()
  })

  it('select all and select none disarm too, and Esc disarms before it closes', async () => {
    await state.openConventionReplace(null)
    await state.confirmConventionReplace()
    state.selectConventionReplaceVisible(true)
    expect(state.conventionReplaceArmed.value).toBe(false)
    await state.confirmConventionReplace()
    expect(state.disarmConventionReplace()).toBe(true)
    expect(state.disarmConventionReplace()).toBe(false)
  })

  it('with no draft sentence selected one press writes, and only the selected spots are sent', async () => {
    replaceMock.mockResolvedValue(committed())
    await state.openConventionReplace(null)
    state.toggleConventionReplaceSpot('1:đảm bảo:0')
    state.toggleConventionReplaceSpot('2:đảm bảo:9')
    await state.confirmConventionReplace()
    expect(replaceMock).toHaveBeenCalledTimes(1)
    const sent = replaceMock.mock.calls[0][0] as { segment_id: number; expected_text: string; spots: unknown[] }[]
    expect(sent.map((g) => g.segment_id)).toEqual([3, 4])
    expect(sent[0]).toEqual({
      segment_id: 3,
      expected_text: TEXT_C,
      spots: [{ start: 17, end: 24, expected_phrase: 'đảm bảo', rule_key: 'đảm bảo' }],
    })
  })

  it('zero selected writes nothing', async () => {
    await state.openConventionReplace(null)
    state.selectConventionReplaceVisible(false)
    await state.confirmConventionReplace()
    expect(replaceMock).not.toHaveBeenCalled()
    expect(state.conventionReplaceArmed.value).toBe(false)
  })

  it('a refused batch names the changed sentences by their numbers, keeps the overlay open and writes nothing locally', async () => {
    replaceMock.mockResolvedValue({
      outcome: null,
      error: {
        code: 'convention.chapter_changed',
        message_key: 'err.convention.chapter_changed',
        params: { segment_ids: '2, 3' },
        retryable: false,
      },
    })
    await state.openConventionReplace(null)
    state.toggleConventionReplaceSpot('1:đảm bảo:0')
    await state.confirmConventionReplace()
    expect(appliedMock).not.toHaveBeenCalled()
    expect(state.conventionReplaceIsOpen.value).toBe(true)
    expect(state.conventionReplaceAlert.value?.params).toEqual({ changed_count: '2', segment_list: 'câu 22, câu 30' })
    await state.confirmConventionReplace()
    expect(replaceMock).toHaveBeenCalledTimes(1)
  })
})

describe('close during the write', () => {
  it('still mirrors a committed batch into the Editor', async () => {
    let resolve: (value: unknown) => void = () => {}
    replaceMock.mockReturnValue(new Promise((r) => { resolve = r }))
    await state.openConventionReplace(null)
    await state.confirmConventionReplace()
    const pending = state.confirmConventionReplace()
    state.closeConventionReplace()
    const outcome = { outcomes: [], replaced: 1, sentences: 1, confirmed: 0, drafts: 1, chapter_started: false }
    resolve({ outcome, error: null })
    await pending
    expect(appliedMock).toHaveBeenCalledWith(outcome)
    expect(state.conventionReplaceIsOpen.value).toBe(false)
  })
})

describe('rescan', () => {
  it('keeps a deselection whose 16 units before and after are unchanged and counts it', async () => {
    await state.openConventionReplace(null)
    state.toggleConventionReplaceSpot('2:đảm bảo:9')
    await state.rescanConventionReplace()
    expect(state.isConventionReplaceSelected('2:đảm bảo:9')).toBe(false)
    expect(state.conventionReplaceTotals.value.selected).toBe(3)
    expect(state.conventionReplaceKeptNote.value).toBe(1)
  })

  it('drops a deselection once the sentence around it changed', async () => {
    await state.openConventionReplace(null)
    state.toggleConventionReplaceSpot('2:đảm bảo:9')
    const changed = 'Giờ thì nhằm bảo đảm bảo vệ đoàn xe, và một đoạn mới.'
    stubScan(() => ({ ...entries(), 2: [changed, [span(12, 19)]] }))
    await state.rescanConventionReplace()
    expect(state.conventionReplaceTotals.value.selected).toBe(4)
    expect(state.conventionReplaceKeptNote.value).toBe(0)
  })

  it('flushes again and scans with a newer generation each time', async () => {
    await state.openConventionReplace(null)
    await state.rescanConventionReplace()
    expect(flushMock).toHaveBeenCalledTimes(2)
    expect(scanMock.mock.calls[1][0]).toBeGreaterThan(scanMock.mock.calls[0][0] as number)
  })

  it('a result that arrives after the overlay closed is dropped', async () => {
    let release: (value: unknown) => void = () => undefined
    scanMock.mockImplementation(
      (generation: number) =>
        new Promise((resolve) => {
          release = () => resolve({ scan: scanOf(generation, entries()), error: null })
        }),
    )
    const opening = state.openConventionReplace(null)
    await Promise.resolve()
    await Promise.resolve()
    state.closeConventionReplace()
    release(null)
    await opening
    expect(state.conventionReplaceTotals.value.matches).toBe(0)
    expect(state.conventionReplaceIsOpen.value).toBe(false)
  })
})
