import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

type Stats = { count: number; bytes: number; by_model: { model: string; count: number }[] }

let stats: Stats
let older: number
let purgeResult: unknown
let purgeGate: Promise<void> | null
let statsError: unknown

const STATS_3: Stats = {
  count: 12480,
  bytes: Math.round(8.4 * 1024 * 1024),
  by_model: [
    { model: 'm-1', count: 12000 },
    { model: 'm-2', count: 480 },
  ],
}

function calls(command: string): unknown[][] {
  return mockInvoke.mock.calls.filter((c) => c[0] === command)
}

beforeEach(() => {
  document.body.innerHTML = ''
  stats = STATS_3
  older = 1204
  purgeGate = null
  statsError = null
  purgeResult = { deleted: 1204, bytes_before: 10 * 1024 * 1024, bytes_after: 8 * 1024 * 1024, compact_error: null }
  mockInvoke.mockReset().mockImplementation(async (command: string, args: Record<string, unknown> | undefined) => {
    switch (command) {
      case 'quick_translation_stats':
        if (statsError !== null) throw statsError
        return stats
      case 'quick_translation_count_older':
        return older + (args?.days === 7 ? 1 : 0)
      case 'quick_translation_purge':
        if (purgeGate !== null) await purgeGate
        if (purgeResult instanceof Error) throw purgeResult
        return purgeResult
      case 'library_index_rebuilt_at':
        return null
      default:
        return undefined
    }
  })
  vi.resetModules()
})

afterEach(() => {
  vi.restoreAllMocks()
})

async function open() {
  const commands = await import('../../src/commands')
  const state = await import('../../src/settingsQuickDataState')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    purgeQuickDataOlder: () => void state.purgeQuickDataOlder(),
    askPurgeQuickDataAll: state.askPurgeQuickDataAll,
    confirmPurgeQuickDataAll: () => void state.confirmPurgeQuickDataAll(),
    cancelPurgeQuickDataAll: state.cancelPurgeQuickDataAll,
  })
  const Section = (await import('../../src/SettingsDataSection.vue')).default
  const wrapper = mount(Section, { attachTo: document.body })
  await flushPromises()
  const q = (name: string) => wrapper.find(`[data-settings-quick-data-${name}]`)
  return { wrapper, q, state }
}

