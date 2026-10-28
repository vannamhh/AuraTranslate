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
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const aiConfigGetMock = vi.fn()
const aiConfigSaveFieldMock = vi.fn()
const aiConfigSaveFieldsMock = vi.fn()
const aiConfigClearOverrideMock = vi.fn()
const aiConfigSaveKeyMock = vi.fn()
const aiConfigDeleteKeyMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigSaveFields: (...args: unknown[]) => aiConfigSaveFieldsMock(...args),
  aiConfigClearOverride: (...args: unknown[]) => aiConfigClearOverrideMock(...args),
  aiConfigSaveKey: (...args: unknown[]) => aiConfigSaveKeyMock(...args),
  aiConfigDeleteKey: (...args: unknown[]) => aiConfigDeleteKeyMock(...args),
  aiConfigTestConnection: vi.fn(),
  aiConfigTestConnectionCancel: vi.fn(),
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
  aiConfigSaveFieldsMock.mockReset()
  aiConfigClearOverrideMock.mockReset()
  aiConfigSaveKeyMock.mockReset()
  aiConfigDeleteKeyMock.mockReset()
  aiConfigGetMock.mockResolvedValue({
    fields: emptyFields(),
    workTierAvailable: false,
    keyConfigured: false,
    error: null,
  })
  aiConfigSaveFieldMock.mockResolvedValue(null)
  aiConfigSaveFieldsMock.mockResolvedValue(null)
  aiConfigClearOverrideMock.mockResolvedValue(null)
  aiConfigSaveKeyMock.mockResolvedValue(null)
  aiConfigDeleteKeyMock.mockResolvedValue(null)
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

  it('temperature và max_tokens: đúng `default` (sau trim) hợp lệ, `Default` và `defaults` thì không', async () => {
    const { isAiConfigValueValid } = await freshState()
    for (const field of ['temperature', 'max_tokens'] as const) {
      expect(isAiConfigValueValid(field, 'default')).toBe(true)
      expect(isAiConfigValueValid(field, '  default ')).toBe(true)
      expect(isAiConfigValueValid(field, 'Default')).toBe(false)
      expect(isAiConfigValueValid(field, 'defaults')).toBe(false)
    }
    expect(isAiConfigValueValid('model', 'default')).toBe(true)
    expect(isAiConfigValueValid('endpoint', 'default')).toBe(false)
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
      keyConfigured: false,
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
    aiConfigGetMock.mockResolvedValue({ fields: null, workTierAvailable: false, keyConfigured: null, error: err })
    const { loadAiConfigSection, aiConfigLoadError } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigLoadError.value).toEqual(err)
  })

  it('workTierAvailable: true trong response ⇒ aiConfigWorkIsOpen đọc true', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
    const { loadAiConfigSection, aiConfigWorkIsOpen } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigWorkIsOpen.value).toBe(true)
  })

  it('workTierAvailable: false trong response ⇒ aiConfigWorkIsOpen đọc false', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    const { loadAiConfigSection, aiConfigWorkIsOpen } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigWorkIsOpen.value).toBe(false)
  })

  it('lượt đọc trượt ⇒ aiConfigWorkIsOpen KHÔNG bị ghi đè bằng một cờ rác của lượt trượt', async () => {
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    // Lượt ĐẦU thành công với work_tier_available: true; lượt SAU trượt — workIsOpen phải giữ
    // nguyên giá trị của lượt thành công gần nhất, không bị một lượt trượt kéo về false.
    aiConfigGetMock.mockResolvedValueOnce({
      fields: emptyFields(),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
    aiConfigGetMock.mockResolvedValueOnce({ fields: null, workTierAvailable: false, keyConfigured: null, error: err })
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
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    const { loadAiConfigSection, setAiConfigDraft, saveAiConfigField } = await freshState()
    await loadAiConfigSection()
    setAiConfigDraft('provider', 'anthropic')

    await saveAiConfigField('provider')

    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveFieldMock).toHaveBeenCalledWith('global', 'provider', 'anthropic')
  })

  it('response mang work_tier_available: true ⇒ lưu tầng work — KHÔNG một tín hiệu chế độ UI nào khác tham gia', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
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
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
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
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: true,
      keyConfigured: false,
      error: null,
    })
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
      keyConfigured: true,
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

// ═════════════════════════════════════════════════════════════════════════════════
// Khoá API (Story 4.3, FR65/FR67, NFR11) — trạng thái BA giá trị, không hai
// ═════════════════════════════════════════════════════════════════════════════════

describe('isAiConfigKeyValueValid — hàm THUẦN, cùng luật core::aiconfig::validate_key', () => {
  it('rỗng hoặc chỉ khoảng trắng ⇒ false', async () => {
    const { isAiConfigKeyValueValid } = await freshState()
    expect(isAiConfigKeyValueValid('')).toBe(false)
    expect(isAiConfigKeyValueValid('   ')).toBe(false)
    expect(isAiConfigKeyValueValid('\t\n')).toBe(false)
  })

  it('chuỗi có nội dung ⇒ true', async () => {
    const { isAiConfigKeyValueValid } = await freshState()
    expect(isAiConfigKeyValueValid('sk-anything')).toBe(true)
    expect(isAiConfigKeyValueValid('  sk-with-surrounding-space  ')).toBe(true)
  })
})

describe('aiConfigKeyConfigured — BA giá trị đọc từ loadAiConfigSection, không hai', () => {
  it.each([true, false, null] as const)('key_configured: %s trong response ⇒ đọc lại đúng %s', async (value) => {
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: value,
      error: null,
    })
    const { loadAiConfigSection, aiConfigKeyConfigured } = await freshState()

    await loadAiConfigSection()

    expect(aiConfigKeyConfigured.value).toBe(value)
  })

  it('lượt đọc trượt ⇒ aiConfigKeyConfigured KHÔNG bị ghi đè bằng một cờ rác của lượt trượt', async () => {
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    aiConfigGetMock.mockResolvedValueOnce({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: true,
      error: null,
    })
    aiConfigGetMock.mockResolvedValueOnce({ fields: null, workTierAvailable: false, keyConfigured: null, error: err })
    const { loadAiConfigSection, aiConfigKeyConfigured } = await freshState()

    await loadAiConfigSection()
    expect(aiConfigKeyConfigured.value).toBe(true)

    await loadAiConfigSection()
    expect(aiConfigKeyConfigured.value).toBe(true)
  })
})

