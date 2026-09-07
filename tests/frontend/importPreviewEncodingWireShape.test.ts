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

/** Khối làm sạch tối giản, 0 luật — Story 6.5. Hình dạng THẬT của
 * `commands::project::CleanupPreviewWire`. */
function cleanupFor(text: string): Record<string, unknown> {
  return { text, spans: [], rules: [], window_truncated: false, final_text: text }
}

/** Khối tách Chương tối giản, một Chương duy nhất — Story 6.6. Hình dạng THẬT của
 * `commands::project::ChapterSplitPreviewWire`. */
function chaptersFor(title: string): Record<string, unknown> {
  return { chapter_count: 1, chapters: [{ ord: 1, title, length: title.length }] }
}

/** Khối tầng 2 tối giản, một khối `paragraph` đang giữ, chưa ai xác nhận — Story 6.9. Hình
 * dạng THẬT của `commands::project::ChapterBlocksPreviewWire` (`BlockWire`/`BlockBodyWire`
 * đúng khuôn `#[serde(tag = "kind", rename_all = "snake_case")]`). */
function blocksFor(text: string): Record<string, unknown> {
  return { blocks: [{ body: { kind: 'paragraph', text }, kept: true, confirmed: false }] }
}

/** Payload dây HỢP LỆ — hình dạng THẬT mà `commands::project::ImportEncodingPreview` (Rust,
 * `serde::Serialize` KHÔNG `rename_all`) trả về. Story 6.4 thêm `normalized` trên mỗi ô —
 * `null` đồng bộ với `preview: null`, một object khi `preview` có chữ. Story 6.5 thêm
 * `cleanup` trên mỗi ô + `self_declared_cleanup`. Story 6.6 thêm `chapters` trên mỗi ô +
 * `self_declared_chapters`, cùng luật đồng bộ `null`. */