describe('hàng Dịch nhanh ở Cài đặt › Dữ liệu', () => {
  it('thống kê: số câu, dung lượng, mỗi mô hình một dòng', async () => {
    const { wrapper, q } = await open()
    expect(q('summary').text()).toBe('12.480 câu · 8,4 MB')
    const lines = wrapper.findAll('[data-settings-quick-data-models] li').map((li) => li.text())
    expect(lines).toEqual(['m-1: 12.000 câu', 'm-2: 480 câu'])
    wrapper.unmount()
  })

  it('kho rỗng: Chưa có câu nào và hai nút tắt', async () => {
    stats = { count: 0, bytes: 0, by_model: [] }
    const { wrapper, q } = await open()
    expect(q('empty').text()).toBe('Chưa có câu nào.')
    expect(q('purge-older').attributes('disabled')).toBeDefined()
    expect(q('purge-all').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('kho lỗi: hiện tError và hai nút tắt', async () => {
    statsError = { code: 'x', message_key: 'err.quick_translation.invalid_days', params: {}, retryable: false }
    const { wrapper, q } = await open()
    expect(q('error').text()).toContain('Số ngày phải là số nguyên')
    expect(q('purge-older').attributes('disabled')).toBeDefined()
    expect(q('purge-all').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('đếm trước, đếm lại khi đổi số; số không hợp lệ báo lỗi và tắt Dọn', async () => {
    const { wrapper, q } = await open()
    expect(q('will-delete').text()).toBe('Sẽ xoá 1.204 câu.')
    expect(calls('quick_translation_count_older')[0]?.[1]).toEqual({ days: 30 })

    await q('days').setValue('7')
    await flushPromises()
    expect(calls('quick_translation_count_older').at(-1)?.[1]).toEqual({ days: 7 })
    expect(q('will-delete').text()).toBe('Sẽ xoá 1.205 câu.')

    const before = calls('quick_translation_count_older').length
    for (const bad of ['', '0', '-3', '2.5']) {
      await q('days').setValue(bad)
      await flushPromises()
      expect(q('days-invalid').exists()).toBe(true)
      expect(q('purge-older').attributes('disabled')).toBeDefined()
    }
    expect(calls('quick_translation_count_older').length).toBe(before)
    wrapper.unmount()
  })

  it('nhớ N: đổi ô rồi rời ô ghi app_config, mở lại thì ô là 7', async () => {
    const first = await open()
    await first.q('days').setValue('7')
    await first.q('days').trigger('change')
    await flushPromises()
    expect(calls('put_config').at(-1)?.[1]).toMatchObject({ kind: 'app_config', key: 'quick_translation_purge_days', value: '7' })
    first.wrapper.unmount()
    vi.resetModules()

    const saved = calls('put_config').at(-1)?.[1] as { value: string }
    const inner = mockInvoke.getMockImplementation()!
    mockInvoke.mockImplementation(async (command: string, args: Record<string, unknown> | undefined) =>
      command === 'bootstrap_config' ? { quick_translation_purge_days: Number(saved.value) } : inner(command, args),
    )
    await (await import('../../src/config/bootstrap')).loadBootstrapConfig()
    const second = await open()
    expect((second.q('days').element as HTMLInputElement).value).toBe('7')
    second.wrapper.unmount()
  })

  it('Dọn quá N: gọi older_than, nêu số câu và byte lấy lại, số liệu cập nhật', async () => {
    const { wrapper, q } = await open()
    stats = { count: 11276, bytes: 8 * 1024 * 1024, by_model: [{ model: 'm-1', count: 11276 }] }
    await q('purge-older').trigger('click')
    await flushPromises()

    expect(calls('quick_translation_purge')[0]?.[1]).toEqual({ scope: { kind: 'older_than', days: 30 } })
    expect(q('result').text()).toBe('Đã xoá 1.204 câu, lấy lại 2 MB. Lần sau dịch lại sẽ gọi AI.')
    expect(q('result').attributes('role')).toBe('status')
    expect(q('summary').text()).toBe('11.276 câu · 8 MB')
    wrapper.unmount()
  })

  it('Dọn tất cả: lần đầu chỉ hỏi tại chỗ, Thôi về nút cũ, xác nhận mới xoá', async () => {
    const { wrapper, q } = await open()
    await q('purge-all').trigger('click')
    expect(q('purge-all-confirm').text()).toBe('Xoá 12.480 câu?')
    expect(q('purge-all-cancel').text()).toBe('Thôi')
    expect(calls('quick_translation_purge')).toHaveLength(0)

    await q('purge-all-cancel').trigger('click')
    expect(q('purge-all').exists()).toBe(true)
    expect(q('purge-all-confirm').exists()).toBe(false)

    await q('purge-all').trigger('click')
    stats = { count: 0, bytes: 0, by_model: [] }
    await q('purge-all-confirm').trigger('click')
    await flushPromises()
    expect(calls('quick_translation_purge')[0]?.[1]).toEqual({ scope: { kind: 'all' } })
    expect(q('empty').exists()).toBe(true)
    wrapper.unmount()
  })

  it('đang dọn: Đang dọn…, hai nút tắt, không gửi lệnh lần hai', async () => {
    let open_: () => void = () => undefined
    purgeGate = new Promise<void>((resolve) => {
      open_ = resolve
    })
    const { wrapper, q, state } = await open()
    await q('purge-older').trigger('click')
    await flushPromises()
    expect(q('busy').text()).toBe('Đang dọn…')
    expect(q('purge-older').attributes('disabled')).toBeDefined()
    expect(q('purge-all').attributes('disabled')).toBeDefined()
    await state.purgeQuickDataOlder()
    expect(calls('quick_translation_purge')).toHaveLength(1)

    open_()
    await flushPromises()
    expect(q('busy').exists()).toBe(false)
    wrapper.unmount()
  })

  it('không thu nhỏ được: câu riêng kèm tError, hàng đã xoá vẫn xoá', async () => {
    purgeResult = { deleted: 2, bytes_before: 100, bytes_after: 100, compact_error: 'disk full' }
    const { wrapper, q } = await open()
    await q('purge-older').trigger('click')
    await flushPromises()
    expect(q('result').text()).toBe('Đã xoá 2 câu nhưng chưa thu nhỏ được tệp.')
    expect(q('compact-error').text()).toContain('disk full')
    wrapper.unmount()
  })

  it('lệnh dọn lỗi: hiện tError, không có câu thành công', async () => {
    purgeResult = { code: 'x', message_key: 'err.quick_translation.invalid_days', params: {}, retryable: false }
    mockInvoke.mockImplementation(async (command: string) => {
      if (command === 'quick_translation_stats') return stats
      if (command === 'quick_translation_count_older') return 1
      if (command === 'quick_translation_purge') throw purgeResult
      return undefined
    })
    const { wrapper, q } = await open()
    await q('purge-older').trigger('click')
    await flushPromises()
    expect(q('purge-error').exists()).toBe(true)
    expect(q('result').exists()).toBe(false)
    wrapper.unmount()
  })

  it('dọn xong bề mặt nào đang mở cũng nhận tín hiệu về Chưa có', async () => {
    const { wrapper, q } = await open()
    const run = await import('../../src/quickTranslateRun')
    expect(run.quickPurgeCount.value).toBe(0)
    await q('purge-older').trigger('click')
    await flushPromises()
    expect(run.quickPurgeCount.value).toBe(1)
    wrapper.unmount()
  })

  it('lưu N lỗi: hiện tError dưới ô và xoá khi lần lưu sau thành công', async () => {
    const failing = { code: 'x', message_key: 'err.quick_translation.invalid_days', params: {}, retryable: false }
    const inner = mockInvoke.getMockImplementation()!
    let fail = true
    mockInvoke.mockImplementation(async (command: string, args: Record<string, unknown> | undefined) => {
      if (command === 'put_config' && fail) throw failing
      return inner(command, args)
    })
    const { wrapper, q } = await open()
    await q('days').setValue('7')
    await q('days').trigger('change')
    await flushPromises()
    expect(q('days-save-error').exists()).toBe(true)

    fail = false
    await q('days').setValue('8')
    await q('days').trigger('change')
    await flushPromises()
    expect(q('days-save-error').exists()).toBe(false)
    wrapper.unmount()
  })

  it('tiêu điểm đi theo nút khi hỏi xác nhận, Thôi và sau khi dọn xong', async () => {
    const { wrapper, q } = await open()
    ;(q('purge-all').element as HTMLElement).focus()
    await q('purge-all').trigger('click')
    await flushPromises()
    expect(document.activeElement).toBe(q('purge-all-confirm').element)

    await q('purge-all-cancel').trigger('click')
    await flushPromises()
    expect(document.activeElement).toBe(q('purge-all').element)

    await q('purge-all').trigger('click')
    await flushPromises()
    await q('purge-all-confirm').trigger('click')
    await flushPromises()
    expect(document.activeElement).toBe(q('purge-all').element)
    wrapper.unmount()
  })

  it('mở lại Cài đặt không còn câu kết quả của lần trước', async () => {
    const first = await open()
    await first.q('purge-older').trigger('click')
    await flushPromises()
    expect(first.q('result').exists()).toBe(true)
    await first.state.loadQuickDataRow()
    await flushPromises()
    expect(first.q('result').exists()).toBe(false)
    first.wrapper.unmount()
  })
})
