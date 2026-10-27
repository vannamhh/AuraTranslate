/**
 * Dòng "Lời nhà cung cấp" (AD-59): hàm đọc `params.provider_message` và bốn chỗ hiện lỗi.
 * Rust đúc `provider_message` ở `ai_translate_wire.rs` và `connection_check_contract.rs`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type { IpcError } from '../../src/i18n'

const singleState: Ref<string> = ref('idle')
const singleError: Ref<IpcError | null> = ref(null)
const batchState: Ref<string> = ref('idle')
const batchError: Ref<IpcError | null> = ref(null)

vi.mock('../../src/aiTranslateState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/aiTranslateState')>()
  return { ...actual, aiTranslateStateValue: singleState, aiTranslateError: singleError }
})

vi.mock('../../src/aiTranslateBatchState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/aiTranslateBatchState')>()
  return {
    ...actual,
    aiTranslateBatchStateValue: batchState,
    aiTranslateBatchError: batchError,
    aiTranslateBatchErrorBlock: () => null,
    aiTranslateBatchTotalCount: ref(1),
  }
})

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {},
}))
vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

const errorOf = (key: string, params: Record<string, string> = {}, retryable = false): IpcError => ({
  code: key,
  message_key: `err.${key}`,
  params,
  retryable,
})

const LABEL = 'Lời nhà cung cấp: '

beforeEach(() => {
  document.body.innerHTML = ''
  singleState.value = 'idle'
  singleError.value = null
  batchState.value = 'idle'
  batchError.value = null
  invokeMock.mockReset()
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
  vi.spyOn(console, 'info').mockImplementation(() => undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('providerMessageOf', () => {
  it('đọc chuỗi không rỗng; vắng, rỗng, sai kiểu hay lỗi null đều ra null', async () => {
    const { providerMessageOf } = await import('../../src/i18n')
    expect(providerMessageOf(errorOf('x', { provider_message: 'het tien' }))).toBe('het tien')
    expect(providerMessageOf(errorOf('x', { provider_message: '' }))).toBeNull()
    expect(providerMessageOf(errorOf('x', { status: '402' }))).toBeNull()
    expect(providerMessageOf(null)).toBeNull()
    const noParams = { code: 'x', message_key: 'err.x', retryable: false } as unknown as IpcError
    expect(providerMessageOf(noParams)).toBeNull()
    const wrongType = { ...errorOf('x'), params: { provider_message: 7 } } as unknown as IpcError
    expect(providerMessageOf(wrongType)).toBeNull()
  })
})

describe('AiTranslationPanel.vue — dòng lời nhà cung cấp', () => {
  async function mountPanel() {
    vi.resetModules()
    const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default
    return mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
  }

  it('một câu: lời hiện sau câu chính, như văn bản chứ không phải HTML', async () => {
    const wrapper = await mountPanel()
    singleState.value = 'error'
    singleError.value = errorOf(
      'ai_translate.provider_payment_required',
      { status: '402', provider_message: '<b>het tien</b>' },
    )
    await wrapper.vm.$nextTick()
    const alert = wrapper.get('[data-ai-translate-state="error"]')
    expect(alert.text()).toContain('Nhà cung cấp AI báo vấn đề thanh toán hoặc số dư (mã 402)')
    expect(alert.text()).toContain(`${LABEL}<b>het tien</b>`)
    expect(alert.find('b').exists()).toBe(false)
    wrapper.unmount()
  })

  it('một câu: không có provider_message thì không có dòng lời', async () => {
    const wrapper = await mountPanel()
    singleState.value = 'error'
    singleError.value = errorOf('ai_translate.provider_server_error', { status: '502' }, true)
    await wrapper.vm.$nextTick()
    const alert = wrapper.get('[data-ai-translate-state="error"]')
    expect(alert.text()).toContain('Sự cố phía nhà cung cấp AI (mã 502)')
    expect(alert.text()).not.toContain(LABEL)
    wrapper.unmount()
  })

  it('lô: lời hiện ở dòng riêng, vắng thì dòng không tồn tại', async () => {
    const wrapper = await mountPanel()
    batchState.value = 'error'
    batchError.value = errorOf('ai_translate.provider_rate_limited', {
      segment_id: '12',
      status: '429',
      provider_message: 'Slow down',
    })
    await wrapper.vm.$nextTick()
    const line = wrapper.get('[data-ai-batch-error-provider-message]')
    expect(line.text()).toBe(`${LABEL}Slow down`)

    batchError.value = errorOf('ai_translate.provider_rate_limited', { segment_id: '12', status: '429' })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-ai-batch-error-provider-message]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('SettingsOverlay — Kiểm tra kết nối hiện lời nhà cung cấp', () => {
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

  async function runTest(error: IpcError) {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'ai_config_get') return Promise.resolve(GET_REPLY)
      if (command === 'ai_config_test_connection') return Promise.reject(error)
      return Promise.resolve(null)
    })
    vi.resetModules()
    const settings = await import('../../src/settingsState')
    const state = await import('../../src/aiConfigState')
    const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
    settings.openSettings()
    settings.selectSettingsSection('ai_and_model')
    await state.loadAiConfigSection()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await flushPromises()
    await wrapper
      .find('[data-ai-test-connection]')
      .element.closest('form')!
      .dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    return wrapper
  }

  it('402 mang lời: câu chính và dòng lời cùng nằm trong kết quả lỗi', async () => {
    const wrapper = await runTest(
      errorOf('ai_config.test_payment_required', { status: '402', provider_message: 'Prepayment depleted.' }),
    )
    const text = wrapper.get('[data-ai-test-result="error"]').text()
    expect(text).toContain('(mã 402)')
    expect(text).toContain(`${LABEL}Prepayment depleted.`)
    wrapper.unmount()
  })

  it('không có lời thì chỉ có câu chính', async () => {
    const wrapper = await runTest(errorOf('ai_config.test_rate_limited', { status: '429' }))
    const text = wrapper.get('[data-ai-test-result="error"]').text()
    expect(text).toContain('(mã 429)')
    expect(text).not.toContain(LABEL)
    wrapper.unmount()
  })
})
