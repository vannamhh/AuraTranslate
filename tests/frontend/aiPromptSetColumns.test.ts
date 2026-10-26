import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const SETS = Array.from({ length: 12 }, (_, i) => ({
  id: i + 1,
  name: `Bộ ${i + 1}`,
  body: '{{glossary_terms}}',
  tier: 'global' as const,
  shadowed_body: null,
  shadowed_id: null,
}))

vi.mock('../../src/config/promptset', () => ({
  promptSetList: async () => ({ sets: SETS, workTierAvailable: false, variables: [], error: null }),
  promptSetChoiceGet: async () => ({ name: null, error: null }),
  promptSetChoiceSet: async () => null,
  promptSetCreate: vi.fn(),
  promptSetDelete: vi.fn(),
  promptSetExport: vi.fn(),
  promptSetExportMany: vi.fn(),
  promptSetRename: vi.fn(),
  promptSetUpdateBody: vi.fn(),
}))

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('panel Đề xuất AI — ô Bộ prompt', () => {
  it('trên 10 bộ: danh sách xếp hai cột và đủ 12 bộ cộng "Chưa chọn" đều có trong đó', async () => {
    vi.resetModules()
    const Panel = (await import('../../src/panels/AiTranslationPanel.vue')).default
    const wrapper = mount(Panel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    const field = wrapper.find('.ai-prompt-select-label')
    expect(field.exists()).toBe(true)
    const list = field.find('.select-options--cols-2')
    expect(list.exists()).toBe(true)
    expect(list.findAll('input[data-select-option]')).toHaveLength(13)
    expect(field.find('.menu--columns').exists()).toBe(true)
    wrapper.unmount()
  })
})
