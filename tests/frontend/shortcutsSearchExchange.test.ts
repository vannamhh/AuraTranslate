/**
 * Phím tắt (FR144): tra ngược, Xuất, Nhập thay toàn bộ ghi đè, Trả về mặc định.
 * Hộp thoại tệp là Rust; ở đây `config/shortcuts` được giả để nội dung tệp đi qua đúng logic kiểm.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const putConfigMock = vi.fn()
const deleteConfigMock = vi.fn()
const exportMock = vi.fn()
const importMock = vi.fn()
const diskRowIds = { value: [] as string[] }

vi.mock('../../src/config/bootstrap', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
  putConfig: (...args: unknown[]) => putConfigMock(...args),
  deleteConfig: (...args: unknown[]) => deleteConfigMock(...args),
  bootstrapShortcutRowIds: diskRowIds,
}))
vi.mock('../../src/config/shortcuts', () => ({
  shortcutsExport: (...args: unknown[]) => exportMock(...args),
  shortcutsImport: (...args: unknown[]) => importMock(...args),
}))

const FORMAT = 'auratranslate-shortcuts'
const file = (bindings: Record<string, string[]>, extra: Record<string, unknown> = {}) =>
  JSON.stringify({ format: FORMAT, version: 1, bindings, ...extra })

const keydown = (code: string, init: Record<string, unknown> = {}) => ({
  code,
  key: code.startsWith('Key') ? code.slice(3).toLowerCase() : code,
  ...init,
})

async function fresh() {
  vi.resetModules()
  Object.defineProperty(navigator, 'platform', { value: 'MacIntel', configurable: true })
  const commands = await import('../../src/commands')
  commands.installCommands({ isMac: true, setMode: () => {} })
  const state = await import('../../src/config/shortcutsState')
  return { commands, state }
}

const chordsOf = (commands: Awaited<ReturnType<typeof fresh>>['commands'], id: string) =>
  commands
    .effectiveBindings()
    .filter((b) => b.id === id)
    .map((b) => b.chord)

const overridesWritten = () => putConfigMock.mock.calls.map((c) => [c[1], c[2]])

beforeEach(() => {
  document.body.innerHTML = ''
  putConfigMock.mockReset().mockResolvedValue(null)
  deleteConfigMock.mockReset().mockResolvedValue(null)
  exportMock.mockReset().mockResolvedValue({ path: '/tmp/x.json', error: null })
  importMock.mockReset().mockResolvedValue({ content: null, error: null })
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('tra ngược trong ô tìm', () => {
  it('gõ ⌘M chỉ còn hàng editor.merge_segments, hiện tổ hợp đã định dạng, không ghi gì', async () => {
    const { state } = await fresh()
    expect(state.handleSearchKey(keydown('KeyM', { metaKey: true }))).toBe(true)
    expect(state.visibleShortcutRows.value.map((r) => r.id)).toEqual(['editor.merge_segments'])
    expect(state.shortcutSearchChordDisplay.value).toBe('⌘M')
    expect(putConfigMock).not.toHaveBeenCalled()
    expect(deleteConfigMock).not.toHaveBeenCalled()
    expect(state.captureIsArmed.value).toBe(false)
  })

  it('tổ hợp không lệnh nào dùng cho ra danh sách rỗng', async () => {
    const { commands, state } = await fresh()
    expect(commands.effectiveBindings().some((b) => b.chord === 'Mod+Alt+Shift+U')).toBe(false)
    expect(state.handleSearchKey(keydown('KeyU', { metaKey: true, altKey: true, shiftKey: true }))).toBe(true)
    expect(state.visibleShortcutRows.value).toEqual([])
  })

  it('phím in được không kèm Mod/Ctrl/Alt vẫn là chữ để tìm theo tên; Tab, Esc, Backspace, Enter cũng vậy', async () => {
    const { state } = await fresh()
    expect(state.handleSearchKey(keydown('KeyM'))).toBe(false)
    for (const code of ['Tab', 'Escape', 'Backspace', 'Enter']) {
      expect(state.handleSearchKey(keydown(code, { key: code }))).toBe(false)
    }
    expect(state.handleSearchKey({ ...keydown('KeyM', { metaKey: true }), isComposing: true })).toBe(false)

    state.setShortcutSearchText('gộp')
    expect(state.visibleShortcutRows.value.map((r) => r.id)).toContain('editor.merge_segments')
    expect(state.visibleShortcutRows.value.length).toBeLessThan(state.shortcutTotalCount.value)
  })

  it('phím không in được (F-key trần) là tra ngược; gõ chữ sau đó xoá tổ hợp, xoá ô trả đủ hàng', async () => {
    const { state } = await fresh()
    expect(state.handleSearchKey(keydown('F3', { key: 'F3' }))).toBe(true)
    expect(state.shortcutSearchChordDisplay.value).not.toBeNull()
    state.setShortcutSearchText('a')
    expect(state.shortcutSearchChordDisplay.value).toBeNull()
    state.clearShortcutSearch()
    expect(state.visibleShortcutRows.value).toHaveLength(state.shortcutTotalCount.value)
  })
})

describe('chord thử nghiệm còn trống', () => {
  it('J, K, L, U với Mod+Alt+Shift chưa lệnh nào giữ — nếu đỏ, đổi chord thử chứ không đổi kỳ vọng', async () => {
    const { commands } = await fresh()
    for (const key of ['J', 'K', 'L', 'U']) {
      expect(commands.effectiveBindings().some((b) => b.chord === `Mod+Alt+Shift+${key}`)).toBe(false)
    }
  })
})

describe('Xuất rồi Nhập', () => {
  async function overrideTwo() {
    const ctx = await fresh()
    importMock.mockResolvedValueOnce({
      content: file({ 'editor.merge_segments': ['Mod+Alt+Shift+J'], 'editor.split_segment': ['Mod+Alt+Shift+K'] }),
      error: null,
    })
    await ctx.state.importShortcuts()
    putConfigMock.mockClear()
    deleteConfigMock.mockClear()
    return ctx
  }

  it('xuất chỉ chứa ghi đè (kể cả bỏ gán), có format và version', async () => {
    const { state } = await fresh()
    importMock.mockResolvedValueOnce({
      content: file({ 'editor.merge_segments': [] }),
      error: null,
    })
    await state.importShortcuts()
    await state.exportShortcuts()
    const written = JSON.parse(exportMock.mock.calls[0][0] as string)
    expect(written).toEqual({ format: FORMAT, version: 1, bindings: { 'editor.merge_segments': [] } })
    expect(state.shortcutFileNotice.value?.key).toBe('shortcuts.export_done')
  })

  it('xuất, gán lại một lệnh, nhập tệp vừa xuất: phím cũ trở về, ghi đè khác giữ đúng tệp', async () => {
    const { commands, state } = await overrideTwo()
    await state.exportShortcuts()
    const exported = exportMock.mock.calls[0][0] as string

    importMock.mockResolvedValueOnce({
      content: file({ 'editor.merge_segments': ['Mod+Alt+Shift+L'], 'editor.split_segment': ['Mod+Alt+Shift+K'] }),
      error: null,
    })
    await state.importShortcuts()
    expect(chordsOf(commands, 'editor.merge_segments')).toEqual(['Mod+Alt+Shift+L'])

    putConfigMock.mockClear()
    importMock.mockResolvedValueOnce({ content: exported, error: null })
    await state.importShortcuts()

    expect(chordsOf(commands, 'editor.merge_segments')).toEqual(['Mod+Alt+Shift+J'])
    expect(chordsOf(commands, 'editor.split_segment')).toEqual(['Mod+Alt+Shift+K'])
    expect(overridesWritten()).toContainEqual(['editor.merge_segments', 'Mod+Alt+Shift+J'])
    expect(state.shortcutFileNotice.value?.key).toBe('shortcuts.import_done')
  })

  it('nhập THAY TOÀN BỘ: ghi đè không còn trong tệp bị xoá hàng và trả về mặc định', async () => {
    const { commands, state } = await overrideTwo()
    importMock.mockResolvedValueOnce({ content: file({ 'editor.merge_segments': ['Mod+Alt+Shift+J'] }), error: null })
    await state.importShortcuts()

    expect(deleteConfigMock).toHaveBeenCalledTimes(1)
    expect(deleteConfigMock.mock.calls[0][1]).toBe('editor.split_segment')
    expect(chordsOf(commands, 'editor.split_segment')).toEqual(commands.defaultChordsFor('editor.split_segment'))
  })

  it('huỷ hộp thoại (Xuất hay Nhập) không ghi gì và không để lại câu nào', async () => {
    const { commands, state } = await overrideTwo()
    const before = JSON.stringify(commands.effectiveBindings())
    exportMock.mockResolvedValueOnce({ path: null, error: null })
    await state.exportShortcuts()
    await state.importShortcuts()
    expect(JSON.stringify(commands.effectiveBindings())).toBe(before)
    expect(putConfigMock).not.toHaveBeenCalled()
    expect(deleteConfigMock).not.toHaveBeenCalled()
    expect(state.shortcutFileNotice.value).toBeNull()
  })

  it('lỗi từ Rust (tệp quá lớn) hiện đúng khoá và không đổi gì', async () => {
    const { commands, state } = await overrideTwo()
    const before = JSON.stringify(commands.effectiveBindings())
    importMock.mockResolvedValueOnce({
      content: null,
      error: { code: 'shortcuts.file_too_large', message_key: 'err.shortcuts.file_too_large', params: {}, retryable: false },
    })
    await state.importShortcuts()
    expect(state.shortcutFileNotice.value?.key).toBe('err.shortcuts.file_too_large')
    expect(JSON.stringify(commands.effectiveBindings())).toBe(before)
    expect(putConfigMock).not.toHaveBeenCalled()
  })
})

describe('Nhập tệp hỏng', () => {
  const bad: Array<[string, string, string]> = [
    ['không phải JSON', '{not json', 'shortcuts.import_not_json'],
    ['JSON không phải đối tượng', '42', 'shortcuts.import_wrong_format'],
    ['format khác', JSON.stringify({ format: 'other', version: 1, bindings: {} }), 'shortcuts.import_wrong_format'],
    ['version khác', JSON.stringify({ format: FORMAT, version: 2, bindings: {} }), 'shortcuts.import_wrong_version'],
    ['bindings là mảng', JSON.stringify({ format: FORMAT, version: 1, bindings: [] }), 'shortcuts.import_wrong_format'],
    ['giá trị không phải mảng chuỗi', file({ 'editor.merge_segments': 'Mod+K' as unknown as string[] }), 'shortcuts.import_wrong_format'],
    ['chord sai', file({ 'editor.merge_segments': ['Mod+NoSuchKey'] }), 'shortcuts.import_rejected'],
    [
      'hai lệnh va nhau',
      file({ 'editor.merge_segments': ['Mod+Alt+Shift+J'], 'editor.split_segment': ['Mod+Alt+Shift+J'] }),
      'shortcuts.import_rejected',
    ],
    ['va với phím mặc định của lệnh khác', file({ 'editor.split_segment': ['Mod+M'] }), 'shortcuts.import_rejected'],
  ]

  it.each(bad)('%s ⇒ lỗi nêu lý do, không hàng shortcut nào đổi, keymap giữ nguyên', async (_name, content, key) => {
    const { commands, state } = await fresh()
    const before = JSON.stringify(commands.effectiveBindings())
    importMock.mockResolvedValueOnce({ content, error: null })
    await state.importShortcuts()

    expect(state.shortcutFileNotice.value?.key).toBe(key)
    expect(putConfigMock).not.toHaveBeenCalled()
    expect(deleteConfigMock).not.toHaveBeenCalled()
    expect(JSON.stringify(commands.effectiveBindings())).toBe(before)
  })
})

describe('Nhập id lạ', () => {
  it('id không còn trong registry bị bỏ và đếm, phần còn lại áp', async () => {
    const { commands, state } = await fresh()
    importMock.mockResolvedValueOnce({
      content: file({
        'editor.merge_segments': ['Mod+Alt+Shift+J'],
        'gone.command_one': ['Mod+Alt+Shift+K'],
        'gone.command_two': ['Mod+Alt+Shift+L'],
      }),
      error: null,
    })
    await state.importShortcuts()

    expect(chordsOf(commands, 'editor.merge_segments')).toEqual(['Mod+Alt+Shift+J'])
    expect(overridesWritten()).toEqual([['editor.merge_segments', 'Mod+Alt+Shift+J']])
    expect(state.shortcutFileNotice.value).toEqual({ key: 'shortcuts.import_done_skipped', params: { skipped: '2' } })
  })

  it('ghi trượt giữa chừng không báo thành công', async () => {
    const { state } = await fresh()
    putConfigMock.mockResolvedValue({ code: 'x', message_key: 'err.unknown', params: {}, retryable: true })
    importMock.mockResolvedValueOnce({ content: file({ 'editor.merge_segments': ['Mod+Alt+Shift+J'] }), error: null })
    await state.importShortcuts()
    expect(state.shortcutFileNotice.value?.key).toBe('shortcuts.save_failed')
  })
})

describe('Trả về mặc định', () => {
  it('hỏi một bước: bấm lần đầu chưa xoá gì; huỷ thì thôi; xác nhận xoá mọi ghi đè', async () => {
    const { commands, state } = await fresh()
    importMock.mockResolvedValueOnce({
      content: file({ 'editor.merge_segments': ['Mod+Alt+Shift+J'], 'editor.split_segment': [] }),
      error: null,
    })
    await state.importShortcuts()
    putConfigMock.mockClear()

    await state.confirmResetAllShortcuts()
    expect(deleteConfigMock).not.toHaveBeenCalled()

    state.askResetAllShortcuts()
    expect(state.shortcutResetAllPending.value).toBe(true)
    state.cancelResetAllShortcuts()
    await state.confirmResetAllShortcuts()
    expect(deleteConfigMock).not.toHaveBeenCalled()
    expect(chordsOf(commands, 'editor.merge_segments')).toEqual(['Mod+Alt+Shift+J'])

    state.askResetAllShortcuts()
    await state.confirmResetAllShortcuts()
    expect(deleteConfigMock.mock.calls.map((c) => c[1]).sort()).toEqual(['editor.merge_segments', 'editor.split_segment'])
    expect(chordsOf(commands, 'editor.merge_segments')).toEqual(commands.defaultChordsFor('editor.merge_segments'))
    expect(commands.currentOverrides()).toEqual({})
    expect(state.shortcutResetAllPending.value).toBe(false)
  })
})

describe('SettingsShortcutsSection — ô tìm', () => {
  async function mountSection() {
    const ctx = await fresh()
    const Section = (await import('../../src/SettingsShortcutsSection.vue')).default
    const wrapper = mount(Section, { attachTo: document.body })
    await flushPromises()
    return { ...ctx, wrapper }
  }

  it('⌘M trong ô chỉ còn một hàng và bộ đếm "1 / tổng"; tổ hợp không ai dùng hiện câu giải thích', async () => {
    const { wrapper, state } = await mountSection()
    const input = wrapper.find('[data-shortcut-search]')
    const total = String(state.shortcutTotalCount.value)

    await input.trigger('keydown', { code: 'KeyM', key: 'm', metaKey: true })
    expect(wrapper.findAll('[data-command-id]').map((r) => r.attributes('data-command-id'))).toEqual([
      'editor.merge_segments',
    ])
    expect(wrapper.find('[data-shortcut-count]').text()).toContain(total)
    expect(wrapper.find('[data-shortcut-search-chord]').text()).toBe('⌘M')
    expect(wrapper.find('[data-shortcut-chord-empty]').exists()).toBe(false)

    await input.trigger('keydown', { code: 'KeyU', key: 'u', metaKey: true, altKey: true, shiftKey: true })
    expect(wrapper.findAll('[data-command-id]')).toHaveLength(0)
    expect(wrapper.find('[data-shortcut-chord-empty]').text()).toContain('⌘')

    await input.trigger('keydown', { code: 'Escape', key: 'Escape' })
    expect(wrapper.findAll('[data-command-id]')).toHaveLength(state.shortcutTotalCount.value)
    wrapper.unmount()
  })

  it('Trả về mặc định hỏi một bước trên màn hình trước khi xoá', async () => {
    const { wrapper } = await mountSection()
    expect(wrapper.find('[data-shortcut-reset-all-confirm]').exists()).toBe(false)
    await wrapper.find('[data-shortcut-reset-all]').trigger('click')
    expect(wrapper.find('[data-shortcut-reset-all-confirm]').exists()).toBe(true)
    expect(deleteConfigMock).not.toHaveBeenCalled()
    await wrapper.find('[data-shortcut-reset-all-cancel]').trigger('click')
    expect(wrapper.find('[data-shortcut-reset-all-confirm]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('hàng shortcut còn trên đĩa mà bàn phím đã từ chối', () => {
  it('Trả về mặc định xoá cả hàng không nằm trong ghi đè đang sống', async () => {
    diskRowIds.value = ['rejected.row']
    const { state } = await fresh()
    state.askResetAllShortcuts()
    await state.confirmResetAllShortcuts()
    expect(deleteConfigMock.mock.calls.map((c) => c[1])).toContain('rejected.row')
    diskRowIds.value = []
  })
})
