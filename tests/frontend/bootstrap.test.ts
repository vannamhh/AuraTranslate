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