describe('Story 4.3 — khoá API KHÔNG BAO GIỜ lộ ra qua bất kỳ hình dạng phản hồi nào state module này đọc', () => {
  it('phản hồi mang thêm trường HÌNH DẠNG một khoá thật (hồi quy giả định phía Rust) ⇒ không accessor nào của module lộ nó ra ngoài', async () => {
    const decoy = 'sk-should-never-leak-anywhere-in-this-module'
    aiConfigGetMock.mockResolvedValue({
      fields: [{ field: 'provider', value: 'anthropic', tier: 'global', shadowed: null }],
      workTierAvailable: false,
      keyConfigured: true,
      error: null,
      // Ba tên trường một khoá thật CÓ THỂ mang nếu Rust hồi quy và trả giá trị qua IPC —
      // module không đọc bất kỳ tên nào trong số này, thứ DUY NHẤT nó đọc là
      // `key_configured` (đã canh ở trên).
      key: decoy,
      value: decoy,
      api_key: decoy,
    })
    const { loadAiConfigSection, aiConfigFieldWire, aiConfigKeyConfigured, aiConfigDraft, AI_CONFIG_FIELDS } =
      await freshState()

    await loadAiConfigSection()

    expect(aiConfigKeyConfigured.value).toBe(true)
    expect(JSON.stringify(aiConfigFieldWire('provider'))).not.toContain(decoy)
    for (const field of AI_CONFIG_FIELDS) expect(aiConfigDraft(field)).not.toContain(decoy)
  })
})

describe('saveAiConfigKey — re-validate trước khi gọi IPC, LUÔN Global, vứt draft sau lượt thành công', () => {
  it('giá trị rỗng/toàn khoảng trắng ⇒ 0 lượt aiConfigSaveKey (I/O Matrix "Save an empty or whitespace-only key")', async () => {
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey } = await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('   ')

    await saveAiConfigKey()

    expect(aiConfigSaveKeyMock).not.toHaveBeenCalled()
  })

  it('giá trị hợp lệ ⇒ gọi aiConfigSaveKey đúng MỘT lần với giá trị ĐÃ TRIM', async () => {
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey } = await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('  sk-my-real-key  ')

    await saveAiConfigKey()

    expect(aiConfigSaveKeyMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveKeyMock).toHaveBeenCalledWith('sk-my-real-key')
  })

  it('lượt Lưu THÀNH CÔNG ⇒ vứt draft NGAY và đọc lại (aiConfigGet gọi thêm một lần)', async () => {
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey, aiConfigKeyDraft } = await freshState()
    await loadAiConfigSection()
    aiConfigGetMock.mockClear()
    setAiConfigKeyDraft('sk-my-real-key')

    await saveAiConfigKey()

    expect(aiConfigKeyDraft()).toBe('')
    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })

  it('lượt Lưu TRƯỢT ⇒ IpcError hiện qua aiConfigKeyError, draft GIỮ NGUYÊN (người dùng còn sửa được)', async () => {
    const err = {
      code: 'ai_config.keychain_unavailable',
      message_key: 'err.ai_config.keychain_unavailable',
      params: {},
      retryable: true,
    }
    aiConfigSaveKeyMock.mockResolvedValue(err)
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey, aiConfigKeyError, aiConfigKeyDraft } =
      await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('sk-my-real-key')

    await saveAiConfigKey()

    expect(aiConfigKeyError.value).toEqual(err)
    expect(aiConfigKeyDraft()).toBe('sk-my-real-key')
  })

  it('đang bận (keyBusy) ⇒ một lượt gọi thứ hai không làm gì (khoá tái nhập)', async () => {
    let resolveSave: (value: null) => void = () => {}
    aiConfigSaveKeyMock.mockReturnValue(
      new Promise((resolve) => {
        resolveSave = resolve
      }),
    )
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey } = await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('sk-my-real-key')

    const first = saveAiConfigKey()
    await saveAiConfigKey() // lượt thứ hai trong lúc lượt đầu còn treo — phải là no-op

    expect(aiConfigSaveKeyMock).toHaveBeenCalledTimes(1)
    resolveSave(null)
    await first
  })
})

describe('deleteAiConfigKey — LUÔN Global, đọc lại sau mỗi lượt', () => {
  it('gọi aiConfigDeleteKey đúng một lần rồi đọc lại (aiConfigGet gọi thêm một lần)', async () => {
    const { loadAiConfigSection, deleteAiConfigKey } = await freshState()
    await loadAiConfigSection()
    aiConfigGetMock.mockClear()

    await deleteAiConfigKey()

    expect(aiConfigDeleteKeyMock).toHaveBeenCalledTimes(1)
    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })

  it('lượt Xoá TRƯỢT ⇒ IpcError hiện qua aiConfigKeyError', async () => {
    const err = {
      code: 'ai_config.keychain_unavailable',
      message_key: 'err.ai_config.keychain_unavailable',
      params: {},
      retryable: true,
    }
    aiConfigDeleteKeyMock.mockResolvedValue(err)
    const { loadAiConfigSection, deleteAiConfigKey, aiConfigKeyError } = await freshState()
    await loadAiConfigSection()

    await deleteAiConfigKey()

    expect(aiConfigKeyError.value).toEqual(err)
  })

  it('I/O Matrix "Delete when none exists" — thành công dù trước đó chưa cấu hình, KHÔNG bị chặn bởi state phía trước', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    const { loadAiConfigSection, deleteAiConfigKey, aiConfigKeyError } = await freshState()
    await loadAiConfigSection()

    await deleteAiConfigKey()

    expect(aiConfigDeleteKeyMock).toHaveBeenCalledTimes(1)
    expect(aiConfigKeyError.value).toBeNull()
  })
})

describe('keyBusy — MỘT cờ dùng chung cho Lưu và Xoá, canh CẢ HAI chiều', () => {
  it('lượt Lưu còn treo ⇒ một lượt Xoá gọi giữa chừng là no-op (không đụng aiConfigDeleteKey)', async () => {
    let resolveSave: (value: null) => void = () => {}
    aiConfigSaveKeyMock.mockReturnValue(
      new Promise((resolve) => {
        resolveSave = resolve
      }),
    )
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey, deleteAiConfigKey } = await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('sk-my-real-key')

    const pendingSave = saveAiConfigKey()
    await deleteAiConfigKey() // gọi giữa lúc Lưu còn treo — phải là no-op

    expect(aiConfigDeleteKeyMock).not.toHaveBeenCalled()
    resolveSave(null)
    await pendingSave
  })

  it('lượt Xoá còn treo ⇒ một lượt Lưu gọi giữa chừng là no-op (không đụng aiConfigSaveKey)', async () => {
    let resolveDelete: (value: null) => void = () => {}
    aiConfigDeleteKeyMock.mockReturnValue(
      new Promise((resolve) => {
        resolveDelete = resolve
      }),
    )
    const { loadAiConfigSection, setAiConfigKeyDraft, saveAiConfigKey, deleteAiConfigKey } = await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('sk-my-real-key')

    const pendingDelete = deleteAiConfigKey()
    await saveAiConfigKey() // gọi giữa lúc Xoá còn treo — phải là no-op

    expect(aiConfigSaveKeyMock).not.toHaveBeenCalled()
    resolveDelete(null)
    await pendingDelete
  })
})

