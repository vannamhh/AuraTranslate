/**
 * The real flush path (`flushEditorNow`) driving the real `conventionState` follower.
 * Mocks sit at the IPC edge only: `config/segment` (the save) and `config/convention` (the scans).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_CHAPTER_ID, readFixture } from './support/segmentFixture'
import type { ConventionScanWire, ConventionSpanWire } from '../../src/config/convention'

type Edit = { id: number; target_text: string }

const saveCalls: Edit[][] = []
let failSave = false
const scanChapterMock = vi.fn()
const scanSegmentsMock = vi.fn()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: async (chapterId: number, edits: readonly Edit[]) => {
      saveCalls.push(edits.map((e) => ({ ...e })))
      if (failSave) {
        return { outcome: null, error: { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: true } }
      }
      return { outcome: { chapter_id: chapterId, saved: edits.length }, error: null }
    },
  }
})

vi.mock('../../src/config/convention', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/convention')>()
  return {
    ...actual,
    scanConventionChapter: (...args: unknown[]) => scanChapterMock(...args),
    scanConventionSegments: (...args: unknown[]) => scanSegmentsMock(...args),
  }
})

function span(start: number, end: number): ConventionSpanWire {
  return { start, end, rule_key: 'đảm bảo', avoid: 'đảm bảo', replacement_cased: 'bảo đảm', tier: 'global' }
}

function wire(generation: number, entries: Record<number, string>): ConventionScanWire {
  return {
    generation,
    chapter_id: FIXTURE_CHAPTER_ID,
    segments: Object.entries(entries).map(([id, text]) => ({ segment_id: Number(id), scanned_text: text, spans: [span(0, 7)] })),
    ignored: 0,
    glossary_suppressed: 0,
    overlapped: 0,
    not_scanned: 0,
  }
}

async function fresh() {
  vi.resetModules()
  const editor = await import('../../src/panels/editorPanelState')
  const convention = await import('../../src/conventionState')
  await editor.ensureSegmentsLoaded()
  scanChapterMock.mockImplementation((generation: number) =>
    Promise.resolve({ scan: wire(generation, { 11: 'Đảm bảo một.', 12: 'Đảm bảo hai.' }), error: null }),
  )
  await convention.scanConventionNow()
  scanSegmentsMock.mockReset().mockImplementation((generation: number) =>
    Promise.resolve({ scan: wire(generation, { 11: 'Đảm bảo mới.' }), error: null }),
  )
  return { editor, convention }
}

async function settle(): Promise<void> {
  for (let i = 0; i < 5; i += 1) await Promise.resolve()
}

beforeEach(() => {
  saveCalls.length = 0
  failSave = false
  scanChapterMock.mockReset()
  scanSegmentsMock.mockReset()
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('rescan after a real flush', () => {
  it('typing makes no scan call; the flush makes exactly one, for exactly the sentences it wrote', async () => {
    const { editor } = await fresh()
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    editor.noteEditorEdit(11, 'Đảm bảo mới hơn.')
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    expect(scanSegmentsMock).not.toHaveBeenCalled()
    expect(await editor.flushEditorNow()).toBe('saved')
    await settle()
    expect(scanSegmentsMock).toHaveBeenCalledTimes(1)
    expect((scanSegmentsMock.mock.calls[0] as [number, number[]])[1]).toEqual([11])
  })

  it('the sentences that were not flushed keep their entry untouched', async () => {
    const { editor, convention } = await fresh()
    const other = convention.conventionResults.value.get(12)
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    await editor.flushEditorNow()
    await settle()
    expect(convention.conventionResults.value.get(11)?.scannedText).toBe('Đảm bảo mới.')
    expect(convention.conventionResults.value.get(12)).toBe(other)
  })

  it('a flush that fails triggers no scan', async () => {
    const { editor } = await fresh()
    failSave = true
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    expect(await editor.flushEditorNow()).toBe('failed')
    await settle()
    expect(scanSegmentsMock).not.toHaveBeenCalled()
  })

  it('a rescan that fails does not fail the save, and the saved stamp still advances on the next flush', async () => {
    const { editor, convention } = await fresh()
    scanSegmentsMock.mockResolvedValue({ scan: null, error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: true } })
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    expect(await editor.flushEditorNow()).toBe('saved')
    const first = editor.editorLastSavedAt.value
    expect(first).not.toBeNull()
    await settle()
    expect(convention.conventionNotice.value?.key).toBe('convention.status.scan_failed')
    await new Promise((resolve) => setTimeout(resolve, 5))
    editor.noteEditorEdit(12, 'Đảm bảo hai mới.')
    expect(await editor.flushEditorNow()).toBe('saved')
    expect(editor.editorLastSavedAt.value).toBeGreaterThan(first as number)
  })

  it('the flush does not wait for a slow scan', async () => {
    const { editor } = await fresh()
    scanSegmentsMock.mockImplementation(() => new Promise(() => undefined))
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    await expect(editor.flushEditorNow()).resolves.toBe('saved')
  })

  it('a Chapter change while the rescan is in flight drops its result', async () => {
    const { editor, convention } = await fresh()
    let release: () => void = () => undefined
    scanSegmentsMock.mockImplementation(
      (generation: number) =>
        new Promise((resolve) => {
          release = () => resolve({ scan: wire(generation, { 11: 'Đảm bảo muộn.' }), error: null })
        }),
    )
    editor.noteEditorEdit(11, 'Đảm bảo muộn.')
    await editor.flushEditorNow()
    convention.resetConvention()
    release()
    await settle()
    expect(convention.conventionResults.value.size).toBe(0)
  })

  it('a sentence still pending after the flush reports sentence_pending on F8', async () => {
    const { editor, convention } = await fresh()
    editor.moveEditorCaretTo(13)
    editor.noteEditorEdit(13, 'đang gõ')
    expect(editor.editorSegmentIsPending(13)).toBe(true)
    convention.clearConventionFor(13)
    convention.resetConvention()
    convention.stepMarks(1)
    expect(convention.conventionNotice.value?.key).toBe('convention.status.sentence_pending')
  })

  it('a throwing follower is contained and the flush still saves', async () => {
    const { editor } = await fresh()
    const spy = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    editor.onEditorFlushed(() => {
      throw new Error('boom')
    })
    editor.noteEditorEdit(11, 'Đảm bảo mới.')
    expect(await editor.flushEditorNow()).toBe('saved')
    expect(spy).toHaveBeenCalled()
    expect(saveCalls).toHaveLength(1)
  })
})
