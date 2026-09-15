/**
 * `modes/libraryImport.ts` — kéo-thả N tệp cùng lúc (Story 6.6b, FR14 mở rộng).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 SỬA 2026-09-16 (vòng rà đối kháng 2, mục 11) — câu trích dưới đây từng gán nhầm cho
 * `AGENTS.md`; nó nằm trong CHÍNH spec này (`spec-6-6b-nhap-nhieu-tep-cung-luc.md`), không
 * phải `AGENTS.md`. Spec ghi thẳng: *"No test anywhere covers `drop_only_first` or
 * `wireDragDropOnce` — changing the drop path breaks nothing, and nothing catches a mistake
 * there."*
 * Tệp này đóng đúng lỗ đó: nó mô phỏng THẬT lượt kéo-thả qua bộ nghe sự kiện thật
 * (`listen()` bị giả lập để BẮT LẠI handler, rồi gọi handler đó với payload N đường dẫn —
 * không gọi tắt `droppedFilePaths.value = […]`), rồi đi tới tận `submitFilePath()`.
 *
 * ⚠️ Khuôn `libraryImportBlocksResubmitWhilePreviewOpen.test.ts`: gọi THẲNG các hàm module
 * (không qua DOM/`.vue`), `config/project`/`config/segment` giả lập bằng `vi.mock`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ImportEncodingPreview } from '../../src/config/project'

const previewFileMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: async () => ({ segments: [], caret_segment_id: null }) }
})

/**
 * 🔴 Bắt lại handler của MỖI sự kiện `listen()` đăng ký — `wireDragDropOnce()` gắn BA bộ
 * nghe (`file-drag-enter`/`file-drag-leave`/`file-dropped`); test này chỉ cần CHÍNH XÁC bộ
 * nghe thứ ba, nhưng bắt cả ba để không phụ thuộc thứ tự gắn dây bên trong module.
 */
const listenHandlers = new Map<string, (payload: unknown) => void>()
vi.mock('@tauri-apps/api/event', () => ({
  listen: async (name: string, handler: (event: { payload: unknown }) => void) => {
    listenHandlers.set(name, (payload: unknown) => handler({ payload }))
    return () => {
      listenHandlers.delete(name)
    }
  },
}))

const DRAG_DROP_EVENT = 'aura://file-dropped'