describe('resetAiConfigSection — vứt cả state khoá API', () => {
  it('vứt keyConfigured/keyDraft/keyError về mặc định', async () => {
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: true,
      error: null,
    })
    const err = {
      code: 'ai_config.keychain_unavailable',
      message_key: 'err.ai_config.keychain_unavailable',
      params: {},
      retryable: true,
    }
    aiConfigDeleteKeyMock.mockResolvedValue(err)
    const {
      loadAiConfigSection,
      resetAiConfigSection,
      setAiConfigKeyDraft,
      deleteAiConfigKey,
      aiConfigKeyConfigured,
      aiConfigKeyDraft,
      aiConfigKeyError,
    } = await freshState()
    await loadAiConfigSection()
    setAiConfigKeyDraft('sk-something-typed')
    await deleteAiConfigKey() // để lại một IpcError trong aiConfigKeyError trước khi reset
    expect(aiConfigKeyConfigured.value).toBe(true)
    expect(aiConfigKeyError.value).toEqual(err)

    resetAiConfigSection()

    expect(aiConfigKeyConfigured.value).toBeNull()
    expect(aiConfigKeyDraft()).toBe('')
    expect(aiConfigKeyError.value).toBeNull()
  })
})

describe('bảng luật dùng chung với Rust — mỗi hàng cùng một phán quyết', () => {
  const table = JSON.parse(
    readFileSync(resolve(process.cwd(), 'tests/frontend/support/ai-config-validity.json'), 'utf8'),
  ) as { rows: { field: 'provider' | 'endpoint' | 'model' | 'temperature' | 'max_tokens'; input: string; valid: boolean }[] }

  it('bảng không rỗng và phủ đủ năm trường', () => {
    expect(table.rows.length).toBeGreaterThan(0)
    expect(new Set(table.rows.map((r) => r.field)).size).toBe(5)
  })

  it('isAiConfigValueValid khớp từng hàng, kể cả max_tokens = 4294967296', async () => {
    const { isAiConfigValueValid } = await freshState()
    const mismatches = table.rows.filter((r) => isAiConfigValueValid(r.field, r.input) !== r.valid)
    expect(mismatches).toEqual([])
    expect(isAiConfigValueValid('max_tokens', '4294967295')).toBe(true)
    expect(isAiConfigValueValid('max_tokens', '4294967296')).toBe(false)
  })
})

describe('bộ chọn tầng Toàn cục/Tác phẩm — tầng xem và tầng ghi', () => {
  const workOverridesModel = {
    fields: [
      { field: 'provider', value: 'openai', tier: 'global', shadowed: null },
      { field: 'endpoint', value: '', tier: 'global', shadowed: null },
      { field: 'model', value: 'work-model', tier: 'work', shadowed: 'global-model' },
      { field: 'temperature', value: '0.5', tier: 'work', shadowed: null },
      { field: 'max_tokens', value: '100', tier: 'global', shadowed: null },
    ],
    workTierAvailable: true,
    keyConfigured: false,
    error: null,
  }

  it('có Tác phẩm mở ⇒ mặc định Tác phẩm; không có ⇒ Toàn cục và Tác phẩm không chọn được', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const open = await freshState()
    await open.loadAiConfigSection()
    expect(open.aiConfigViewTier.value).toBe('work')

    aiConfigGetMock.mockResolvedValue({ ...workOverridesModel, workTierAvailable: false })
    const closed = await freshState()
    await closed.loadAiConfigSection()
    closed.selectAiConfigViewTier('work')
    expect(closed.aiConfigViewTier.value).toBe('global')
  })

  it('xem Toàn cục: trường bị Work ghi đè hiện `shadowed`, không hiện giá trị Work; trường khác hiện `value`', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const s = await freshState()
    await s.loadAiConfigSection()
    expect(s.aiConfigDraft('model')).toBe('work-model')

    s.selectAiConfigViewTier('global')
    expect(s.aiConfigDraft('model')).toBe('global-model')
    expect(s.aiConfigDraft('temperature')).toBe('')
    expect(s.aiConfigDraft('provider')).toBe('openai')
    expect(s.aiConfigDraft('max_tokens')).toBe('100')

    s.selectAiConfigViewTier('work')
    expect(s.aiConfigDraft('model')).toBe('work-model')
    expect(s.aiConfigDraft('temperature')).toBe('0.5')
  })

  it('Toàn cục được chọn khi Work mở ⇒ lưu tầng global; đổi lại Tác phẩm ⇒ lưu tầng work', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const s = await freshState()
    await s.loadAiConfigSection()
    s.selectAiConfigViewTier('global')
    s.setAiConfigDraft('model', 'new-global')
    await s.saveAiConfigField('model')
    expect(aiConfigSaveFieldMock).toHaveBeenLastCalledWith('global', 'model', 'new-global')
    expect(s.aiConfigViewTier.value).toBe('global')

    s.selectAiConfigViewTier('work')
    s.setAiConfigDraft('model', 'new-work')
    await s.saveAiConfigField('model')
    expect(aiConfigSaveFieldMock).toHaveBeenLastCalledWith('work', 'model', 'new-work')
  })

  it('lượt lưu re-check workIsOpen tại thời điểm gọi: Work biến mất giữa chừng ⇒ ghi global, không ghi work', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const s = await freshState()
    await s.loadAiConfigSection()
    s.setAiConfigDraft('provider', 'x')

    aiConfigGetMock.mockResolvedValue({ ...workOverridesModel, workTierAvailable: false })
    await s.loadAiConfigSection()
    s.setAiConfigDraft('provider', 'x')
    await s.saveAiConfigField('provider')

    expect(aiConfigSaveFieldMock).toHaveBeenLastCalledWith('global', 'provider', 'x')
  })

  it('lưu Toàn cục không gọi aiConfigClearOverride và không đụng bản ghi đè của Work', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const s = await freshState()
    await s.loadAiConfigSection()
    s.selectAiConfigViewTier('global')
    s.setAiConfigDraft('model', 'g2')
    await s.saveAiConfigField('model')
    expect(aiConfigClearOverrideMock).not.toHaveBeenCalled()
    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(1)
  })

  it('resetAiConfigViewTier trả lựa chọn về mặc định Tác phẩm khi Work còn mở', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const s = await freshState()
    await s.loadAiConfigSection()
    s.selectAiConfigViewTier('global')
    s.resetAiConfigViewTier()
    expect(s.aiConfigViewTier.value).toBe('work')
    expect(s.aiConfigDraft('model')).toBe('work-model')
  })

  it('lượt nạp lại sau khi lưu KHÔNG đặt lại tầng đã chọn', async () => {
    aiConfigGetMock.mockResolvedValue(workOverridesModel)
    const s = await freshState()
    await s.loadAiConfigSection()
    s.selectAiConfigViewTier('global')
    await s.loadAiConfigSection()
    expect(s.aiConfigViewTier.value).toBe('global')
  })
})

