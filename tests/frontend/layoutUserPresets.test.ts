/**
 * Preset bố cục của người dùng (FR17, FR18, A22): vòng đời lưu · đổi tên · áp · xoá trong state,
 * thân mục Bố cục, và dock THẬT cho `currentLayoutJson` / `applyLayoutJson`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import type { SerializedGrid, GridNode } from '../../src/layout/dockTree'

const putConfigMock = vi.fn()
const deleteConfigMock = vi.fn()

vi.mock('../../src/config/bootstrap', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
  putConfig: (...args: unknown[]) => putConfigMock(...args),
  deleteConfig: (...args: unknown[]) => deleteConfigMock(...args),
}))

const disk = new Map<string, string>()
const ipcFailure = { code: 'x', message_key: 'err.unknown', params: {}, retryable: true }

beforeEach(() => {
  document.body.innerHTML = ''
  disk.clear()
  putConfigMock.mockReset().mockImplementation(async (scope: string, key: string, value: string) => {
    if (scope === 'layout_preset') disk.set(key, value)
    return null
  })
  deleteConfigMock.mockReset().mockImplementation(async (scope: string, key: string) => {
    if (scope === 'layout_preset') disk.delete(key)
    return null
  })
  vi.resetModules()
})

afterEach(() => {
  vi.restoreAllMocks()
})

const names = (state: typeof import('../../src/layout/layoutPresetState')) =>
  state.userLayoutPresets.value.map((p) => p.name)

describe('layoutPresetState — vòng đời preset người dùng', () => {
  it('lưu, đổi tên, áp, xoá: danh sách phản ánh từng bước và đĩa khớp', async () => {
    const state = await import('../../src/layout/layoutPresetState')
    expect(await state.saveUserLayoutPreset('  Đọc song song ', '{"a":1}')).toBe(true)
    expect(names(state)).toEqual(['Đọc song song'])
    expect([...disk.keys()]).toEqual(['Đọc song song'])

    expect(await state.renameUserLayoutPreset('Đọc song song', 'Soát lỗi')).toBe(true)
    expect(names(state)).toEqual(['Soát lỗi'])
    expect([...disk.entries()]).toEqual([['Soát lỗi', '{"a":1}']])

    const applied: string[] = []
    expect(state.applyUserLayoutPreset('Soát lỗi', (json) => (applied.push(json), true))).toBe(true)
    expect(applied).toEqual(['{"a":1}'])

    expect(await state.deleteUserLayoutPreset('Soát lỗi')).toBe(true)
    expect(names(state)).toEqual([])
    expect(disk.size).toBe(0)
  })

  it('còn sau khi mở lại app: initUserLayoutPresets nạp đúng thứ đã ghi', async () => {
    const first = await import('../../src/layout/layoutPresetState')
    await first.saveUserLayoutPreset('B', '{"b":1}')
    await first.saveUserLayoutPreset('A', '{"a":1}')

    vi.resetModules()
    const reopened = await import('../../src/layout/layoutPresetState')
    expect(names(reopened)).toEqual([])
    reopened.initUserLayoutPresets(Object.fromEntries(disk))
    expect(names(reopened)).toEqual(['A', 'B'])
  })

  it('tên rỗng, trùng hoặc quá dài bị từ chối và không ghi hàng nào', async () => {
    const state = await import('../../src/layout/layoutPresetState')
    await state.saveUserLayoutPreset('Một', '{}')
    putConfigMock.mockClear()

    expect(await state.saveUserLayoutPreset('   ', '{}')).toBe(false)
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_name_empty')
    expect(await state.saveUserLayoutPreset('Một', '{}')).toBe(false)
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_name_taken')
    expect(await state.saveUserLayoutPreset('x'.repeat(state.MAX_PRESET_NAME_LENGTH + 1), '{}')).toBe(false)
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_name_long')
    expect(putConfigMock).not.toHaveBeenCalled()
  })

  it('không có Workspace (json null) thì không ghi gì và nói lý do', async () => {
    const state = await import('../../src/layout/layoutPresetState')
    expect(await state.saveUserLayoutPreset('Tên', null)).toBe(false)
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_no_workspace')
    expect(putConfigMock).not.toHaveBeenCalled()
  })

  it('ghi trượt giữ danh sách cũ; xoá trượt giữ preset; đổi tên trượt ở bước xoá để cả hai tên cùng hiện', async () => {
    const state = await import('../../src/layout/layoutPresetState')
    putConfigMock.mockResolvedValueOnce(ipcFailure)
    expect(await state.saveUserLayoutPreset('Hỏng', '{}')).toBe(false)
    expect(names(state)).toEqual([])
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_save_failed')

    await state.saveUserLayoutPreset('Cũ', '{}')
    deleteConfigMock.mockResolvedValueOnce(ipcFailure)
    expect(await state.deleteUserLayoutPreset('Cũ')).toBe(false)
    expect(names(state)).toEqual(['Cũ'])

    deleteConfigMock.mockResolvedValueOnce(ipcFailure)
    expect(await state.renameUserLayoutPreset('Cũ', 'Mới')).toBe(false)
    expect(names(state)).toEqual(['Cũ', 'Mới'])
    expect([...disk.keys()].sort()).toEqual(['Cũ', 'Mới'])
  })

  it('preset hỏng hoặc dock từ chối ⇒ false kèm câu "preset hỏng", không ném', async () => {
    const state = await import('../../src/layout/layoutPresetState')
    state.initUserLayoutPresets({ NotJson: '{oops', Bad: '{"x":1}' })
    expect(state.applyUserLayoutPreset('NotJson', () => true)).toBe(false)
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_preset_broken')
    expect(state.applyUserLayoutPreset('Bad', () => false)).toBe(false)
    expect(state.layoutPresetErrorKey.value).toBe('settings.layout.error_preset_broken')
    expect(state.applyUserLayoutPreset('missing', () => true)).toBe(false)
  })
})

describe('SettingsLayoutSection — thân mục Bố cục', () => {
  async function mountSection(live: { json: string | null; apply: (json: string) => boolean } | null) {
    const dock = await import('../../src/layout/dockController')
    if (live !== null) {
      dock.setDockController({
        applyPreset: () => true,
        togglePanel: () => true,
        visiblePanelsInLayoutOrder: () => [],
        activePanelId: () => null,
        currentLayoutJson: () => live.json,
        applyLayoutJson: live.apply,
      } as never)
    }
    const state = await import('../../src/layout/layoutPresetState')
    const Section = (await import('../../src/SettingsLayoutSection.vue')).default
    const wrapper = mount(Section, { attachTo: document.body })
    await flushPromises()
    return { wrapper, state, dock }
  }

  it('đặt tên, lưu, đổi tên, áp rồi xoá qua giao diện', async () => {
    const apply = vi.fn(() => true)
    const { wrapper } = await mountSection({ json: '{"live":1}', apply })

    await wrapper.find('[data-layout-new-name]').setValue('Của tôi')
    await wrapper.find('[data-layout-save]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(wrapper.find('[data-layout-user="Của tôi"]').exists()).toBe(true)
    expect(disk.get('Của tôi')).toBe('{"live":1}')

    await wrapper.find('[data-layout-rename-input]').setValue('Đổi tên')
    await wrapper.find('[data-layout-rename]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(wrapper.find('[data-layout-user="Của tôi"]').exists()).toBe(false)
    expect(wrapper.find('[data-layout-user="Đổi tên"]').exists()).toBe(true)

    await wrapper.find('[data-layout-apply]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(apply).toHaveBeenCalledWith('{"live":1}')
    expect(wrapper.find('[data-layout-error]').exists()).toBe(false)

    await wrapper.find('[data-layout-delete]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(wrapper.find('[data-layout-user]').exists()).toBe(false)
    expect(disk.size).toBe(0)
    wrapper.unmount()
  })

  it('dock từ chối preset (hình dạng sai) ⇒ hiện câu lỗi, không ném', async () => {
    const { wrapper, state } = await mountSection({ json: '{}', apply: () => false })
    state.initUserLayoutPresets({ Hỏng: '{"grid":1}' })
    await nextTick()
    await wrapper.find('[data-layout-apply]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(wrapper.find('[data-layout-error]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('ngoài Workspace các nút áp/lưu bị khoá và có câu nói rõ', async () => {
    const { wrapper } = await mountSection(null)
    expect(wrapper.find('[data-layout-no-workspace]').exists()).toBe(true)
    expect(wrapper.find('[data-layout-save]').attributes('disabled')).toBeDefined()
    expect(wrapper.find('[data-layout-builtin="layout.preset_grid"] button').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('Review Mode mở ⇒ nút khoá và hiện câu Review Mode, kể cả khi dock còn sống', async () => {
    const dock = await import('../../src/layout/dockController')
    dock.setDockSuspended(true)
    const { wrapper } = await mountSection({ json: '{}', apply: () => true })
    expect(wrapper.find('[data-layout-review-open]').exists()).toBe(true)
    expect(wrapper.find('[data-layout-save]').attributes('disabled')).toBeDefined()
    expect(dock.currentLayoutJson()).toBeNull()
    expect(dock.applyLayoutJson('{}')).toBe(false)
    dock.resetDockSuspended()
    wrapper.unmount()
  })
})

describe('dock THẬT — currentLayoutJson và applyLayoutJson', () => {
  function shapeOf(json: string): string {
    const grid = (JSON.parse(json) as { grid: SerializedGrid }).grid
    const walk = (node: GridNode, orientation: SerializedGrid['orientation']): string => {
      if (node.type === 'leaf') return ((node.data as { views?: string[] }).views ?? []).join(',')
      const kids = node.data as GridNode[]
      const next: SerializedGrid['orientation'] = orientation === 'HORIZONTAL' ? 'VERTICAL' : 'HORIZONTAL'
      return `(${kids.map((k) => walk(k, next)).join(orientation === 'HORIZONTAL' ? '|' : '/')})`
    }
    return walk(grid.root, grid.orientation)
  }

  async function liveDock() {
    Object.defineProperty(window, 'innerWidth', { configurable: true, value: 1200 })
    Object.defineProperty(window, 'innerHeight', { configurable: true, value: 1000 })
    document.documentElement.style.setProperty('--space-titlebar-height', '40px')
    document.documentElement.style.setProperty('--space-status-height', '34px')
    const WorkspaceDock = (await import('../../src/layout/WorkspaceDock.vue')).default
    const dock = await import('../../src/layout/dockController')
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))
    await nextTick()
    return { wrapper, dock }
  }

  it('lưu bố cục hiện tại, đổi sang preset khác, áp preset đã lưu ⇒ hình dạng trở lại', async () => {
    const { wrapper, dock } = await liveDock()
    try {
      expect(dock.applyPreset('layout.preset_grid')).toBe(true)
      const saved = dock.currentLayoutJson()
      expect(saved).not.toBeNull()
      const savedShape = shapeOf(saved!)

      expect(dock.applyPreset('layout.preset_columns')).toBe(true)
      expect(shapeOf(dock.currentLayoutJson()!)).not.toBe(savedShape)

      expect(dock.applyLayoutJson(saved!)).toBe(true)
      expect(shapeOf(dock.currentLayoutJson()!)).toBe(savedShape)
    } finally {
      wrapper.unmount()
    }
  })

  it('preset sai hình dạng ⇒ false và bố cục đang hiện không đổi', async () => {
    const { wrapper, dock } = await liveDock()
    try {
      vi.spyOn(console, 'error').mockImplementation(() => undefined)
      const before = shapeOf(dock.currentLayoutJson()!)
      for (const broken of ['{"grid":1}', '{}', '{"grid":{"root":{"type":"leaf","data":{"views":[]}}},"panels":{}}']) {
        expect(dock.applyLayoutJson(broken), broken).toBe(false)
        expect(shapeOf(dock.currentLayoutJson()!), broken).toBe(before)
      }
    } finally {
      wrapper.unmount()
    }
  })
})
