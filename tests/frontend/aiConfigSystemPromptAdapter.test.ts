import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

async function freshAdapter() {
  vi.resetModules()
  mockInvoke.mockReset()
  return import('../../src/config/aiconfig')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('config/aiconfig.ts — system_prompt trên dây', () => {
  it('aiConfigGet mang văn bản mặc định của app và dòng system_prompt nguyên văn', async () => {
    const adapter = await freshAdapter()
    const text = 'Dòng một.\n\n  Dòng ba.  '
    mockInvoke.mockResolvedValueOnce({
      work_tier_available: false,
      fields: [{ field: 'system_prompt', value: text, tier: 'global', shadowed: null }],
      default_system_prompt: text,
      key_configured: false,
      ready: false,
      not_ready: [],
    })
    const got = await adapter.aiConfigGet()
    expect(got.error).toBeNull()
    expect(got.defaultSystemPrompt).toBe(text)
    expect(got.fields?.[0]?.value).toBe(text)
  })

  it('aiConfigClearOverride không có tầng gửi {field}; có tầng gửi {field, tier}', async () => {
    const adapter = await freshAdapter()
    mockInvoke.mockResolvedValue(undefined)
    expect(await adapter.aiConfigClearOverride('model')).toBeNull()
    expect(mockInvoke).toHaveBeenLastCalledWith('ai_config_clear_override', { field: 'model' })
    expect(await adapter.aiConfigClearOverride('system_prompt', 'global')).toBeNull()
    expect(mockInvoke).toHaveBeenLastCalledWith('ai_config_clear_override', { field: 'system_prompt', tier: 'global' })
    expect(await adapter.aiConfigClearOverride('system_prompt', 'work')).toBeNull()
    expect(mockInvoke).toHaveBeenLastCalledWith('ai_config_clear_override', { field: 'system_prompt', tier: 'work' })
  })
})