describe('lưu khi rời ô, chuỗi nhà cung cấp, trả toàn bộ về kế thừa', () => {
  const fieldRows = (overrides: Partial<Record<string, { value: string; tier: 'global' | 'work' }>> = {}) =>
    ['provider', 'endpoint', 'model', 'temperature', 'max_tokens'].map((field) => ({
      field,
      value: overrides[field]?.value ?? '',
      tier: overrides[field]?.tier ?? 'global',
      shadowed: null,
    }))

  async function loaded(workTierAvailable: boolean, rows = fieldRows()) {
    aiConfigGetMock.mockResolvedValue({ fields: rows, workTierAvailable, keyConfigured: false, error: null })
    const state = await freshState()
    await state.loadAiConfigSection()
    return state
  }

  it('commitAiConfigField bỏ qua giá trị rỗng, không hợp lệ hoặc không đổi', async () => {
    const state = await loaded(false, fieldRows({ model: { value: 'm', tier: 'global' } }))
    state.setAiConfigDraft('model', '   ')
    await state.commitAiConfigField('model')
    state.setAiConfigDraft('model', 'm')
    await state.commitAiConfigField('model')
    state.setAiConfigDraft('temperature', '9')
    await state.commitAiConfigField('temperature')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()

    state.setAiConfigDraft('model', 'n')
    await state.commitAiConfigField('model')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'model', 'n')
  })

  const fieldsOf = (provider: string, endpoint = '', model = '', temperature = '0.3') =>
    fieldRows({
      provider: { value: provider, tier: 'global' },
      endpoint: { value: endpoint, tier: 'global' },
      model: { value: model, tier: 'global' },
      temperature: { value: temperature, tier: 'global' },
    })

  it('bảng preset có đủ chín nhà cung cấp, "Endpoint khác" không mang endpoint hay mô hình', async () => {
    const { AI_PROVIDERS, AI_PROVIDER_PRESETS } = await freshState()
    expect([...AI_PROVIDERS]).toEqual([
      'anthropic', 'openai', 'gemini', 'openrouter', 'mistral', 'xai', 'ollama', 'lmstudio', 'custom',
    ])
    expect(AI_PROVIDER_PRESETS.gemini).toEqual({
      endpoint: 'https://generativelanguage.googleapis.com/v1beta/openai',
      model: 'gemini-3.8-flash',
      temperature: 'default',
      maxTokens: 'default',
      keyPlaceholder: 'AIza…',
    })
    expect(AI_PROVIDER_PRESETS.custom).toEqual({
      endpoint: '',
      model: '',
      temperature: '',
      maxTokens: '',
      keyPlaceholder: '',
    })
  })

  it('bảng preset: nhiệt độ và số token theo Design Notes, khoá API đúng placeholder', async () => {
    const { AI_PROVIDER_PRESETS: P } = await freshState()
    const row = (k: keyof typeof P) => [P[k].temperature, P[k].maxTokens, P[k].keyPlaceholder]
    expect(row('anthropic')).toEqual(['default', '8192', 'sk-ant-…'])
    expect(row('openai')).toEqual(['default', 'default', 'sk-…'])
    expect(row('gemini')).toEqual(['default', 'default', 'AIza…'])
    expect(row('openrouter')).toEqual(['default', 'default', 'sk-or-…'])
    expect(row('mistral')).toEqual(['0.2', 'default', ''])
    expect(row('xai')).toEqual(['default', 'default', 'xai-…'])
    expect(row('ollama')).toEqual(['0.2', 'default', ''])
    expect(row('lmstudio')).toEqual(['0.2', 'default', ''])
    expect(row('custom')).toEqual(['', '', ''])
  })

  it('bắt đầu trống: bấm Gemini ghi cả năm trường qua MỘT lời gọi, không hỏi, nạp lại đúng một lần', async () => {
    const state = await loaded(false)
    aiConfigGetMock.mockClear()
    await state.selectAiConfigProvider('gemini')
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'gemini' },
      { field: 'endpoint', value: 'https://generativelanguage.googleapis.com/v1beta/openai' },
      { field: 'model', value: 'gemini-3.8-flash' },
      { field: 'temperature', value: 'default' },
      { field: 'max_tokens', value: 'default' },
    ])
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(aiConfigGetMock).toHaveBeenCalledTimes(1)
  })

  it('ghi đè: endpoint đang là URL OpenAI, bấm Anthropic ⇒ hỏi, chưa có lượt IPC ghi nào', async () => {
    const state = await loaded(false, fieldsOf('openai', 'https://api.openai.com/v1', 'gpt-6.1-sol'))
    await state.selectAiConfigProvider('anthropic')
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(state.aiConfigPresetConfirm.value).toEqual({
      provider: 'anthropic',
      changes: [
        { field: 'endpoint', oldValue: 'https://api.openai.com/v1', newValue: 'https://api.anthropic.com/v1' },
        { field: 'model', oldValue: 'gpt-6.1-sol', newValue: 'claude-sonnet-5-5' },
        { field: 'temperature', oldValue: '0.3', newValue: 'default' },
      ],
    })
  })

  it('Áp dụng ghi cả năm trường rồi đóng hộp; Huỷ đóng hộp và không ghi gì, các trường giữ giá trị đã lưu', async () => {
    const state = await loaded(false, fieldsOf('openai', 'https://api.openai.com/v1', 'gpt-6.1-sol'))
    await state.selectAiConfigProvider('anthropic')
    state.cancelAiConfigPreset()
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(state.aiConfigDraft('endpoint')).toBe('https://api.openai.com/v1')
    expect(state.aiConfigDraft('model')).toBe('gpt-6.1-sol')
    expect(state.aiConfigDraft('temperature')).toBe('0.3')

    await state.selectAiConfigProvider('anthropic')
    await state.confirmAiConfigPreset()
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'anthropic' },
      { field: 'endpoint', value: 'https://api.anthropic.com/v1' },
      { field: 'model', value: 'claude-sonnet-5-5' },
      { field: 'temperature', value: 'default' },
      { field: 'max_tokens', value: '8192' },
    ])
  })

  it('bấm nút ngay sau khi gõ endpoint: lượt lưu khi rời ô xong trước rồi mới so ⇒ hỏi, không ghi đè im lặng', async () => {
    const state = await loaded(false, fieldsOf('openai', '', '', ''))
    let releaseSave: (value: null) => void = () => undefined
    aiConfigSaveFieldMock.mockImplementationOnce(
      () =>
        new Promise<null>((resolve) => {
          releaseSave = resolve
        }),
    )
    state.setAiConfigDraft('endpoint', 'https://proxy.local/v1')
    void state.commitAiConfigField('endpoint')
    const picked = state.selectAiConfigProvider('gemini')
    aiConfigGetMock.mockResolvedValue({
      fields: fieldsOf('openai', 'https://proxy.local/v1', '', ''),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    releaseSave(null)
    await picked
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(state.aiConfigPresetConfirm.value?.changes).toEqual([
      { field: 'endpoint', oldValue: 'https://proxy.local/v1', newValue: 'https://generativelanguage.googleapis.com/v1beta/openai' },
    ])
  })

  it('đã khớp: endpoint, model và nhiệt độ bằng giá trị preset ⇒ ghi, không hỏi', async () => {
    const state = await loaded(false, fieldsOf('openai', 'https://api.anthropic.com/v1', 'claude-sonnet-5-5', 'default'))
    await state.selectAiConfigProvider('anthropic')
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledTimes(1)
  })

  it('preset không có mô hình: bấm Ollama giữ `qwen`, không ghi model, không hỏi', async () => {
    const state = await loaded(false, fieldsOf('openai', '', 'qwen', '0.2'))
    await state.selectAiConfigProvider('ollama')
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'ollama' },
      { field: 'endpoint', value: 'http://localhost:11434/v1' },
      { field: 'temperature', value: '0.2' },
      { field: 'max_tokens', value: 'default' },
    ])
  })

  it('"Endpoint khác" chỉ ghi provider, kể cả khi endpoint và model đang có giá trị', async () => {
    const state = await loaded(false, fieldsOf('openai', 'https://api.openai.com/v1', 'gpt-6.1-sol'))
    await state.selectAiConfigProvider('custom')
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [{ field: 'provider', value: 'custom' }])
  })

  it('tầng Tác phẩm: bấm Mistral từ trống ghi năm trường ở tầng work, không hỏi', async () => {
    const state = await loaded(true)
    expect(state.aiConfigViewTier.value).toBe('work')
    await state.selectAiConfigProvider('mistral')
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('work', [
      { field: 'provider', value: 'mistral' },
      { field: 'endpoint', value: 'https://api.mistral.ai/v1' },
      { field: 'model', value: 'mistral-large-latest' },
      { field: 'temperature', value: '0.2' },
      { field: 'max_tokens', value: 'default' },
    ])
  })

  it('so với giá trị ở tầng đang xem: tầng Tác phẩm kế thừa endpoint Toàn cục khác preset ⇒ hỏi', async () => {
    const state = await loaded(true, fieldsOf('openai', 'https://api.openai.com/v1', ''))
    await state.selectAiConfigProvider('xai')
    expect(state.aiConfigPresetConfirm.value?.changes.map((c) => c.field)).toEqual(['endpoint', 'temperature'])
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
  })

  it('lượt ghi trượt: lỗi hiện ở dòng provider, không nạp lại, không thử lại', async () => {
    const error = { code: 'store.write_failed', message: 'x', params: {} }
    aiConfigSaveFieldsMock.mockResolvedValue(error)
    const state = await loaded(false)
    aiConfigGetMock.mockClear()
    await state.selectAiConfigProvider('gemini')
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledTimes(1)
    expect(aiConfigGetMock).not.toHaveBeenCalled()
    expect(state.aiConfigSaveErrorFor('provider')).toEqual(error)
    expect(state.aiConfigIsSaving('provider')).toBe(false)
  })

  it('mỗi preset ghi đúng nhiệt độ và số token của nó; Endpoint khác không ghi hai trường này', async () => {
    const state = await loaded(false, fieldsOf('openai', '', '', ''))
    for (const provider of state.AI_PROVIDERS) {
      aiConfigSaveFieldsMock.mockClear()
      await state.selectAiConfigProvider(provider)
      const preset = state.AI_PROVIDER_PRESETS[provider]
      const sent = aiConfigSaveFieldsMock.mock.calls.flatMap((c) => c[1] as { field: string; value: string }[])
      const valueOf = (f: string) => sent.find((x) => x.field === f)?.value
      expect(valueOf('temperature')).toBe(preset.temperature === '' ? undefined : preset.temperature)
      expect(valueOf('max_tokens')).toBe(preset.maxTokens === '' ? undefined : preset.maxTokens)
    }
  })

  it('xác nhận ghi đè liệt kê nhiệt độ và số token khác preset; trống hoặc cùng giá trị thì không liệt kê', async () => {
    const rows = fieldRows({
      provider: { value: 'mistral', tier: 'global' },
      temperature: { value: '0.3', tier: 'global' },
      max_tokens: { value: '4096', tier: 'global' },
    })
    const state = await loaded(false, rows)
    await state.selectAiConfigProvider('anthropic')
    expect(state.aiConfigPresetConfirm.value?.changes).toEqual([
      { field: 'temperature', oldValue: '0.3', newValue: 'default' },
      { field: 'max_tokens', oldValue: '4096', newValue: '8192' },
    ])
    state.cancelAiConfigPreset()

    const equalNumeric = await loaded(
      false,
      fieldRows({ temperature: { value: '0.20', tier: 'global' }, max_tokens: { value: 'default', tier: 'global' } }),
    )
    await equalNumeric.selectAiConfigProvider('mistral')
    expect(equalNumeric.aiConfigPresetConfirm.value).toBeNull()
  })

  it('I/O Matrix "Áp preset đè nhiệt độ": 0.3 rồi Anthropic ⇒ dòng 0.3 → default; Huỷ không ghi gì', async () => {
    const state = await loaded(false, fieldsOf('openai', '', '', '0.3'))
    await state.selectAiConfigProvider('anthropic')
    expect(state.aiConfigPresetConfirm.value?.changes).toContainEqual({
      field: 'temperature',
      oldValue: '0.3',
      newValue: 'default',
    })
    state.cancelAiConfigPreset()
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
  })

  it('I/O Matrix "Áp preset từ trống": Mistral ghi 0.2 và default cùng ba trường kia, không hỏi', async () => {
    const state = await loaded(false)
    await state.selectAiConfigProvider('mistral')
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledExactlyOnceWith('global', [
      { field: 'provider', value: 'mistral' },
      { field: 'endpoint', value: 'https://api.mistral.ai/v1' },
      { field: 'model', value: 'mistral-large-latest' },
      { field: 'temperature', value: '0.2' },
      { field: 'max_tokens', value: 'default' },
    ])
  })

  it('giá trị lưu là `default` và preset cũng `default` ⇒ không hỏi; preset số thì VẪN hỏi', async () => {
    const stored = fieldRows({
      temperature: { value: 'default', tier: 'global' },
      max_tokens: { value: 'default', tier: 'global' },
    })
    const state = await loaded(false, stored)
    await state.selectAiConfigProvider('openai')
    expect(state.aiConfigPresetConfirm.value).toBeNull()
    expect(aiConfigSaveFieldsMock).toHaveBeenCalledTimes(1)

    const other = await loaded(false, stored)
    await other.selectAiConfigProvider('mistral')
    expect(other.aiConfigPresetConfirm.value?.changes).toEqual([
      { field: 'temperature', oldValue: 'default', newValue: '0.2' },
    ])
  })

  it('placeholder khoá API theo nhà cung cấp; OpenAI là `sk-…`, nhà cung cấp lạ và chưa chọn là rỗng', async () => {
    const openai = await loaded(false, fieldRows({ provider: { value: 'openai', tier: 'global' } }))
    expect(openai.aiConfigKeyPlaceholder()).toBe('sk-…')
    const anthropic = await loaded(false, fieldRows({ provider: { value: 'anthropic', tier: 'global' } }))
    expect(anthropic.aiConfigKeyPlaceholder()).toBe('sk-ant-…')
    const mistral = await loaded(false, fieldRows({ provider: { value: 'mistral', tier: 'global' } }))
    expect(mistral.aiConfigKeyPlaceholder()).toBe('')
    const unknown = await loaded(false, fieldRows({ provider: { value: 'groq', tier: 'global' } }))
    expect(unknown.aiConfigKeyPlaceholder()).toBe('')
    expect((await loaded(false)).aiConfigKeyPlaceholder()).toBe('')
  })

  it('đổi tầng xem hoặc đặt lại mục thì hộp xác nhận bị gỡ', async () => {
    const state = await loaded(true, fieldsOf('openai', 'https://api.openai.com/v1', ''))
    await state.selectAiConfigProvider('xai')
    expect(state.aiConfigPresetConfirm.value).not.toBeNull()
    state.selectAiConfigViewTier('global')
    expect(state.aiConfigPresetConfirm.value).toBeNull()

    await state.selectAiConfigProvider('xai')
    expect(state.aiConfigPresetConfirm.value).not.toBeNull()
    state.resetAiConfigSection()
    expect(state.aiConfigPresetConfirm.value).toBeNull()
  })

  it('placeholder theo nhà cung cấp; giá trị lạ là "Endpoint khác" và không có placeholder', async () => {
    const state = await loaded(false, fieldRows({ provider: { value: 'ollama', tier: 'global' } }))
    expect(state.aiConfigEndpointPlaceholder()).toBe('http://localhost:11434/v1')
    expect(state.aiConfigProviderChoice()).toBe('ollama')

    const other = await loaded(false, fieldRows({ provider: { value: 'groq', tier: 'global' } }))
    expect(other.aiConfigProviderChoice()).toBeNull()
    expect(other.aiConfigProviderIsOther()).toBe(true)
    expect(other.aiConfigEndpointPlaceholder()).toBe('')
  })

  it('clearAllAiConfigOverrides xoá đúng các trường tier work, rồi đọc lại; không có Tác phẩm thì không làm gì', async () => {
    const rows = fieldRows({
      model: { value: 'w', tier: 'work' },
      temperature: { value: '0.5', tier: 'work' },
      max_tokens: { value: '10', tier: 'global' },
    })
    const state = await loaded(true, rows)
    expect(state.aiConfigOverrideCount.value).toBe(2)
    await state.clearAllAiConfigOverrides()
    expect(aiConfigClearOverrideMock.mock.calls.map((c) => c[0]).sort()).toEqual(['model', 'temperature'])

    aiConfigClearOverrideMock.mockClear()
    const closed = await loaded(false, rows)
    await closed.clearAllAiConfigOverrides()
    expect(aiConfigClearOverrideMock).not.toHaveBeenCalled()
  })
})

