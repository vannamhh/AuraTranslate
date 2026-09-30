import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const aiConfigGetMock = vi.fn()
const aiConfigSaveFieldMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigClearOverride: vi.fn(),
  aiConfigSaveKey: vi.fn(),
  aiConfigDeleteKey: vi.fn(),
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

async function open(workTierAvailable: boolean) {
  vi.resetModules()
  aiConfigGetMock.mockReset()
  aiConfigSaveFieldMock.mockReset()
  aiConfigSaveFieldMock.mockResolvedValue(null)
  aiConfigGetMock.mockResolvedValue({ fields, workTierAvailable, keyConfigured: false, error: null })
  const settingsState = await import('../../src/settingsState')
  const aiConfigState = await import('../../src/aiConfigState')
  const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
  settingsState.openSettings()
  settingsState.selectSettingsSection('ai_and_model')
  await aiConfigState.loadAiConfigSection()
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

describe('SettingsOverlay.vue — bộ chọn tầng của mục AI', () => {
  it('có Tác phẩm mở: mặc định Tác phẩm, hai lựa chọn đều bật', async () => {
    const { wrapper } = await open(true)
    const radios = wrapper.findAll('.ai-tier input[type="radio"]')
    expect(radios).toHaveLength(2)
    expect((radios[0].element as HTMLInputElement).checked).toBe(false)
    expect((radios[1].element as HTMLInputElement).checked).toBe(true)
    expect(radios[1].attributes('disabled')).toBeUndefined()
    expect(inputs(wrapper)[2]).toBe('work-model')
    wrapper.unmount()
  })

  it('không có Tác phẩm: Tác phẩm bị khoá, Toàn cục được chọn, câu giải thích hiện', async () => {
    const { wrapper } = await open(false)
    const radios = wrapper.findAll('.ai-tier input[type="radio"]')
    expect(radios[1].attributes('disabled')).toBeDefined()
    expect((radios[0].element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.text()).toContain('Chưa mở Tác phẩm nào')
    wrapper.unmount()
  })

  it('chọn Toàn cục: ô hiện giá trị Toàn cục bị che, lưu ghi tầng global, nút "Trả về kế thừa" ẩn', async () => {
    const { wrapper } = await open(true)
    expect(wrapper.findAll('.ai-field-clear-form')).toHaveLength(2)

    await wrapper.findAll('.ai-tier input[type="radio"]')[0].setValue(true)
    await flushPromises()
    expect(inputs(wrapper)[2]).toBe('global-model')
    expect(wrapper.findAll('.ai-field-clear-form')).toHaveLength(0)
    expect(wrapper.text()).toContain('Đang lưu ở tầng Toàn cục')

    const modelForm = wrapper.findAll('.ai-field-form')[2]
    await modelForm.find('input').setValue('g2')
    await modelForm.trigger('submit')
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
    expect(inputs(wrapper)[2]).toBe('work-model')
    wrapper.unmount()
  })
})
