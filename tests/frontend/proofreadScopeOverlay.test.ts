import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const invokeMock = vi.fn()

function seg(id: number, patch: Partial<ChapterSegment> = {}): ChapterSegment {
  return { ...FIXTURE_SEGMENTS[0], id, ord: id, target_text: `dich ${String(id)}`, ...patch }
}

async function fresh() {
  vi.resetModules()
  invokeMock.mockReset()
  invokeMock.mockResolvedValue({ state: 'done', not_scanned: 0 })
  const caret = ref<number | null>(3)
  const segments = ref<ChapterSegment[]>(
    Array.from({ length: 30 }, (_, i) => {
      const id = i + 1
      if (id <= 3) return seg(id, { target_text: '' })
      if (id === 4) return seg(id, { is_omitted: true })
      return seg(id)
    }),
  )
  const selection = ref<number[]>([])
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => invokeMock(...args),
    Channel: class {
      onmessage: (v: unknown) => void = () => {}
    },
  }))
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caret,
    editorSegments: segments,
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
  }))
  vi.doMock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: selection }))
  const commands = await import('../../src/commands')
  const { proofreadHandlers } = await import('../../src/proofreadHandlers')
  const state = await import('../../src/proofreadState')
  const Overlay = (await import('../../src/ProofreadScopeOverlay.vue')).default
  commands.installCommands({ ...proofreadHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, caret, selection, state, Overlay }
}

function scanCalls(): { segmentIds: number[] }[] {
  return invokeMock.mock.calls.filter((c) => c[0] === 'ai_proofread_segments').map((c) => c[1] as { segmentIds: number[] })
}

beforeEach(() => {
  vi.doUnmock('@tauri-apps/api/core')
})
afterEach(() => {
  document.body.innerHTML = ''
})

describe('ProofreadScopeOverlay', () => {
  it('renders nothing until opened, then shows three cells with the number of sentences each will scan', async () => {
    const { state, Overlay, selection } = await fresh()
    selection.value = [1, 2, 5, 6]
    const wrapper = mount(Overlay, { attachTo: document.body })
    expect(wrapper.find('[data-proofread-scope]').exists()).toBe(false)
    state.openProofreadScope()
    await nextTick()
    const cell = (kind: string) => wrapper.find(`[data-proofread-scope="${kind}"]`)
    expect(cell('segment').text()).toContain('0')
    expect(cell('selection').text()).toContain('2')
    expect(cell('chapter').text()).toContain('26')
    wrapper.unmount()
  })

  it('disables the selection cell when nothing is selected', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    state.openProofreadScope()
    await nextTick()
    expect(wrapper.find('[data-proofread-scope="selection"]').attributes('disabled')).toBeDefined()
    expect(wrapper.find('[data-proofread-scope="chapter"]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it.each([
    ['selection', [5, 6, 7]],
    ['chapter', Array.from({ length: 30 }, (_, i) => i + 1)],
  ] as const)('the %s cell dispatches its own command and scans that scope', async (kind, ids) => {
    const { state, Overlay, selection } = await fresh()
    selection.value = [5, 6, 7]
    const wrapper = mount(Overlay, { attachTo: document.body })
    state.openProofreadScope()
    await nextTick()
    const button = wrapper.find(`[data-proofread-scope="${kind}"]`)
    expect(button.attributes('disabled')).toBeUndefined()
    await button.trigger('click')
    await vi.waitFor(() => {
      expect(scanCalls()).toHaveLength(1)
    })
    expect(scanCalls()[0]?.segmentIds).toEqual(ids)
    expect(state.proofreadScopeIsOpen.value).toBe(false)
    wrapper.unmount()
  })

  it('the segment cell scans the caret segment when it is scannable', async () => {
    const { state, Overlay, caret } = await fresh()
    caret.value = 9
    const wrapper = mount(Overlay, { attachTo: document.body })
    state.openProofreadScope()
    await nextTick()
    await wrapper.find('[data-proofread-scope="segment"]').trigger('click')
    await vi.waitFor(() => {
      expect(scanCalls()).toHaveLength(1)
    })
    expect(scanCalls()[0]?.segmentIds).toEqual([9])
    wrapper.unmount()
  })

  it('keeps the last caret segment for the segment scope when a pointer-opened chooser nulls the caret', async () => {
    const { state, Overlay, caret } = await fresh()
    caret.value = 9
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    caret.value = null
    state.openProofreadScope()
    await nextTick()
    const cell = wrapper.find('[data-proofread-scope="segment"]')
    expect(cell.attributes('disabled')).toBeUndefined()
    await cell.trigger('click')
    await vi.waitFor(() => {
      expect(scanCalls()).toHaveLength(1)
    })
    expect(scanCalls()[0]?.segmentIds).toEqual([9])
    wrapper.unmount()
  })

  it('Escape and the close button close it without a scan', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    state.openProofreadScope()
    await nextTick()
    await wrapper.find('.pfscope-close').trigger('click')
    expect(state.proofreadScopeIsOpen.value).toBe(false)
    state.openProofreadScope()
    await nextTick()
    await wrapper.find('.pfscope-scrim').trigger('keydown', { key: 'Escape' })
    expect(state.proofreadScopeIsOpen.value).toBe(false)
    expect(scanCalls()).toHaveLength(0)
    wrapper.unmount()
  })
})