describe('chip nhiệt độ và số token — trạng thái bật, chọn chip, Tự nhập', () => {
  const rows = (o: Partial<Record<string, { value: string; tier: 'global' | 'work' }>> = {}) =>
    ['provider', 'endpoint', 'model', 'temperature', 'max_tokens'].map((field) => ({
      field,
      value: o[field]?.value ?? '',
      tier: o[field]?.tier ?? 'global',
      shadowed: null,
    }))

  async function loaded(workTierAvailable: boolean, r = rows()) {
    aiConfigGetMock.mockResolvedValue({ fields: r, workTierAvailable, keyConfigured: false, error: null })
    const state = await freshState()
    await state.loadAiConfigSection()
    return state
  }

  const lit = (state: Awaited<ReturnType<typeof freshState>>, field: 'temperature' | 'max_tokens') =>
    state.aiConfigChipChoices(field).filter((c) => state.aiConfigChipIsOn(field, c))

  it('bảng chip: ba giá trị số mỗi trường', async () => {
    const { AI_CHIP_VALUES, AI_CHIP_FIELDS } = await freshState()
    expect([...AI_CHIP_FIELDS]).toEqual(['temperature', 'max_tokens'])
    expect(AI_CHIP_VALUES.temperature).toEqual(['0.2', '0.5', '1'])
    expect(AI_CHIP_VALUES.max_tokens).toEqual(['4096', '8192', '16384'])
  })

  it('AC: giá trị lưu `default` hoặc không có hàng ⇒ chỉ chip Mặc định bật, ô Tự nhập đóng', async () => {
    const empty = await loaded(false)
    const stored = await loaded(
      false,
      rows({ temperature: { value: 'default', tier: 'global' }, max_tokens: { value: 'default', tier: 'global' } }),
    )
    for (const state of [empty, stored]) {
      expect(lit(state, 'temperature')).toEqual(['default'])
      expect(lit(state, 'max_tokens')).toEqual(['default'])
      expect(state.aiConfigChipInputIsOpen('temperature')).toBe(false)
    }
    expect(stored.aiConfigChipInputValue('temperature')).toBe('')
  })

  it('giá trị khớp đúng một chip số thì chip đó bật, kể cả cách viết khác (`0.50`, `+4096`)', async () => {
    const state = await loaded(
      false,
      rows({ temperature: { value: '0.50', tier: 'global' }, max_tokens: { value: '4096', tier: 'global' } }),
    )
    expect(lit(state, 'temperature')).toEqual(['0.5'])
    expect(lit(state, 'max_tokens')).toEqual(['4096'])
  })

  it('giá trị hợp lệ không khớp chip nào ⇒ chip Tự nhập bật, ô nhập hiện giá trị đó', async () => {
    const state = await loaded(
      false,
      rows({ temperature: { value: '0.35', tier: 'global' }, max_tokens: { value: '5000', tier: 'global' } }),
    )
    expect(lit(state, 'temperature')).toEqual(['custom'])
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(true)
    expect(state.aiConfigChipInputValue('temperature')).toBe('0.35')
    expect(lit(state, 'max_tokens')).toEqual(['custom'])
    expect(state.aiConfigChipInputValue('max_tokens')).toBe('5000')
  })

  it('giá trị lưu không hợp lệ ⇒ không chip nào bật', async () => {
    const state = await loaded(false, rows({ temperature: { value: '7', tier: 'global' } }))
    expect(lit(state, 'temperature')).toEqual([])
  })

  it('I/O Matrix "Chọn chip số": bấm 0.5 ⇒ một lượt aiConfigSaveField(global, temperature, 0.5), chip 0.5 bật sau khi nạp lại', async () => {
    const state = await loaded(false)
    aiConfigGetMock.mockResolvedValue({
      fields: rows({ temperature: { value: '0.5', tier: 'global' } }),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await state.selectAiConfigChip('temperature', '0.5')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'temperature', '0.5')
    expect(aiConfigSaveFieldsMock).not.toHaveBeenCalled()
    expect(lit(state, 'temperature')).toEqual(['0.5'])
  })

  it('bấm chip Mặc định ghi đúng chuỗi `default`, không xoá hàng', async () => {
    const state = await loaded(false, rows({ max_tokens: { value: '8192', tier: 'global' } }))
    await state.selectAiConfigChip('max_tokens', 'default')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'max_tokens', 'default')
    expect(aiConfigClearOverrideMock).not.toHaveBeenCalled()
  })

  it('bấm chip đang bật thì không ghi gì', async () => {
    const state = await loaded(false, rows({ temperature: { value: '0.2', tier: 'global' } }))
    await state.selectAiConfigChip('temperature', '0.2')
    await state.selectAiConfigChip('max_tokens', 'default')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
  })

  it('I/O Matrix "Mặc định ở tầng Tác phẩm": Toàn cục 0.3, tầng work bấm Mặc định ⇒ chỉ ghi tầng work', async () => {
    const state = await loaded(true, rows({ temperature: { value: '0.3', tier: 'global' } }))
    expect(state.aiConfigViewTier.value).toBe('work')
    await state.selectAiConfigChip('temperature', 'default')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('work', 'temperature', 'default')
    expect(aiConfigClearOverrideMock).not.toHaveBeenCalled()
  })

  it('AC: bấm chip ở tầng Tác phẩm biến trường thành ghi đè (đếm ghi đè tăng) — Trả về kế thừa xoá đúng hàng đó', async () => {
    const state = await loaded(true, rows({ temperature: { value: '0.5', tier: 'work' } }))
    expect(state.aiConfigOverrideCount.value).toBe(1)
    expect(lit(state, 'temperature')).toEqual(['0.5'])
    await state.clearAiConfigOverride('temperature')
    expect(aiConfigClearOverrideMock).toHaveBeenCalledExactlyOnceWith('temperature')
  })

  it('I/O Matrix "Tự nhập": bấm Tự nhập chỉ mở ô, không ghi; gõ 0.35 rồi rời ô lưu 0.35', async () => {
    const state = await loaded(false)
    await state.selectAiConfigChip('temperature', 'custom')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(lit(state, 'temperature')).toEqual(['custom'])
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(true)

    state.setAiConfigDraft('temperature', '0.35')
    await state.commitAiConfigField('temperature')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'temperature', '0.35')
  })

  it('Tự nhập: `3` bị từ chối, 0 lượt ghi', async () => {
    const state = await loaded(false)
    await state.selectAiConfigChip('temperature', 'custom')
    state.setAiConfigDraft('temperature', '3')
    await state.commitAiConfigField('temperature')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(true)
  })

  it('Tự nhập đang mở, bấm chip khác thì ô đóng và chip đó được ghi', async () => {
    const state = await loaded(false)
    await state.selectAiConfigChip('max_tokens', 'custom')
    await state.selectAiConfigChip('max_tokens', '16384')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledExactlyOnceWith('global', 'max_tokens', '16384')
    expect(state.aiConfigChipInputIsOpen('max_tokens')).toBe(false)
  })

  it('ô Tự nhập đóng khi đổi tầng xem và khi đặt lại mục', async () => {
    const state = await loaded(true)
    await state.selectAiConfigChip('temperature', 'custom')
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(true)
    state.selectAiConfigViewTier('global')
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(false)

    await state.selectAiConfigChip('temperature', 'custom')
    state.resetAiConfigSection()
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(false)
  })

  it('lượt ghi chip trượt ⇒ lỗi hiện ở trường đó', async () => {
    const error = { code: 'store.write_failed', message: 'x', params: {} }
    aiConfigSaveFieldMock.mockResolvedValue(error)
    const state = await loaded(false)
    await state.selectAiConfigChip('max_tokens', '4096')
    expect(state.aiConfigSaveErrorFor('max_tokens')).toEqual(error)
  })
})

