import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'

const listMock = vi.fn()
const createMock = vi.fn()
const deleteMock = vi.fn()
const choiceSetMock = vi.fn(async (..._args: unknown[]) => null as unknown)
const proofreadChoiceGetMock = vi.fn()
const setGlobalMock = vi.fn(async (..._args: unknown[]) => null as unknown)
const setWorkMock = vi.fn(async (..._args: unknown[]) => null as unknown)
const consequenceMock = vi.fn()
const openImportMock = vi.fn()
const confirmImportMock = vi.fn()

vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...a: unknown[]) => listMock(...a),
  promptSetCreate: (...a: unknown[]) => createMock(...a),
  promptSetRename: vi.fn(async () => null),
  promptSetUpdateBody: vi.fn(async () => ({ warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })),
  promptSetDelete: (...a: unknown[]) => deleteMock(...a),
  promptSetExport: vi.fn(),
  promptSetExportMany: vi.fn(),
  promptSetChoiceGet: async () => ({ name: 'Tiên hiệp', error: null }),
  promptSetChoiceSet: (...a: unknown[]) => choiceSetMock(...a),
  promptSetProofreadChoiceGet: (...a: unknown[]) => proofreadChoiceGetMock(...a),
  promptSetProofreadChoiceSetGlobal: (...a: unknown[]) => setGlobalMock(...a),
  promptSetProofreadChoiceSetWork: (...a: unknown[]) => setWorkMock(...a),
  promptSetProofreadDeleteConsequence: (...a: unknown[]) => consequenceMock(...a),
  promptSetOpenImportPreview: (...a: unknown[]) => openImportMock(...a),
  promptSetConfirmImport: (...a: unknown[]) => confirmImportMock(...a),
  promptSetCancelImport: vi.fn(async () => null),
}))

type Row = { id: number; name: string; body: string; tier: 'global' | 'work'; shadowed_body: null; shadowed_id: null }
const row = (over: Partial<Row>): Row => ({
  id: 1,
  name: 'x',
  body: '',
  tier: 'global',
  shadowed_body: null,
  shadowed_id: null,
  ...over,
})

const TRANSLATE_SET = row({ id: 1, name: 'Tiên hiệp', body: '{{glossary_terms}} {{source_segment}}' })
const PROOFREAD_SET = row({ id: 2, name: 'Soát câu', body: 'so với {{source_segment}}' })

function noWarn() {
  return { unknown_markers: [], glossary_terms_missing: false }
}

beforeEach(() => {
  document.body.innerHTML = ''
  for (const m of [listMock, createMock, deleteMock, choiceSetMock, proofreadChoiceGetMock, setGlobalMock, setWorkMock, consequenceMock, openImportMock, confirmImportMock]) {
    m.mockClear()
  }
  listMock.mockImplementation(async (kind: string) =>
    kind === 'proofread'
      ? { sets: [PROOFREAD_SET], workTierAvailable: true, variables: [], error: null }
      : { sets: [TRANSLATE_SET], workTierAvailable: true, variables: ['glossary_terms', 'source_segment'], error: null },
  )
  proofreadChoiceGetMock.mockResolvedValue({
    choice: { global: { name: 'Soát câu', missing: false }, work: { mode: 'follow_global', name: null, missing: false } },
    error: null,
  })
  createMock.mockResolvedValue({ id: 3, warnings: noWarn(), error: null })
  deleteMock.mockResolvedValue(null)
  consequenceMock.mockResolvedValue({ consequence: 'none', error: null })
})

async function openOverlay() {
  vi.resetModules()
  const state = await import('../../src/promptSetState')
  const libraryState = await import('../../src/promptLibraryState')
  const importState = await import('../../src/promptSetImportState')
  const Overlay = (await import('../../src/PromptLibraryOverlay.vue')).default
  libraryState.openPromptLibrary()
  await flushPromises()
  const wrapper = mount(Overlay, { attachTo: document.body })
  await flushPromises()
  return { state, libraryState, importState, wrapper }
}

async function toProofreadTab(wrapper: Awaited<ReturnType<typeof openOverlay>>['wrapper']) {
  await wrapper.find('[data-prompt-kind-option="proofread"]').setValue(true)
  await flushPromises()
}

