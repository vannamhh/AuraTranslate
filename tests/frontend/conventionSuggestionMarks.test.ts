import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { ConventionSpanWire, ConventionSuggestionScanWire } from '../../src/config/convention'

type Row = { segmentId: number; status: string; text: string }

const scanMock = vi.fn()
const singleState = ref('idle')
const singleText = ref('')
const singleSegment = ref<number | null>(null)
const batchState = ref('not_configured')
const batchRows = ref<Row[]>([])
const generation = ref(1)

vi.mock('../../src/config/convention', () => ({
  scanConventionSuggestions: (...args: unknown[]) => scanMock(...args),
}))
vi.mock('../../src/aiTranslateState', () => ({
  aiTranslateStateValue: singleState,
  aiTranslateAccumulatedText: singleText,
  aiTranslateRunSegmentId: singleSegment,
}))
vi.mock('../../src/aiTranslateBatchState', () => ({
  aiTranslateBatchStateValue: batchState,
  aiTranslateBatchRows: batchRows,
}))
vi.mock('../../src/conventionState', () => ({ conventionGeneration: generation }))

const marks = await import('../../src/conventionSuggestionMarks')

function span(start: number, end: number, replacement: string | null = 'bảo đảm'): ConventionSpanWire {
  return { start, end, rule_key: 'đảm bảo', avoid: 'đảm bảo', replacement_cased: replacement, tier: 'global' }
}

function scanOf(
  items: { segment_id: number; scanned_text: string; spans: ConventionSpanWire[] }[],
  gen = generation.value,
): ConventionSuggestionScanWire {
  return { generation: gen, chapter_id: 1, items, ignored: 0, glossary_suppressed: 0, overlapped: 0 }
}

async function settle(): Promise<void> {
  for (let i = 0; i < 5; i += 1) await Promise.resolve()
}

beforeEach(async () => {
  singleState.value = 'idle'
  singleText.value = ''
  singleSegment.value = null
  batchState.value = 'not_configured'
  batchRows.value = []
  generation.value += 1
  marks.resetConventionSuggestions()
  scanMock.mockReset()
  await settle()
})

describe('one-sentence suggestion', () => {
  it('a suggestion that is still streaming is never scanned and has no mark', async () => {
    singleSegment.value = 5
    singleText.value = 'Đảm bảo an toàn.'
    singleState.value = 'generating'
    await settle()
    expect(scanMock).not.toHaveBeenCalled()
    expect(marks.conventionSuggestionSingle.value).toBeNull()
  })

  it('a finished suggestion is scanned once with its segment id and text, and gets a mark', async () => {
    singleSegment.value = 5
    singleText.value = 'Đảm bảo an toàn.'
    scanMock.mockResolvedValue({ scan: scanOf([{ segment_id: 5, scanned_text: 'Đảm bảo an toàn.', spans: [span(0, 7)] }]), error: null })
    singleState.value = 'generating'
    singleState.value = 'done'
    await settle()
    expect(scanMock).toHaveBeenCalledTimes(1)
    expect(scanMock).toHaveBeenCalledWith(generation.value, [{ segment_id: 5, text: 'Đảm bảo an toàn.' }])
    expect(marks.conventionSuggestionSingle.value?.segmentId).toBe(5)
    const parts = marks.conventionSuggestionParts('Đảm bảo an toàn.', marks.conventionSuggestionSingle.value)
    expect(parts.map((p) => [p.text, p.title !== null])).toEqual([['Đảm bảo', true], [' an toàn.', false]])
  })

  it('a new run drops the previous marks before it streams', async () => {
    singleSegment.value = 5
    singleText.value = 'Đảm bảo an toàn.'
    scanMock.mockResolvedValue({ scan: scanOf([{ segment_id: 5, scanned_text: 'Đảm bảo an toàn.', spans: [span(0, 7)] }]), error: null })
    singleState.value = 'done'
    await settle()
    expect(marks.conventionSuggestionSingle.value).not.toBeNull()
    singleState.value = 'generating'
    expect(marks.conventionSuggestionSingle.value).toBeNull()
  })

  it('a result that arrives after the generation moved is dropped', async () => {
    singleSegment.value = 5
    singleText.value = 'Đảm bảo an toàn.'
    let release: () => void = () => undefined
    scanMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          release = () => resolve({ scan: scanOf([{ segment_id: 5, scanned_text: 'Đảm bảo an toàn.', spans: [span(0, 7)] }]), error: null })
        }),
    )
    singleState.value = 'done'
    generation.value += 1
    release()
    await settle()
    expect(marks.conventionSuggestionSingle.value).toBeNull()
  })

  it('a failed scan leaves no mark and throws nothing', async () => {
    singleSegment.value = 5
    singleText.value = 'Đảm bảo an toàn.'
    scanMock.mockResolvedValue({ scan: null, error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: true } })
    singleState.value = 'done'
    await settle()
    expect(marks.conventionSuggestionSingle.value).toBeNull()
  })

  it('a mark never applies to text other than the text it scanned', () => {
    const entry = { scannedText: 'Đảm bảo an toàn.', spans: [span(0, 7)] }
    expect(marks.conventionSuggestionParts('Đảm bảo an toàn!', entry)).toEqual([{ text: 'Đảm bảo an toàn!', title: null }])
    expect(marks.conventionSuggestionParts('x', null)).toEqual([{ text: 'x', title: null }])
  })

  it('a rule-list or Chapter change clears the marks', async () => {
    singleSegment.value = 5
    singleText.value = 'Đảm bảo an toàn.'
    scanMock.mockResolvedValue({ scan: scanOf([{ segment_id: 5, scanned_text: 'Đảm bảo an toàn.', spans: [span(0, 7)] }]), error: null })
    singleState.value = 'done'
    await settle()
    generation.value += 1
    expect(marks.conventionSuggestionSingle.value).toBeNull()
  })
})

