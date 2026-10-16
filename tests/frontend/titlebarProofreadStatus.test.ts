import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const runMock = vi.fn()
const chapterId = ref<number | null>(9)
const chapters = ref([{ chapter_id: 9, ord: 47 }])

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: vi.fn(),
  getProofreadDecisionCounts: () => Promise.resolve({ counts: { accepted: 0, not_error: 0, skipped: 0, by_kind: [] }, error: null }),
}))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorCaretSegmentId: ref<number | null>(null),
  editorSegments: ref([]),
  editorChapterId: chapterId,
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))
vi.mock('../../src/modes/libraryChapters', () => ({ libraryChapters: chapters }))

const proofread = await import('../../src/proofreadState')
const { proofreadStatusText } = await import('../../src/titlebarProofreadStatus')

function batch(ids: number[]) {
  return {
    kind: 'batch',
    usage: null,
    segments: ids.map((id) => ({ segment_id: id, scanned_text: 'x', findings: [], unlocated: 0, ambiguous: 0 })),
  }
}

async function scan(ids: number[]): Promise<void> {
  runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
    onEvent(batch(ids))
    return Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0 }, error: null })
  })
  await proofread.runProofread(ids)
}

beforeEach(() => {
  proofread.resetProofread()
  runMock.mockReset()
  chapterId.value = 9
  chapters.value = [{ chapter_id: 9, ord: 47 }]
})

describe('Proofreader status in the title bar', () => {
  it('is hidden before any scan', () => {
    expect(proofreadStatusText.value).toBeNull()
  })

  it('names the open Chapter and the scanned sentence count after a finished scan', async () => {
    await scan([1, 2, 3])
    expect(proofreadStatusText.value).toBe('Chương 47 · đã quét 3 câu')
  })

  it('hides when the open Chapter is no longer the listed one, and after a reset', async () => {
    await scan([1, 2, 3])
    chapterId.value = 10
    expect(proofreadStatusText.value).toBeNull()
    chapterId.value = 9
    expect(proofreadStatusText.value).not.toBeNull()
    proofread.resetProofread()
    expect(proofreadStatusText.value).toBeNull()
  })
})
