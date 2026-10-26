import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'

const aiConfigGetMock = vi.fn()
const aiConfigClearOverrideMock = vi.fn()

vi.mock('../../src/config/aiconfig', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/aiconfig')>()
  return {
    ...actual,
    aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
    aiConfigClearOverride: (...args: unknown[]) => aiConfigClearOverrideMock(...args),
  }
})
vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

type Row = { field: string; value: string; tier: 'global' | 'work'; shadowed: string | null }

const g = (field: string, value: string): Row => ({ field, value, tier: 'global', shadowed: null })

function wire(opts: {
  rows: Row[]
  notReady: string[]
  workOpen?: boolean
  ready?: boolean | null
}) {
  return {
    fields: opts.rows,
    workTierAvailable: opts.workOpen ?? false,
    keyConfigured: opts.ready === null ? null : false,
    ready: opts.ready === undefined ? opts.notReady.length === 0 : opts.ready,
    notReady: opts.notReady,
    error: null,
  }
}

const OLLAMA_FULL = [g('provider', 'ollama'), g('endpoint', 'http://localhost:11434'), g('model', 'qwen')]

async function mountPanel(result: unknown) {
  vi.resetModules()
  aiConfigGetMock.mockReset()
  aiConfigClearOverrideMock.mockReset()
  aiConfigGetMock.mockResolvedValue(result)
  const commands = await import('../../src/commands')
  const aiConfigState = await import('../../src/aiConfigState')
  const Panel = (await import('../../src/panels/AiTranslationPanel.vue')).default
  const settingsState = await import('../../src/settingsState')
  commands.installCommands({
    openSettingsToAiConfig: settingsState.openSettingsToAiConfig,
  } as unknown as CommandDeps)
  const wrapper = mount(Panel, { props: { params: { params: {} } }, attachTo: document.body })
  await flushPromises()
  return { wrapper, aiConfigState, settingsState }
}