function preview(): ImportEncodingPreview {
  return {
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

function batchFor(paths: string[]) {
  return {
    batch: {
      items: paths.map((p) => ({ path: p, ok: true, error: null })),
      encoding_preview: preview(),
    },
    error: null,
  }
}

async function freshModule() {
  vi.resetModules()
  listenHandlers.clear()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  const nhap = await import('../../src/modes/libraryImport')
  nhap.wireDragDropOnce()
  await Promise.resolve() // để `listen(...).then(...)` (microtask) đăng ký xong handler
  return nhap
}

function fireDrop(paths: string[]): void {
  const handler = listenHandlers.get(DRAG_DROP_EVENT)
  if (handler === undefined) throw new Error('bo nghe file-dropped chua duoc gan day')
  handler(paths)
}

describe('libraryImport — kéo-thả N tệp cùng lúc (Story 6.6b)', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('thả 3 tệp giữ NGUYÊN cả ba đường dẫn, KHÔNG chỉ tệp đầu (retire drop_only_first)', async () => {
    const nhap = await freshModule()

    fireDrop(['/tmp/a.txt', '/tmp/b.md', '/tmp/c.txt'])

    expect(nhap.droppedFilePaths.value).toEqual(['/tmp/a.txt', '/tmp/b.md', '/tmp/c.txt'])
    expect(nhap.effectiveFilePaths.value).toEqual(['/tmp/a.txt', '/tmp/b.md', '/tmp/c.txt'])
    // N > 1: ô đường dẫn để TRỐNG (không có chỗ hiện N đường dẫn có nghĩa) — dữ liệu THẬT
    // sống ở `droppedFilePaths`, không ở `filePath`.
    expect(nhap.filePath.value).toBe('')
    expect(nhap.noticeKey.value).toBe('mode.library.drop_multiple_files')
  })

  it('thả ĐÚNG một tệp giữ hành vi CŨ — điền thẳng vào ô đường dẫn, không có thông báo', async () => {
    const nhap = await freshModule()

    fireDrop(['/tmp/mot-tep.txt'])

    expect(nhap.droppedFilePaths.value).toEqual(['/tmp/mot-tep.txt'])
    expect(nhap.filePath.value).toBe('/tmp/mot-tep.txt')
    expect(nhap.noticeKey.value).toBeNull()
  })

  it('N đường dẫn đã thả sống sót tới `submitFilePath()`, gửi ĐÚNG cả N — không phải phần tử đầu', async () => {
    const nhap = await freshModule()
    fireDrop(['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt'])
    previewFileMock.mockResolvedValue(batchFor(['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt']))

    await nhap.submitFilePath()

    expect(previewFileMock).toHaveBeenCalledWith(['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt'], 'zh', null)
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // §Always spec 6.6b — MỘT luật ưu tiên, hai chiều
  // ─────────────────────────────────────────────────────────────────────────────

  it('luật ưu tiên, chiều 1 — danh sách THẢ thắng khi không rỗng, ô gõ tay (còn giá trị cũ) bị bỏ qua', async () => {
    const nhap = await freshModule()
    nhap.filePath.value = '/tmp/da-go-truoc-do.txt'

    fireDrop(['/tmp/dropped-a.txt', '/tmp/dropped-b.txt'])

    expect(nhap.effectiveFilePaths.value).toEqual(['/tmp/dropped-a.txt', '/tmp/dropped-b.txt'])
  })

  it('luật ưu tiên, chiều 2 — GÕ vào ô đường dẫn dọn SẠCH danh sách thả', async () => {
    const nhap = await freshModule()
    fireDrop(['/tmp/dropped-a.txt', '/tmp/dropped-b.txt'])
    expect(nhap.droppedFilePaths.value).toHaveLength(2)

    nhap.onFilePathInput('/tmp/moi-go.txt')

    expect(nhap.droppedFilePaths.value).toEqual([])
    expect(nhap.effectiveFilePaths.value).toEqual(['/tmp/moi-go.txt'])
    expect(nhap.filePath.value).toBe('/tmp/moi-go.txt')
  })

  it('cả hai nguồn rỗng ⇒ `effectiveFilePaths` rỗng, `submitFilePath()` no-op (0 lời gọi IPC)', async () => {
    const nhap = await freshModule()
    expect(nhap.effectiveFilePaths.value).toEqual([])

    await nhap.submitFilePath()

    expect(previewFileMock).not.toHaveBeenCalled()
  })

  it('lượt tạo Tác phẩm THÀNH CÔNG xoá SẠCH cả `filePath` lẫn `droppedFilePaths`', async () => {
    const nhap = await freshModule()
    fireDrop(['/tmp/a.txt', '/tmp/b.txt'])
    previewFileMock.mockResolvedValue(batchFor(['/tmp/a.txt', '/tmp/b.txt']))
    await nhap.submitFilePath()

    const created = {
      meta: {
        meta_schema_version: 1,
        work_id: 'w1',
        name: 'Ten',
        source_lang: 'zh',
        genre: '',
        created_at: '',
        updated_at: '',
        chapter_count: 2,
      },
      folder: '/tmp/Ten.atproj',
      images_saved: 0,
      images_failed: 0,
    }
    nhap.finishImportSubmission(created, null)

    expect(nhap.filePath.value).toBe('')
    expect(nhap.droppedFilePaths.value).toEqual([])
  })

  it('bỏ qua phần tử thả rỗng/không phải chuỗi một cách phòng thủ, không panic', async () => {
    const nhap = await freshModule()

    fireDrop(['', '/tmp/that.txt', '']) // hàng rào phòng thủ — Rust không bao giờ gửi chuỗi rỗng thật

    expect(nhap.droppedFilePaths.value).toEqual(['/tmp/that.txt'])
  })
})
