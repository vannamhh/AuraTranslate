/**
 * Pending-translation strip, TM proposals (FR146): numbering, digit keys vs the free-text
 * input, Hán Việt de-duplication, lookup failure, mark meta line.
 *
 * Only the IPC boundary is faked; the real state module and `GlossaryConfirmStrip.vue` run.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type { GlossarySegmentSource } from '../../src/panels/glossaryMarksMap'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

type TmSuggestion = { target_text: string; count: number }

function markWire(overrides: Record<string, unknown> = {}) {
  return {
    start: 0,
    end: 3,
    tier: 'work',
    is_confirmed: false,
    translation: null,
    id: 7,
    source_term: '聽潮閣',
    han_viet_suggestion: 'Thính Triều Các',
    han_viet_status: 'ok',
    occurrence_count: 644,
    term_origin: 'import_scan',
    category: 'place',
    ...overrides,
  }
}

async function mountStrip(options: { mark?: Record<string, unknown>; tm?: TmSuggestion[] | Error; hidden?: number } = {}) {
  vi.resetModules()
  mockInvoke.mockReset()

  const caretSegmentId: Ref<number | null> = ref(null)
  const segments: Ref<GlossarySegmentSource[]> = ref([])
  const dispatchMock = vi.fn()

  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caretSegmentId,
    editorSegments: segments,
  }))
  vi.doMock('../../src/glossaryQuickAddState', () => ({ quickAddIsOpen: ref(false) }))
  vi.doMock('../../src/commands', () => ({ dispatch: (id: string) => dispatchMock(id) }))
  vi.doMock('../../src/config/shortcutsState', () => ({ shortcutRows: ref([]) }))

  const state = await import('../../src/glossaryConfirmStripState')
  const marksState = await import('../../src/panels/glossaryMarksState')
  const GlossaryConfirmStrip = (await import('../../src/GlossaryConfirmStrip.vue')).default

  const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '聽潮閣' }]
  segments.value = segs
  mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([markWire(options.mark)])
    if (cmd === 'glossary_tm_suggestions') {
      expect(args).toEqual({ sourceTerm: '聽潮閣' })
      const tm = options.tm ?? []
      return tm instanceof Error
        ? Promise.reject({ code: 'tm.broken', message_key: 'err.unknown', params: {}, retryable: false })
        : Promise.resolve({ suggestions: tm, hidden_count: options.hidden ?? 0 })
    }
    return Promise.reject(new Error(`unexpected command: ${cmd}`))
  })
  await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
  caretSegmentId.value = 1

  const wrapper = mount(GlossaryConfirmStrip, { attachTo: document.body })
  await vi.waitFor(() => {
    expect(mockInvoke).toHaveBeenCalledWith('glossary_tm_suggestions', { sourceTerm: '聽潮閣' })
  })
  await wrapper.vm.$nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
  return { state, wrapper, dispatchMock }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

function optionTexts(wrapper: Awaited<ReturnType<typeof mountStrip>>['wrapper']): string[] {
  return wrapper
    .findAll('.gcs-option')
    .map((b) => ['.gcs-key', '.gcs-option-text', '.gcs-option-source'].map((c) => b.get(c).text()).join(' '))
}

describe('hidden TM pairs (only-mine switch)', () => {
  it('no proposal but a hidden count ⇒ the note names the count; Hán Việt stays option 1', async () => {
    const { wrapper } = await mountStrip({ tm: [], hidden: 4 })
    expect(wrapper.get('[data-tm-hidden-note]').text()).toContain('4')
    expect(optionTexts(wrapper)).toEqual(['1 Thính Triều Các âm Hán Việt'])
    wrapper.unmount()
  })

  it('a zero hidden count shows no note', async () => {
    const { wrapper } = await mountStrip({ tm: [{ target_text: 'Gác Nghe Sóng', count: 2 }], hidden: 0 })
    expect(wrapper.find('[data-tm-hidden-note]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('numbered proposals', () => {
  it('lists Hán Việt as 1, then TM translations by count', async () => {
    const { wrapper } = await mountStrip({
      tm: [
        { target_text: 'Gác Nghe Sóng', count: 2 },
        { target_text: 'Lầu Triều', count: 1 },
      ],
    })
    expect(optionTexts(wrapper)).toEqual([
      '1 Thính Triều Các âm Hán Việt',
      '2 Gác Nghe Sóng TM · 2 lần',
      '3 Lầu Triều TM · 1 lần',
    ])
    wrapper.unmount()
  })

  it('shows one row when the TM translation equals the Hán Việt reading', async () => {
    const { wrapper } = await mountStrip({ tm: [{ target_text: 'Thính Triều Các', count: 3 }] })
    expect(optionTexts(wrapper)).toEqual(['1 Thính Triều Các âm Hán Việt · TM 3 lần'])
    wrapper.unmount()
  })

  it('numbers TM from 1 when there is no dictionary data, and keeps the dictionary notice', async () => {
    const { wrapper } = await mountStrip({
      mark: { han_viet_suggestion: null, han_viet_status: 'dict_unavailable' },
      tm: [{ target_text: 'Gác Nghe Sóng', count: 1 }],
    })
    expect(optionTexts(wrapper)).toEqual(['1 Gác Nghe Sóng TM · 1 lần'])
    expect(wrapper.find('#gcs-status-msg').exists()).toBe(true)
    wrapper.unmount()
  })

  it('keeps Hán Việt and the input and shows one error line when the TM lookup fails', async () => {
    const { wrapper } = await mountStrip({ tm: new Error('boom') })
    expect(optionTexts(wrapper)).toEqual(['1 Thính Triều Các âm Hán Việt'])
    expect(wrapper.find('.gcs-input').exists()).toBe(true)
    expect(wrapper.findAll('.gcs-error')).toHaveLength(1)
    wrapper.unmount()
  })

  it('caps the list at nine proposals', async () => {
    const tm = Array.from({ length: 12 }, (_, i) => ({ target_text: `Bản ${i}`, count: 1 }))
    const { wrapper } = await mountStrip({ tm })
    expect(wrapper.findAll('.gcs-option')).toHaveLength(9)
    wrapper.unmount()
  })
})

describe('digit keys', () => {
  const key = (code: string) => new KeyboardEvent('keydown', { code, bubbles: true, cancelable: true })

  it('digit 2 outside the input aims the second proposal and dispatches save', async () => {
    const { wrapper, dispatchMock, state } = await mountStrip({ tm: [{ target_text: 'Gác Nghe Sóng', count: 2 }] })
    wrapper.get('.glossary-confirm-strip').element.dispatchEvent(key('Digit2'))
    expect(state.confirmStripAimedOption.value).toBe(1)
    expect(dispatchMock).toHaveBeenCalledWith('glossary.confirm.save')
    wrapper.unmount()
  })

  it('a digit typed in the input is text: nothing is aimed or dispatched', async () => {
    const { wrapper, dispatchMock, state } = await mountStrip({ tm: [{ target_text: 'Gác Nghe Sóng', count: 2 }] })
    const input = wrapper.get('.gcs-input')
    input.element.dispatchEvent(key('Digit2'))
    expect(state.confirmStripAimedOption.value).toBeNull()
    expect(dispatchMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('a digit past the last proposal does nothing', async () => {
    const { wrapper, dispatchMock } = await mountStrip()
    wrapper.get('.glossary-confirm-strip').element.dispatchEvent(key('Digit5'))
    expect(dispatchMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('saving with the second proposal aimed writes that translation', async () => {
    const { wrapper, state } = await mountStrip({ tm: [{ target_text: 'Gác Nghe Sóng', count: 2 }] })
    mockInvoke.mockImplementation(() => Promise.resolve(null))
    state.aimGlossaryConfirmOption(1)
    await state.confirmGlossaryConfirmStrip(1, [{ id: 1, source_text: '聽潮閣' }], 'zh')
    expect(mockInvoke).toHaveBeenCalledWith('glossary_confirm_pending_translation', {
      tier: 'work',
      id: 7,
      translation: 'Gác Nghe Sóng',
    })
    wrapper.unmount()
  })

  it('a Hán Việt reading arriving after the aim does not move it to another translation', async () => {
    const { wrapper, state } = await mountStrip({
      mark: { han_viet_suggestion: null, han_viet_status: 'dict_unavailable' },
      tm: [
        { target_text: 'Gác Nghe Sóng', count: 2 },
        { target_text: 'Lầu Nghe Triều', count: 1 },
      ],
    })
    state.aimGlossaryConfirmOption(1)
    state.syncGlossaryConfirmStripTarget(1, [{ id: 1, source_text: '聽潮閣' }], true, [markWire() as never])
    expect(state.confirmStripOptions.value.map((o) => o.text)).toEqual(['Thính Triều Các', 'Gác Nghe Sóng', 'Lầu Nghe Triều'])

    mockInvoke.mockImplementation(() => Promise.resolve(null))
    await state.confirmGlossaryConfirmStrip(1, [{ id: 1, source_text: '聽潮閣' }], 'zh')
    expect(mockInvoke).toHaveBeenCalledWith('glossary_confirm_pending_translation', {
      tier: 'work',
      id: 7,
      translation: 'Lầu Nghe Triều',
    })
    wrapper.unmount()
  })

  it('typed text wins over the first proposal when nothing is aimed', async () => {
    const { wrapper, state } = await mountStrip()
    mockInvoke.mockImplementation(() => Promise.resolve(null))
    state.confirmStripTranslationInput.value = 'Thính Triều Lầu'
    await state.confirmGlossaryConfirmStrip(1, [{ id: 1, source_text: '聽潮閣' }], 'zh')
    expect(mockInvoke).toHaveBeenCalledWith('glossary_confirm_pending_translation', {
      tier: 'work',
      id: 7,
      translation: 'Thính Triều Lầu',
    })
    wrapper.unmount()
  })
})

describe('mark meta line', () => {
  it('shows origin and category, and drops a clause it cannot name', async () => {
    const full = await mountStrip()
    expect(full.wrapper.get('.gcs-origin').text()).toBe('xuất xứ: Quét khi nhập · phân loại: Địa danh')
    full.wrapper.unmount()

    const unknown = await mountStrip({ mark: { term_origin: 'something_new' } })
    expect(unknown.wrapper.get('.gcs-origin').text()).toBe('phân loại: Địa danh')
    unknown.wrapper.unmount()
  })

  it('shows the occurrence count inside the pending line', async () => {
    const { wrapper } = await mountStrip()
    expect(wrapper.get('.gcs-occurrence').text()).toBe('chờ chốt bản dịch · xuất hiện 644 lần trong Tác phẩm')
    wrapper.unmount()
  })
})