const reasons = (w: ReturnType<typeof mount>) =>
  w.findAll('[data-ai-unconfigured-reason]').map((r) => `${r.attributes('data-ai-unconfigured-reason-kind')}|${r.text()}`)

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('panel Đề xuất AI — lý do chưa sẵn sàng (I/O Matrix)', () => {
  it('ghi đè cần khoá: câu "Nhà cung cấp Anthropic cần khoá API", dòng ghi đè, nút Tác phẩm mở mục AI ở tầng Tác phẩm', async () => {
    const rows: Row[] = [
      { field: 'provider', value: 'anthropic', tier: 'work', shadowed: 'ollama' },
      g('endpoint', 'http://localhost:11434'),
      g('model', 'qwen'),
    ]
    const { wrapper, aiConfigState, settingsState } = await mountPanel(wire({ rows, notReady: ['key_missing'], workOpen: true }))

    expect(reasons(wrapper)).toEqual(['key_missing|Nhà cung cấp Anthropic cần khoá API.'])
    expect(wrapper.get('[data-ai-unconfigured-overrides]').text()).toBe('Tác phẩm này đang ghi đè: Nhà cung cấp.')
    aiConfigState.selectAiConfigViewTier('global')
    expect(aiConfigState.aiConfigViewTier.value).toBe('global')
    const button = wrapper.get('[data-ai-unconfigured-open-settings]')
    expect(button.text()).toBe('Mở Cài đặt ở phạm vi Tác phẩm này')
    expect(wrapper.findAll('[data-ai-unconfigured] button')).toHaveLength(1)
    await button.trigger('click')
    await flushPromises()
    expect(settingsState.settingsOverlayIsOpen.value).toBe(true)
    expect(settingsState.settingsActiveSection.value).toBe('ai_and_model')
    expect(aiConfigState.aiConfigWorkIsOpen.value).toBe(true)
    expect(aiConfigState.aiConfigViewTier.value).toBe('work')
    wrapper.unmount()
  })

  it('thiếu endpoint và mô hình: hai câu theo thứ tự, nút Mở Cài đặt ở tầng Toàn cục, không dòng ghi đè, không nút Tác phẩm', async () => {
    const { wrapper, aiConfigState, settingsState } = await mountPanel(wire({ rows: [g('provider', 'ollama')], notReady: ['endpoint_missing', 'model_missing'] }))

    expect(reasons(wrapper)).toEqual(['endpoint_missing|Chưa có endpoint.', 'model_missing|Chưa chọn mô hình.'])
    expect(wrapper.find('[data-ai-unconfigured-overrides]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-unconfigured-open-settings]').text()).toBe('Mở Cài đặt')
    await wrapper.get('[data-ai-unconfigured-open-settings]').trigger('click')
    await flushPromises()
    expect(settingsState.settingsOverlayIsOpen.value).toBe(true)
    expect(aiConfigState.aiConfigViewTier.value).toBe('global')
    wrapper.unmount()
  })

  it('chỉ thiếu mô hình: một câu, không câu về khoá', async () => {
    const rows = [g('provider', 'ollama'), g('endpoint', 'http://localhost:11434')]
    const { wrapper } = await mountPanel(wire({ rows, notReady: ['model_missing'] }))

    expect(reasons(wrapper)).toEqual(['model_missing|Chưa chọn mô hình.'])
    expect(wrapper.text()).not.toContain('cần khoá API')
    wrapper.unmount()
  })

  it('nhà cung cấp trống: "Chưa chọn nhà cung cấp — cần khoá API."', async () => {
    const rows = [g('provider', ''), g('endpoint', 'http://x.example'), g('model', 'm')]
    const { wrapper } = await mountPanel(wire({ rows, notReady: ['key_missing'] }))

    expect(reasons(wrapper)).toEqual(['key_missing|Chưa chọn nhà cung cấp — cần khoá API.'])
    wrapper.unmount()
  })

  it('nhà cung cấp lạ `groq`: dùng nguyên văn giá trị', async () => {
    const rows = [g('provider', 'groq'), g('endpoint', 'http://x.example'), g('model', 'm')]
    const { wrapper } = await mountPanel(wire({ rows, notReady: ['key_missing'] }))

    expect(reasons(wrapper)).toEqual(['key_missing|Nhà cung cấp groq cần khoá API.'])
    wrapper.unmount()
  })

  it('keychain không trả lời: gợi ý key_unchecked, không khối chưa sẵn sàng, không câu "cần khoá"', async () => {
    const { wrapper } = await mountPanel(wire({ rows: OLLAMA_FULL, notReady: [], ready: null }))

    expect(wrapper.find('[data-ai-key-unchecked]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('cần khoá API')
    wrapper.unmount()
  })

  it('sẵn sàng: không khối chưa sẵn sàng', async () => {
    const { wrapper } = await mountPanel(wire({ rows: OLLAMA_FULL, notReady: [] }))

    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('panel Đề xuất AI — Acceptance Criteria', () => {
  it('AC1: "Trả về kế thừa" nhà cung cấp (clearAiConfigOverride thật + đọc lại) làm khối chưa sẵn sàng biến mất khi panel vẫn mở', async () => {
    const overridden: Row[] = [
      { field: 'provider', value: 'anthropic', tier: 'work', shadowed: 'ollama' },
      g('endpoint', 'http://localhost:11434'),
      g('model', 'qwen'),
    ]
    const { wrapper, aiConfigState } = await mountPanel(wire({ rows: overridden, notReady: ['key_missing'], workOpen: true }))
    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(true)

    aiConfigClearOverrideMock.mockResolvedValue(null)
    aiConfigGetMock.mockResolvedValue(wire({ rows: OLLAMA_FULL, notReady: [], workOpen: true }))
    await aiConfigState.clearAiConfigOverride('provider')
    await flushPromises()

    expect(aiConfigClearOverrideMock).toHaveBeenCalledWith('provider')
    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-unconfigured-overrides]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('AC2: không có Tác phẩm nào mở ⇒ không dòng ghi đè, không nút phạm vi Tác phẩm', async () => {
    const rows: Row[] = [
      { field: 'provider', value: 'anthropic', tier: 'work', shadowed: 'ollama' },
      g('endpoint', 'http://localhost:11434'),
      g('model', 'qwen'),
    ]
    const { wrapper } = await mountPanel(wire({ rows, notReady: ['key_missing'], workOpen: false }))

    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-unconfigured-overrides]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-unconfigured-open-settings]').text()).toBe('Mở Cài đặt')
    wrapper.unmount()
  })
})
