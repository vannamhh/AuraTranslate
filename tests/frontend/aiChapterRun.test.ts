import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref, shallowRef } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const invokeMock = vi.fn()

function seg(id: number, patch: Partial<ChapterSegment> = {}): ChapterSegment {
  return { ...FIXTURE_SEGMENTS[0], id, ord: id, source_text: `src ${String(id)}`, target_text: '', ...patch }
}

type BatchArgs = {
  scope: { kind: 'selection'; segment_ids: number[] } | { kind: 'chapter' }
  promptSetName: string | null
}

function batchCalls(): BatchArgs[] {
  return invokeMock.mock.calls.filter((c) => c[0] === 'ai_translate_batch').map((c) => c[1] as BatchArgs)
}

type Channelish = { onmessage: (v: unknown) => void }

function script(events: unknown[]): void {
  invokeMock.mockImplementationOnce((_cmd: string, args: { channel: Channelish }) => {
    for (const ev of events) args.channel.onmessage(ev)
    return Promise.resolve({ state: 'done', usage: null, maybe_truncated: false })
  })
}

async function fresh() {
  vi.resetModules()
  invokeMock.mockReset()
  invokeMock.mockResolvedValue({ state: 'done', usage: null, maybe_truncated: false })
  const segments = ref<ChapterSegment[]>([seg(1), seg(2), seg(3)])
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
  const batch = await import('../../src/aiTranslateBatchState')
  const scope = await import('../../src/chapterTranslateScope')
  commands.installCommands({ ...aiTranslateHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, batch, segments, shapes, scope }
}

const PLANNED = { kind: 'planned', calls: [{ call_index: 1, segment_ids: [1, 2, 3] }] }

describe('ai.translate.chapter_run', () => {
  beforeEach(() => {
    vi.doUnmock('@tauri-apps/api/core')
  })

  it('sends scope chapter with no segment ids: Rust chooses the sentences', async () => {
    const { commands } = await fresh()
    commands.dispatch('ai.translate.chapter_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    expect(batchCalls()[0]?.scope).toEqual({ kind: 'chapter' })
    expect(JSON.stringify(batchCalls()[0])).not.toContain('skipTranslated')
  })

  it('counts only blank, non-omitted sentences, by typing-buffer shape when present', async () => {
    const { segments, shapes, scope } = await fresh()
    segments.value = [seg(1, { target_text: 'x' }), seg(2), seg(3, { target_text: '  ' }), seg(4, { is_omitted: true }), seg(5)]
    expect(scope.chapterUntranslatedIds.value).toEqual([2, 3, 5])
    shapes.value = new Map([[2, 2]])
    expect(scope.chapterUntranslatedIds.value).toEqual([3, 5])
    shapes.value = new Map([[1, 0]])
    expect(scope.chapterUntranslatedIds.value).toEqual([1, 2, 3, 5])
  })

  it('makes no call when every sentence is translated or omitted', async () => {
    const { commands, segments } = await fresh()
    segments.value = [seg(1, { target_text: 'x' }), seg(2, { is_omitted: true })]
    commands.dispatch('ai.translate.chapter_run')
    await new Promise((r) => setTimeout(r, 10))
    expect(batchCalls()).toHaveLength(0)
  })

  it('selection run sends scope selection with the selected ids', async () => {
    const { commands } = await fresh()
    commands.dispatch('ai.translate.batch_run')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(1)
    })
    expect(batchCalls()[0]?.scope).toEqual({ kind: 'selection', segment_ids: [1, 2] })
  })

  it('retry after a chapter run is a selection run over missing and not_sent rows only', async () => {
    const { commands, batch } = await fresh()
    script([
      PLANNED,
      { kind: 'call_started', call_index: 1 },
      { kind: 'received', call_index: 1, segment_id: 1, text: 'Một', has_han: false, source_hash: 'h1' },
      { kind: 'discarded', call_index: 1, segment_id: 2, reason: 'missing' },
    ])
    commands.dispatch('ai.translate.chapter_run')
    await vi.waitFor(() => {
      expect(batch.aiTranslateBatchStateValue.value).toBe('done')
    })
    expect(batch.aiTranslateBatchRows.value.map((r) => r.status)).toEqual(['received', 'missing', 'not_sent'])

    commands.dispatch('ai.translate.batch_retry')
    await vi.waitFor(() => {
      expect(batchCalls()).toHaveLength(2)
    })
    expect(batchCalls()[1]?.scope).toEqual({ kind: 'selection', segment_ids: [2, 3] })
  })

  it('does not retry when nothing is missing or unsent', async () => {
    const { commands, batch } = await fresh()
    script([
      { kind: 'planned', calls: [{ call_index: 1, segment_ids: [1] }] },
      { kind: 'received', call_index: 1, segment_id: 1, text: 'Một', has_han: false, source_hash: 'h1' },
    ])
    commands.dispatch('ai.translate.chapter_run')
    await vi.waitFor(() => {
      expect(batch.aiTranslateBatchStateValue.value).toBe('done')
    })
    commands.dispatch('ai.translate.batch_retry')
    await new Promise((r) => setTimeout(r, 10))
    expect(batchCalls()).toHaveLength(1)
  })
})
