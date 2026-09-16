/**
 * State của mục "AI và mô hình" trong lớp phủ Cài đặt — Story 4.2, FR68.
 *
 * ⚠️ Cùng khuôn `glossarySettings.test.ts`: `config/aiconfig.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const aiConfigGetMock = vi.fn()
const aiConfigSaveFieldMock = vi.fn()
const aiConfigClearOverrideMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigClearOverride: (...args: unknown[]) => aiConfigClearOverrideMock(...args),
}))

/** Nạp lại module mỗi ca — state là module-level singleton, cùng lý do `freshState` của
 * `glossarySettings.test.ts`. */
async function freshState() {
  vi.resetModules()
  return import('../../src/aiConfigState')
}

function emptyFields() {
  return []
}

beforeEach(() => {
  aiConfigGetMock.mockReset()
  aiConfigSaveFieldMock.mockReset()
  aiConfigClearOverrideMock.mockReset()
  aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), error: null })
  aiConfigSaveFieldMock.mockResolvedValue(null)
  aiConfigClearOverrideMock.mockResolvedValue(null)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('isAiConfigValueValid — hàm THUẦN, đọc lại đúng ràng buộc phía Rust', () => {
  it('provider/model: rỗng hoặc chỉ khoảng trắng ⇒ false', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('provider', '')).toBe(false)
    expect(isAiConfigValueValid('provider', '   ')).toBe(false)
    expect(isAiConfigValueValid('model', '')).toBe(false)
  })

  it('provider/model: chuỗi có nội dung ⇒ true', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('provider', 'anthropic')).toBe(true)
    expect(isAiConfigValueValid('model', 'claude')).toBe(true)
  })

  it('temperature: biên đóng [0, 2] đều hợp lệ', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('temperature', '0')).toBe(true)
    expect(isAiConfigValueValid('temperature', '2')).toBe(true)
    expect(isAiConfigValueValid('temperature', '0.7')).toBe(true)
  })

  it('temperature: "-1", "3", "abc" đều bị từ chối (I/O Matrix spec 4.2)', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('temperature', '-1')).toBe(false)
    expect(isAiConfigValueValid('temperature', '3')).toBe(false)
    expect(isAiConfigValueValid('temperature', 'abc')).toBe(false)
  })

  it('temperature: cú pháp `f64::from_str` phía Rust chấp nhận (".5"/"5."/"1e-1"/"+0.5") cũng phải qua được ở đây', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('temperature', '.5')).toBe(true)
    expect(isAiConfigValueValid('temperature', '5.')).toBe(false) // hợp cú pháp, nhưng 5 > 2 — khoảng chặn, không cú pháp
    expect(isAiConfigValueValid('temperature', '.5e0')).toBe(true)
    expect(isAiConfigValueValid('temperature', '1e-1')).toBe(true)
    expect(isAiConfigValueValid('temperature', '+0.5')).toBe(true)
  })

  it('temperature: "Infinity"/"NaN"/hex đều bị từ chối NGAY Ở CÚ PHÁP, không nhờ khoảng [0, 2]', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('temperature', 'Infinity')).toBe(false)
    expect(isAiConfigValueValid('temperature', 'NaN')).toBe(false)
    expect(isAiConfigValueValid('temperature', '0x1F')).toBe(false)
    expect(isAiConfigValueValid('temperature', '')).toBe(false)
  })

  it('max_tokens: "0", "-5", "1.5", "abc" đều bị từ chối (I/O Matrix spec 4.2)', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('max_tokens', '0')).toBe(false)
    expect(isAiConfigValueValid('max_tokens', '-5')).toBe(false)
    expect(isAiConfigValueValid('max_tokens', '1.5')).toBe(false)
    expect(isAiConfigValueValid('max_tokens', 'abc')).toBe(false)
  })

  it('max_tokens: số nguyên dương ⇒ true', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('max_tokens', '2048')).toBe(true)
  })

  it('endpoint: "localhost:11434" (không lược đồ) và chuỗi rỗng đều bị từ chối', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('endpoint', 'localhost:11434')).toBe(false)
    expect(isAiConfigValueValid('endpoint', '')).toBe(false)
  })

  it('endpoint: URL tuyệt đối http/https mang host ⇒ true', async () => {
    const { isAiConfigValueValid } = await freshState()
    expect(isAiConfigValueValid('endpoint', 'https://api.anthropic.com')).toBe(true)
    expect(isAiConfigValueValid('endpoint', 'http://localhost:11434')).toBe(true)
  })
})

