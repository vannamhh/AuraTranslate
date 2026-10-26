import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'

const promptSetListMock = vi.fn()
const promptSetCreateMock = vi.fn()

vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...args: unknown[]) => promptSetListMock(...args),
  promptSetCreate: (...args: unknown[]) => promptSetCreateMock(...args),
  promptSetRename: vi.fn(),
  promptSetUpdateBody: vi.fn(),
  promptSetDelete: vi.fn(),
  promptSetExport: vi.fn(),
}))

type Row = { id: number; name: string; body: string; tier: 'global' | 'work'; shadowed_body: string | null; shadowed_id: number | null }
const row = (over: Partial<Row>): Row => ({ id: 1, name: 'X', body: 'b', tier: 'global', shadowed_body: null, shadowed_id: null, ...over })

async function openPicker(sets: Row[], workTierAvailable = false) {
  vi.resetModules()
  document.body.innerHTML = ''
  promptSetListMock.mockReset()
  promptSetCreateMock.mockReset()
  promptSetListMock.mockResolvedValue({ sets, workTierAvailable, variables: [], error: null })
  promptSetCreateMock.mockResolvedValue({ id: 1, warnings: null, error: null })
  const state = await import('../../src/promptSetState')
  const Picker = (await import('../../src/PromptTemplatePicker.vue')).default
  const i18n = await import('../../src/i18n')
  await state.loadPromptSets()
  const wrapper = mount(Picker, { attachTo: document.body })
  await flushPromises()
  return { wrapper, state, i18n }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('PromptTemplatePicker.vue', () => {
  it('liệt kê 29 mẫu theo nhãn thể loại và xem được thân từng mẫu', async () => {
    const { wrapper } = await openPicker([])
    const items = wrapper.findAll('li[data-template-id]')
    expect(items).toHaveLength(29)
    const { PROMPT_TEMPLATES } = await import('../../src/promptTemplates')
    const { genreLabel } = await import('../../src/modes/libraryWorkCard')
    expect(items[0]!.text()).toContain(genreLabel(items[0]!.attributes('data-template-id') as never))
    expect(wrapper.find('[data-template-preview-body]').exists()).toBe(false)
    await wrapper.get('li[data-template-id="legal"] form').trigger('submit')
    expect(wrapper.get('[data-template-preview-body] pre').text()).toBe(PROMPT_TEMPLATES.legal.trim())
  })

  it('mẫu đã có hiện "Đã có" và bị khoá', async () => {
    const { wrapper, i18n } = await openPicker([row({ name: 'tiên hiệp' })])
    const li = wrapper.get('li[data-template-id="xianxia"]')
    expect(li.get('[data-template-exists]').text()).toBe(i18n.t('prompt.library.template_exists_badge'))
    expect((li.get('[data-template-check]').element as HTMLInputElement).disabled).toBe(true)
    expect(wrapper.find('li[data-template-id="wuxia"] [data-template-exists]').exists()).toBe(false)
  })

  it('đếm N theo số mẫu được chọn, "Chọn mọi mẫu còn thiếu" chọn 28, thêm thì tạo 28 bộ', async () => {
    const { wrapper, i18n } = await openPicker([row({ name: 'Tiên hiệp' })])
    const add = wrapper.get('[data-template-add]')
    expect((add.element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('li[data-template-id="wuxia"] [data-template-check]').setValue(true)
    expect(add.text()).toBe(i18n.t('prompt.library.template_add_button', { count: '1' }))
    await wrapper.get('[data-template-pick-missing]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await wrapper.vm.$nextTick()
    expect(add.text()).toBe(i18n.t('prompt.library.template_add_button', { count: '28' }))
    await add.element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await vi.waitFor(() => expect(promptSetCreateMock).toHaveBeenCalledTimes(28))
    expect(promptSetCreateMock.mock.calls.every((c) => c[0] === 'global')).toBe(true)
  })

  it('đã có cả 29 mẫu: mọi mẫu "Đã có", không còn gì để thêm', async () => {
    const { genreLabel, GENRE_IDS } = await import('../../src/modes/libraryWorkCard')
    const sets = GENRE_IDS.map((id, i) => row({ id: i + 1, name: genreLabel(id) ?? id }))
    const { wrapper } = await openPicker(sets)
    expect(wrapper.findAll('[data-template-exists]')).toHaveLength(29)
    expect(wrapper.find('[data-template-all-present]').exists()).toBe(true)
    expect((wrapper.get('[data-template-pick-missing]').element as HTMLButtonElement).disabled).toBe(true)
    expect((wrapper.get('[data-template-add]').element as HTMLButtonElement).disabled).toBe(true)
  })

  it('tầng Tác phẩm ẩn khi không có Tác phẩm, hiện khi có; đổi sang Tác phẩm tạo ở tầng work', async () => {
    const none = await openPicker([], false)
    expect(none.wrapper.find('[data-template-tier]').exists()).toBe(false)
    const some = await openPicker([], true)
    expect(some.wrapper.find('[data-template-tier="work"]').exists()).toBe(true)
    await some.wrapper.get('[data-template-tier="work"]').setValue(true)
    await some.wrapper.get('li[data-template-id="wuxia"] [data-template-check]').setValue(true)
    await some.wrapper.get('[data-template-add]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(promptSetCreateMock.mock.calls[0]![0]).toBe('work')
  })

  it('Thư viện prompt có hàng mở bộ chọn mẫu', async () => {
    vi.resetModules()
    promptSetListMock.mockResolvedValue({ sets: [], workTierAvailable: false, variables: [], error: null })
    const state = await import('../../src/promptSetState')
    const lib = await import('../../src/promptLibraryState')
    const Overlay = (await import('../../src/PromptLibraryOverlay.vue')).default
    lib.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()
    await wrapper.get('[data-prompt-template-open]').trigger('submit')
    await flushPromises()
    expect(wrapper.findAll('li[data-template-id]')).toHaveLength(29)
    wrapper.unmount()
    lib.resetPromptLibrary()
    state.resetPromptSets()
  })

  async function openOverlay() {
    vi.resetModules()
    document.body.innerHTML = ''
    promptSetListMock.mockReset()
    promptSetCreateMock.mockReset()
    promptSetListMock.mockResolvedValue({ sets: [], workTierAvailable: false, variables: [], error: null })
    const state = await import('../../src/promptSetState')
    const lib = await import('../../src/promptLibraryState')
    const Overlay = (await import('../../src/PromptLibraryOverlay.vue')).default
    lib.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()
    await wrapper.get('[data-prompt-template-open]').trigger('submit')
    await flushPromises()
    return { wrapper, lib, state }
  }

  it('Escape trong Thư viện prompt đóng bộ chọn mẫu trước, lớp phủ vẫn mở', async () => {
    const { wrapper, lib, state } = await openOverlay()
    expect(wrapper.find('[data-prompt-template-picker]').exists()).toBe(true)
    await wrapper.get('.pl-scrim').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(wrapper.find('[data-prompt-template-picker]').exists()).toBe(false)
    expect(wrapper.find('.pl-scrim').exists()).toBe(true)
    wrapper.unmount()
    lib.resetPromptLibrary()
    state.resetPromptSets()
  })

  it('tạo lỗi: hiện dòng role=alert qua tError, không có dòng "đã thêm"', async () => {
    const { wrapper, i18n } = await openPicker([])
    promptSetCreateMock.mockResolvedValue({ warnings: null, error: { code: 'io.write', message_key: 'err.unknown', params: {}, retryable: false } })
    await wrapper.get('li[data-template-id="wuxia"] [data-template-check]').setValue(true)
    await wrapper.get('[data-template-add]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    const alert = wrapper.get('[role="alert"]')
    expect(alert.text()).toBe(i18n.tError({ code: 'io.write', message_key: 'err.unknown', params: {}, retryable: false } as never))
    expect(wrapper.find('[data-template-added]').exists()).toBe(false)
  })
})
