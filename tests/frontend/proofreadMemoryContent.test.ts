/**
 * "Chỗ đã bỏ qua": date and block count per row (a NULL counter shows no count), and focus
 * returning to the Proofreader strip link that opened it (UX-DR17).
 */
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { flushPromises } from './support/flushMicrotasks'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const invokeMock = vi.fn()

const ROWS = [
  { id: 3, kind: 'grammar', phrase: 'nhin', context_before: '', context_after: '', created_at: '2026-07-14T12:00:00.000Z', blocked_count: 2 },
  { id: 2, kind: 'spelling', phrase: 'sai', context_before: '', context_after: '', created_at: '2026-07-15T12:00:00.000Z', blocked_count: 0 },
  { id: 1, kind: 'spelling', phrase: 'cu', context_before: '', context_after: '', created_at: '2026-07-16T12:00:00.000Z', blocked_count: null },
]

async function fresh() {
  vi.resetModules()
  invokeMock.mockReset()
  invokeMock.mockImplementation((cmd: string) => Promise.resolve(cmd === 'ai_proofread_ignore_list' ? ROWS : null))
  const caret = ref<number | null>(5)
  const segments = ref<ChapterSegment[]>([{ ...FIXTURE_SEGMENTS[0], id: 5, ord: 5, target_text: 'toi nhin thay sai' }])
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => invokeMock(...args),
    Channel: class {
      onmessage: (v: unknown) => void = () => {}
    },
  }))
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caret,
    editorLastCaretSegmentId: caret,
    editorSegments: segments,
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
  }))
  vi.doMock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: ref<number[]>([]) }))
  const commands = await import('../../src/commands')
  const { proofreadHandlers } = await import('../../src/proofreadHandlers')
  const state = await import('../../src/proofreadState')
  const Overlay = (await import('../../src/ProofreadMemoryOverlay.vue')).default
  const Strip = (await import('../../src/ProofreaderStrip.vue')).default
  commands.installCommands({ ...proofreadHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, state, Overlay, Strip }
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('row meta', () => {
  it('shows the date and the block count, and no count for a row that predates the counter', async () => {
    const { commands, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    commands.dispatch('ai.proofread.memory.open')
    await flushPromises()
    await nextTick()

    expect(wrapper.findAll('.pfmem-meta').map((m) => m.text())).toEqual([
      'bỏ qua 14-07 · đã chặn 2 lần',
      'bỏ qua 15-07 · đã chặn 0 lần',
      'bỏ qua 16-07',
    ])
    wrapper.unmount()
  })
})

describe('focus return', () => {
  it('closing the list from the strip link puts focus back on that link', async () => {
    const { commands, state, Overlay, Strip } = await fresh()
    const cfg = await import('../../src/config/proofread')
    vi.spyOn(cfg, 'runProofreadSegments').mockImplementation(((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 1, batches: 1 })
      onEvent({
        kind: 'batch',
        usage: null,
        segments: [
          {
            segment_id: 5,
            scanned_text: 'toi nhin thay sai',
            findings: [{ kind: 'grammar', start: 4, end: 8, explanation: 'e', suggestion: 'nhìn' }],
            unlocated: 0,
            ambiguous: 0,
          },
        ],
      })
      return Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    }) as never)
    const strip = mount(Strip, { attachTo: document.body })
    const overlay = mount(Overlay, { attachTo: document.body })
    await state.runProofread([5])
    await flushPromises()

    const link = strip.get('[data-proofread-strip-memory]')
    await link.trigger('mousedown')
    await link.trigger('click')
    await flushPromises()
    await nextTick()
    expect(overlay.find('.pfmem-panel').exists()).toBe(true)
    expect(document.activeElement).not.toBe(link.element)

    commands.dispatch('ai.proofread.memory.close')
    await nextTick()
    expect(overlay.find('.pfmem-panel').exists()).toBe(false)
    expect(document.activeElement).toBe(link.element)
    strip.unmount()
    overlay.unmount()
  })
})
