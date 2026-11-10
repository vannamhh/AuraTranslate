/**
 * The convention strip's two ways to leave a spot: "Bỏ qua" only moves on and calls no Rust,
 * "Không phải lỗi" is its own button wired to its own command.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import { flushPromises } from './support/flushMicrotasks'

const dispatchMock = vi.fn()
const ignoreMock = vi.fn()
const segments = ref([{ id: 1, target_text: 'Đảm bảo an toàn.', status: 'draft', is_omitted: false }])
const caret = ref<number | null>(1)

vi.mock('../../src/config/convention', () => ({
  scanConventionChapter: (generation: number) =>
    Promise.resolve({
      scan: {
        generation,
        chapter_id: 1,
        segments: [
          {
            segment_id: 1,
            scanned_text: 'Đảm bảo an toàn.',
            spans: [{ start: 0, end: 7, rule_key: 'đảm bảo', avoid: 'đảm bảo', replacement_cased: 'bảo đảm', tier: 'global' }],
          },
        ],
        ignored: 0,
        glossary_suppressed: 0,
        overlapped: 0,
        not_scanned: 0,
      },
      error: null,
    }),
  scanConventionSegments: () => Promise.resolve({ scan: null, error: null }),
  addConventionIgnore: (...args: unknown[]) => ignoreMock(...args),
}))
vi.mock('../../src/commands', () => ({ dispatch: (...args: unknown[]) => dispatchMock(...args) }))
vi.mock('../../src/config/shortcutsState', () => ({ shortcutRows: ref([]), commandTitle: (id: string) => id }))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorCaretSegmentId: caret,
  editorLastCaretSegmentId: caret,
  editorSegmentIsPending: () => false,
  editorSegments: segments,
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
  moveEditorCaretTo: () => true,
  onEditorFlushed: () => () => undefined,
  replaceEditorSegment: () => undefined,
}))
vi.mock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: ref<number[]>([]) }))
vi.mock('../../src/openWorkIdentity', () => ({ openWorkId: ref('work-1') }))

const state = await import('../../src/conventionState')
const { resetProofread } = await import('../../src/proofreadState')
const ProofreaderStrip = (await import('../../src/ProofreaderStrip.vue')).default

beforeEach(async () => {
  state.resetConvention()
  ignoreMock.mockReset().mockResolvedValue({ ok: true, error: null })
  dispatchMock.mockReset()
  await state.scanConventionNow()
  state.openConventionAt(1, 0)
})

afterEach(() => {
  resetProofread()
  document.body.innerHTML = ''
})

describe('convention strip buttons', () => {
  it('renders Bỏ qua and Không phải lỗi as two different buttons, each with its own command', async () => {
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await nextTick()
    const skip = wrapper.find('[data-convention-skip]')
    const ignore = wrapper.find('[data-convention-ignore]')
    expect(skip.exists()).toBe(true)
    expect(ignore.exists()).toBe(true)
    expect(skip.text()).toBe('Bỏ qua')
    expect(ignore.text()).toBe('Không phải lỗi')

    await skip.trigger('click')
    expect(dispatchMock).toHaveBeenLastCalledWith('convention.skip')
    await ignore.trigger('click')
    expect(dispatchMock).toHaveBeenLastCalledWith('convention.ignore')
    expect(ignoreMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('keeps focus off the button on mousedown, like the other strip buttons', async () => {
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await nextTick()
    const event = new MouseEvent('mousedown', { bubbles: true, cancelable: true })
    wrapper.find('[data-convention-ignore]').element.dispatchEvent(event)
    expect(event.defaultPrevented).toBe(true)
    wrapper.unmount()
  })

  it('disables Không phải lỗi while a write is in flight', async () => {
    let release: (value: unknown) => void = () => undefined
    ignoreMock.mockImplementation(() => new Promise((resolve) => (release = resolve)))
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await nextTick()
    const pending = state.ignoreConventionSpot()
    await flushPromises()
    await nextTick()
    expect(wrapper.find('[data-convention-ignore]').attributes('disabled')).toBeDefined()
    release({ ok: true, error: null })
    await pending
    wrapper.unmount()
  })
})
