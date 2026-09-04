/**
 * `config/project.ts::{isImportEncodingPreview, callPreviewImportEncoding}` — bộ phân giải
 * hình dạng dây THẬT của lượt xem trước bảng mã (Story 6.3, FR126).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — vòng rà đối kháng 2, mục 2
 * ─────────────────────────────────────────────────────────────────────────────
 * `tests/frontend/importPreviewEncoding.test.ts`, `libraryImportResetsSegmentHistory.test.ts`,
 * và `libraryImportRetryAfterFailedConfirm.test.ts` đều `vi.mock('../../src/config/project', …)`
 * — TRỌN adapter, kể cả `previewImportEncodingFromText`/`previewImportEncodingFromFile`. Hai
 * hàm đó (và `isImportEncodingPreview` bên trong `callPreviewImportEncoding`) vì thế CHƯA
 * TỪNG chạy thật trong bộ test — một payload sai hình dạng (ví dụ Rust thêm
 * `#[serde(rename_all = "camelCase")]`) đi lọt HOÀN TOÀN mà không cổng nào thấy, đúng khuôn
 * `bootstrap.test.ts:12-16`/`glossaryMarksRefresh.test.ts` đã đóng cho hai adapter khác.
 *
 * Mock `@tauri-apps/api/core` trực tiếp — KHÔNG mock `src/config/project` — để hai hàm đó
 * chạy THẬT.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

/** Payload dây HỢP LỆ — hình dạng THẬT mà `commands::project::ImportEncodingPreview` (Rust,
 * `serde::Serialize` KHÔNG `rename_all`) trả về. Story 6.4 thêm `normalized` trên mỗi ô —
 * `null` đồng bộ với `preview: null`, một object khi `preview` có chữ. */
function validWirePreview(): Record<string, unknown> {
  return {
    confidence: 'low',
    selected_encoding: 'GBK',
    candidates: [
      { label: 'UTF-8', encoding: 'UTF-8', preview: null, normalized: null },
      {
        label: 'GB18030',
        encoding: 'gb18030',
        preview: '萧炎',
        normalized: { text: '萧炎', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
      },
      {
        label: 'GBK',
        encoding: 'GBK',
        preview: '萧炎',
        normalized: { text: '萧炎', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
      },
      {
        label: 'Big5',
        encoding: 'Big5',
        preview: '達鍁',
        normalized: { text: '達鍁', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
      },
      {
        label: 'UTF-16',
        encoding: 'UTF-16LE',
        preview: '扡摣',
        normalized: { text: '扡摣', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
      },
    ],
    // candidates khong rong -- doc .normalized cua ung vien dang chon, khong doc truong nay.
    self_declared_normalized: null,
  }
}

beforeEach(() => {
  vi.resetModules()
  mockInvoke.mockReset()
})

describe('previewImportEncodingFromText/_FromFile — hình dạng dây THẬT (không mock adapter)', () => {
  it('payload snake_case hợp lệ đi qua isImportEncodingPreview thật, không bị bác', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('plain ascii', 'en')

    expect(result.error).toBeNull()
    expect(result.preview).not.toBeNull()
    expect(result.preview?.selected_encoding).toBe('GBK')
    expect(result.preview?.candidates).toHaveLength(5)
    expect(result.preview?.candidates[0]?.preview).toBeNull()
    expect(result.preview?.candidates[1]?.preview).toBe('萧炎')
  })

  // 🔴 Đối chứng cho chính lỗ hổng mục 2 tả: nếu Rust ĐỔI sang camelCase
  // (`#[serde(rename_all = "camelCase")]`), payload thật sẽ có hình dạng NÀY — ca này chứng
  // minh `isImportEncodingPreview` THẬT SỰ bác nó, không lặng lẽ chấp nhận.
  it('payload camelCase (mô phỏng một lượt Rust đổi rename_all) BỊ BÁC, không đi lọt', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'low',
      selectedEncoding: 'GBK', // camelCase -- sai hinh dang
      candidates: [],
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('plain ascii', 'en')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
    expect(result.error?.message_key).toBe('err.unknown')
  })

  // Item 23 — một phần tử THIẾU trường trong `candidates` phải bị bác, không lọt qua thành
  // `undefined` hiện lên dải.
  it('một phần tử `candidates` thiếu trường (`encoding` vắng mặt) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'low',
      selected_encoding: 'GBK',
      candidates: [{ label: 'UTF-8', preview: null }], // thieu `encoding`
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('plain ascii', 'en')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('`preview: undefined` (JSON.stringify sẽ bỏ hẳn trường) trong một phần tử bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [{ label: 'UTF-8', encoding: 'UTF-8' }], // thieu `preview` han
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Story 6.4 — `normalized` VẮNG MẶT (không phải `null`, không phải một object hợp lệ)
  // phải bác CẢ payload, đúng lý lẽ ghi ở `isEncodingCandidateWire` (`src/config/project.ts`):
  // một backend cũ chưa nâng cấp trả object thiếu trường này lọt qua sẽ làm `.vue` đọc
  // `candidate.normalized.text` trên `undefined` rồi vỡ trắng màn hình.
  it('candidates[].normalized VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [{ label: 'UTF-8', encoding: 'UTF-8', preview: 'abc' }], // thieu `normalized` han
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Story 6.4 — `normalized` mang một object THIẾU trường con (`joined_lines` vắng mặt)
  // cũng phải bác CẢ payload — cùng lý lẽ, một tầng sâu hơn.
  it('candidates[].normalized THIẾU trường con (`joined_lines` vắng mặt) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'abc',
          normalized: { text: 'abc', blank_lines_removed: 0, window_truncated: false }, // thieu joined_lines
        },
      ],
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Vá vòng rà 1, mục 1 — `self_declared_normalized` VẮNG MẶT (không phải `null`) làm CẢ
  // payload bị bác, cùng lý lẽ đã áp cho `candidates[].normalized`.
  it('self_declared_normalized VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    const payload = validWirePreview()
    delete payload.self_declared_normalized
    mockInvoke.mockResolvedValue(payload)
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Ca DƯƠNG bổ khuyết — nhánh TỰ KHAI (0 ứng viên) với `self_declared_normalized` hợp lệ
  // phải đi qua thật.
  it('payload TỰ KHAI (0 ứng viên) với self_declared_normalized hợp lệ đi qua thật', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'self_declared',
      selected_encoding: 'UTF-8',
      candidates: [],
      self_declared_normalized: { text: 'da dan', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('da dan', 'en')

    expect(result.error).toBeNull()
    expect(result.preview?.candidates).toHaveLength(0)
    expect(result.preview?.self_declared_normalized?.text).toBe('da dan')
  })

  it('nhánh TỆP dùng CHUNG bộ phân giải — payload hợp lệ đi qua thật', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewImportEncodingFromFile } = await import('../../src/config/project')

    const result = await previewImportEncodingFromFile('/tmp/gbk.txt', 'zh')

    expect(result.error).toBeNull()
    expect(result.preview?.candidates).toHaveLength(5)
  })
})
