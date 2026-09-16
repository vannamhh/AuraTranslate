/**
 * `config/project.ts::{isBilingualImportEncodingPreview, previewBilingualImportFromFile}` — bộ
 * phân giải hình dạng dây THẬT của lượt xem trước song ngữ (Story 6.16, FR115; trường
 * `chapters` mới của Story 6.16b).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — spec 6.16b §Code Map "Tests that move"
 * ─────────────────────────────────────────────────────────────────────────────
 * `tests/frontend/importPreviewBilingual.test.ts` `vi.mock('../../src/config/project', …)`
 * TRỌN ba hàm gọi Rust (`previewBilingualImportFromFile`/`rebuildBilingualImportPreview`/
 * `confirmBilingualImport`) — `isBilingualImportEncodingPreview` (sống BÊN TRONG
 * `previewBilingualImportFromFile`) vì thế CHƯA TỪNG chạy thật trong bộ test, khác hẳn song
 * sinh đơn ngữ của nó (`importPreviewEncodingWireShape.test.ts`, 24 ca không mock adapter). Một
 * payload sai hình dạng (Rust đổi `rename_all`, hoặc một trường `Option` mới thiếu vế `null`
 * trong guard) đi lọt HOÀN TOÀN mà không cổng nào thấy.
 *
 * Mock `@tauri-apps/api/core` trực tiếp — KHÔNG mock `src/config/project` — để
 * `isBilingualImportEncodingPreview`/`previewBilingualImportFromFile` chạy THẬT.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

/** Bốn trường xuất xứ RỖNG — hình dạng THẬT của `commands::project::ChapterOriginWire`
 * (Story 6.15), cùng stub `importPreviewEncodingWireShape.test.ts::originStub` dùng cho đường
 * đơn ngữ. */
function originStub(): Record<string, unknown> {
  return {
    author: null,
    site_name: null,
    url: null,
    published_at: null,
    author_confirmed: false,
    site_name_confirmed: false,
    url_confirmed: false,
    published_at_confirmed: false,
  }
}

/** Khối tách Chương tối giản, một Chương duy nhất — hình dạng THẬT của
 * `commands::project::ChapterSplitPreviewWire` (Story 6.6/6.10), TÁI DÙNG nguyên trên đường
 * song ngữ (Story 6.16b) qua `BilingualEncodingCandidateWire.chapters`. */
function chaptersFor(title: string): Record<string, unknown> {
  return {
    chapter_count: 1,
    chapters: [
      {
        ord: 1,
        title,
        length: title.length,
        cleanup_match_count: 0,
        joined_line_count_in_chapter: null,
        needs_review: false,
        review_causes: [],
        origin: originStub(),
        source_file: null,
      },
    ],
    broken_item_count: 0,
    needs_review_count: 0,
    clean_count: 1,
    any_signal_participated: false,
  }
}

/** Một ứng viên hợp lệ — hình dạng THẬT của `commands::project::BilingualEncodingCandidateWire`. */
function candidateFor(label: string, preview: string | null): Record<string, unknown> {
  return {
    label,
    encoding: label,
    preview,
    row_count: preview === null ? 0 : 1,
    chapter_count: preview === null ? 0 : 1,
    pair_count: 0,
    skipped_target_sentence_count: 0,
    mismatches: [],
    chapters: preview === null ? null : chaptersFor(preview),
  }
}

/** Payload dây HỢP LỆ — hình dạng THẬT mà `commands::project::BilingualImportEncodingPreview`
 * trả về (`serde::Serialize` KHÔNG `rename_all`). */
function validWirePreview(): Record<string, unknown> {
  return {
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [candidateFor('UTF-16', null), candidateFor('UTF-8', 'a,b')],
    sample_rows: [['a', 'b']],
    row_count: 1,
    column_count: 2,
  }
}

beforeEach(() => {
  vi.resetModules()
  mockInvoke.mockReset()
})

