/**
 * `config/bootstrap.ts::loadBootstrapConfig` — nhánh phân giải `glossary_scan_threshold`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — rà ba lớp 2026-08-22 bắt được một khoảng hở nghiệm thu
 * ─────────────────────────────────────────────────────────────────────────────
 * Trước tệp này, nhánh phân giải `glossary_scan_threshold` (`bootstrap.ts:226-230`) không
 * có test nào chạy QUA nó: `glossarySettings.test.ts` mock TRỌN `config/bootstrap` (không
 * `importOriginal`), nên vệ `typeof === 'number' && Number.isInteger(...) && > 0` chưa từng
 * được thi hành thật trong bộ test. Nếu vệ đó hỏng, một giá trị sai (chuỗi, số 0, số âm, số
 * thập phân) đi thẳng vào ô nhập lần đầu mở lớp phủ mà **0 test đỏ**.
 *
 * Lệch parity với khuôn nó chép: `isGlossaryMarkArray` (`config/glossary.ts`) có
 * `glossaryMarksRefresh.test.ts` chạy qua bộ phân giải THẬT bằng cách mock đúng biên IPC
 * (`@tauri-apps/api/core`), không mock trọn adapter. Tệp này đóng khoảng hở tương ứng cho
 * `bootstrap.ts`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

/** Payload dây HỢP LỆ, đủ mọi trường — chỉ `glossary_scan_threshold` đổi theo từng ca. */
function wirePayload(glossaryScanThreshold: unknown): Record<string, unknown> {
  const base: Record<string, unknown> = {
    theme: 'light',
    mode: 'library',
    shortcuts: {},
    layout_presets: {},
    workspace_layout: '',
    dict_sources_disabled: '',
  }
  if (glossaryScanThreshold !== undefined) {
    base.glossary_scan_threshold = glossaryScanThreshold
  }
  return base
}

beforeEach(() => {
  mockInvoke.mockReset()
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
})

describe('loadBootstrapConfig — phân giải glossary_scan_threshold qua bộ phân giải THẬT', () => {
  it('thiếu trường (bản Rust trước Story 3.5) ⇒ mặc định 5', async () => {
    mockInvoke.mockResolvedValueOnce(wirePayload(undefined))
    const { loadBootstrapConfig, bootstrapGlossaryScanThreshold, DEFAULT_GLOSSARY_SCAN_THRESHOLD } =
      await import('../../src/config/bootstrap')

    await loadBootstrapConfig()

    expect(bootstrapGlossaryScanThreshold.value).toBe(DEFAULT_GLOSSARY_SCAN_THRESHOLD)
    expect(DEFAULT_GLOSSARY_SCAN_THRESHOLD).toBe(5)
  })

  it('chuỗi "5" (sai KIỂU — Rust luôn gửi số) ⇒ mặc định 5', async () => {
    mockInvoke.mockResolvedValueOnce(wirePayload('5'))
    const { loadBootstrapConfig, bootstrapGlossaryScanThreshold, DEFAULT_GLOSSARY_SCAN_THRESHOLD } =
      await import('../../src/config/bootstrap')

    await loadBootstrapConfig()

    expect(bootstrapGlossaryScanThreshold.value).toBe(DEFAULT_GLOSSARY_SCAN_THRESHOLD)
  })

  it('0 ⇒ mặc định 5 — cùng luật `parse_glossary_scan_threshold` phía Rust (ngưỡng 0 tắt hết bộ lọc)', async () => {
    mockInvoke.mockResolvedValueOnce(wirePayload(0))
    const { loadBootstrapConfig, bootstrapGlossaryScanThreshold, DEFAULT_GLOSSARY_SCAN_THRESHOLD } =
      await import('../../src/config/bootstrap')

    await loadBootstrapConfig()

    expect(bootstrapGlossaryScanThreshold.value).toBe(DEFAULT_GLOSSARY_SCAN_THRESHOLD)
  })

  it('-1 ⇒ mặc định 5', async () => {
    mockInvoke.mockResolvedValueOnce(wirePayload(-1))
    const { loadBootstrapConfig, bootstrapGlossaryScanThreshold, DEFAULT_GLOSSARY_SCAN_THRESHOLD } =
      await import('../../src/config/bootstrap')

    await loadBootstrapConfig()

    expect(bootstrapGlossaryScanThreshold.value).toBe(DEFAULT_GLOSSARY_SCAN_THRESHOLD)
  })

  it('3.5 (không phải số nguyên) ⇒ mặc định 5', async () => {
    mockInvoke.mockResolvedValueOnce(wirePayload(3.5))
    const { loadBootstrapConfig, bootstrapGlossaryScanThreshold, DEFAULT_GLOSSARY_SCAN_THRESHOLD } =
      await import('../../src/config/bootstrap')

    await loadBootstrapConfig()

    expect(bootstrapGlossaryScanThreshold.value).toBe(DEFAULT_GLOSSARY_SCAN_THRESHOLD)
  })

  it('số nguyên dương hợp lệ (12) ⇒ chính số đó, không phải mặc định', async () => {
    mockInvoke.mockResolvedValueOnce(wirePayload(12))
    const { loadBootstrapConfig, bootstrapGlossaryScanThreshold } = await import('../../src/config/bootstrap')

    await loadBootstrapConfig()

    expect(bootstrapGlossaryScanThreshold.value).toBe(12)
  })
})

