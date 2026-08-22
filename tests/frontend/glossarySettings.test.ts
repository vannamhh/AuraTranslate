/**
 * State của lớp phủ "Cài đặt ngưỡng quét Glossary" — Story 3.5, FR47.
 *
 * ⚠️ Cùng khuôn `glossaryQuickAdd.test.ts`: `config/bootstrap.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

const putConfigMock = vi.fn()
/** Ref giả — chỉ cần `.value`, cùng khuôn `readonly(ref(...))` phía thật. */
let bootstrapThreshold = { value: 5 }

vi.mock('../../src/config/bootstrap', () => ({
  KEY_GLOSSARY_SCAN_THRESHOLD: 'glossary_scan_threshold',
  SCOPE_APP_CONFIG: 'app_config',
  get bootstrapGlossaryScanThreshold() {
    return bootstrapThreshold
  },
  putConfig: (...args: unknown[]) => putConfigMock(...args),
}))

/**
 * Nạp lại module mỗi ca — state là module-level singleton.
 *
 * ⚠️ Chỉ `vi.resetModules()` — KHÔNG đặt lại `putConfigMock`/`bootstrapThreshold` ở đây.
 * `beforeEach` (dưới) đã đặt chúng về mặc định TRƯỚC khi thân ca chạy; một ca cần tuỳ biến
 * (`mockResolvedValue`, `bootstrapThreshold = {...}`) phải làm điều đó TRƯỚC khi gọi hàm
 * này, và một lượt "dọn lại" thứ hai ở đây sẽ xoá mất chính tuỳ biến ca vừa đặt.
 */
async function freshState() {
  vi.resetModules()
  return import('../../src/glossarySettingsState')
}

beforeEach(() => {
  document.body.innerHTML = ''
  putConfigMock.mockReset()
  bootstrapThreshold = { value: 5 }
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('parsedGlossaryScanThreshold — hàm THUẦN, đọc lại đúng ràng buộc phía Rust', () => {
  it('chuỗi rỗng ⇒ null', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('')).toBeNull()
  })

  it('không phải số ("abc") ⇒ null', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('abc')).toBeNull()
  })

  it('"0" ⇒ null — ngưỡng 0 tắt hết bộ lọc, cùng luật `parse_glossary_scan_threshold` phía Rust', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('0')).toBeNull()
  })

  it('số âm ("-3") ⇒ null', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('-3')).toBeNull()
  })

  it('số thập phân ("3.5") ⇒ null', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('3.5')).toBeNull()
  })

  it('số nguyên dương hợp lệ ⇒ chính số đó', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('12')).toBe(12)
  })

  it('khoảng trắng biên bị cắt trước khi phân tích', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('  7  ')).toBe(7)
  })

  // ── Rà ba lớp 2026-08-22 — hai lớp phân giải phải khớp `u32::from_str` phía Rust THẬT ──

  it('dấu "+" ở đầu ⇒ chấp nhận, cùng `u32::from_str` phía Rust (bản trước từ chối, LỆCH)', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('+5')).toBe(5)
  })

  it('đúng bằng u32::MAX (4294967295) ⇒ chấp nhận', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('4294967295')).toBe(4294967295)
  })

  it('vượt u32::MAX đúng một (4294967296) ⇒ null — Rust `parse::<u32>()` từ chối giá trị này', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('4294967296')).toBeNull()
  })

  it('vượt xa u32::MAX (5000000000, "5 tỷ") ⇒ null — bản trước cho qua RỒI GHI, Rust âm thầm rơi về mặc định', async () => {
    const { parsedGlossaryScanThreshold } = await freshState()
    expect(parsedGlossaryScanThreshold('5000000000')).toBeNull()
  })
})

describe('openGlossarySettings/closeGlossarySettings', () => {
  it('mở lớp phủ nạp giá trị TỪ bootstrap vào ô nhập, và xoá lỗi cũ', async () => {
    bootstrapThreshold = { value: 8 }
    const { openGlossarySettings, glossarySettingsOverlayIsOpen, glossarySettingsThresholdInput } =
      await freshState()

    openGlossarySettings()

    expect(glossarySettingsOverlayIsOpen.value).toBe(true)
    expect(glossarySettingsThresholdInput.value).toBe('8')
  })

  it('đóng lớp phủ KHÔNG gọi putConfig', async () => {
    const { openGlossarySettings, closeGlossarySettings, glossarySettingsOverlayIsOpen } =
      await freshState()

    openGlossarySettings()
    closeGlossarySettings()

    expect(glossarySettingsOverlayIsOpen.value).toBe(false)
    expect(putConfigMock).not.toHaveBeenCalled()
  })

  it('close bị chặn trong lúc save pending; save thành công mới tự đóng', async () => {
    let finishSave: ((value: null) => void) | undefined
    putConfigMock.mockImplementationOnce(
      () => new Promise<null>((resolve) => {
        finishSave = resolve
      }),
    )
    const {
      openGlossarySettings,
      closeGlossarySettings,
      saveGlossarySettings,
      glossarySettingsThresholdInput,
      glossarySettingsOverlayIsOpen,
      glossarySettingsSaving,
    } = await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = '6'
    const pending = saveGlossarySettings()
    expect(glossarySettingsSaving.value).toBe(true)

    closeGlossarySettings()
    expect(glossarySettingsOverlayIsOpen.value).toBe(true)

    if (finishSave === undefined) throw new Error('fixture save chưa được gọi')
    finishSave(null)
    await pending
    expect(glossarySettingsSaving.value).toBe(false)
    expect(glossarySettingsOverlayIsOpen.value).toBe(false)
  })
})

