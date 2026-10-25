import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const aiConfigGetMock = vi.fn()
const aiConfigSaveFieldMock = vi.fn()
const aiConfigSaveFieldsMock = vi.fn()
const aiConfigClearOverrideMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigSaveFields: (...args: unknown[]) => aiConfigSaveFieldsMock(...args),
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
  aiConfigSaveFieldsMock.mockReset()
  aiConfigSaveFieldsMock.mockResolvedValue(null)
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
    confirmAiConfigPreset: () => void aiConfigState.confirmAiConfigPreset(),
    cancelAiConfigPreset: aiConfigState.cancelAiConfigPreset,
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

  const emptyRows = () =>
    ['provider', 'endpoint', 'model', 'temperature', 'max_tokens'].map((field) => ({
      field,
      value: field === 'temperature' ? '0.3' : '',
      tier: 'global',
      shadowed: null,
    }))
  const pick = async (wrapper: ReturnType<typeof mount>, provider: string) => {
    await wrapper.find(`[data-ai-provider="${provider}"]`).element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
  }

  it('có đúng chín nút nhà cung cấp theo thứ tự của danh sách', async () => {
    const { wrapper } = await open(false)
    expect(wrapper.findAll('[data-ai-provider]').map((b) => b.attributes('data-ai-provider'))).toEqual([
      'anthropic', 'openai', 'gemini', 'openrouter', 'mistral', 'xai', 'ollama', 'lmstudio', 'custom',
    ])
    wrapper.unmount()
  })

  it('chọn "Gemini" khi trống: một lời gọi ghi ba trường, không hỏi, ô hiện giá trị đã lưu và nút Gemini bật', async () => {
    const { wrapper } = await open(false, emptyRows())
    aiConfigGetMock.mockResolvedValue({
      fields: [
        { field: 'provider', value: 'gemini', tier: 'global', shadowed: null },
        { field: 'endpoint', value: 'https://generativelanguage.googleapis.com/v1beta/openai', tier: 'global', shadowed: null },
        { field: 'model', value: 'gemini-flash-latest', tier: 'global', shadowed: null },
        ...emptyRows().slice(3),
      ],
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await pick(wrapper, 'gemini')

    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'gemini' },
      { field: 'endpoint', value: 'https://generativelanguage.googleapis.com/v1beta/openai' },
      { field: 'model', value: 'gemini-flash-latest' },
    ])
    expect(wrapper.find('[data-ai-provider="gemini"]').attributes('aria-pressed')).toBe('true')
    expect((wrapper.find('[data-ai-field="endpoint"] input').element as HTMLInputElement).value).toBe(
      'https://generativelanguage.googleapis.com/v1beta/openai',
    )
    expect((wrapper.find('[data-ai-field="model"] input').element as HTMLInputElement).value).toBe('gemini-flash-latest')
    wrapper.unmount()
  })

  it('chọn "Ollama" khi endpoint trống và model là `qwen`: ghi provider và endpoint, giữ `qwen`', async () => {
    const rows = emptyRows().map((r) => (r.field === 'model' ? { ...r, value: 'qwen' } : r))
    const { wrapper } = await open(false, rows)
    await pick(wrapper, 'ollama')

    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'ollama' },
      { field: 'endpoint', value: 'http://localhost:11434/v1' },
    ])
    wrapper.unmount()
  })

  it('endpoint đang khác preset: hộp alertdialog nêu giá trị cũ và mới; Huỷ không ghi gì, Áp dụng ghi cả ba', async () => {
    const { wrapper } = await open(false)
    await pick(wrapper, 'anthropic')

    const dialog = wrapper.find('[role="alertdialog"][data-ai-preset-confirm]')
    expect(dialog.exists()).toBe(true)
    expect(dialog.attributes('data-ai-preset-provider')).toBe('anthropic')
    expect(dialog.find('[data-ai-preset-change="endpoint"]').text()).toContain('https://e.example.com')
    expect(dialog.find('[data-ai-preset-change="endpoint"]').text()).toContain('https://api.anthropic.com/v1')
    expect(dialog.find('[data-ai-preset-change="model"]').exists()).toBe(true)
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()

    await dialog.find('[data-ai-preset-cancel]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect((wrapper.find('[data-ai-field="endpoint"] input').element as HTMLInputElement).value).toBe('https://e.example.com')

    await pick(wrapper, 'anthropic')
    await wrapper.find('[data-ai-preset-apply]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'anthropic' },
      { field: 'endpoint', value: 'https://api.anthropic.com/v1' },
      { field: 'model', value: 'claude-sonnet-5-5' },
    ])
    wrapper.unmount()
  })

  it('tầng Tác phẩm: bấm Mistral (endpoint và model khác preset) rồi Áp dụng ghi ba trường ở tầng work', async () => {
    const { wrapper } = await open(true)
    await pick(wrapper, 'mistral')
    await wrapper.find('[data-ai-preset-apply]').trigger('click')
    await flushPromises()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('work', [
      { field: 'provider', value: 'mistral' },
      { field: 'endpoint', value: 'https://api.mistral.ai/v1' },
      { field: 'model', value: 'mistral-large-latest' },
    ])
    wrapper.unmount()
  })

  it('lượt ghi trượt: lỗi hiện dưới hàng nút nhà cung cấp và hộp không mở lại', async () => {
    aiConfigSaveFieldsMock.mockResolvedValue({ code: 'ai_config.work_tier_unavailable', message: 'x', params: {} })
    const { wrapper } = await open(false, emptyRows())
    aiConfigSaveFieldsMock.mockResolvedValue({ code: 'ai_config.work_tier_unavailable', message: 'x', params: {} })
    await pick(wrapper, 'gemini')
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledTimes(1)
    expect(wrapper.find('[data-ai-field="provider"] [role="alert"]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
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

describe('main.ts — dây hai command của hộp xác nhận preset', () => {
  it('installCommands nhận confirmAiConfigPreset và cancelAiConfigPreset từ aiConfigState', async () => {
    const { readFileSync } = await import('node:fs')
    const { resolve } = await import('node:path')
    const code = readFileSync(resolve(process.cwd(), 'src/main.ts'), 'utf8')
      .split('\n')
      .filter((line) => !/^\s*(\/\/|\*|\/\*)/.test(line))
      .join('\n')
    expect(code).toMatch(/^import \{ cancelAiConfigPreset, confirmAiConfigPreset \} from '\.\/aiConfigState'$/m)
    expect(code).toMatch(/^\s*confirmAiConfigPreset: \(\) => \{\n\s*void confirmAiConfigPreset\(\)\n\s*\},$/m)
    expect(code).toMatch(/^\s*cancelAiConfigPreset,$/m)
  })
})
