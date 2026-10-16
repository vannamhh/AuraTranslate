import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const aiConfigGetMock = vi.fn()
const aiConfigSaveFieldMock = vi.fn()
const aiConfigClearOverrideMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigClearOverride: (...args: unknown[]) => aiConfigClearOverrideMock(...args),
  aiConfigSaveKey: vi.fn(),
  aiConfigDeleteKey: vi.fn(),
  aiConfigTestConnection: vi.fn(),
  aiConfigTestConnectionCancel: vi.fn(),
}))
vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

const fields = [
  { field: 'provider', value: 'openai', tier: 'global', shadowed: null },
  { field: 'endpoint', value: 'https://e.example.com', tier: 'global', shadowed: null },
  { field: 'model', value: 'work-model', tier: 'work', shadowed: 'global-model' },
  { field: 'temperature', value: '0.5', tier: 'work', shadowed: null },
  { field: 'max_tokens', value: '100', tier: 'global', shadowed: null },
]

async function open(workTierAvailable: boolean, rows: readonly object[] = fields) {
  vi.resetModules()
  aiConfigGetMock.mockReset()
  aiConfigSaveFieldMock.mockReset()
  aiConfigSaveFieldMock.mockResolvedValue(null)
  aiConfigClearOverrideMock.mockReset()
  aiConfigClearOverrideMock.mockResolvedValue(null)
  aiConfigGetMock.mockResolvedValue({ fields: rows, workTierAvailable, keyConfigured: false, error: null })
  const settingsState = await import('../../src/settingsState')
  const aiConfigState = await import('../../src/aiConfigState')
  const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
  settingsState.openSettings()
  settingsState.selectSettingsSection('ai_and_model')
  await aiConfigState.loadAiConfigSection()
  const commands = await import('../../src/commands')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    clearAllAiConfigOverrides: () => void aiConfigState.clearAllAiConfigOverrides(),
  })
  const wrapper = mount(SettingsOverlay, { attachTo: document.body })
  await flushPromises()
  return { wrapper, settingsState, aiConfigState }
}

function inputs(wrapper: ReturnType<typeof mount>): string[] {
  return wrapper.findAll('.ai-field-form .ai-field-input').map((i) => (i.element as HTMLInputElement).value)
}

beforeEach(() => {
  document.body.innerHTML = ''
})

const global = (wrapper: ReturnType<typeof mount>) => wrapper.find('[data-settings-scope-global]')
const work = (wrapper: ReturnType<typeof mount>) => wrapper.find('[data-settings-scope-work]')
const pressed = (w: { attributes: (n: string) => string | undefined }) => w.attributes('aria-pressed')

