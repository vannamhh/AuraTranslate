import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

import { promptSetExportMany, promptSetList, promptSetCreate, promptSetOpenImportPreview } from '../../src/config/promptset'

const file = { tier: 'global', kind: 'translate', id: 1, file_name: 'a.prompt.md', path: '/d/a.prompt.md', error: null }

beforeEach(() => {
  mockInvoke.mockReset()
})

describe('promptSetExportMany — biên IPC, không bao giờ ném', () => {
  it('gọi `prompt_set_export_many` với `{ sets }` và trả danh sách tệp', async () => {
    mockInvoke.mockResolvedValue([file])
    const result = await promptSetExportMany([{ tier: 'global', kind: 'translate', id: 1 }])
    expect(mockInvoke).toHaveBeenCalledWith('prompt_set_export_many', { sets: [{ tier: 'global', kind: 'translate', id: 1 }] })
    expect(result).toEqual({ outcome: 'done', files: [file] })
  })

  it('null (huỷ hộp thoại) ⇒ cancelled, im lặng', async () => {
    mockInvoke.mockResolvedValue(null)
    expect(await promptSetExportMany([{ tier: 'global', kind: 'translate', id: 1 }])).toEqual({ outcome: 'cancelled' })
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

  it('một tệp trả về thiếu `kind` hoặc mang `kind` lạ ⇒ error', async () => {
    const { kind: _kind, ...withoutKind } = file
    mockInvoke.mockResolvedValue([withoutKind])
    expect((await promptSetExportMany([])).outcome).toBe('error')
    mockInvoke.mockResolvedValue([{ ...file, kind: 'x' }])
    expect((await promptSetExportMany([])).outcome).toBe('error')
  })
})

describe('promptSetOpenImportPreview và các lệnh theo loại — biên IPC', () => {
  const preview = {
    file_name: 'a.prompt.md',
    set_kind: 'translate',
    name: 'A',
    body: 'x',
    warnings: { unknown_markers: [], glossary_terms_missing: false },
    global: { kind: 'new', existing_body: null },
    work: null,
  }

  it('bản xem trước có `set_kind` hợp lệ ⇒ loaded', async () => {
    mockInvoke.mockResolvedValue(preview)
    expect(await promptSetOpenImportPreview()).toEqual({ outcome: 'loaded', preview })
  })

  it('bản xem trước thiếu `set_kind` hoặc mang giá trị lạ ⇒ error', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const { set_kind: _setKind, ...withoutKind } = preview
    mockInvoke.mockResolvedValue(withoutKind)
    expect((await promptSetOpenImportPreview()).outcome).toBe('error')
    mockInvoke.mockResolvedValue({ ...preview, set_kind: 'x' })
    expect((await promptSetOpenImportPreview()).outcome).toBe('error')
  })

  it('promptSetList và promptSetCreate gửi `kind` xuống dây', async () => {
    mockInvoke.mockResolvedValue({ sets: [], work_tier_available: false })
    await promptSetList('translate')
    expect(mockInvoke).toHaveBeenCalledWith('prompt_set_list', { kind: 'translate' })
    mockInvoke.mockResolvedValue({ id: 1, warnings: { unknown_markers: [], glossary_terms_missing: false } })
    await promptSetCreate('global', 'proofread', 'A', 'b')
    expect(mockInvoke).toHaveBeenCalledWith('prompt_set_create', { tier: 'global', kind: 'proofread', name: 'A', body: 'b' })
  })
})
