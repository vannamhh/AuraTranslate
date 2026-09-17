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
const aiConfigSaveKeyMock = vi.fn()
const aiConfigDeleteKeyMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigClearOverride: (...args: unknown[]) => aiConfigClearOverrideMock(...args),
  aiConfigSaveKey: (...args: unknown[]) => aiConfigSaveKeyMock(...args),
  aiConfigDeleteKey: (...args: unknown[]) => aiConfigDeleteKeyMock(...args),
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
  aiConfigSaveKeyMock.mockReset()
  aiConfigDeleteKeyMock.mockReset()
  aiConfigGetMock.mockResolvedValue({
    fields: emptyFields(),
    workTierAvailable: false,
    keyConfigured: false,
    error: null,
  })
  aiConfigSaveFieldMock.mockResolvedValue(null)
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
