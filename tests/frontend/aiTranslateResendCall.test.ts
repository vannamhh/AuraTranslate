/**
 * Adapter `runAiTranslateResendCall`: lệnh `ai_translate_resend` chỉ nhận `channel` (webview không
 * đưa prompt hay id câu lên dây), hình dạng sai là lỗi, và token chảy qua Channel.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()
const channels: { onmessage: (text: string) => void }[] = []

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {
    onmessage: (text: string) => void = () => undefined
    constructor() {
      channels.push(this)
    }
  },
}))

async function freshAdapter() {
  vi.resetModules()
  invokeMock.mockReset()
  channels.length = 0
  return import('../../src/config/aitranslate')
}

beforeEach(() => {
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
  vi.spyOn(console, 'info').mockImplementation(() => undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
})

describe('runAiTranslateResendCall', () => {
  it('gọi `ai_translate_resend` với payload chỉ gồm `channel`, token tới theo thứ tự, kết quả cuối trả nguyên', async () => {
    const adapter = await freshAdapter()
    invokeMock.mockImplementation(async () => {
      channels[0].onmessage('Xin ')
      channels[0].onmessage('chào')
      return { state: 'done', usage: null }
    })
    const tokens: string[] = []

    const result = await adapter.runAiTranslateResendCall((t) => tokens.push(t))

    expect(invokeMock).toHaveBeenCalledTimes(1)
    const [command, payload] = invokeMock.mock.calls[0] as [string, Record<string, unknown>]
    expect(command).toBe('ai_translate_resend')
    expect(Object.keys(payload)).toEqual(['channel'])
    expect(tokens).toEqual(['Xin ', 'chào'])
    expect(result).toEqual({ value: { state: 'done', usage: null }, error: null })
  })

  it('hình dạng kết quả sai ⇒ { value: null, error: lỗi không rõ }', async () => {
    const adapter = await freshAdapter()
    invokeMock.mockResolvedValue({ state: 'nonsense' })

    const result = await adapter.runAiTranslateResendCall(() => undefined)

    expect(result.value).toBeNull()
    expect(result.error).toEqual(adapter.UNKNOWN_IPC_ERROR)
  })

  it('invoke ném IpcError (không có bản ghi để gửi lại) ⇒ lỗi đó được trả nguyên', async () => {
    const adapter = await freshAdapter()
    const err = {
      code: 'ai_translate.no_prompt_to_resend',
      message_key: 'err.ai_translate.no_prompt_to_resend',
      params: {},
      retryable: false,
    }
    invokeMock.mockRejectedValue(err)

    const result = await adapter.runAiTranslateResendCall(() => undefined)

    expect(result).toEqual({ value: null, error: err })
  })

  it('ngoài Tauri (không cầu IPC) ⇒ { value: null, error: null }, không lỗi giả', async () => {
    const adapter = await freshAdapter()
    invokeMock.mockRejectedValue(new Error('no bridge'))

    const result = await adapter.runAiTranslateResendCall(() => undefined)

    expect(result).toEqual({ value: null, error: null })
  })
})
