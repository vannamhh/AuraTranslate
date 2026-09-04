/**
 * `modes/libraryImport.ts::finishImportSubmission` — **một lượt xác nhận TRƯỢT rồi THÀNH
 * CÔNG vẫn phải xoá đúng ô đã nộp.** Story 6.3, phản biện Ice 2026-09-04.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản đầu của `lastSubmittedFrom` (khi ô đó còn sống trong `libraryImport.ts`) xoá nó VÔ
 * ĐIỀU KIỆN ở cuối `finishImportSubmission`, kể cả trên nhánh TRƯỢT (`created === null`).
 * `tests/frontend/importPreviewEncoding.test.ts` đã đối chứng mệnh đề Ở TẦNG STATE
 * (`importPreviewLastSubmittedFrom` sống sót qua một lượt trượt). Tệp NÀY đối chứng mệnh đề
 * đó Ở TẦNG SẢN PHẨM — gọi thẳng `submitFilePath`/`finishImportSubmission` thật, không dựng
 * lại logic bằng tay — để một lượt hồi quy tương lai (ví dụ ai đó thêm lại một dòng xoá vô
 * điều kiện) bị bắt ở ĐÚNG chỗ người dùng chạm tới, không chỉ ở tầng state bên dưới.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const previewFileMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromFile: (path: string) => previewFileMock(path),
    confirmImportWithEncoding: (name: string, sourceLang: string, genre: string, encoding: string) =>
      confirmMock(name, sourceLang, genre, encoding),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: async () => ({ segments: [], caret_segment_id: null }) }
})

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

beforeEach(() => {
  vi.resetModules()
  previewFileMock.mockReset()
  confirmMock.mockReset()
})

describe('🔴 lượt xác nhận TRƯỢT rồi THÀNH CÔNG — ô đường dẫn vẫn được xoá đúng lúc', () => {
  it('filePath sống sót qua lượt trượt, rồi bị xoá SAU lượt thành công', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')

    nhap.filePath.value = '/tmp/gbk.txt'
    previewFileMock.mockResolvedValue({
      preview: {
        confidence: 'low',
        selected_encoding: 'Big5',
        candidates: [
          { label: 'UTF-8', encoding: 'UTF-8', preview: null },
          { label: 'GB18030', encoding: 'gb18030', preview: '萧炎' },
          { label: 'GBK', encoding: 'GBK', preview: '萧炎' },
          { label: 'Big5', encoding: 'Big5', preview: '達鍁' },
          { label: 'UTF-16', encoding: 'UTF-16LE', preview: '扡摣' },
        ],
      },
      error: null,
    })

    await nhap.submitFilePath()
    expect(preview.importPreviewIsOpen.value).toBe(true)
    expect(preview.importPreviewLastSubmittedFrom.value).toBe('file')

    // Lượt 1 — chọn nhầm Big5 (mặc định Rust chọn), TRƯỢT.
    const err = {
      code: 'import.undecodable_bytes',
      message_key: 'err.import.undecodable_bytes',
      params: {},
      retryable: false,
    }
    confirmMock.mockResolvedValue({ created: null, error: err })
    const failed = await preview.confirmImportPreview()
    if (failed.created !== null || failed.error !== null) {
      nhap.finishImportSubmission(failed.created, failed.error)
    }

    // Chưa bị xoá — lượt xác nhận vừa rồi TRƯỢT, không có gì để nói "đã nộp xong".
    expect(nhap.filePath.value).toBe('/tmp/gbk.txt')
    expect(preview.importPreviewIsOpen.value).toBe(true) // lớp phủ vẫn mở để chọn lại

    // Lượt 2 — chọn lại GBK, THÀNH CÔNG.
    preview.selectImportPreviewCandidate('GBK')
    const created = { meta: { work_id: 'w1', name: 'Ten' }, folder: '/tmp/Ten.atproj' }
    confirmMock.mockResolvedValue({ created, error: null })
    const ok = await preview.confirmImportPreview()
    if (ok.created !== null || ok.error !== null) {
      nhap.finishImportSubmission(ok.created, ok.error)
    }

    expect(nhap.createdWork.value).toEqual(created)
    // 🔴 Đây là dòng đỏ trên bản lỗi: `lastSubmittedFrom` bị xoá sớm ở lượt 1 làm nhánh
    // `if (lastSubmittedFrom.value === 'file')` ở `finishImportSubmission` không khớp nữa,
    // và `filePath` KHÔNG được xoá dù lượt nhập đã thành công thật.
    expect(nhap.filePath.value).toBe('')
  })
})