describe('batch suggestions', () => {
  function fiftyRows(): Row[] {
    return Array.from({ length: 50 }, (_, i) => ({
      segmentId: 100 + i,
      status: i % 5 === 0 ? 'queued' : i % 7 === 0 ? 'missing' : 'received',
      text: i % 5 === 0 || i % 7 === 0 ? `Nguồn ${i}` : `Đảm bảo ${i}`,
    }))
  }

  it('a 50-row batch makes one call carrying only the received rows, and marks only those', async () => {
    const rows = fiftyRows()
    batchRows.value = rows
    const received = rows.filter((r) => r.status === 'received')
    scanMock.mockResolvedValue({
      scan: scanOf(received.map((r) => ({ segment_id: r.segmentId, scanned_text: r.text, spans: [span(0, 7)] }))),
      error: null,
    })
    batchState.value = 'generating'
    batchState.value = 'done'
    await settle()
    expect(scanMock).toHaveBeenCalledTimes(1)
    const items = (scanMock.mock.calls[0] as [number, { segment_id: number; text: string }[]])[1]
    expect(items.map((i) => i.segment_id)).toEqual(received.map((r) => r.segmentId))
    expect(items.length).toBeLessThan(50)
    const queued = rows.find((r) => r.status === 'queued')
    const missing = rows.find((r) => r.status === 'missing')
    expect(marks.conventionSuggestionBatch.value.size).toBe(received.length)
    expect(marks.conventionSuggestionBatch.value.has(queued?.segmentId ?? -1)).toBe(false)
    expect(marks.conventionSuggestionBatch.value.has(missing?.segmentId ?? -1)).toBe(false)
  })

  it('rows still generating never trigger a call', async () => {
    batchRows.value = fiftyRows()
    batchState.value = 'generating'
    await settle()
    expect(scanMock).not.toHaveBeenCalled()
  })

  it('a batch with no received row makes no call', async () => {
    batchRows.value = [{ segmentId: 1, status: 'missing', text: 'Nguồn' }]
    batchState.value = 'done'
    await settle()
    expect(scanMock).not.toHaveBeenCalled()
  })

  it('a new batch run drops the previous batch marks', async () => {
    batchRows.value = [{ segmentId: 1, status: 'received', text: 'Đảm bảo' }]
    scanMock.mockResolvedValue({ scan: scanOf([{ segment_id: 1, scanned_text: 'Đảm bảo', spans: [span(0, 7)] }]), error: null })
    batchState.value = 'done'
    await settle()
    expect(marks.conventionSuggestionBatch.value.size).toBe(1)
    batchState.value = 'generating'
    expect(marks.conventionSuggestionBatch.value.size).toBe(0)
  })
})
