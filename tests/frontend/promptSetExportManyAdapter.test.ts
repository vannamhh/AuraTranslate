import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

import { promptSetExportMany } from '../../src/config/promptset'

const file = { tier: 'global', id: 1, file_name: 'a.prompt.md', path: '/d/a.prompt.md', error: null }

beforeEach(() => {
  mockInvoke.mockReset()
})

describe('promptSetExportMany — biên IPC, không bao giờ ném', () => {
  it('gọi `prompt_set_export_many` với `{ sets }` và trả danh sách tệp', async () => {
    mockInvoke.mockResolvedValue([file])
    const result = await promptSetExportMany([{ tier: 'global', id: 1 }])
    expect(mockInvoke).toHaveBeenCalledWith('prompt_set_export_many', { sets: [{ tier: 'global', id: 1 }] })
    expect(result).toEqual({ outcome: 'done', files: [file] })
  })

  it('null (huỷ hộp thoại) ⇒ cancelled, im lặng', async () => {
    mockInvoke.mockResolvedValue(null)
    expect(await promptSetExportMany([{ tier: 'global', id: 1 }])).toEqual({ outcome: 'cancelled' })
  })

  it('hình dạng sai (một dòng mang cả path lẫn error, hoặc không phải mảng) ⇒ error, không lọt vào UI', async () => {
    const err = { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
    mockInvoke.mockResolvedValue([{ ...file, error: err }])
    expect((await promptSetExportMany([])).outcome).toBe('error')
    mockInvoke.mockResolvedValue('nope')
    expect((await promptSetExportMany([])).outcome).toBe('error')
  })

  it('IpcError ném ra ⇒ error mang đúng IpcError', async () => {
    const err = { code: 'promptset.x', message_key: 'err.unknown', params: {}, retryable: false }
    mockInvoke.mockRejectedValue(err)
    expect(await promptSetExportMany([])).toEqual({ outcome: 'error', error: err })
  })
})