describe('thư viện prompt — thẻ chỉ dẫn Proofreader (9-15)', () => {
  it('đổi thẻ, tạo, sửa, xoá bộ chỉ dẫn không chạm lựa chọn bộ dịch', async () => {
    const { state, wrapper } = await openOverlay()
    await state.restorePromptSetForWork('')
    await flushPromises()
    expect(state.selectedPromptSetName.value).toBe('Tiên hiệp')
    choiceSetMock.mockClear()

    await toProofreadTab(wrapper)
    await wrapper.find('.pl-row-form').trigger('submit')
    await flushPromises()
    await wrapper.find('.pl-delete-form').trigger('submit')
    await flushPromises()
    await wrapper.find('.pl-delete-form').trigger('submit')
    await flushPromises()
    await wrapper.find('[data-prompt-kind-option="translate"]').setValue(true)
    await flushPromises()

    expect(deleteMock).toHaveBeenCalledWith('global', 'proofread', 2)
    expect(choiceSetMock).not.toHaveBeenCalled()
    expect(state.selectedPromptSetName.value).toBe('Tiên hiệp')
    wrapper.unmount()
  })

  it('thân chứa marker: không chip biến, không tô, không cảnh báo', async () => {
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    await wrapper.find('.pl-row-form').trigger('submit')
    await flushPromises()

    expect(wrapper.find('.pl-vars').exists()).toBe(false)
    expect(wrapper.find('.pl-var').exists()).toBe(false)
    expect(wrapper.find('.pl-warn').exists()).toBe(false)
    expect(wrapper.find('.pl-ta-mirror').text()).toContain('{{source_segment}}')
    wrapper.unmount()
  })

  it('tên mất do Rust báo hiện kèm dấu thiếu ở cả hai ô', async () => {
    proofreadChoiceGetMock.mockResolvedValue({
      choice: { global: { name: 'Cũ G', missing: true }, work: { mode: 'named', name: 'Cũ W', missing: true } },
      error: null,
    })
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)

    expect(wrapper.find('[data-proofread-choice-global]').text()).toContain('Cũ G (thiếu)')
    expect(wrapper.find('[data-proofread-choice-work]').text()).toContain('Cũ W (thiếu)')
    expect(wrapper.find('[data-proofread-choice-missing="global"]').exists()).toBe(true)
    expect(wrapper.find('[data-proofread-choice-missing="work"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('chọn ở hai ô gọi đúng lệnh ghi rồi đọc lại từ Rust', async () => {
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    proofreadChoiceGetMock.mockClear()

    await wrapper.find('[data-proofread-choice-global]').setValue('')
    await flushPromises()
    expect(setGlobalMock).toHaveBeenCalledWith(null)

    await wrapper.find('[data-proofread-choice-work]').setValue('off')
    await flushPromises()
    expect(setWorkMock).toHaveBeenLastCalledWith(null, true)

    await wrapper.find('[data-proofread-choice-work]').setValue('named:Soát câu')
    await flushPromises()
    expect(setWorkMock).toHaveBeenLastCalledWith('Soát câu', false)
    expect(proofreadChoiceGetMock).toHaveBeenCalledTimes(3)
    wrapper.unmount()
  })

  it('chưa chọn gì: ô Global "Không chọn", ô Tác phẩm "Theo Toàn cục"', async () => {
    proofreadChoiceGetMock.mockResolvedValue({
      choice: { global: null, work: { mode: 'follow_global', name: null, missing: false } },
      error: null,
    })
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    const g = wrapper.find('[data-proofread-choice-global]').element as HTMLSelectElement
    const w = wrapper.find('[data-proofread-choice-work]').element as HTMLSelectElement
    expect(g.selectedOptions[0].textContent).toBe('Không chọn')
    expect(w.selectedOptions[0].textContent).toBe('Theo Toàn cục')
    expect(wrapper.find('[data-proofread-choice-missing]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('đổi tên bộ đang chọn: ô chọn đọc lại từ Rust và hiện tên mới', async () => {
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    await wrapper.find('.pl-row-form').trigger('submit')
    await flushPromises()
    listMock.mockImplementation(async () => ({
      sets: [{ ...PROOFREAD_SET, name: 'Soát mới' }],
      workTierAvailable: true,
      variables: [],
      error: null,
    }))
    proofreadChoiceGetMock.mockResolvedValue({
      choice: { global: { name: 'Soát mới', missing: false }, work: { mode: 'follow_global', name: null, missing: false } },
      error: null,
    })
    const { renamePromptSet } = await import('../../src/promptSetState')
    await renamePromptSet('global', 2, 'Soát mới', 'proofread')
    await flushPromises()
    const g = wrapper.find('[data-proofread-choice-global]').element as HTMLSelectElement
    expect(g.selectedOptions[0].textContent).toBe('Soát mới')
    wrapper.unmount()
  })

  it('ghi lựa chọn trượt: lỗi hiện trong thẻ', async () => {
    setGlobalMock.mockResolvedValueOnce({ code: 'prompt_set.invalid_name', message_key: 'err.prompt_set.invalid_name', params: {}, retryable: false })
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    await wrapper.find('[data-proofread-choice-global]').setValue('')
    await flushPromises()
    expect(wrapper.find('[data-proofread-choice-error]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('không có Tác phẩm mở: ô Tác phẩm bị khoá', async () => {
    proofreadChoiceGetMock.mockResolvedValue({ choice: { global: null, work: null }, error: null })
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    expect(wrapper.find('[data-proofread-choice-work]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it.each([
    ['global_takes_over', 'thay vào'],
    ['set_missing', 'báo thiếu bộ'],
    ['none', 'một lần nữa để xoá'],
  ])('xoá bộ chỉ dẫn: hậu quả %s do Rust tính được hiện ở lượt bấm đầu', async (consequence, text) => {
    consequenceMock.mockResolvedValue({ consequence, error: null })
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    await wrapper.find('.pl-row-form').trigger('submit')
    await flushPromises()
    await wrapper.find('.pl-delete-form').trigger('submit')
    await flushPromises()

    expect(consequenceMock).toHaveBeenCalledWith('global', 2)
    expect(deleteMock).not.toHaveBeenCalled()
    const hint = wrapper.find('[data-delete-hint]')
    expect(hint.attributes('data-delete-consequence')).toBe(consequence)
    expect(hint.text()).toContain(text)
    wrapper.unmount()
  })

  it('nhập tệp loại proofread chuyển thư viện sang thẻ chỉ dẫn', async () => {
    openImportMock.mockResolvedValue({
      outcome: 'loaded',
      preview: {
        file_name: 'a.proofread.prompt.md',
        set_kind: 'proofread',
        name: 'Soát câu',
        body: 'x',
        warnings: noWarn(),
        global: { kind: 'new', existing_body: null },
        work: null,
      },
    })
    confirmImportMock.mockResolvedValue({ outcome: 'inserted', error: null })
    const { wrapper, importState } = await openOverlay()
    expect(wrapper.find('[data-proofread-choice]').exists()).toBe(false)

    await importState.openPromptImportPreviewOverlay()
    await importState.confirmPromptImportPreview()
    await flushPromises()

    expect(wrapper.find('[data-proofread-choice]').exists()).toBe(true)
    expect(listMock).toHaveBeenCalledWith('proofread')
    wrapper.unmount()
  })

  it('Global trỏ tới tên chỉ có ở tầng Tác phẩm (không thiếu): ô vẫn hiện tên đó', async () => {
    proofreadChoiceGetMock.mockResolvedValue({
      choice: { global: { name: 'B', missing: false }, work: { mode: 'follow_global', name: null, missing: false } },
      error: null,
    })
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)

    const select = wrapper.find('[data-proofread-choice-global]')
    const selected = select.findAll('option').find((o) => (o.element as HTMLOptionElement).selected)
    expect(selected?.text()).toBe('B')
    wrapper.unmount()
  })

  it.each([
    ['error', { consequence: null, error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false } }],
    ['null', { consequence: null, error: null }],
  ])('lệnh hậu quả xoá trượt (%s): không armed, lượt bấm sau không xoá', async (_label, reply) => {
    consequenceMock.mockResolvedValue(reply)
    const { wrapper } = await openOverlay()
    await toProofreadTab(wrapper)
    await wrapper.find('.pl-row-form').trigger('submit')
    await flushPromises()
    await wrapper.find('.pl-delete-form').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[data-delete-hint]').exists()).toBe(false)
    await wrapper.find('.pl-delete-form').trigger('submit')
    await flushPromises()
    expect(deleteMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
