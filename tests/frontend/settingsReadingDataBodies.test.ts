/** Thân mục Chế độ đọc và Dữ liệu (A23): dùng đúng state và lệnh sẵn có của hai chế độ, không đường song song. */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const IDLE_MS = 6000

beforeEach(() => {
  document.body.innerHTML = ''
  mockInvoke.mockReset().mockImplementation(async (command: string) =>
    command === 'library_index_rebuilt_at' ? null : undefined,
  )
  vi.resetModules()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('SettingsReadingSection', () => {
  async function open(deps: Record<string, unknown> = {}) {
    const commands = await import('../../src/commands')
    commands.installCommands({ isMac: true, setMode: () => {}, ...deps })
    const reading = await import('../../src/modes/readingState')
    const Section = (await import('../../src/SettingsReadingSection.vue')).default
    const wrapper = mount(Section, { attachTo: document.body })
    await flushPromises()
    return { wrapper, reading }
  }

  it('mỗi nút mức và nút song ngữ là đúng lệnh của Chế độ đọc', async () => {
    const dense = vi.fn()
    const airy = vi.fn()
    const toggle = vi.fn()
    const { wrapper } = await open({
      setReadingLevelDense: dense,
      setReadingLevelAiry: airy,
      toggleReadingBilingual: toggle,
    })
    await wrapper.find('[data-settings-reading-level="sm"]').trigger('click')
    await wrapper.find('[data-settings-reading-level="lg"]').trigger('click')
    await wrapper.find('[data-settings-reading-bilingual]').trigger('click')
    expect([dense.mock.calls.length, airy.mock.calls.length, toggle.mock.calls.length]).toEqual([1, 1, 1])
    wrapper.unmount()
  })

  it('thanh trượt đọc và ghi cùng state với Chế độ đọc, và ghi một lượt app_config sau khi yên', async () => {
    vi.useFakeTimers()
    const { wrapper, reading } = await open()
    const size = wrapper.find('[data-settings-reading-font-size]')
    await size.setValue('22')
    expect(reading.effectiveFontSize.value).toBe(22)

    reading.setReadingLevel('lg')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-settings-reading-level="lg"]').attributes('aria-pressed')).toBe('true')
    expect(wrapper.find('[data-settings-reading-level="sm"]').attributes('aria-pressed')).toBe('false')

    await wrapper.find('[data-settings-reading-line-height]').setValue('1.2')
    expect(reading.effectiveLineHeight.value).toBe(reading.READING_LINE_HEIGHT_FLOOR)

    await vi.advanceTimersByTimeAsync(IDLE_MS)
    const writes = mockInvoke.mock.calls.filter((c) => c[0] === 'put_config' && c[1].key === 'reading_preferences')
    expect(writes).toHaveLength(1)
    expect(JSON.parse(writes[0][1].value)).toMatchObject({ level: 'lg', lineHeightOverride: 1.66 })
    wrapper.unmount()
  })
})

describe('SettingsDataSection', () => {
  async function open(deps: Record<string, unknown> = {}) {
    const commands = await import('../../src/commands')
    commands.installCommands({ isMac: true, setMode: () => {}, ...deps })
    const Section = (await import('../../src/SettingsDataSection.vue')).default
    const wrapper = mount(Section, { attachTo: document.body })
    await flushPromises()
    return { wrapper }
  }

  it('ba nút dùng ba lệnh Thư viện sẵn có; chưa quét lần nào thì nói chưa quét', async () => {
    const choose = vi.fn()
    const rescan = vi.fn()
    const rebuild = vi.fn()
    const { wrapper } = await open({
      chooseLibraryRootFolder: choose,
      rescanLibraryFolder: rescan,
      rebuildLibraryIndexFolder: rebuild,
    })
    expect(wrapper.find('[data-settings-data-root]').text()).not.toBe('')
    await wrapper.find('[data-settings-data-choose]').trigger('click')
    await wrapper.find('[data-settings-data-rescan]').trigger('click')
    await wrapper.find('[data-settings-data-rebuild]').trigger('click')
    expect([choose.mock.calls.length, rescan.mock.calls.length, rebuild.mock.calls.length]).toEqual([1, 1, 1])
    wrapper.unmount()
  })

  it('mốc dựng chỉ mục đọc được thì hiện; đọc trượt thì hiện lỗi chứ không để trống im lặng', async () => {
    mockInvoke.mockImplementation(async (command: string) =>
      command === 'library_index_rebuilt_at' ? '2026-10-16T08:00:00Z' : undefined,
    )
    const first = await open()
    expect(first.wrapper.find('[data-settings-data-rebuilt-at]').exists()).toBe(true)
    expect(first.wrapper.find('[data-settings-data-error]').exists()).toBe(false)
    first.wrapper.unmount()

    vi.resetModules()
    mockInvoke.mockImplementation(async (command: string) => {
      if (command === 'library_index_rebuilt_at') {
        throw { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
      }
      return undefined
    })
    const second = await open()
    expect(second.wrapper.find('[data-settings-data-rebuilt-at]').exists()).toBe(false)
    expect(second.wrapper.find('[data-settings-data-error]').exists()).toBe(true)
    second.wrapper.unmount()
  })
})