describe('system_prompt — trạng thái, lưu nguyên văn, Khôi phục mặc định', () => {
  const APP_TEXT = 'Văn bản của app.\nDòng hai.'
  const row = (value: string, tier: 'global' | 'work' = 'global', shadowed: string | null = null) => ({
    field: 'system_prompt',
    value,
    tier,
    shadowed,
  })

  async function loaded(workTierAvailable: boolean, system: ReturnType<typeof row>) {
    aiConfigGetMock.mockResolvedValue({
      fields: [system],
      defaultSystemPrompt: APP_TEXT,
      workTierAvailable,
      keyConfigured: false,
      error: null,
    })
    const state = await freshState()
    await state.loadAiConfigSection()
    return state
  }

  it('AI_CONFIG_FIELDS có sáu trường, system_prompt cuối; isAiConfigValueValid luôn true, kể cả rỗng và toàn khoảng trắng', async () => {
    const { AI_CONFIG_FIELDS, isAiConfigValueValid } = await freshState()
    expect(AI_CONFIG_FIELDS).toHaveLength(6)
    expect(AI_CONFIG_FIELDS[5]).toBe('system_prompt')
    expect(isAiConfigValueValid('system_prompt', '')).toBe(true)
    expect(isAiConfigValueValid('system_prompt', '  \n ')).toBe(true)
  })

  it('trạng thái: văn bản của app ⇒ default, văn bản khác ⇒ edited, rỗng ⇒ off', async () => {
    expect((await loaded(false, row(APP_TEXT))).aiConfigSystemPromptStatus()).toBe('default')
    expect((await loaded(false, row('Của Ice'))).aiConfigSystemPromptStatus()).toBe('edited')
    expect((await loaded(false, row(''))).aiConfigSystemPromptStatus()).toBe('off')
    expect((await loaded(false, row('  \n '))).aiConfigSystemPromptStatus()).toBe('off')
    expect((await loaded(false, row(APP_TEXT))).aiConfigDefaultSystemPrompt()).toBe(APP_TEXT)
  })

  it('lưu giữ NGUYÊN văn xuống dòng và khoảng trắng đầu/cuối (không trim); ô rỗng lưu chuỗi rỗng', async () => {
    const state = await loaded(false, row(APP_TEXT))
    const typed = '  Một.\n\n Ba. \n'
    state.setAiConfigDraft('system_prompt', typed)
    await state.commitAiConfigField('system_prompt')
    expect(aiConfigSaveFieldMock).toHaveBeenLastCalledWith('global', 'system_prompt', typed)

    state.setAiConfigDraft('system_prompt', '')
    await state.commitAiConfigField('system_prompt')
    expect(aiConfigSaveFieldMock).toHaveBeenLastCalledWith('global', 'system_prompt', '')
    expect(aiConfigSaveFieldMock).toHaveBeenCalledTimes(2)
  })

  it('nháp bằng giá trị đang hiện ⇒ không ghi', async () => {
    const state = await loaded(false, row(APP_TEXT))
    await state.commitAiConfigField('system_prompt')
    expect(aiConfigSaveFieldMock).not.toHaveBeenCalled()
  })

  it('Khôi phục: Toàn cục xoá dòng ở tầng global, Tác phẩm xoá ở tầng work, rồi đọc lại', async () => {
    const global = await loaded(false, row('Của Ice'))
    expect(global.aiConfigSystemPromptCanRestore()).toBe(true)
    await global.restoreAiConfigSystemPrompt()
    expect(aiConfigClearOverrideMock).toHaveBeenLastCalledWith('system_prompt', 'global')

    const work = await loaded(true, row('Của Tác phẩm', 'work', 'Toàn cục'))
    expect(work.aiConfigSystemPromptCanRestore()).toBe(true)
    await work.restoreAiConfigSystemPrompt()
    expect(aiConfigClearOverrideMock).toHaveBeenLastCalledWith('system_prompt', 'work')
  })

  it('Khôi phục không có việc để làm: Toàn cục đang là văn bản của app, hay Tác phẩm không có dòng riêng', async () => {
    expect((await loaded(false, row(APP_TEXT))).aiConfigSystemPromptCanRestore()).toBe(false)
    expect((await loaded(true, row('Của Ice', 'global'))).aiConfigSystemPromptCanRestore()).toBe(false)
  })

  it('xem Toàn cục khi Tác phẩm tắt (rỗng) và Toàn cục có văn bản: hiện văn bản Toàn cục, không phải rỗng', async () => {
    const state = await loaded(true, row('', 'work', 'Toàn cục A'))
    expect(state.aiConfigSystemPromptStatus()).toBe('off')
    state.selectAiConfigViewTier('global')
    expect(state.aiConfigDraft('system_prompt')).toBe('Toàn cục A')
    expect(state.aiConfigSystemPromptStatus()).toBe('edited')
  })

  it('lưu trượt ⇒ lỗi hiện qua aiConfigSaveErrorFor, nháp giữ nguyên, không đọc lại', async () => {
    const state = await loaded(false, row(APP_TEXT))
    const err = { code: 'store.write_failed', message_key: 'err.unknown', params: {}, retryable: false }
    aiConfigSaveFieldMock.mockResolvedValue(err)
    const reads = aiConfigGetMock.mock.calls.length
    state.setAiConfigDraft('system_prompt', 'Bản nháp\ndở')
    await state.commitAiConfigField('system_prompt')
    expect(state.aiConfigSaveErrorFor('system_prompt')).toEqual(err)
    expect(state.aiConfigDraft('system_prompt')).toBe('Bản nháp\ndở')
    expect(aiConfigGetMock.mock.calls.length).toBe(reads)
  })
})

