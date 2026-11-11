import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invokeMock(...args), Channel: class {} }))
const fields = [
  { field: 'quick_provider', value: 'ollama' },
  { field: 'quick_endpoint', value: 'http://localhost/v1' },
  { field: 'quick_model', value: 'small' },
]
const wire = { fields, source: 'dedicated', key_configured: null, ready: true, not_ready: [] }
const denied = { code: 'ai_config.keychain_unavailable', message_key: 'err.ai_config.keychain_unavailable', params: {}, retryable: true }

async function fresh() {
  vi.resetModules()
  const state = await import('../../src/quickAiConfigState')
  const main = await import('../../src/aiConfigState')
  await main.loadAiConfigSection()
  main.selectAiConfigViewTier('global')
  await state.loadQuickAiConfigSection()
  return { state, main, adapter: await import('../../src/config/quickAiConfig'), component: (await import('../../src/SettingsQuickAiSection.vue')).default }
}

beforeEach(() => {
  invokeMock.mockReset()
  invokeMock.mockImplementation(async (command: string) => command === 'quick_ai_config_get' ? wire : command === 'ai_config_get' ? { fields: [], work_tier_available: true, key_configured: false, ready: false, not_ready: [], default_system_prompt: '' } : null)
})
afterEach(() => { vi.restoreAllMocks() })

