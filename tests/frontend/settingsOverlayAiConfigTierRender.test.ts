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

const DEFAULT_SYSTEM = 'Mặc định của app.\nDòng hai.'

const fields = [
  { field: 'provider', value: 'openai', tier: 'global', shadowed: null },
  { field: 'endpoint', value: 'https://e.example.com', tier: 'global', shadowed: null },
  { field: 'model', value: 'work-model', tier: 'work', shadowed: 'global-model' },
  { field: 'temperature', value: '0.5', tier: 'work', shadowed: null },
  { field: 'max_tokens', value: '100', tier: 'global', shadowed: null },
  { field: 'system_prompt', value: DEFAULT_SYSTEM, tier: 'global', shadowed: null },
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
  aiConfigGetMock.mockResolvedValue({ fields: rows, defaultSystemPrompt: DEFAULT_SYSTEM, workTierAvailable, keyConfigured: false, error: null })
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
      value: '',
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

  it('chọn "Gemini" khi trống: một lời gọi ghi năm trường, không hỏi, ô hiện giá trị đã lưu và nút Gemini bật', async () => {
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
      { field: 'temperature', value: 'default' },
      { field: 'max_tokens', value: 'default' },
    ])
    expect(wrapper.find('[data-ai-provider="gemini"]').attributes('aria-pressed')).toBe('true')
    expect((wrapper.find('[data-ai-field="endpoint"] input').element as HTMLInputElement).value).toBe(
      'https://generativelanguage.googleapis.com/v1beta/openai',
    )
    expect((wrapper.find('[data-ai-field="model"] input').element as HTMLInputElement).value).toBe('gemini-flash-latest')
    wrapper.unmount()
  })

  it('chọn "Ollama" khi endpoint trống và model là `qwen`: ghi provider, endpoint, nhiệt độ và số token, giữ `qwen`', async () => {
    const rows = emptyRows().map((r) => (r.field === 'model' ? { ...r, value: 'qwen' } : r))
    const { wrapper } = await open(false, rows)
    await pick(wrapper, 'ollama')

    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'ollama' },
      { field: 'endpoint', value: 'http://localhost:11434/v1' },
      { field: 'temperature', value: '0.2' },
      { field: 'max_tokens', value: 'default' },
    ])
    wrapper.unmount()
  })

  it('endpoint đang khác preset: hộp alertdialog nêu giá trị cũ và mới; Huỷ không ghi gì, Áp dụng ghi cả năm', async () => {
    const { wrapper } = await open(false)
    await pick(wrapper, 'anthropic')

    const dialog = wrapper.find('[role="alertdialog"][data-ai-preset-confirm]')
    expect(dialog.exists()).toBe(true)
    expect(dialog.attributes('data-ai-preset-provider')).toBe('anthropic')
    expect(dialog.find('[data-ai-preset-change="endpoint"]').text()).toContain('https://e.example.com')
    expect(dialog.find('[data-ai-preset-change="endpoint"]').text()).toContain('https://api.anthropic.com/v1')
    expect(dialog.find('[data-ai-preset-change="model"]').exists()).toBe(true)
    expect(dialog.find('[data-ai-preset-change="temperature"]').exists()).toBe(false)
    expect(dialog.find('[data-ai-preset-change="max_tokens"]').text()).toContain('100')
    expect(dialog.find('[data-ai-preset-change="max_tokens"]').text()).toContain('8192')
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
      { field: 'temperature', value: 'default' },
      { field: 'max_tokens', value: '8192' },
    ])
    wrapper.unmount()
  })

  it('tầng Tác phẩm: bấm Mistral (endpoint và model khác preset) rồi Áp dụng ghi năm trường ở tầng work', async () => {
    const { wrapper } = await open(true)
    await pick(wrapper, 'mistral')
    await wrapper.find('[data-ai-preset-apply]').trigger('click')
    await flushPromises()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('work', [
      { field: 'provider', value: 'mistral' },
      { field: 'endpoint', value: 'https://api.mistral.ai/v1' },
      { field: 'model', value: 'mistral-large-latest' },
      { field: 'temperature', value: '0.2' },
      { field: 'max_tokens', value: 'default' },
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

describe('SettingsOverlay.vue — hàng chip nhiệt độ và số token', () => {
  const rowsWith = (temperature: string, maxTokens: string, tier = 'global') =>
    ['provider', 'endpoint', 'model', 'temperature', 'max_tokens'].map((field) => ({
      field,
      value: field === 'temperature' ? temperature : field === 'max_tokens' ? maxTokens : '',
      tier: field === 'temperature' ? tier : 'global',
      shadowed: null,
    }))
  const pick = async (wrapper: ReturnType<typeof mount>, provider: string) => {
    await wrapper.find(`[data-ai-provider="${provider}"]`).element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
  }
  const chips = (wrapper: ReturnType<typeof mount>, field: string) =>
    wrapper.findAll(`[data-ai-field="${field}"] [data-ai-chip]`).map((b) => b.attributes('data-ai-chip'))
  const litChips = (wrapper: ReturnType<typeof mount>, field: string) =>
    wrapper
      .findAll(`[data-ai-field="${field}"] [data-ai-chip]`)
      .filter((b) => b.attributes('aria-pressed') === 'true')
      .map((b) => b.attributes('data-ai-chip'))
  const click = async (wrapper: ReturnType<typeof mount>, field: string, chip: string) => {
    await wrapper
      .find(`[data-ai-field="${field}"] [data-ai-chip="${chip}"]`)
      .element.closest('form')!
      .dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
  }

  it('mỗi trường là một hàng chip theo thứ tự Mặc định · số · Tự nhập, không còn ô nhập thường', async () => {
    const { wrapper } = await open(false, rowsWith('', ''))
    expect(chips(wrapper, 'temperature')).toEqual(['default', '0.2', '0.5', '1', 'custom'])
    expect(chips(wrapper, 'max_tokens')).toEqual(['default', '4096', '8192', '16384', 'custom'])
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip="default"]').text()).toBe('Mặc định nhà cung cấp')
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip="0.2"]').text()).toContain('Sát nghĩa')
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip="0.5"]').text()).toContain('Cân bằng')
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip="1"]').text()).toContain('Tự nhiên')
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip="custom"]').text()).toBe('Tự nhập')
    expect(wrapper.findAll('[data-ai-chip-input]')).toHaveLength(0)
    wrapper.unmount()
  })

  it('AC: giá trị lưu là `default` hoặc không có hàng ⇒ chip Mặc định bật', async () => {
    for (const stored of ['', 'default']) {
      const { wrapper } = await open(false, rowsWith(stored, stored))
      expect(litChips(wrapper, 'temperature')).toEqual(['default'])
      expect(litChips(wrapper, 'max_tokens')).toEqual(['default'])
      wrapper.unmount()
    }
    const { wrapper } = await open(false, [])
    expect(litChips(wrapper, 'temperature')).toEqual(['default'])
    wrapper.unmount()
  })

  it('giá trị khớp chip số thì chip đó bật; giá trị khác bật Tự nhập kèm ô nhập hiện giá trị', async () => {
    const { wrapper } = await open(false, rowsWith('0.5', '5000'))
    expect(litChips(wrapper, 'temperature')).toEqual(['0.5'])
    expect(litChips(wrapper, 'max_tokens')).toEqual(['custom'])
    expect((wrapper.find('[data-ai-field="max_tokens"] [data-ai-chip-input]').element as HTMLInputElement).value).toBe('5000')
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip-input]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('I/O Matrix "Chọn chip số": bấm 0.5 lưu temperature = 0.5 ở tầng Toàn cục, chip 0.5 bật', async () => {
    const { wrapper } = await open(false, rowsWith('', ''))
    aiConfigGetMock.mockResolvedValue({
      fields: rowsWith('0.5', ''),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await click(wrapper, 'temperature', '0.5')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'temperature', '0.5')
    expect(litChips(wrapper, 'temperature')).toEqual(['0.5'])
    wrapper.unmount()
  })

  it('AC tầng Tác phẩm: bấm chip chỉ ghi tầng work, và hàng ghi đè có nút "Trả về kế thừa"', async () => {
    const { wrapper } = await open(true, rowsWith('0.3', ''))
    expect(wrapper.findAll('[data-ai-field="temperature"] .ai-field-clear-form')).toHaveLength(0)
    aiConfigGetMock.mockResolvedValue({
      fields: rowsWith('default', '', 'work'),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
    await click(wrapper, 'temperature', 'default')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('work', 'temperature', 'default')
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(aiConfigClearOverrideMock).not.toHaveBeenCalled()
    expect(litChips(wrapper, 'temperature')).toEqual(['default'])
    expect(wrapper.findAll('[data-ai-field="temperature"] .ai-field-clear-form')).toHaveLength(1)
    expect(wrapper.findAll('[data-ai-field="max_tokens"] .ai-field-clear-form')).toHaveLength(0)
    wrapper.unmount()
  })

  it('I/O Matrix "Tự nhập": bấm Tự nhập mở ô không ghi; gõ 0.35 rời ô lưu 0.35; gõ 3 hiện câu từ chối, 0 lượt ghi', async () => {
    const { wrapper } = await open(false, rowsWith('', ''))
    await click(wrapper, 'temperature', 'custom')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(litChips(wrapper, 'temperature')).toEqual(['custom'])

    const input = wrapper.find('[data-ai-field="temperature"] [data-ai-chip-input]')
    await input.setValue('3')
    await input.trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(wrapper.find('[data-ai-field="temperature"] .ai-field-alert').exists()).toBe(true)

    aiConfigGetMock.mockResolvedValue({
      fields: rowsWith('0.35', ''),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await input.setValue('0.35')
    await input.trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'temperature', '0.35')
    expect(litChips(wrapper, 'temperature')).toEqual(['custom'])
    expect((wrapper.find('[data-ai-field="temperature"] [data-ai-chip-input]').element as HTMLInputElement).value).toBe('0.35')
    wrapper.unmount()
  })

  it('ô Tự nhập đang lưu khi rời ô thì các chip của trường vẫn bấm được, nên cú bấm làm rời ô không bị nuốt', async () => {
    const { wrapper } = await open(false, rowsWith('', ''))
    await click(wrapper, 'temperature', 'custom')
    aiConfigSaveFieldMock.mockImplementationOnce(() => new Promise(() => undefined))
    const input = wrapper.find('[data-ai-field="temperature"] [data-ai-chip-input]')
    await input.setValue('0.35')
    await input.trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledOnce()
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip="0.5"]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('đóng ô Tự nhập bằng chip đang bật thì câu từ chối của giá trị gõ dở biến mất', async () => {
    const { wrapper } = await open(false, rowsWith('', ''))
    await click(wrapper, 'temperature', 'custom')
    const input = wrapper.find('[data-ai-field="temperature"] [data-ai-chip-input]')
    await input.setValue('3')
    await input.trigger('blur')
    await flushPromises()
    expect(wrapper.find('[data-ai-field="temperature"] .ai-field-alert').exists()).toBe(true)
    await click(wrapper, 'temperature', 'default')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(wrapper.find('[data-ai-field="temperature"] [data-ai-chip-input]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-field="temperature"] .ai-field-alert').exists()).toBe(false)
    wrapper.unmount()
  })

  it('AC: placeholder ô khoá API theo nhà cung cấp, OpenAI là `sk-…`, Mistral không có', async () => {
    const withProvider = (provider: string) =>
      rowsWith('', '').map((r) => (r.field === 'provider' ? { ...r, value: provider } : r))
    const keyInput = (w: ReturnType<typeof mount>) => w.find('.ai-key-field input').element as HTMLInputElement
    const openai = await open(false, withProvider('openai'))
    expect(keyInput(openai.wrapper).placeholder).toBe('sk-…')
    openai.wrapper.unmount()
    const mistral = await open(false, withProvider('mistral'))
    expect(keyInput(mistral.wrapper).placeholder).toBe('')
    mistral.wrapper.unmount()
  })

  it('I/O Matrix "Áp preset đè nhiệt độ": 0.3 rồi Anthropic ⇒ dòng "Nhiệt độ: 0.3 → Mặc định nhà cung cấp"; Huỷ không ghi gì', async () => {
    const { wrapper } = await open(false, rowsWith('0.3', ''))
    await pick(wrapper, 'anthropic')
    const line = wrapper.find('[data-ai-preset-change="temperature"]')
    expect(line.text()).toContain('0.3')
    expect(line.text()).toContain('Mặc định nhà cung cấp')
    await wrapper.find('[data-ai-preset-cancel]').trigger('click')
    await flushPromises()
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('I/O Matrix "Áp preset từ trống": Mistral ghi 0.2 và default cùng ba trường kia, không hỏi', async () => {
    const { wrapper } = await open(false, rowsWith('', ''))
    await pick(wrapper, 'mistral')
    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'mistral' },
      { field: 'endpoint', value: 'https://api.mistral.ai/v1' },
      { field: 'model', value: 'mistral-large-latest' },
      { field: 'temperature', value: '0.2' },
      { field: 'max_tokens', value: 'default' },
    ])
    wrapper.unmount()
  })

  it('giá trị lưu `default` và preset cũng `default` ⇒ áp OpenAI không hỏi', async () => {
    const { wrapper } = await open(false, rowsWith('default', 'default'))
    await pick(wrapper, 'openai')
    expect(wrapper.find('[data-ai-preset-confirm]').exists()).toBe(false)
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })
})

describe('SettingsOverlay.vue — ô System prompt', () => {
  const base = fields.filter((f) => f.field !== 'system_prompt')
  const rows = (system: object) => [...base, system]
  const box = (wrapper: ReturnType<typeof mount>) => wrapper.get('textarea[data-ai-system-prompt]')
  const boxValue = (wrapper: ReturnType<typeof mount>) => (box(wrapper).element as HTMLTextAreaElement).value
  const status = (wrapper: ReturnType<typeof mount>) => wrapper.get('[data-ai-system-prompt-status]').text()
  const restore = (wrapper: ReturnType<typeof mount>) => wrapper.find('[data-ai-system-prompt-restore]')
  const submitRestore = async (wrapper: ReturnType<typeof mount>) => {
    await restore(wrapper).element.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()
  }

  it('I/O Matrix "Mặc định" + AC Khôi phục: không dòng nào ⇒ ô hiện văn bản của app, "Mặc định của app", không nút Khôi phục', async () => {
    const { wrapper } = await open(false, rows({ field: 'system_prompt', value: DEFAULT_SYSTEM, tier: 'global', shadowed: null }))
    expect(boxValue(wrapper)).toBe(DEFAULT_SYSTEM)
    expect(status(wrapper)).toBe('Mặc định của app')
    expect(restore(wrapper).exists()).toBe(false)
    wrapper.unmount()
  })

  it('I/O Matrix "Ice sửa": văn bản riêng ở Toàn cục ⇒ "Đã sửa"; Khôi phục xoá dòng ở tầng global rồi ô trở về văn bản của app', async () => {
    const { wrapper } = await open(false, rows({ field: 'system_prompt', value: 'Của Ice', tier: 'global', shadowed: null }))
    expect(boxValue(wrapper)).toBe('Của Ice')
    expect(status(wrapper)).toBe('Đã sửa')
    aiConfigGetMock.mockResolvedValue({
      fields: rows({ field: 'system_prompt', value: DEFAULT_SYSTEM, tier: 'global', shadowed: null }),
      defaultSystemPrompt: DEFAULT_SYSTEM,
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await submitRestore(wrapper)
    expect(aiConfigClearOverrideMock).toHaveBeenCalledExactlyOnceWith('system_prompt', 'global')
    expect(boxValue(wrapper)).toBe(DEFAULT_SYSTEM)
    expect(status(wrapper)).toBe('Mặc định của app')
    expect(restore(wrapper).exists()).toBe(false)
    wrapper.unmount()
  })

  it('I/O Matrix "Tắt": ô rỗng lưu ở Toàn cục ⇒ trạng thái Tắt và vẫn có nút Khôi phục', async () => {
    const { wrapper } = await open(false, rows({ field: 'system_prompt', value: '', tier: 'global', shadowed: null }))
    expect(boxValue(wrapper)).toBe('')
    expect(status(wrapper)).toContain('Tắt')
    expect(restore(wrapper).exists()).toBe(true)
    wrapper.unmount()
  })

  it('lưu khi rời ô giữ nguyên văn xuống dòng và khoảng trắng đầu/cuối, ghi đúng tầng đang xem', async () => {
    const { wrapper } = await open(false)
    const typed = '  Dòng một.\n\n  Dòng ba.  \n'
    await box(wrapper).setValue(typed)
    await box(wrapper).trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'system_prompt', typed)
    wrapper.unmount()
  })

  it('xoá hết ô rồi rời ô lưu chuỗi rỗng (Tắt), không bật lại giá trị cũ, và ô không đổi thì không ghi', async () => {
    const { wrapper } = await open(false)
    await box(wrapper).trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    await box(wrapper).setValue('')
    await box(wrapper).trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'system_prompt', '')
    wrapper.unmount()
  })

  it('I/O Matrix "Lỗi lưu": ghi trượt ⇒ hiện lỗi và bản nháp giữ nguyên trong ô', async () => {
    const { wrapper } = await open(false)
    aiConfigSaveFieldMock.mockResolvedValue({ code: 'x.fail', message_key: 'err.unknown', params: {}, retryable: false })
    await box(wrapper).setValue('Bản nháp\ndở dang')
    await box(wrapper).trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(wrapper.find('[data-ai-field="system_prompt"] [role="alert"]').exists()).toBe(true)
    expect(boxValue(wrapper)).toBe('Bản nháp\ndở dang')
    wrapper.unmount()
  })

  it('I/O Matrix "Ghi đè Tác phẩm" + "Tắt ở một Tác phẩm": Tác phẩm tắt ⇒ Tắt, Toàn cục vẫn "Đã sửa"; Khôi phục ở Tác phẩm xoá tầng work; lưu ghi tầng work', async () => {
    const { wrapper, aiConfigState } = await open(
      true,
      rows({ field: 'system_prompt', value: '', tier: 'work', shadowed: 'Toàn cục A' }),
    )
    expect(boxValue(wrapper)).toBe('')
    expect(status(wrapper)).toContain('Tắt')

    aiConfigState.selectAiConfigViewTier('global')
    await flushPromises()
    expect(boxValue(wrapper)).toBe('Toàn cục A')
    expect(status(wrapper)).toBe('Đã sửa')
    aiConfigState.selectAiConfigViewTier('work')
    await flushPromises()

    await box(wrapper).setValue('Của Tác phẩm B')
    await box(wrapper).trigger('blur')
    await flushPromises()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('work', 'system_prompt', 'Của Tác phẩm B')

    await submitRestore(wrapper)
    expect(aiConfigClearOverrideMock).toHaveBeenCalledExactlyOnceWith('system_prompt', 'work')
    wrapper.unmount()
  })

  it('Toàn cục bị Tác phẩm che mà không có dòng Toàn cục ⇒ xem Toàn cục hiện văn bản của app', async () => {
    const { wrapper, aiConfigState } = await open(
      true,
      rows({ field: 'system_prompt', value: 'Của Tác phẩm', tier: 'work', shadowed: null }),
    )
    aiConfigState.selectAiConfigViewTier('global')
    await flushPromises()
    expect(boxValue(wrapper)).toBe(DEFAULT_SYSTEM)
    expect(status(wrapper)).toBe('Mặc định của app')
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