describe('đọc lại sau mỗi lượt ghi KHÔNG bật aiConfigLoading — Cài đặt thay cả mục bằng dòng đang tải theo cờ đó', () => {
  type State = Awaited<ReturnType<typeof freshState>>
  const workModelRow = {
    fields: [{ field: 'model', value: '', tier: 'work', shadowed: null }],
    workTierAvailable: true,
    keyConfigured: true,
    error: null,
  }
  const writes: ReadonlyArray<[string, (s: State) => Promise<unknown>]> = [
    [
      'saveAiConfigField',
      (s) => {
        s.setAiConfigDraft('model', 'new-model')
        return s.saveAiConfigField('model')
      },
    ],
    ['restoreAiConfigSystemPrompt', (s) => s.restoreAiConfigSystemPrompt()],
    ['clearAiConfigOverride', (s) => s.clearAiConfigOverride('model')],
    [
      'saveAiConfigKey',
      (s) => {
        s.setAiConfigKeyDraft('sk-test')
        return s.saveAiConfigKey()
      },
    ],
    ['deleteAiConfigKey', (s) => s.deleteAiConfigKey()],
    ['selectAiConfigProvider', (s) => s.selectAiConfigProvider('openai')],
    ['clearAllAiConfigOverrides', (s) => s.clearAllAiConfigOverrides()],
  ]

  it.each(writes)('%s', async (_name, write) => {
    const state = await freshState()
    aiConfigGetMock.mockResolvedValue(workModelRow)
    await state.loadAiConfigSection()

    let finishReload: (value: unknown) => void = () => {}
    aiConfigGetMock.mockReturnValueOnce(new Promise((resolve) => (finishReload = resolve)))
    const pending = write(state)
    await vi.waitFor(() => expect(aiConfigGetMock).toHaveBeenCalledTimes(2))

    expect(state.aiConfigLoading.value).toBe(false)
    finishReload(workModelRow)
    await pending
    expect(state.aiConfigLoading.value).toBe(false)
  })

  it('loadAiConfigSection vẫn bật cờ trong lúc nạp', async () => {
    const state = await freshState()
    let finishLoad: (value: unknown) => void = () => {}
    aiConfigGetMock.mockReturnValueOnce(new Promise((resolve) => (finishLoad = resolve)))
    const pending = state.loadAiConfigSection()
    expect(state.aiConfigLoading.value).toBe(true)
    finishLoad(workModelRow)
    await pending
    expect(state.aiConfigLoading.value).toBe(false)
  })
})