describe('previewBilingualImportFromFile — hình dạng dây THẬT (không mock adapter)', () => {
  it('payload snake_case hợp lệ đi qua isBilingualImportEncodingPreview thật, không bị bác', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    const result = await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, false)

    expect(result.error).toBeNull()
    expect(result.preview).not.toBeNull()
    expect(result.preview?.selected_encoding).toBe('UTF-8')
    expect(result.preview?.candidates).toHaveLength(2)
    expect(result.preview?.candidates[0]?.chapters).toBeNull()
    expect(result.preview?.candidates[1]?.chapters?.chapter_count).toBe(1)
  })

  // Đối chứng: nếu Rust đổi sang camelCase (`#[serde(rename_all = "camelCase")]`), payload
  // thật sẽ có hình dạng NÀY — ca này chứng minh guard THẬT SỰ bác nó, không lặng lẽ chấp nhận.
  it('payload camelCase (mô phỏng một lượt Rust đổi rename_all) BỊ BÁC, không đi lọt', async () => {
    mockInvoke.mockResolvedValue({ confidence: 'high', selectedEncoding: 'UTF-8', candidates: [] })
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    const result = await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, false)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
    expect(result.error?.message_key).toBe('err.unknown')
  })

  // Payload thiếu hẳn `row_count` (trường bắt buộc cấp cao nhất) làm CẢ payload bị bác.
  it('payload thiếu trường cấp cao nhất (`row_count` vắng mặt) làm CẢ payload bị bác', async () => {
    const payload = validWirePreview()
    delete payload.row_count
    mockInvoke.mockResolvedValue(payload)
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    const result = await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, false)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Một phần tử `candidates` thiếu trường (`encoding` vắng mặt) phải bác CẢ payload, không lọt
  // qua thành `undefined` hiện lên dải — cùng khuôn `importPreviewEncodingWireShape.test.ts`.
  it('một phần tử `candidates` thiếu trường (`encoding` vắng mặt) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [{ label: 'UTF-8', preview: null }],
      sample_rows: [],
      row_count: 0,
      column_count: 0,
    })
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    const result = await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, false)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // **THÊM (Story 6.16b)** — `candidates[].chapters` VẮNG MẶT (thiếu trường hẳn, không phải
  // `null`) phải bác CẢ payload — đúng lý lẽ ghi ở `isBilingualEncodingCandidateWire`: một
  // backend cũ chưa nâng cấp trả object thiếu trường này lọt qua sẽ làm `.vue` đọc
  // `candidate.chapters.chapter_count` trên `undefined` rồi vỡ trắng màn hình.
  it('candidates[].chapters VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'a,b',
          row_count: 1,
          chapter_count: 1,
          pair_count: 0,
          skipped_target_sentence_count: 0,
          mismatches: [],
          // thieu `chapters` han
        },
      ],
      sample_rows: [['a', 'b']],
      row_count: 1,
      column_count: 2,
    })
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    const result = await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, false)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // **THÊM (Story 6.16b)** — `candidates[].chapters` mang một object THIẾU trường con
  // (`any_signal_participated` vắng mặt) cũng phải bác CẢ payload — cùng lý lẽ, một tầng sâu
  // hơn, tái dùng ĐÚNG `isChapterSplitPreviewWire` mà đường đơn ngữ đã canh.
  it('candidates[].chapters THIẾU trường con (`any_signal_participated` vắng mặt) làm CẢ payload bị bác', async () => {
    const badChapters = chaptersFor('a,b')
    delete badChapters.any_signal_participated
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'a,b',
          row_count: 1,
          chapter_count: 1,
          pair_count: 0,
          skipped_target_sentence_count: 0,
          mismatches: [],
          chapters: badChapters,
        },
      ],
      sample_rows: [['a', 'b']],
      row_count: 1,
      column_count: 2,
    })
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    const result = await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, false)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('gửi đúng camelCase args cho invoke, `chapterPattern: null` không bị đúc thành object rỗng', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewBilingualImportFromFile } = await import('../../src/config/project')

    await previewBilingualImportFromFile('/tmp/a.csv', 'en', null, 0, 1, true)

    expect(mockInvoke).toHaveBeenCalledWith('preview_bilingual_import_from_file', {
      path: '/tmp/a.csv',
      sourceLang: 'en',
      chapterPattern: null,
      sourceColumn: 0,
      targetColumn: 1,
      hasHeader: true,
      regroupings: [],
    })
  })
})