describe('saveGlossarySettings — ô nhập từ chối giá trị hỏng, giá trị hợp lệ đi tới put_config đúng một lần', () => {
  it('giá trị không phải số ⇒ 0 lượt putConfig, lớp phủ vẫn mở', async () => {
    const { openGlossarySettings, saveGlossarySettings, glossarySettingsThresholdInput, glossarySettingsOverlayIsOpen } =
      await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = 'abc'
    await saveGlossarySettings()

    expect(putConfigMock).not.toHaveBeenCalled()
    expect(glossarySettingsOverlayIsOpen.value).toBe(true)
  })

  it('giá trị ≤ 0 ⇒ 0 lượt putConfig', async () => {
    const { openGlossarySettings, saveGlossarySettings, glossarySettingsThresholdInput } = await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = '0'
    await saveGlossarySettings()

    expect(putConfigMock).not.toHaveBeenCalled()
  })

  it('giá trị hợp lệ ⇒ ĐÚNG MỘT lượt putConfig, đúng kind/key/value, rồi đóng lớp phủ', async () => {
    putConfigMock.mockResolvedValue(null)
    const { openGlossarySettings, saveGlossarySettings, glossarySettingsThresholdInput, glossarySettingsOverlayIsOpen } =
      await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = '12'
    await saveGlossarySettings()

    expect(putConfigMock).toHaveBeenCalledTimes(1)
    expect(putConfigMock).toHaveBeenCalledWith('app_config', 'glossary_scan_threshold', '12')
    expect(glossarySettingsOverlayIsOpen.value).toBe(false)
  })

  it('lượt lưu TRƯỢT ⇒ IpcError hiện qua glossarySettingsSaveError, lớp phủ KHÔNG tự đóng', async () => {
    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    putConfigMock.mockResolvedValue(err)
    const { openGlossarySettings, saveGlossarySettings, glossarySettingsThresholdInput, glossarySettingsOverlayIsOpen, glossarySettingsSaveError } =
      await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = '12'
    await saveGlossarySettings()

    expect(putConfigMock).toHaveBeenCalledTimes(1)
    expect(glossarySettingsSaveError.value).toEqual(err)
    expect(glossarySettingsOverlayIsOpen.value).toBe(true)
  })

  it('lỗi IPC lạ đã chuẩn hoá thành err.unknown ⇒ lớp phủ ở lại để hiện lỗi', async () => {
    const unknown = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    putConfigMock.mockResolvedValue(unknown)
    const {
      openGlossarySettings,
      saveGlossarySettings,
      glossarySettingsThresholdInput,
      glossarySettingsOverlayIsOpen,
      glossarySettingsSaveError,
    } = await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = '12'
    await saveGlossarySettings()

    expect(glossarySettingsSaveError.value).toEqual(unknown)
    expect(glossarySettingsOverlayIsOpen.value).toBe(true)
  })

  it('mở lại SAU một lượt lưu thành công nạp giá trị VỪA LƯU, không giá trị bootstrap cũ', async () => {
    putConfigMock.mockResolvedValue(null)
    bootstrapThreshold = { value: 5 }
    const { openGlossarySettings, saveGlossarySettings, glossarySettingsThresholdInput } = await freshState()

    openGlossarySettings()
    glossarySettingsThresholdInput.value = '9'
    await saveGlossarySettings()

    openGlossarySettings()
    expect(glossarySettingsThresholdInput.value).toBe('9')
  })
})

describe('GlossarySettingsOverlay — bề mặt selection thật của modal', () => {
  it('vùng chọn chữ thật trong modal vai display không trở thành nguồn Auto-Lookup', async () => {
    const state = await freshState()
    state.openGlossarySettings()
    const [{ default: GlossarySettingsOverlay }, selectionContract] = await Promise.all([
      import('../../src/GlossarySettingsOverlay.vue'),
      import('../../src/panels/selectionContract'),
    ])
    const wrapper = mount(GlossarySettingsOverlay, { attachTo: document.body })
    await nextTick()

    const text = wrapper.get('.gs-intro').element.firstChild
    if (!(text instanceof Text) || text.data.length === 0) {
      wrapper.unmount()
      throw new Error('modal thật phải render một text node để dựng vùng chọn')
    }
    const range = document.createRange()
    range.setStart(text, 0)
    range.setEnd(text, Math.min(8, text.data.length))
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)
    const selectedText = selection?.toString() ?? ''

    expect(selectedText.length).toBeGreaterThan(0)
    expect(selectionContract.currentSelectionText()).toBe('')
    expect(selectionContract.currentSelectionTextForGlossaryQuickAdd()).toBe(selectedText)

    wrapper.unmount()
    selection?.removeAllRanges()
  })
})