describe('quick configuration', () => {
  it('mounts Global values, saves on blur, then shows the persisted values locked at Work scope', async () => {
    const { state, main, component } = await fresh()
    invokeMock.mockImplementation(async (command: string) => command === 'quick_ai_config_get' ? { ...wire, fields: fields.map(field => field.field === 'quick_model' ? { ...field, value: 'tiny' } : field) } : null)
    const wrapper = mount(component)
    expect(wrapper.text()).toContain('Dùng chung mọi Tác phẩm')
    await wrapper.get('[data-quick-ai-field="quick_model"]').setValue('tiny')
    await wrapper.get('[data-quick-ai-field="quick_model"]').trigger('blur')
    await flushPromises()
    expect(invokeMock).toHaveBeenCalledWith('quick_ai_config_save_fields', { tier: 'global', fields: [{ field: 'quick_model', value: 'tiny' }] })
    state.setQuickAiConfigKeyDraft('unsaved-secret')
    main.selectAiConfigViewTier('work')
    await state.loadQuickAiConfigSection()
    await flushPromises()
    expect((wrapper.get('[data-quick-ai-field="quick_model"]').element as HTMLInputElement).value).toBe('tiny')
    expect(wrapper.get('[data-quick-ai-locked]').text()).toBe('Sửa ở tầng Toàn cục.')
    expect(wrapper.get('fieldset').attributes('disabled')).toBeDefined()
    expect(state.quickAiConfigKeyDraft.value).toBe('')
    invokeMock.mockClear()
    state.setQuickAiConfigDraft('quick_model', 'forbidden')
    await state.commitQuickAiConfigDrafts()
    await state.saveQuickAiConfigKey()
    await state.deleteQuickAiConfigKey()
    await state.runQuickAiConfigTest()
    expect(invokeMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('renders each Rust-selected status and preserves unknown key state', async () => {
    const { state, component } = await fresh()
    const wrapper = mount(component)
    const messages = new Set<string>()
    for (const [source, expected] of [
      ['dedicated', 'Tự dịch chữ đang hiện ở Bảng chờ Glossary và màn xem trước nhập URL.'],
      ['main_global', 'Để trống thì bấm tay dùng cấu hình chính ở tầng Toàn cục. Tự dịch cần cấu hình riêng.'],
      ['incomplete', 'Cần cả endpoint và mô hình.'],
    ]) {
      invokeMock.mockResolvedValue({ ...wire, source, key_configured: null })
      await state.loadQuickAiConfigSection()
      await flushPromises()
      messages.add(wrapper.get('[data-quick-ai-config-status]').text())
      expect(wrapper.get('[data-quick-ai-config-status]').text()).toBe(expected)
      expect(wrapper.get('[data-quick-ai-key-status]').text()).toContain('Không kiểm tra được')
    }
    expect(messages.size).toBe(3)
    wrapper.unmount()
  })

  it('disposes secrets on failure, flush and reset, rendering translated key errors', async () => {
    const { state, component } = await fresh()
    const wrapper = mount(component)
    state.setQuickAiConfigKeyDraft('secret')
    invokeMock.mockRejectedValueOnce(denied)
    await state.saveQuickAiConfigKey()
    await flushPromises()
    expect(state.quickAiConfigKeyDraft.value).toBe('')
    expect(state.quickAiConfigKeyError.value).toEqual(denied)
    expect(wrapper.find('[role="alert"]').exists()).toBe(true)
    expect(wrapper.text()).not.toContain('secret')
    state.setQuickAiConfigKeyDraft('discard')
    state.flushQuickAiConfigDrafts()
    expect(state.quickAiConfigKeyDraft.value).toBe('')
    state.setQuickAiConfigKeyDraft('discard again')
    state.resetQuickAiConfig()
    expect(state.quickAiConfigKeyDraft.value).toBe('')
    wrapper.unmount()
  })

  it('cancels only the quick connection test and skips network while a save is pending', async () => {
    const { state } = await fresh()
    let release: (value: unknown) => void = () => undefined
    invokeMock.mockImplementation((command: string) => command === 'quick_ai_config_test_connection' ? new Promise(resolve => { release = resolve }) : Promise.resolve(command === 'quick_ai_config_get' ? wire : null))
    const running = state.runQuickAiConfigTest()
    await flushPromises()
    await state.cancelQuickAiConfigTest()
    expect(invokeMock).toHaveBeenCalledWith('quick_ai_config_test_connection_cancel')
    expect(invokeMock.mock.calls.some(([command]) => command === 'ai_config_test_connection_cancel')).toBe(false)
    release({ kind: 'cancelled' })
    await running
    expect(state.quickAiConfigTestStatus.value).toBe('cancelled')
    let finish: () => void = () => undefined
    invokeMock.mockImplementation((command: string) => command === 'quick_ai_config_save_fields' ? new Promise(resolve => { finish = () => resolve(null) }) : Promise.resolve(wire))
    invokeMock.mockClear()
    state.setQuickAiConfigDraft('quick_model', 'new')
    const testing = state.runQuickAiConfigTest()
    await flushPromises()
    await state.cancelQuickAiConfigTest()
    finish()
    await testing
    expect(invokeMock.mock.calls.some(([command]) => command === 'quick_ai_config_test_connection')).toBe(false)
    expect(state.quickAiConfigTestStatus.value).toBe('cancelled')
  })

  it('shows saved Global values after a rejected draft is closed and reopened at Work scope', async () => {
    const { state, main, component } = await fresh()
    invokeMock.mockImplementation(async (command: string) => command === 'quick_ai_config_save_fields' ? Promise.reject(denied) : command === 'quick_ai_config_get' ? wire : null)
    const wrapper = mount(component)
    await wrapper.get('[data-quick-ai-field="quick_endpoint"]').setValue('invalid-endpoint')
    await wrapper.get('[data-quick-ai-field="quick_endpoint"]').trigger('blur')
    await flushPromises()
    expect(state.quickAiConfigSaveError.value).toEqual(denied)
    state.flushQuickAiConfigDrafts()
    await flushPromises()
    wrapper.unmount()
    main.selectAiConfigViewTier('work')
    await state.loadQuickAiConfigSection()
    const reopened = mount(component)
    expect((reopened.get('[data-quick-ai-field="quick_endpoint"]').element as HTMLInputElement).value).toBe('http://localhost/v1')
    expect(reopened.get('fieldset').attributes('disabled')).toBeDefined()
    main.selectAiConfigViewTier('global')
    await flushPromises()
    expect((reopened.get('[data-quick-ai-field="quick_endpoint"]').element as HTMLInputElement).value).toBe('invalid-endpoint')
    reopened.unmount()
  })

  it('keeps cancellation available when a running test switches to Work scope', async () => {
    const { state, main, component } = await fresh()
    const commands = await import('../../src/commands')
    commands.installCommands({ isMac: true, setMode: () => {}, cancelQuickAiConfigTest: () => { void state.cancelQuickAiConfigTest() } })
    let release: (value: unknown) => void = () => undefined
    invokeMock.mockImplementation((command: string) => command === 'quick_ai_config_test_connection' ? new Promise(resolve => { release = resolve }) : Promise.resolve(null))
    const wrapper = mount(component)
    const running = state.runQuickAiConfigTest()
    await flushPromises()
    main.selectAiConfigViewTier('work')
    await flushPromises()
    const button = wrapper.findAll('button').find(button => button.text() === 'Huỷ thử kết nối')!
    expect(button.attributes('disabled')).toBeUndefined()
    await button.trigger('click')
    await flushPromises()
    expect(invokeMock).toHaveBeenCalledWith('quick_ai_config_test_connection_cancel')
    release({ kind: 'cancelled' })
    await running
    expect(state.quickAiConfigTestStatus.value).toBe('cancelled')
    wrapper.unmount()
  })

  it('saves dirty fields before testing and retains the verdict for the newly saved configuration', async () => {
    const { state } = await fresh()
    let finish: () => void = () => undefined
    invokeMock.mockImplementation((command: string) => command === 'quick_ai_config_save_fields' ? new Promise(resolve => { finish = () => resolve(null) }) : Promise.resolve(command === 'quick_ai_config_get' ? wire : { kind: 'connected', model_count: 2 }))
    state.setQuickAiConfigDraft('quick_model', 'new-model')
    invokeMock.mockClear()
    const testing = state.runQuickAiConfigTest()
    await flushPromises()
    expect(invokeMock.mock.calls.some(([command]) => command === 'quick_ai_config_test_connection')).toBe(false)
    finish()
    await testing
    expect(invokeMock).toHaveBeenCalledWith('quick_ai_config_test_connection')
    expect(state.quickAiConfigTestStatus.value).toBe('connected')
    expect(state.quickAiConfigTestModelCount.value).toBe(2)
  })

  it.each(['fields', 'save_key', 'delete_key'])('invalidates a completed verdict after successfully saving %s', async action => {
    const { state } = await fresh()
    invokeMock.mockImplementation(async (command: string) => command === 'quick_ai_config_test_connection' ? { kind: 'connected', model_count: 3 } : command === 'quick_ai_config_get' ? wire : null)
    await state.runQuickAiConfigTest()
    expect(state.quickAiConfigTestStatus.value).toBe('connected')
    if (action === 'fields') {
      state.setQuickAiConfigDraft('quick_model', 'new-model')
      await state.commitQuickAiConfigDrafts()
    } else if (action === 'save_key') {
      state.setQuickAiConfigKeyDraft('new-key')
      await state.saveQuickAiConfigKey()
    } else {
      await state.deleteQuickAiConfigKey()
    }
    expect(state.quickAiConfigTestStatus.value).toBe('idle')
    expect(state.quickAiConfigTestModelCount.value).toBe(0)
    expect(state.quickAiConfigTestError.value).toBeNull()
  })

  it.each(['fields', 'save_key', 'delete_key'])('ignores an old-config success arriving after %s changes', async action => {
    const { state } = await fresh()
    let release: (value: unknown) => void = () => undefined
    invokeMock.mockImplementation((command: string) => command === 'quick_ai_config_test_connection' ? new Promise(resolve => { release = resolve }) : Promise.resolve(command === 'quick_ai_config_get' ? wire : null))
    const running = state.runQuickAiConfigTest()
    await flushPromises()
    if (action === 'fields') {
      state.setQuickAiConfigDraft('quick_model', 'new-model')
      await state.commitQuickAiConfigDrafts()
    } else if (action === 'save_key') {
      state.setQuickAiConfigKeyDraft('new-key')
      await state.saveQuickAiConfigKey()
    } else {
      await state.deleteQuickAiConfigKey()
    }
    expect(state.quickAiConfigTestStatus.value).toBe('idle')
    release({ kind: 'connected', model_count: 3 })
    await running
    expect(state.quickAiConfigTestStatus.value).toBe('idle')
    expect(state.quickAiConfigTestModelCount.value).toBe(0)
  })

  it('rejects malformed IPC and passes errors without exposing returned data', async () => {
    const { adapter } = await fresh()
    for (const invalid of [null, { ...wire, fields: [fields[0], fields[0], fields[1]] }, { ...wire, ready: 0 }, { ...wire, source: 'work' }]) {
      invokeMock.mockResolvedValueOnce(invalid)
      expect((await adapter.quickAiConfigGet()).error?.code).toBe('ipc.unknown')
    }
    invokeMock.mockResolvedValueOnce({ kind: 'connected' })
    expect((await adapter.quickAiConfigTestConnection()).error?.code).toBe('ipc.unknown')
    invokeMock.mockRejectedValueOnce(denied)
    expect(await adapter.quickAiConfigDeleteKey()).toEqual(denied)
  })
})
