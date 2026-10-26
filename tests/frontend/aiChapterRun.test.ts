import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref, shallowRef } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const invokeMock = vi.fn()

function seg(id: number, patch: Partial<ChapterSegment> = {}): ChapterSegment {
  return { ...FIXTURE_SEGMENTS[0], id, ord: id, source_text: `src ${String(id)}`, target_text: `dich ${String(id)}`, ...patch }
}

function batchCalls(): { segmentIds: number[]; skipTranslated: boolean }[] {
  return invokeMock.mock.calls
    .filter((c) => c[0] === 'ai_translate_batch')
    .map((c) => c[1] as { segmentIds: number[]; skipTranslated: boolean })
}

async function fresh() {
  vi.resetModules()
  invokeMock.mockReset()
  invokeMock.mockResolvedValue({ state: 'done', usage: null })
  const segments = ref<ChapterSegment[]>([
    seg(1),
    seg(2, { target_text: '' }),
    seg(3, { target_text: '  ' }),
    seg(4, { is_omitted: true, target_text: '' }),
    seg(5, { target_text: '' }),
  ])
  const shapes = shallowRef(new Map<number, 0 | 1 | 2>())
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => invokeMock(...args),
    Channel: class {
      onmessage: (v: unknown) => void = () => {}
    },
  }))
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: ref(null),
    editorActiveSegmentId: ref(null),
    editorSegments: segments,
    editorEditedShape: shapes,
    flushEditorBeforeDiscreteWrite: vi.fn(() => Promise.resolve('clean')),
    promoteAiTranslationToEditor: vi.fn(),
  }))
  vi.doMock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: ref([1, 2]) }))
  const commands = await import('../../src/commands')
  const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
  const scope = await import('../../src/chapterTranslateScope')
  commands.installCommands({ ...aiTranslateHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, segments, shapes, scope }
}

describe('ai.translate.chapter_run', () => {
  beforeEach(() => {
    vi.doUnmock('@tauri-apps/api/core')
  })

  it('sends only blank, non-omitted segments in document order with skipTranslated', async () => {
    const { commands, scope } = await fresh()
    expect(scope.chapterUntranslatedIds.value).toEqual([2, 3, 5])
    commands.dispatch('ai.translate.chapter_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    expect(batchCalls()[0]).toMatchObject({ segmentIds: [2, 3, 5], skipTranslated: true })
  })

  it('counts a segment being typed by its typing-buffer shape, not by the stored row', async () => {
    const { commands, shapes, scope } = await fresh()
    shapes.value = new Map([[2, 2]])
    expect(scope.chapterUntranslatedIds.value).toEqual([3, 5])
    shapes.value = new Map([[1, 0]])
    expect(scope.chapterUntranslatedIds.value).toEqual([1, 2, 3, 5])
    commands.dispatch('ai.translate.chapter_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    expect(batchCalls()[0]?.segmentIds).toEqual([1, 2, 3, 5])
  })

  it('makes no call when every segment is translated or omitted', async () => {
    const { commands, segments, scope } = await fresh()
    segments.value = [seg(1), seg(2, { is_omitted: true, target_text: '' })]
    expect(scope.chapterUntranslatedIds.value).toEqual([])
    commands.dispatch('ai.translate.chapter_run')
    await new Promise((r) => setTimeout(r, 10))
    expect(batchCalls()).toHaveLength(0)
  })

  it('leaves the selection run unchanged: skipTranslated false and the selected ids', async () => {
    const { commands } = await fresh()
    commands.dispatch('ai.translate.batch_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    expect(batchCalls()[0]).toMatchObject({ segmentIds: [1, 2], skipTranslated: false })
  })

  it('retry after a retryable error keeps the chapter mode', async () => {
    const { commands } = await fresh()
    invokeMock.mockRejectedValueOnce({
      code: 'ai_translate.provider_unreachable',
      message_key: 'err.ai_translate.provider_unreachable',
      params: { segment_id: '3' },
      retryable: true,
    })
    commands.dispatch('ai.translate.chapter_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    await new Promise((r) => setTimeout(r, 10))
    commands.dispatch('ai.translate.batch_retry')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(2)
    })
    expect(batchCalls()[1]?.skipTranslated).toBe(true)
  })

  it('retry after a selection run keeps skipTranslated false', async () => {
    const { commands } = await fresh()
    invokeMock.mockRejectedValueOnce({
      code: 'ai_translate.provider_unreachable',
      message_key: 'err.ai_translate.provider_unreachable',
      params: { segment_id: '2' },
      retryable: true,
    })
    commands.dispatch('ai.translate.batch_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    await new Promise((r) => setTimeout(r, 10))
    commands.dispatch('ai.translate.batch_retry')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(2)
    })
    expect(batchCalls()[1]?.skipTranslated).toBe(false)
  })
})
