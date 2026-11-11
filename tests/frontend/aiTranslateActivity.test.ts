import { describe, expect, it, vi, beforeEach } from 'vitest'
const invokeMock = vi.fn()
const listenMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: (...args: unknown[]) => listenMock(...args) }))

beforeEach(() => { vi.resetModules(); invokeMock.mockReset(); listenMock.mockReset() })

describe('real translation activity subscription', () => {
  it('subscribes before snapshot and ignores older snapshot/event revisions', async () => {
    let receive: (event: { payload: unknown }) => void = () => undefined
    const unlisten = vi.fn()
    listenMock.mockImplementation(async (_name, callback) => { receive = callback; return unlisten })
    invokeMock.mockImplementation(async () => {
      receive({ payload: { revision: 3, active_count: 1 } })
      return { revision: 2, active_count: 0 }
    })
    const state = await import('../../src/config/aiTranslateActivity')
    expect((await state.subscribeAiTranslateActivity()).unlisten).toBe(unlisten)
    expect(listenMock).toHaveBeenCalledWith('ai-translate-activity-v1', expect.any(Function))
    expect(state.aiTranslateActivity.value).toEqual({ revision: 3, active_count: 1 })
    receive({ payload: { revision: 1, active_count: 0 } })
    expect(state.aiTranslateActivity.value?.active_count).toBe(1)
    receive({ payload: { revision: 4, active_count: 0 } })
    expect(state.aiTranslateActivity.value?.active_count).toBe(0)
  })

  it('rejects malformed counts and reports snapshot failure without throwing', async () => {
    const state = await import('../../src/config/aiTranslateActivity')
    for (const value of [null, {}, { revision: -1, active_count: 0 }, { revision: 1, active_count: 0.5 }, { revision: Number.MAX_SAFE_INTEGER + 1, active_count: 0 }]) {
      expect(state.isAiTranslateActivityWire(value)).toBe(false)
    }
    listenMock.mockResolvedValue(vi.fn())
    invokeMock.mockRejectedValue(new Error('offline'))
    expect((await state.subscribeAiTranslateActivity()).error?.code).toBe('ipc.unknown')
    expect(state.aiTranslateActivity.value).toBeNull()
  })
})