function validWirePreview(): Record<string, unknown> {
  return {
    confidence: 'low',
    selected_encoding: 'GBK',
    candidates: [
      { label: 'UTF-8', encoding: 'UTF-8', preview: null, normalized: null, cleanup: null, chapters: null, blocks: null },
      {
        label: 'GB18030',
        encoding: 'gb18030',
        preview: '萧炎',
        normalized: { text: '萧炎', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: cleanupFor('萧炎'),
        chapters: chaptersFor('萧炎'),
        blocks: blocksFor('萧炎'),
      },
      {
        label: 'GBK',
        encoding: 'GBK',
        preview: '萧炎',
        normalized: { text: '萧炎', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: cleanupFor('萧炎'),
        chapters: chaptersFor('萧炎'),
        blocks: blocksFor('萧炎'),
      },
      {
        label: 'Big5',
        encoding: 'Big5',
        preview: '達鍁',
        normalized: { text: '達鍁', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: cleanupFor('達鍁'),
        chapters: chaptersFor('達鍁'),
        blocks: blocksFor('達鍁'),
      },
      {
        label: 'UTF-16',
        encoding: 'UTF-16LE',
        preview: '扡摣',
        normalized: { text: '扡摣', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: cleanupFor('扡摣'),
        chapters: chaptersFor('扡摣'),
        blocks: blocksFor('扡摣'),
      },
    ],
    // candidates khong rong -- doc .normalized/.cleanup/.chapters cua ung vien dang chon,
    // khong doc bon truong nay.
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
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

    const result = await previewImportEncodingFromText('plain ascii', 'en', null)

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

    const result = await previewImportEncodingFromText('plain ascii', 'en', null)

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

    const result = await previewImportEncodingFromText('plain ascii', 'en', null)

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

    const result = await previewImportEncodingFromText('x', 'en', null)

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

    const result = await previewImportEncodingFromText('x', 'en', null)

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

    const result = await previewImportEncodingFromText('x', 'en', null)

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

    const result = await previewImportEncodingFromText('x', 'en', null)

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
      self_declared_cleanup: cleanupFor('da dan'),
      self_declared_chapters: chaptersFor('da dan'),
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('da dan', 'en', null)

    expect(result.error).toBeNull()
    expect(result.preview?.candidates).toHaveLength(0)
    expect(result.preview?.self_declared_normalized?.text).toBe('da dan')
  })

  it('nhánh TỆP dùng CHUNG bộ phân giải — payload hợp lệ đi qua thật', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewImportEncodingFromFile } = await import('../../src/config/project')

    const result = await previewImportEncodingFromFile('/tmp/gbk.txt', 'zh', null)

    expect(result.error).toBeNull()
    expect(result.preview?.candidates).toHaveLength(5)
  })

  // ── Story 6.5 — khối làm sạch (tầng 3) trên dây ──────────────────────────────────

  // Cùng lý lẽ mục 23/vá vòng rà 1 đã áp cho `normalized` — `cleanup` VẮNG MẶT (không phải
  // `null`) phải bác CẢ payload, không lọt qua thành `undefined` hiện lên `.vue`.
  it('candidates[].cleanup VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'abc',
          normalized: { text: 'abc', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          // thieu `cleanup` han
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('candidates[].cleanup.spans THIẾU một trường con (`end` vắng mặt) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'abc',
          normalized: { text: 'abc', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          cleanup: {
            text: 'abc',
            spans: [{ tier: 'global', id: 1, start: 0 }], // thieu `end`
            rules: [],
            window_truncated: false,
            final_text: 'abc',
          },
          chapters: null,
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('self_declared_cleanup VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    const payload = validWirePreview()
    delete payload.self_declared_cleanup
    mockInvoke.mockResolvedValue(payload)
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Ca DƯƠNG — một khối làm sạch mang luật + span THẬT (không rỗng) phải đi qua nguyên vẹn,
  // không bị Kiểm TYPE cắt bớt trường nào.
  it('payload mang khối làm sạch KHÔNG rỗng (luật + span thật) đi qua nguyên vẹn', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'quang cao abc',
          normalized: { text: 'quang cao abc', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          cleanup: {
            text: 'quang cao abc',
            spans: [{ tier: 'global', id: 1, start: 0, end: 9 }],
            rules: [
              {
                tier: 'global',
                id: 1,
                pattern: 'quang cao',
                kind: 'literal',
                enabled: true,
                count_in_chapter: 1,
                count_in_import: 1,
              },
            ],
            window_truncated: false,
            final_text: 'abc',
          },
          chapters: chaptersFor('quang cao abc'),
          blocks: blocksFor('quang cao abc'),
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('quang cao abc', 'en', null)

    expect(result.error).toBeNull()
    const cleanup = result.preview?.candidates[0]?.cleanup
    expect(cleanup?.spans).toHaveLength(1)
    expect(cleanup?.rules[0]?.pattern).toBe('quang cao')
    expect(cleanup?.final_text).toBe('abc')
  })

  // ── Story 6.6 — khối tách Chương (tầng 4) trên dây ────────────────────────────────

  // Cùng lý lẽ đã áp cho `cleanup` (Story 6.5) — `chapters` VẮNG MẶT (không phải `null`)
  // phải bác CẢ payload, không lọt qua thành `undefined` hiện lên `.vue`.
  it('candidates[].chapters VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'abc',
          normalized: { text: 'abc', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          cleanup: cleanupFor('abc'),
          // thieu `chapters` han
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('self_declared_chapters VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    const payload = validWirePreview()
    delete payload.self_declared_chapters
    mockInvoke.mockResolvedValue(payload)
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Ca DƯƠNG — một khối tách Chương mang N > 1 Chương THẬT (title + length) phải đi qua
  // nguyên vẹn, không bị Kiểm TYPE cắt bớt trường nào.
  it('payload mang khối tách Chương N > 1 Chương đi qua nguyên vẹn', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'Chuong 1 Chuong 2',
          normalized: {
            text: 'Chuong 1 Chuong 2',
            joined_lines: 0,
            blank_lines_removed: 0,
            window_truncated: false,
          },
          cleanup: cleanupFor('Chuong 1 Chuong 2'),
          chapters: {
            chapter_count: 2,
            chapters: [
              { ord: 1, title: 'Chuong 1', length: 8 },
              { ord: 2, title: 'Chuong 2', length: 8 },
            ],
          },
          blocks: blocksFor('Chuong 1 Chuong 2'),
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.error).toBeNull()
    const chapters = result.preview?.candidates[0]?.chapters
    expect(chapters?.chapter_count).toBe(2)
    expect(chapters?.chapters).toHaveLength(2)
    expect(chapters?.chapters[1]?.title).toBe('Chuong 2')
  })

  // ── Story 6.9 — khối tầng 2, ranh giới bóc (FR123) trên dây ───────────────────────

  // Cùng lý lẽ đã áp cho `cleanup`/`chapters` — `blocks` VẮNG MẶT (không phải `null`) phải
  // bác CẢ payload, không lọt qua thành `undefined` hiện lên `.vue`.
  it('candidates[].blocks VẮNG MẶT (thiếu trường, không phải null) làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'abc',
          normalized: { text: 'abc', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          cleanup: cleanupFor('abc'),
          chapters: chaptersFor('abc'),
          // thieu `blocks` han
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('candidates[].blocks[].body THIẾU `kind` làm CẢ payload bị bác', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'abc',
          normalized: { text: 'abc', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          cleanup: cleanupFor('abc'),
          chapters: chaptersFor('abc'),
          blocks: { blocks: [{ body: { text: 'abc' }, kept: true, confirmed: false }] }, // thieu `kind`
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.preview).toBeNull()
    expect(result.error).not.toBeNull()
  })

  // Ca DƯƠNG — một dãy khối THẬT (ba nhánh thân, ba vạch lề) phải đi qua nguyên vẹn, không bị
  // Kiểm TYPE cắt bớt trường nào — khuôn `the_chapter_blocks_preview_wire_shape_carries_all_three_body_kinds_and_all_three_visible_states`
  // phía Rust (`segment_contract.rs`), cùng payload thật để hai bên không lệch nhau.
  it('payload mang dãy khối tầng 2 THẬT (ba nhánh thân, ba vạch lề) đi qua nguyên vẹn', async () => {
    mockInvoke.mockResolvedValue({
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'Khung dieu huong. Doan than bai.',
          normalized: {
            text: 'Khung dieu huong. Doan than bai.',
            joined_lines: 0,
            blank_lines_removed: 0,
            window_truncated: false,
          },
          cleanup: cleanupFor('Khung dieu huong. Doan than bai.'),
          chapters: chaptersFor('Khung dieu huong. Doan than bai.'),
          blocks: {
            blocks: [
              { body: { kind: 'paragraph', text: 'Khung dieu huong' }, kept: false, confirmed: false },
              { body: { kind: 'paragraph', text: 'Doan than bai' }, kept: true, confirmed: false },
              { body: { kind: 'caption', text: 'Chu thich anh' }, kept: true, confirmed: true },
              { body: { kind: 'image', src: 'https://example.com/a.jpg', alt: null }, kept: true, confirmed: false },
            ],
          },
        },
      ],
      self_declared_normalized: null,
      self_declared_cleanup: null,
      self_declared_chapters: null,
    })
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    const result = await previewImportEncodingFromText('x', 'en', null)

    expect(result.error).toBeNull()
    const blocks = result.preview?.candidates[0]?.blocks?.blocks
    expect(blocks).toHaveLength(4)
    expect(blocks?.[0]).toEqual({
      body: { kind: 'paragraph', text: 'Khung dieu huong' },
      kept: false,
      confirmed: false,
    })
    expect(blocks?.[2]).toEqual({ body: { kind: 'caption', text: 'Chu thich anh' }, kept: true, confirmed: true })
    expect(blocks?.[3]).toEqual({
      body: { kind: 'image', src: 'https://example.com/a.jpg', alt: null },
      kept: true,
      confirmed: false,
    })
  })

  // ── Story 6.6 — tham số `chapterPattern` gửi lên Rust ĐÚNG HÌNH DẠNG dây ──────────

  it('chapterPattern null gửi nguyên văn `null` cho invoke, không bị đúc thành một object rỗng', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewImportEncodingFromText } = await import('../../src/config/project')

    await previewImportEncodingFromText('van ban', 'en', null)

    expect(mockInvoke).toHaveBeenCalledWith('preview_import_encoding_from_text', {
      text: 'van ban',
      sourceLang: 'en',
      chapterPattern: null,
    })
  })

  it('chapterPattern { pattern, kind } gửi ĐÚNG hình dạng { pattern, kind } cho invoke', async () => {
    mockInvoke.mockResolvedValue(validWirePreview())
    const { previewImportEncodingFromFile } = await import('../../src/config/project')

    await previewImportEncodingFromFile('/tmp/x.txt', 'zh', { pattern: '第.*章', kind: 'regex' })

    expect(mockInvoke).toHaveBeenCalledWith('preview_import_encoding_from_file', {
      path: '/tmp/x.txt',
      sourceLang: 'zh',
      chapterPattern: { pattern: '第.*章', kind: 'regex' },
    })
  })
})
