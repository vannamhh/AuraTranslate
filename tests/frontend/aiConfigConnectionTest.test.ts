/**
 * Thử kết nối (FR143): adapter `aiConfigTestConnection`, state `runAiConfigTest` và nút trong
 * mục AI. Provider thật không giả được ở đây; Rust có máy chủ giả riêng (`connection_check_contract`).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const invokeMock = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {},
}))
vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

const GET_REPLY = {
  fields: [
    { field: 'provider', value: 'openai', tier: 'global', shadowed: null },
    { field: 'endpoint', value: 'https://e.example.com/v1', tier: 'global', shadowed: null },
    { field: 'model', value: 'm', tier: 'global', shadowed: null },
    { field: 'temperature', value: '', tier: 'global', shadowed: null },
    { field: 'max_tokens', value: '', tier: 'global', shadowed: null },
  ],
  work_tier_available: false,
  key_configured: true,
  ready: true,
}

const ipcError = (key: string, params: Record<string, string> = {}) => ({
  code: key,
  message_key: `err.${key}`,
  params,
  retryable: false,
})

function route(handlers: Record<string, (args: unknown) => unknown>) {
  invokeMock.mockImplementation(async (command: string, args: unknown) => {
    if (command === 'ai_config_get') return GET_REPLY
    const handler = (handlers as Partial<typeof handlers>)[command]
    if (handler === undefined) return null
    return handler(args)
  })
}

async function fresh() {
  vi.resetModules()
  const state = await import('../../src/aiConfigState')
  const adapter = await import('../../src/config/aiconfig')
  return { state, adapter }
}

beforeEach(() => {
  invokeMock.mockReset()
  document.body.innerHTML = ''
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
  vi.spyOn(console, 'info').mockImplementation(() => undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
})

describe('aiConfigTestConnection — adapter', () => {
  it('sends only the tier and returns the three outcome shapes', async () => {
    const { adapter } = await fresh()
    for (const outcome of [
      { kind: 'connected', model_count: 4 },
      { kind: 'not_configured' },
      { kind: 'cancelled' },
    ]) {
      invokeMock.mockResolvedValueOnce(outcome)
      expect(await adapter.aiConfigTestConnection('global')).toEqual({ outcome, error: null })
    }
    expect(invokeMock).toHaveBeenCalledWith('ai_config_test_connection', { tier: 'global' })
  })

  it('a strange shape is an unknown error, never an outcome', async () => {
    const { adapter } = await fresh()
    invokeMock.mockResolvedValueOnce({ kind: 'connected' })
    const result = await adapter.aiConfigTestConnection('work')
    expect(result.outcome).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('an IpcError passes through; no bridge gives neither outcome nor error; cancel never throws', async () => {
    const { adapter } = await fresh()
    const refused = ipcError('ai_config.test_key_rejected')
    invokeMock.mockRejectedValueOnce(refused)
    expect(await adapter.aiConfigTestConnection('global')).toEqual({ outcome: null, error: refused })

    invokeMock.mockRejectedValueOnce(new Error('no ipc'))
    expect(await adapter.aiConfigTestConnection('global')).toEqual({ outcome: null, error: null })

    vi.spyOn(console, 'warn').mockImplementation(() => undefined)
    invokeMock.mockRejectedValueOnce(new Error('x'))
    await expect(adapter.aiConfigTestConnectionCancel()).resolves.toBeUndefined()
    expect(invokeMock).toHaveBeenLastCalledWith('ai_config_test_connection_cancel')
  })
})

describe('runAiConfigTest — state', () => {
  it('connected carries the model count; the tested tier is the viewed one', async () => {
    route({ ai_config_test_connection: () => ({ kind: 'connected', model_count: 7 }) })
    const { state } = await fresh()
    await state.loadAiConfigSection()
    await state.runAiConfigTest()
    expect(state.aiConfigTestStatus.value).toBe('connected')
    expect(state.aiConfigTestModelCount.value).toBe(7)
    expect(invokeMock).toHaveBeenCalledWith('ai_config_test_connection', { tier: 'global' })
  })

  it('a rejected key is an error with its message key, not an automatic second call', async () => {
    route({
      ai_config_test_connection: () => {
        throw ipcError('ai_config.test_key_rejected')
      },
    })
    const { state } = await fresh()
    await state.loadAiConfigSection()
    await state.runAiConfigTest()
    expect(state.aiConfigTestStatus.value).toBe('error')
    expect(state.aiConfigTestError.value?.message_key).toBe('err.ai_config.test_key_rejected')
    expect(invokeMock.mock.calls.filter((c) => c[0] === 'ai_config_test_connection')).toHaveLength(1)
  })

  it('not configured and cancelled are states of their own', async () => {
    const { state } = await fresh()
    await state.loadAiConfigSection()
    for (const kind of ['not_configured', 'cancelled'] as const) {
      route({ ai_config_test_connection: () => ({ kind }) })
      await state.runAiConfigTest()
      expect(state.aiConfigTestStatus.value).toBe(kind)
      expect(state.aiConfigTestError.value).toBeNull()
    }
  })

  it('a second press while running does not start a second call; cancel only works while running', async () => {
    let release: (v: unknown) => void = () => undefined
    route({
      ai_config_test_connection: () => new Promise((r) => (release = r)),
    })
    const { state } = await fresh()
    await state.loadAiConfigSection()

    await state.cancelAiConfigTest()
    expect(invokeMock.mock.calls.some((c) => c[0] === 'ai_config_test_connection_cancel')).toBe(false)

    const first = state.runAiConfigTest()
    await flushPromises()
    void state.runAiConfigTest()
    await flushPromises()
    expect(invokeMock.mock.calls.filter((c) => c[0] === 'ai_config_test_connection')).toHaveLength(1)
    expect(state.aiConfigTestStatus.value).toBe('running')

    await state.cancelAiConfigTest()
    expect(invokeMock.mock.calls.filter((c) => c[0] === 'ai_config_test_connection_cancel')).toHaveLength(1)
    release({ kind: 'cancelled' })
    await first
    expect(state.aiConfigTestStatus.value).toBe('cancelled')
  })

  it('waits for a blur save still in flight, so the test reads the value just typed', async () => {
    const order: string[] = []
    let finishSave: () => void = () => undefined
    route({
      ai_config_save_field: () =>
        new Promise((r) => {
          finishSave = () => {
            order.push('saved')
            r(null)
          }
        }),
      ai_config_test_connection: () => {
        order.push('tested')
        return { kind: 'connected', model_count: 1 }
      },
    })
    const { state } = await fresh()
    await state.loadAiConfigSection()
    state.setAiConfigDraft('model', 'typed')
    const saving = state.commitAiConfigField('model')
    const testing = state.runAiConfigTest()
    await flushPromises()
    expect(order).toEqual([])
    finishSave()
    await saving
    await testing
    expect(order).toEqual(['saved', 'tested'])
  })

  it('a cancel pressed while waiting for the blur save skips the invoke and ends cancelled', async () => {
    let finishSave: () => void = () => undefined
    let tested = false
    route({
      ai_config_save_field: () => new Promise((r) => (finishSave = () => r(null))),
      ai_config_test_connection: () => {
        tested = true
        return { kind: 'connected', model_count: 1 }
      },
    })
    const { state } = await fresh()
    await state.loadAiConfigSection()
    state.setAiConfigDraft('model', 'typed')
    const saving = state.commitAiConfigField('model')
    const testing = state.runAiConfigTest()
    await flushPromises()
    await state.cancelAiConfigTest()
    finishSave()
    await saving
    await testing
    expect(tested).toBe(false)
    expect(invokeMock.mock.calls.some((c) => c[0] === 'ai_config_test_connection_cancel')).toBe(false)
    expect(state.aiConfigTestStatus.value).toBe('cancelled')
  })

  it('switching the viewed tier clears a finished result, but not a running one', async () => {
    route({ ai_config_test_connection: () => ({ kind: 'connected', model_count: 2 }) })
    const { state } = await fresh()
    await state.loadAiConfigSection()
    await state.runAiConfigTest()
    expect(state.aiConfigTestStatus.value).toBe('connected')
    state.selectAiConfigViewTier('global')
    expect(state.aiConfigTestStatus.value).toBe('idle')
  })
})

describe('SettingsOverlay — Thử kết nối button', () => {
  async function open() {
    vi.resetModules()
    const settings = await import('../../src/settingsState')
    const state = await import('../../src/aiConfigState')
    const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
    settings.openSettings()
    settings.selectSettingsSection('ai_and_model')
    await state.loadAiConfigSection()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await flushPromises()
    return { wrapper }
  }

  async function press(wrapper: ReturnType<typeof mount>) {
    await wrapper.find('[data-ai-test-connection]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
  }

  it('wrong key shows the rejected-key sentence, from the message key', async () => {
    route({
      ai_config_test_connection: () => {
        throw ipcError('ai_config.test_key_rejected')
      },
    })
    const { wrapper } = await open()
    await press(wrapper)
    const result = wrapper.find('[data-ai-test-result="error"]')
    expect(result.text()).toContain('từ chối khoá API')
    wrapper.unmount()
  })

  it('missing configuration is an invitation, not an alert', async () => {
    route({ ai_config_test_connection: () => ({ kind: 'not_configured' }) })
    const { wrapper } = await open()
    await press(wrapper)
    const result = wrapper.find('[data-ai-test-result="not_configured"]')
    expect(result.attributes('role')).toBe('status')
    expect(wrapper.find('[data-ai-test-result="error"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('dead network shows the unreachable sentence and a running test shows Huỷ', async () => {
    let fail: (e: unknown) => void = () => undefined
    route({ ai_config_test_connection: () => new Promise((_, reject) => (fail = reject)) })
    const { wrapper } = await open()
    await press(wrapper)
    expect(wrapper.find('[data-ai-test-result="running"]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-test-cancel]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-test-connection]').attributes('disabled')).toBeDefined()

    fail(ipcError('ai_config.test_unreachable'))
    await flushPromises()
    expect(wrapper.find('[data-ai-test-result="error"]').text()).toContain('Không tới được máy chủ')
    expect(wrapper.find('[data-ai-test-cancel]').exists()).toBe(false)
    wrapper.unmount()
  })
})
