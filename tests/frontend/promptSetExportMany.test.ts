import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const promptSetListMock = vi.fn()
const promptSetExportManyMock = vi.fn()

vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...args: unknown[]) => promptSetListMock(...args),
  promptSetExportMany: (...args: unknown[]) => promptSetExportManyMock(...args),
  promptSetExport: vi.fn(),
}))

const ROWS = [
  { id: 5, name: 'Tiên hiệp', body: 'a', tier: 'work', shadowed_body: null, shadowed_id: null },
  { id: 7, name: 'Tiên hiệp', body: 'b', tier: 'global', shadowed_body: null, shadowed_id: null },
  { id: 8, name: 'a/b', body: 'c', tier: 'global', shadowed_body: null, shadowed_id: null },
]

const failure = {
  code: 'prompt_set.export_write_failed',
  message_key: 'err.prompt_set.export_write_failed',
  params: {},
  retryable: false,
}

async function fresh() {
  vi.resetModules()
  promptSetListMock.mockReset()
  promptSetExportManyMock.mockReset()
  promptSetListMock.mockResolvedValue({ sets: ROWS, workTierAvailable: true, variables: [], error: null })
  const state = await import('../../src/promptSetState')
  const gate = await import('../../src/glossaryExchangeGate')
  const libraryState = await import('../../src/promptLibraryState')
  const PromptLibraryOverlay = (await import('../../src/PromptLibraryOverlay.vue')).default
  return { state, gate, libraryState, PromptLibraryOverlay }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('exportPromptSets — một hộp thoại, một cờ dùng chung, kết quả từng tệp', () => {
  it('gửi đúng danh sách (tier, id) trong MỘT lượt gọi và giữ kết quả từng tệp, kể cả tệp lỗi', async () => {
    const { state } = await fresh()
    const files = [
      { tier: 'work', id: 5, file_name: 'Tiên hiệp.prompt.md', path: '/d/Tiên hiệp.prompt.md', error: null },
      { tier: 'global', id: 7, file_name: 'Tiên hiệp-2.prompt.md', path: '/d/Tiên hiệp-2.prompt.md', error: null },
      { tier: 'global', id: 8, file_name: null, path: null, error: failure },
    ]
    promptSetExportManyMock.mockResolvedValue({ outcome: 'done', files })

    await state.exportPromptSets([
      { tier: 'work', id: 5 },
      { tier: 'global', id: 7 },
      { tier: 'global', id: 8 },
    ])

    expect(promptSetExportManyMock).toHaveBeenCalledTimes(1)
    expect(promptSetExportManyMock.mock.calls[0][0]).toEqual([
      { tier: 'work', id: 5 },
      { tier: 'global', id: 7 },
      { tier: 'global', id: 8 },
    ])
    expect(state.promptSetExportFiles.value).toEqual(files)
    expect(state.promptSetExportError.value).toBeNull()
  })

  it('cờ dùng chung bật suốt hộp thoại: lượt thứ hai khi hộp thoại còn mở là no-op, và cờ hạ sau khi xong', async () => {
    const { state, gate } = await fresh()
    let settle: (v: unknown) => void = () => {}
    promptSetExportManyMock.mockImplementation(() => new Promise((r) => (settle = r)))

    const first = state.exportPromptSets([{ tier: 'global', id: 7 }])
    expect(gate.glossaryExchangeBusy.value).toBe(true)
    await state.exportPromptSets([{ tier: 'global', id: 8 }])
    expect(promptSetExportManyMock).toHaveBeenCalledTimes(1)

    settle({ outcome: 'cancelled' })
    await first
    expect(gate.glossaryExchangeBusy.value).toBe(false)
    expect(state.promptSetExportFiles.value).toBeNull()
  })

  it('cờ đang bận bởi một hộp thoại khác ⇒ 0 lượt gọi; danh sách rỗng ⇒ 0 lượt gọi', async () => {
    const { state, gate } = await fresh()
    await state.exportPromptSets([])
    gate.setGlossaryExchangeBusy(true)
    await state.exportPromptSets([{ tier: 'global', id: 7 }])
    expect(promptSetExportManyMock).not.toHaveBeenCalled()
  })

  it('lỗi cả lượt ⇒ promptSetExportManyError (không phải lỗi xuất một bộ), không có danh sách tệp', async () => {
    const { state } = await fresh()
    promptSetExportManyMock.mockResolvedValue({ outcome: 'error', error: failure })
    await state.exportPromptSets([{ tier: 'global', id: 7 }])
    expect(state.promptSetExportManyError.value).toEqual(failure)
    expect(state.promptSetExportError.value).toBeNull()
    expect(state.promptSetExportFiles.value).toBeNull()
  })
})

describe('PromptLibraryOverlay.vue — chọn nhiều bộ và báo từng tệp', () => {
  it('chọn ba bộ (hai trùng tên khác tầng) rồi xuất ⇒ một lượt gọi với cả ba, mỗi tệp một dòng kết quả', async () => {
    const { libraryState, PromptLibraryOverlay } = await fresh()
    promptSetExportManyMock.mockResolvedValue({
      outcome: 'done',
      files: [
        { tier: 'work', id: 5, file_name: 'Tiên hiệp.prompt.md', path: '/d/x', error: null },
        { tier: 'global', id: 7, file_name: 'Tiên hiệp-2.prompt.md', path: '/d/y', error: null },
        { tier: 'global', id: 8, file_name: null, path: null, error: failure },
      ],
    })
    libraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const boxes = wrapper.findAll('.pl-export-many input[type="checkbox"]')
    expect(boxes).toHaveLength(3)
    const submit = wrapper.get('.pl-export-many button[type="submit"]')
    expect(submit.attributes('disabled')).toBeDefined()
    for (const box of boxes) await box.setValue(true)
    expect(submit.attributes('disabled')).toBeUndefined()

    await wrapper.get('.pl-export-many').trigger('submit')
    await flushPromises()

    expect(promptSetExportManyMock).toHaveBeenCalledTimes(1)
    expect(promptSetExportManyMock.mock.calls[0][0]).toHaveLength(3)
    const lines = wrapper.findAll('.pl-export-files li').map((li) => li.text())
    expect(lines).toHaveLength(3)
    expect(lines[0]).toContain('Tiên hiệp.prompt.md')
    expect(lines[1]).toContain('Tiên hiệp-2.prompt.md')
    expect(lines[2]).toContain('a/b')
    expect(lines[2]).not.toContain('prompt.md')

    wrapper.unmount()
  })

  it('bộ đã tick rồi biến mất sau lần nạp lại ⇒ nút khoá và không gọi IPC', async () => {
    const { state, libraryState, PromptLibraryOverlay } = await fresh()
    libraryState.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.findAll('.pl-export-many input[type="checkbox"]')[0].setValue(true)
    promptSetListMock.mockResolvedValue({ sets: [ROWS[2]], workTierAvailable: true, variables: [], error: null })
    await state.loadPromptSets()
    await flushPromises()

    const submit = wrapper.get('.pl-export-many button[type="submit"]')
    expect(submit.attributes('disabled')).toBeDefined()
    await wrapper.get('.pl-export-many').trigger('submit')
    expect(promptSetExportManyMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('checkbox nằm trong fieldset có legend', async () => {
    const { libraryState, PromptLibraryOverlay } = await fresh()
    libraryState.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('fieldset.pl-export-set > legend').exists()).toBe(true)
    wrapper.unmount()
  })

  it('lỗi cả lượt của xuất nhiều bộ hiện đúng MỘT lần, trong khối xuất nhiều bộ', async () => {
    const { libraryState, PromptLibraryOverlay } = await fresh()
    promptSetExportManyMock.mockResolvedValue({ outcome: 'error', error: failure })
    libraryState.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.findAll('.pl-export-many input[type="checkbox"]')[0].setValue(true)
    await wrapper.get('.pl-export-many').trigger('submit')
    await flushPromises()

    expect(wrapper.findAll('.pl-export-many [role="alert"]')).toHaveLength(1)
    expect(wrapper.findAll('[role="alert"]')).toHaveLength(1)
    wrapper.unmount()
  })
})
