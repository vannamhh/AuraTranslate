/**
 * Adapter `runAiTranslateResendCall`: lệnh `ai_translate_resend` chỉ nhận `channel` (webview không
 * đưa prompt hay id câu lên dây), hình dạng sai là lỗi, và token chảy qua Channel.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()
const channels: { onmessage: (value: never) => void }[] = []

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  Channel: class {
    onmessage: (value: never) => void = () => undefined
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
      channels[0].onmessage('Xin ' as never)
      channels[0].onmessage('chào' as never)
      return { state: 'done', usage: null, maybe_truncated: false }
    })
    const tokens: string[] = []

    const result = await adapter.runAiTranslateResendCall((t) => tokens.push(t))

    expect(invokeMock).toHaveBeenCalledTimes(1)
    const [command, payload] = invokeMock.mock.calls[0] as [string, Record<string, unknown>]
    expect(command).toBe('ai_translate_resend')
    expect(Object.keys(payload)).toEqual(['channel'])
    expect(tokens).toEqual(['Xin ', 'chào'])
    expect(result).toEqual({ value: { state: 'done', usage: null, maybe_truncated: false }, error: null })
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

describe('resendAiCall', () => {
  it('calls `ai_translate_resend_call` with camelCase operationId and callIndex plus the channel', async () => {
    const adapter = await freshAdapter()
    invokeMock.mockImplementation(async () => {
      channels[0].onmessage({ kind: 'planned', calls: [{ call_index: 3, segment_ids: [7] }] } as never)
      channels[0].onmessage({ kind: 'call_started', call_index: 3 } as never)
      return { state: 'done', usage: null, maybe_truncated: false }
    })
    const kinds: string[] = []

    const result = await adapter.resendAiCall(12, 3, (e) => kinds.push(e.kind))

    const [command, payload] = invokeMock.mock.calls[0] as [string, Record<string, unknown>]
    expect(command).toBe('ai_translate_resend_call')
    expect(Object.keys(payload).sort()).toEqual(['callIndex', 'channel', 'operationId'])
    expect(payload.operationId).toBe(12)
    expect(payload.callIndex).toBe(3)
    expect(kinds).toEqual(['planned', 'call_started'])
    expect(result.error).toBeNull()
  })

  it('skips a frame of the old shape instead of delivering it', async () => {
    const adapter = await freshAdapter()
    invokeMock.mockImplementation(async () => {
      channels[0].onmessage({ kind: 'done', segment_id: 1, usage: null } as never)
      channels[0].onmessage({ kind: 'token', segment_id: 1, text: 'x' } as never)
      return { state: 'done', usage: null, maybe_truncated: false }
    })
    const seen: unknown[] = []

    await adapter.resendAiCall(1, 1, (e) => seen.push(e))

    expect(seen).toEqual([])
  })

  it('an outcome without maybe_truncated is a wrong shape', async () => {
    const adapter = await freshAdapter()
    invokeMock.mockResolvedValue({ state: 'done', usage: null })

    const result = await adapter.resendAiCall(1, 1, () => undefined)

    expect(result.value).toBeNull()
    expect(result.error).toEqual(adapter.UNKNOWN_IPC_ERROR)
  })
})
