/**
 * State của mục "AI và mô hình" trong lớp phủ Cài đặt — Story 4.2, FR68.
 *
 * ⚠️ Cùng khuôn `glossarySettings.test.ts`: `config/aiconfig.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `workTierAvailable` ĐẾN TỪ MOCK CỦA `aiConfigGet()`, KHÔNG một tham số của
 * `loadAiConfigSection` — đối chứng khuyết tật `currentMode !== 'library'`
 * ─────────────────────────────────────────────────────────────────────────────
 * Trước bản vá này, `loadAiConfigSection(isWorkOpen: boolean)` nhận tầng từ CHỖ GỌI
 * (`settingsState.ts` tính bằng `currentMode.value !== 'library'` — một proxy chế độ UI
 * KHÔNG tương đương `OpenWorkState` phía Rust). `loadAiConfigSection` giờ KHÔNG còn tham số:
 * mọi ca dưới đây điều khiển tầng ghi bằng CÁCH DUY NHẤT còn lại — trường
 * `workTierAvailable` của giá trị `aiConfigGetMock` trả về, đúng như `config/aiconfig.ts`
 * đọc nó từ `AiConfigGetWire.work_tier_available` (Rust).
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
  aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: false, error: null })
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

describe('loadAiConfigSection — không tham số, workIsOpen lấy từ response', () => {
  it('nạp năm trường vào draft, xoá lỗi cũ', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: [
        { field: 'provider', value: 'anthropic', tier: 'global', shadowed: null },
        { field: 'endpoint', value: 'https://api.anthropic.com', tier: 'global', shadowed: null },
      ],
      workTierAvailable: false,
      error: null,
    })
    const { loadAiConfigSection, aiConfigDraft, aiConfigLoadError } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigDraft('provider')).toBe('anthropic')
    expect(aiConfigDraft('endpoint')).toBe('https://api.anthropic.com')
    expect(aiConfigDraft('model')).toBe('')
    expect(aiConfigLoadError.value).toBeNull()
  })

  it('lượt đọc trượt ⇒ lỗi hiện qua aiConfigLoadError', async () => {
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    aiConfigGetMock.mockResolvedValue({ fields: null, workTierAvailable: false, error: err })
    const { loadAiConfigSection, aiConfigLoadError } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigLoadError.value).toEqual(err)
  })

  it('workTierAvailable: true trong response ⇒ aiConfigWorkIsOpen đọc true', async () => {
    aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: true, error: null })
    const { loadAiConfigSection, aiConfigWorkIsOpen } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigWorkIsOpen.value).toBe(true)
  })

  it('workTierAvailable: false trong response ⇒ aiConfigWorkIsOpen đọc false', async () => {
    aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: false, error: null })
    const { loadAiConfigSection, aiConfigWorkIsOpen } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigWorkIsOpen.value).toBe(false)
  })

  it('lượt đọc trượt ⇒ aiConfigWorkIsOpen KHÔNG bị ghi đè bằng một cờ rác của lượt trượt', async () => {
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    // Lượt ĐẦU thành công với work_tier_available: true; lượt SAU trượt — workIsOpen phải giữ
    // nguyên giá trị của lượt thành công gần nhất, không bị một lượt trượt kéo về false.
    aiConfigGetMock.mockResolvedValueOnce({ fields: emptyFields(), workTierAvailable: true, error: null })
    aiConfigGetMock.mockResolvedValueOnce({ fields: null, workTierAvailable: false, error: err })
    const { loadAiConfigSection, aiConfigWorkIsOpen } = await freshState()

    await loadAiConfigSection()
    expect(aiConfigWorkIsOpen.value).toBe(true)

    await loadAiConfigSection()
    expect(aiConfigWorkIsOpen.value).toBe(true)
  })
})

describe('saveAiConfigField — re-validate trước khi gọi IPC, tầng ghi theo work_tier_available của response', () => {
  it('giá trị không hợp lệ ⇒ 0 lượt aiConfigSaveField', async () => {
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection()
    setAiConfigDraft('temperature', '3')

    await saveAiConfigField('temperature')

    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
  })

  it('response mang work_tier_available: false ⇒ lưu tầng global', async () => {
    aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: false, error: null })
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection()
    setAiConfigDraft('provider', 'anthropic')

    await saveAiConfigField('provider')

    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('global', 'provider', 'anthropic')
  })

  it('response mang work_tier_available: true ⇒ lưu tầng work — KHÔNG một tín hiệu chế độ UI nào khác tham gia', async () => {
    aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: true, error: null })
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection()
    setAiConfigDraft('endpoint', 'https://local.example.com')

    await saveAiConfigField('endpoint')

    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('work', 'endpoint', 'https://local.example.com')
  })

  it('lượt lưu TRƯỢT ⇒ IpcError hiện qua aiConfigSaveErrorFor(field)', async () => {
    const err = { code: 'ai_config.invalid_value', message_key: 'err.ai_config.invalid_value', params: { field: 'temperature' }, retryable: false }
    aiConfigSaveFieldMock.mockResolvedValue(err)
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField, aiConfigSaveErrorFor } = await freshState()
    await loadAiConfigSection()
    setAiConfigDraft('temperature', '0.7')

    await saveAiConfigField('temperature')

    expect(aiConfigSaveErrorFor('temperature')).toEqual(err)
  })

  it('lượt lưu THÀNH CÔNG ⇒ đọc lại năm trường (aiConfigGet gọi thêm một lần)', async () => {
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection()
    aiConfigGetMock.mockClear()
    setAiConfigDraft('model', 'claude')

    await saveAiConfigField('model')

    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })
})

describe('clearAiConfigOverride', () => {
  it('gọi aiConfigClearOverride đúng một lần rồi đọc lại', async () => {
    aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: true, error: null })
    const { loadAiConfigSection, clearAiConfigOverride } = await freshState()
    await loadAiConfigSection()
    aiConfigGetMock.mockClear()

    await clearAiConfigOverride('endpoint')

    expect(aiConfigClearOverrideMock).toHaveBeenCalledTimes(1)
    expect(aiConfigClearOverrideMock).toHaveBeenCalledWith('endpoint')
    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })

  it('lượt xoá TRƯỢT ⇒ IpcError hiện qua aiConfigSaveErrorFor(field)', async () => {
    const err = { code: 'ai_config.work_tier_unavailable', message_key: 'err.ai_config.work_tier_unavailable', params: {}, retryable: false }
    aiConfigClearOverrideMock.mockResolvedValue(err)
    aiConfigGetMock.mockResolvedValue({ fields: emptyFields(), workTierAvailable: true, error: null })
    const { loadAiConfigSection, clearAiConfigOverride, aiConfigSaveErrorFor } = await freshState()
    await loadAiConfigSection()

    await clearAiConfigOverride('endpoint')

    expect(aiConfigSaveErrorFor('endpoint')).toEqual(err)
  })
})

describe('resetAiConfigSection', () => {
  it('vứt draft/lỗi/trạng thái tải về mặc định', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: [{ field: 'provider', value: 'anthropic', tier: 'global', shadowed: null }],
      workTierAvailable: true,
      error: null,
    })
    const {
      loadAiConfigSection,
      resetAiConfigSection,
      aiConfigDraft,
      aiConfigLoadError,
      aiConfigWorkIsOpen,
    } = await freshState()

    await loadAiConfigSection()
    expect(aiConfigDraft('provider')).toBe('anthropic')
    expect(aiConfigWorkIsOpen.value).toBe(true)

    resetAiConfigSection()

    expect(aiConfigDraft('provider')).toBe('')
    expect(aiConfigLoadError.value).toBeNull()
    expect(aiConfigWorkIsOpen.value).toBe(false)
  })
})
