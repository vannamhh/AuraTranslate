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
 * `serde::Serialize` KHÔNG `rename_all`) trả về. */
function validWirePreview(): Record<string, unknown> {
  return {
    confidence: 'low',
    selected_encoding: 'GBK',
    candidates: [
      { label: 'UTF-8', encoding: 'UTF-8', preview: null },
      { label: 'GB18030', encoding: 'gb18030', preview: '萧炎' },
      { label: 'GBK', encoding: 'GBK', preview: '萧炎' },
      { label: 'Big5', encoding: 'Big5', preview: '達鍁' },
      { label: 'UTF-16', encoding: 'UTF-16LE', preview: '扡摣' },
    ],
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

    const result = await previewImportEncodingFromText('plain ascii')

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

    const result = await previewImportEncodingFromText('plain ascii')

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

    const result = await previewImportEncodingFromText('plain ascii')

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

    const result = await previewImportEncodingFromText('x')

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('nhánh TỆP dùng CHUNG bộ phân giải — payload hợp lệ đi qua thật', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewImportEncodingFromFile } = await import('../../src/config/project')

    const result = await previewImportEncodingFromFile('/tmp/gbk.txt')

    expect(result.error).toBeNull()
    expect(result.preview?.candidates).toHaveLength(5)
  })
})