describe('đọc lại sau một lượt ghi chỉ làm mới ô VỪA GHI — chữ đang gõ ở ô khác không bị ghi đè', () => {
  function stored(model: string, endpoint: string) {
    return {
      fields: [
        { field: 'endpoint', value: endpoint, tier: 'global', shadowed: null },
        { field: 'model', value: model, tier: 'global', shadowed: null },
      ],
      workTierAvailable: false,
      keyConfigured: true,
      error: null,
    }
  }

  it('gõ ô Model và mở Tự nhập của nhiệt độ trong lúc đọc lại sau khi lưu Endpoint ⇒ cả hai còn nguyên', async () => {
    const state = await freshState()
    aiConfigGetMock.mockResolvedValue(stored('old-model', 'http://old'))
    await state.loadAiConfigSection()

    let finishReload: (value: unknown) => void = () => {}
    aiConfigGetMock.mockReturnValueOnce(new Promise((resolve) => (finishReload = resolve)))
    state.setAiConfigDraft('endpoint', '  http://new  ')
    const pending = state.saveAiConfigField('endpoint')
    await vi.waitFor(() => expect(aiConfigGetMock).toHaveBeenCalledTimes(2))

    state.setAiConfigDraft('model', 'typing-now')
    await state.selectAiConfigChip('temperature', 'custom')
    finishReload(stored('old-model', 'http://new'))
    await pending

    expect(state.aiConfigDraft('endpoint')).toBe('http://new')
    expect(state.aiConfigDraft('model')).toBe('typing-now')
    expect(state.aiConfigChipInputIsOpen('temperature')).toBe(true)
  })

  it('lượt đọc của lần ghi trước bị lượt sau vượt mặt ⇒ ô của lần ghi trước vẫn được làm mới', async () => {
    const state = await freshState()
    aiConfigGetMock.mockResolvedValue(stored('old-model', 'http://old'))
    await state.loadAiConfigSection()

    let finishFirst: (value: unknown) => void = () => {}
    aiConfigGetMock.mockReturnValueOnce(new Promise((resolve) => (finishFirst = resolve)))
    state.setAiConfigDraft('endpoint', '  http://new  ')
    const first = state.saveAiConfigField('endpoint')
    await vi.waitFor(() => expect(aiConfigGetMock).toHaveBeenCalledTimes(2))

    aiConfigGetMock.mockResolvedValueOnce(stored('new-model', 'http://new'))
    state.setAiConfigDraft('model', 'new-model')
    await state.saveAiConfigField('model')
    finishFirst(stored('old-model', 'http://new'))
    await first

    expect(state.aiConfigDraft('endpoint')).toBe('http://new')
    expect(state.aiConfigDraft('model')).toBe('new-model')
  })
})
