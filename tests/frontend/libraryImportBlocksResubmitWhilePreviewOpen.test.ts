/**
 * `modes/libraryImport.ts::submitPastedText`/`submitFilePath` — **`busy` không còn phủ đúng
 * cửa sổ ghi thật.** Story 6.3, vòng rà đối kháng 2, mục 9.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * `busy` hạ NGAY sau khi màn xem trước bảng mã MỞ (`submitPastedText`/`submitFilePath` tự
 * hạ nó ngay dưới lời gọi `openImportPreviewFrom*`) — lượt GHI thật (`create_work` phía
 * Rust) chỉ chạy SAU đó, ở `confirmImportPreview()`. Nút "Nhập" trên form hết `:disabled`
 * ngay khi màn xem trước mở, và chỉ còn bị chặn THỊ GIÁC bởi lớp phủ scrim của
 * `ImportPreviewOverlay.vue` — một lối vào KHÔNG đi qua DOM (`dispatch('library.import_text')`
 * gọi thẳng, cùng cách một phím tắt toàn cục/command palette sẽ gọi) vẫn mở được một lượt
 * xem trước THỨ HAI trong khi lượt ĐẦU còn đang mở/xác nhận — đè lên nguồn đang chờ.
 *
 * Test này gọi THẲNG `submitFilePath`/`submitPastedText` (không qua DOM, đúng đường mà một
 * lối vào không-DOM sẽ đi) để đối chứng việc chặn giờ nằm ở TẦNG HÀM, không chỉ ở `:disabled`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ImportEncodingPreview } from '../../src/config/project'

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string) => previewTextMock(text),
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

function preview(): ImportEncodingPreview {
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
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
})

describe('🔴 mục 9 — một màn xem trước đang MỞ chặn lượt nộp form THỨ HAI, dù `busy` đã hạ', () => {
  it('submitFilePath() thứ hai, TRONG LÚC lượt đầu còn mở, là no-op', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    const state = await import('../../src/importPreviewState')

    nhap.filePath.value = '/tmp/a.txt'
    previewFileMock.mockResolvedValue({ preview: preview(), error: null })
    await nhap.submitFilePath()

    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(nhap.busy.value).toBe(false) // đúng bẫy mục 9 -- busy đã hạ, dài trước lượt xác nhận
    expect(previewFileMock).toHaveBeenCalledTimes(1)

    // Lượt nộp THỨ HAI -- mô phỏng một lối vào không-DOM (dispatch trực tiếp) trong khi màn
    // xem trước ĐẦU còn mở. `:disabled` của nút không còn liên quan ở đây.
    nhap.filePath.value = '/tmp/b.txt'
    await nhap.submitFilePath()

    expect(previewFileMock).toHaveBeenCalledTimes(1) // KHÔNG một lượt gọi IPC thứ hai
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')
  })

  it('submitPastedText() TRONG LÚC một lượt xem trước-từ-tệp còn mở là no-op (chặn CHÉO nhánh)', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    const state = await import('../../src/importPreviewState')

    nhap.filePath.value = '/tmp/a.txt'
    previewFileMock.mockResolvedValue({ preview: preview(), error: null })
    await nhap.submitFilePath()
    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')

    nhap.pastedText.value = 'mot doan van ban dan tay'
    await nhap.submitPastedText()

    expect(previewTextMock).not.toHaveBeenCalled()
    // Nguồn "đã nộp từ đâu" KHÔNG bị ghi đè sang 'text' -- lượt thứ hai bị chặn TRƯỚC khi
    // đụng tới `openImportPreviewFromText` (nơi đổi ô này).
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')
  })
})