describe('loadAiConfigSection', () => {
  it('nạp năm trường vào draft, xoá lỗi cũ', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: [
        { field: 'provider', value: 'anthropic', tier: 'global', shadowed: null },
        { field: 'endpoint', value: 'https://api.anthropic.com', tier: 'global', shadowed: null },
      ],
      error: null,
    })
    const { loadAiConfigSection, aiConfigDraft, aiConfigLoadError } = await freshState()

    await loadAiConfigSection(false)

    expect(aiConfigDraft('provider')).toBe('anthropic')
    expect(aiConfigDraft('endpoint')).toBe('https://api.anthropic.com')
    expect(aiConfigDraft('model')).toBe('')
    expect(aiConfigLoadError.value).toBeNull()
  })

  it('lượt đọc trượt ⇒ lỗi hiện qua aiConfigLoadError', async () => {
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    aiConfigGetMock.mockResolvedValue({ fields: null, error: err })
    const { loadAiConfigSection, aiConfigLoadError } = await freshState()

    await loadAiConfigSection(false)

    expect(aiConfigLoadError.value).toEqual(err)
  })
})

describe('saveAiConfigField — re-validate trước khi gọi IPC, tầng ghi theo trạng thái Tác phẩm', () => {
  it('giá trị không hợp lệ ⇒ 0 lượt aiConfigSaveField', async () => {
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection(false)
    setAiConfigDraft('temperature', '3')

    await saveAiConfigField('temperature')

    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
  })

  it('không Tác phẩm nào mở ⇒ lưu tầng global', async () => {
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection(false)
    setAiConfigDraft('provider', 'anthropic')

    await saveAiConfigField('provider')

    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('global', 'provider', 'anthropic')
  })

  it('có Tác phẩm đang mở ⇒ lưu tầng work', async () => {
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection(true)
    setAiConfigDraft('endpoint', 'https://local.example.com')

    await saveAiConfigField('endpoint')

    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('work', 'endpoint', 'https://local.example.com')
  })

  it('lượt lưu TRƯỢT ⇒ IpcError hiện qua aiConfigSaveErrorFor(field)', async () => {
    const err = { code: 'ai_config.invalid_value', message_key: 'err.ai_config.invalid_value', params: { field: 'temperature' }, retryable: false }
    aiConfigSaveFieldMock.mockResolvedValue(err)
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField, aiConfigSaveErrorFor } = await freshState()
    await loadAiConfigSection(false)
    setAiConfigDraft('temperature', '0.7')

    await saveAiConfigField('temperature')

    expect(aiConfigSaveErrorFor('temperature')).toEqual(err)
  })

  it('lượt lưu THÀNH CÔNG ⇒ đọc lại năm trường (aiConfigGet gọi thêm một lần)', async () => {
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection(false)
    aiConfigGetMock.mockClear()
    setAiConfigDraft('model', 'claude')

    await saveAiConfigField('model')

    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })
})

describe('clearAiConfigOverride', () => {
  it('gọi aiConfigClearOverride đúng một lần rồi đọc lại', async () => {
    const { loadAiConfigSection, clearAiConfigOverride } = await freshState()
    await loadAiConfigSection(true)
    aiConfigGetMock.mockClear()

    await clearAiConfigOverride('endpoint')

    expect(aiConfigClearOverrideMock).toHaveBeenCalledTimes(1)
    expect(aiConfigClearOverrideMock).toHaveBeenCalledWith('endpoint')
    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })

  it('lượt xoá TRƯỢT ⇒ IpcError hiện qua aiConfigSaveErrorFor(field)', async () => {
    const err = { code: 'ai_config.work_tier_unavailable', message_key: 'err.ai_config.work_tier_unavailable', params: {}, retryable: false }
    aiConfigClearOverrideMock.mockResolvedValue(err)
    const { loadAiConfigSection, clearAiConfigOverride, aiConfigSaveErrorFor } = await freshState()
    await loadAiConfigSection(true)

    await clearAiConfigOverride('endpoint')

    expect(aiConfigSaveErrorFor('endpoint')).toEqual(err)
  })
})

describe('resetAiConfigSection', () => {
  it('vứt draft/lỗi/trạng thái tải về mặc định', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: [{ field: 'provider', value: 'anthropic', tier: 'global', shadowed: null }],
      error: null,
    })
    const {
      loadAiConfigSection,
      resetAiConfigSection,
      aiConfigDraft,
      aiConfigLoadError,
      aiConfigWorkIsOpen,
    } = await freshState()

    await loadAiConfigSection(true)
    expect(aiConfigDraft('provider')).toBe('anthropic')

    resetAiConfigSection()

    expect(aiConfigDraft('provider')).toBe('')
    expect(aiConfigLoadError.value).toBeNull()
    expect(aiConfigWorkIsOpen.value).toBe(false)
  })
})
