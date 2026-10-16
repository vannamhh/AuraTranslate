/** Công tắc "Hiện token và ước tính chi phí" (A25): mặc định bật, khoá `app_config` Toàn cục, chỉ `off` là ẩn. */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const wire = (usage: unknown) => ({
  theme: 'light',
  mode: 'library',
  shortcuts: {},
  layout_presets: {},
  workspace_layout: '',
  dict_sources_disabled: '',
  ...(usage === undefined ? {} : { ai_usage_visible: usage }),
})

async function started(usage: unknown) {
  vi.resetModules()
  mockInvoke.mockReset()
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  mockInvoke.mockResolvedValueOnce(wire(usage))
  const bootstrap = await import('../../src/config/bootstrap')
  await bootstrap.loadBootstrapConfig()
  return import('../../src/aiUsageVisibleState')
}

beforeEach(() => {
  vi.spyOn(console, 'info').mockImplementation(() => undefined)
})

describe('aiUsageVisible', () => {
  it.each([[''], [undefined], ['on'], ['rác']])('khởi động với %j ⇒ bật', async (raw) => {
    const state = await started(raw)
    expect(state.aiUsageVisible.value).toBe(true)
  })

  it('khởi động với "off" ⇒ ẩn', async () => {
    const state = await started('off')
    expect(state.aiUsageVisible.value).toBe(false)
  })

  it('tắt ghi "off", bật ghi "on" vào khoá Toàn cục, và đổi hiệu lực ngay', async () => {
    const state = await started('')
    mockInvoke.mockResolvedValue(undefined)
    await state.setAiUsageVisible(false)
    expect(state.aiUsageVisible.value).toBe(false)
    const [, args] = mockInvoke.mock.calls.at(-1) as [string, Record<string, unknown>]
    expect(args).toMatchObject({ kind: 'app_config', key: 'ai_usage_visible', value: 'off' })

    await state.setAiUsageVisible(true)
    expect(state.aiUsageVisible.value).toBe(true)
    expect((mockInvoke.mock.calls.at(-1) as [string, Record<string, unknown>])[1]).toMatchObject({ value: 'on' })
  })

  it('ghi trượt hiện lỗi nhưng công tắc vẫn đổi, không im lặng', async () => {
    const state = await started('')
    mockInvoke.mockRejectedValueOnce({ code: 'x', message_key: 'err.unknown', params: {}, retryable: true })
    await state.setAiUsageVisible(false)
    expect(state.aiUsageVisibleSaveError.value?.code).toBe('x')
    expect(state.aiUsageVisible.value).toBe(false)
  })
})