describe('lỗi không có hình dạng IpcError — phân biệt Tauri thật với trình duyệt thường', () => {
  const unknownError = {
    code: 'ipc.unknown',
    message_key: 'err.unknown',
    params: {},
    retryable: false,
  }

  it('bootstrap_config reject Error khi cầu Tauri tồn tại ⇒ err.unknown và config null', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    mockInvoke.mockRejectedValueOnce(new Error('bridge rejected'))
    const { loadBootstrapConfig, configError } = await import('../../src/config/bootstrap')

    const result = await loadBootstrapConfig()

    expect(result).toEqual({ config: null, error: unknownError })
    expect(configError.value).toEqual(unknownError)
  })

  it('bootstrap_config reject Error ngoài Tauri ⇒ error null để npm run dev dùng mặc định', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('no bridge'))
    const { loadBootstrapConfig } = await import('../../src/config/bootstrap')

    await expect(loadBootstrapConfig()).resolves.toEqual({ config: null, error: null })
  })

  it('put_config reject Error khi cầu Tauri tồn tại ⇒ err.unknown, không báo lưu thành công giả', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    mockInvoke.mockRejectedValueOnce(new Error('bridge rejected'))
    const { putConfig } = await import('../../src/config/bootstrap')

    const error = await putConfig('app_config', 'glossary_scan_threshold', '6')

    expect(error).toEqual(unknownError)
  })

  it('put_config reject Error ngoài Tauri ⇒ null để npm run dev tiếp tục dùng được', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('no bridge'))
    const { putConfig } = await import('../../src/config/bootstrap')

    await expect(putConfig('app_config', 'glossary_scan_threshold', '6')).resolves.toBeNull()
  })

  it('delete_config reject Error khi cầu Tauri tồn tại ⇒ err.unknown, không báo xoá thành công giả', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    mockInvoke.mockRejectedValueOnce(new Error('bridge rejected'))
    const { deleteConfig } = await import('../../src/config/bootstrap')

    await expect(deleteConfig('app_config', 'mode.library')).resolves.toEqual(unknownError)
  })

  it('delete_config reject Error ngoài Tauri ⇒ null để npm run dev tiếp tục dùng được', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('no bridge'))
    const { deleteConfig } = await import('../../src/config/bootstrap')

    await expect(deleteConfig('app_config', 'mode.library')).resolves.toBeNull()
  })
})
