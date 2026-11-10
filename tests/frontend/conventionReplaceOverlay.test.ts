import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { ConventionScanWire } from '../../src/config/convention'

const dispatchMock = vi.fn()
const scanMock = vi.fn()
const segments = ref([
  { id: 1, ord: 9, target_text: 'Đảm bảo rằng ổn.', status: 'draft' },
  { id: 2, ord: 22, target_text: 'nhằm bảo đảm bảo vệ', status: 'confirmed' },
])

vi.mock('../../src/commands', () => ({ dispatch: (...args: unknown[]) => dispatchMock(...args) }))
vi.mock('../../src/commands/focus', () => ({ focusReturnTargetOnOpen: () => null }))
vi.mock('../../src/config/convention', () => ({
  scanConventionSegments: () => Promise.resolve({ scan: null, error: null }),
  scanConventionChapterWithProgress: (...args: unknown[]) => scanMock(...args),
  replaceConventionChapter: vi.fn(),
}))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorSegments: segments,
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))
vi.mock('../../src/conventionState', () => ({ applyConventionReplaced: vi.fn() }))

const state = await import('../../src/conventionReplaceState')
const Overlay = (await import('../../src/ConventionReplaceOverlay.vue')).default

function scan(generation: number): ConventionScanWire {
  const span = (start: number, end: number) => ({
    start,
    end,
    rule_key: 'đảm bảo',
    avoid: 'đảm bảo',
    replacement_cased: 'bảo đảm',
    tier: 'global' as const,
  })
  return {
    generation,
    chapter_id: 1,
    segments: [
      { segment_id: 1, scanned_text: 'Đảm bảo rằng ổn.', spans: [span(0, 7)] },
      { segment_id: 2, scanned_text: 'nhằm bảo đảm bảo vệ', spans: [span(9, 16)] },
    ],
    ignored: 0,
    glossary_suppressed: 0,
    overlapped: 0,
    not_scanned: 0,
  }
}

beforeEach(() => {
  state.closeConventionReplace()
  dispatchMock.mockReset()
  scanMock.mockReset().mockImplementation((generation: number) => Promise.resolve({ scan: scan(generation), error: null }))
})

afterEach(() => {
  state.closeConventionReplace()
})

describe('ConventionReplaceOverlay', () => {
  it('lists every spot selected, labels draft and confirmed sentences in words, and says nothing was filtered', async () => {
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openConventionReplace(null)
    await flushPromises()
    const spots = wrapper.findAll('[data-convention-replace-spot]')
    expect(spots.map((s) => s.attributes('aria-selected'))).toEqual(['true', 'true'])
    const text = wrapper.text()
    expect(text).toContain('Câu 9')
    expect(text).toContain('Bản nháp — không hoàn tác được')
    expect(text).toContain('Đã xác nhận')
    expect(wrapper.get('[data-convention-replace-hidden]').text()).toBe('Không có chỗ nào bị lọc.')
    wrapper.unmount()
  })

  it('arrow down then Space deselects the spot under the cursor and shows the sentence whole', async () => {
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openConventionReplace(null)
    await flushPromises()
    const list = wrapper.get('[data-convention-replace-list]')
    await list.trigger('keydown', { key: 'ArrowDown' })
    await list.trigger('keydown', { key: ' ' })
    const spots = wrapper.findAll('[data-convention-replace-spot]')
    expect(spots[1]?.attributes('aria-selected')).toBe('false')
    expect(spots[1]?.text()).toBe('nhằm bảo đảm bảo vệ')
    expect(spots[0]?.text()).toContain('Đảm bảo')
    expect(wrapper.get('[data-convention-replace-summary]').text()).toContain('đã chọn 1')
    wrapper.unmount()
  })

  it('Esc undoes a pending second press first and only then dispatches the close command', async () => {
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openConventionReplace(null)
    await flushPromises()
    await state.confirmConventionReplace()
    await flushPromises()
    expect(wrapper.find('[data-convention-replace-armed]').exists()).toBe(true)
    expect(wrapper.get('[data-convention-replace-confirm]').text()).toBe('Xác nhận thay 2 chỗ')
    await wrapper.get('.crv-scrim').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[data-convention-replace-armed]').exists()).toBe(false)
    expect(dispatchMock).not.toHaveBeenCalled()
    await wrapper.get('.crv-scrim').trigger('keydown', { key: 'Escape' })
    expect(dispatchMock).toHaveBeenCalledWith('convention.replace_chapter_close')
    wrapper.unmount()
  })

  it('the primary button locks at zero selected and every button dispatches a command', async () => {
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openConventionReplace(null)
    await flushPromises()
    await wrapper.get('[data-convention-replace-select-none]').trigger('click')
    expect(dispatchMock).toHaveBeenLastCalledWith('convention.replace_chapter_select_none')
    state.selectConventionReplaceVisible(false)
    await flushPromises()
    expect(wrapper.get('[data-convention-replace-confirm]').attributes('disabled')).toBeDefined()
    await wrapper.get('[data-convention-replace-close]').trigger('click')
    expect(dispatchMock).toHaveBeenLastCalledWith('convention.replace_chapter_close')
    wrapper.unmount()
  })

  it('a refused batch shows the alert frame with Quét lại and Đóng instead of the primary button', async () => {
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openConventionReplace(null)
    await flushPromises()
    const convention = await import('../../src/config/convention')
    vi.mocked(convention.replaceConventionChapter).mockResolvedValue({
      outcome: null,
      error: {
        code: 'convention.chapter_changed',
        message_key: 'err.convention.chapter_changed',
        params: { segment_ids: '2' },
        retryable: false,
      },
    })
    state.toggleConventionReplaceSpot('1:đảm bảo:0')
    await state.confirmConventionReplace()
    await flushPromises()
    expect(wrapper.get('[data-convention-replace-refused]').text()).toContain('câu 22')
    expect(wrapper.find('[data-convention-replace-confirm]').exists()).toBe(false)
    expect(wrapper.find('[data-convention-replace-rescan]').exists()).toBe(true)
    wrapper.unmount()
  })
})