describe('SettingsOverlay.vue — thanh Phạm vi của mục AI', () => {
  it('có Tác phẩm mở: mặc định Tác phẩm, hai nút đều bật, đếm đúng hai trường ghi đè', async () => {
    const { wrapper } = await open(true)
    expect(wrapper.find('[data-settings-scope]').exists()).toBe(true)
    expect(wrapper.findAll('.ai-tier input[type="radio"]')).toHaveLength(0)
    expect(pressed(work(wrapper))).toBe('true')
    expect(pressed(global(wrapper))).toBe('false')
    expect(work(wrapper).attributes('disabled')).toBeUndefined()
    expect(inputs(wrapper)[1]).toBe('work-model')
    expect(wrapper.find('[data-settings-scope-overrides]').text()).toContain('2')
    wrapper.unmount()
  })

  it('không có Tác phẩm: Tác phẩm bị khoá, Toàn cục được chọn, câu giải thích hiện', async () => {
    const { wrapper } = await open(false)
    expect(work(wrapper).attributes('disabled')).toBeDefined()
    expect(pressed(global(wrapper))).toBe('true')
    expect(wrapper.find('[data-settings-scope-work-unavailable]').text()).toContain('Chưa mở Tác phẩm nào')
    wrapper.unmount()
  })

  it('chọn Toàn cục: ô hiện giá trị Toàn cục bị che, lưu ghi tầng global, nút "Trả về kế thừa" ẩn', async () => {
    const { wrapper } = await open(true)
    expect(wrapper.findAll('[data-ai-field] .ai-field-clear-form')).toHaveLength(2)

    await global(wrapper).element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
    expect(inputs(wrapper)[1]).toBe('global-model')
    expect(wrapper.findAll('[data-ai-field] .ai-field-clear-form')).toHaveLength(0)
    expect(wrapper.find('[data-settings-scope-msg]').text()).toContain('Toàn cục')

    const modelInput = wrapper.find('[data-ai-field="model"] input')
    await modelInput.setValue('g2')
    await modelInput.trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('global', 'model', 'g2')
    wrapper.unmount()
  })

  it('rời mục AI rồi chọn lại ⇒ tầng xem về mặc định Tác phẩm dù trước đó đã chọn Toàn cục', async () => {
    const { wrapper, settingsState, aiConfigState } = await open(true)
    aiConfigState.selectAiConfigViewTier('global')
    expect(aiConfigState.aiConfigViewTier.value).toBe('global')

    settingsState.selectSettingsSection('privacy')
    settingsState.selectSettingsSection('ai_and_model')
    await flushPromises()

    expect(aiConfigState.aiConfigViewTier.value).toBe('work')
    expect(inputs(wrapper)[1]).toBe('work-model')
    wrapper.unmount()
  })

  it('không có nút Lưu nào trong mục: rời ô là lưu, và giá trị không đổi thì không ghi', async () => {
    const { wrapper } = await open(true)
    const saveButtons = wrapper.findAll('button').filter((b) => b.text() === 'Lưu')
    expect(saveButtons).toHaveLength(0)

    const input = wrapper.find('[data-ai-field="model"] input')
    await input.trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()

    await input.setValue('edited')
    await input.trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('work', 'model', 'edited')
    wrapper.unmount()
  })

  it('"Trả toàn bộ mục về kế thừa" gọi ClearOverride cho đúng hai trường ghi đè và đếm về 0', async () => {
    const { wrapper } = await open(true)
    expect(wrapper.find('[data-ai-clear-all]').attributes('disabled')).toBeUndefined()
    aiConfigGetMock.mockResolvedValue({
      fields: fields.map((f) => ({ ...f, tier: 'global', shadowed: null })),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
    await wrapper.find('[data-ai-clear-all]').trigger('click')
    await flushPromises()

    expect(aiConfigClearOverrideMock.mock.calls.map((c) => c[0]).sort()).toEqual(['model', 'temperature'])
    expect(wrapper.find('[data-settings-scope-overrides]').text()).toContain('0')
    expect(wrapper.find('[data-ai-clear-all]').attributes('disabled')).toBeDefined()
    expect(wrapper.findAll('[data-ai-field] .ai-field-clear-form')).toHaveLength(0)
    wrapper.unmount()
  })

  it('chọn "Ollama" khi endpoint trống: chỉ ghi provider, endpoint vẫn trống, hiện placeholder', async () => {
    const emptyRows = ['provider', 'endpoint', 'model', 'temperature', 'max_tokens'].map((field) => ({
      field,
      value: '',
      tier: 'global',
      shadowed: null,
    }))
    const { wrapper } = await open(false, emptyRows)
    aiConfigGetMock.mockResolvedValue({
      fields: [
        { field: 'provider', value: 'ollama', tier: 'global', shadowed: null },
        ...emptyRows.slice(1),
      ],
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await wrapper.find('[data-ai-provider="ollama"]').element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()

    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('global', 'provider', 'ollama')
    const endpoint = wrapper.find('[data-ai-field="endpoint"] input')
    expect((endpoint.element as HTMLInputElement).value).toBe('')
    expect(endpoint.attributes('placeholder')).toBe('http://localhost:11434/v1')
    wrapper.unmount()
  })

  it('giá trị provider ngoài danh sách bật "Endpoint khác" và không nút nào khác', async () => {
    const { wrapper } = await open(false, fields.map((f) => (f.field === 'provider' ? { ...f, value: 'groq' } : f)))
    const on = wrapper
      .findAll('[data-ai-provider]')
      .filter((b) => b.attributes('aria-pressed') === 'true')
      .map((b) => b.attributes('data-ai-provider'))
    expect(on).toEqual(['custom'])
    wrapper.unmount()
  })
})
