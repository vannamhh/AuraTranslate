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
 * không gọi tắt `importFiles.value = […]`), rồi đi tới tận `submitFilePath()`.
 *
 * ⚠️ Khuôn `libraryImportBlocksResubmitWhilePreviewOpen.test.ts`: gọi THẲNG các hàm module
 * (không qua DOM/`.vue`), `config/project`/`config/segment` giả lập bằng `vi.mock`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ImportEncodingPreview } from '../../src/config/project'
import { fakeClassification } from './support/importClassify'

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

const pickFilesMock = vi.fn()

vi.mock('../../src/config/library', async (importOriginal) => {
  const { mockImportLibrary } = await import('./support/importClassify')
  return mockImportLibrary(
    importOriginal as () => Promise<typeof import('../../src/config/library')>,
    (...args) => pickFilesMock(...args),
  )
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: async (): ReturnType<typeof actual.readOpenChapterSegments> => ({
      loaded: { chapter_id: 1, segments: [], caret_segment_id: null, assets: [], assets_dir: '', tm_filled_segment_ids: [], tm_prefill: { kind: 'ran', hidden_count: 0 } },
      error: null,
    }),
  }
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
    source_lang: 'en',
    source_lang_auto: false,
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
      items: paths.map((p) => ({ path: p, ok: true, byte_len: null, error: null })),
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
  pickFilesMock.mockReset()
  const nhap = await import('../../src/modes/libraryImport')
  const overlay = await import('../../src/modes/libraryImportOverlayState')
  overlay.openLibraryImportOverlay()
  nhap.wireDragDropOnce()
  await Promise.resolve() // để `listen(...).then(...)` (microtask) đăng ký xong handler
  return nhap
}

async function fireDrop(paths: string[]): Promise<void> {
  const handler = listenHandlers.get(DRAG_DROP_EVENT)
  if (handler === undefined) throw new Error('bo nghe file-dropped chua duoc gan day')
  handler(paths)
  await new Promise((resolve) => setTimeout(resolve, 0))
}

function listedPaths(nhap: Awaited<ReturnType<typeof freshModule>>): string[] {
  return nhap.importFiles.value.map((f) => f.path)
}

describe('libraryImport — kéo-thả N tệp cùng lúc (Story 6.6b, 6.25)', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('thả 3 tệp giữ NGUYÊN cả ba đường dẫn, KHÔNG chỉ tệp đầu, và chuyển sang tab Tệp', async () => {
    const nhap = await freshModule()
    expect(nhap.importActiveTab.value).toBe('paste')

    await fireDrop(['/tmp/a.txt', '/tmp/b.md', '/tmp/c.txt'])

    expect(listedPaths(nhap)).toEqual(['/tmp/a.txt', '/tmp/b.md', '/tmp/c.txt'])
    expect(nhap.importFilesVerdict.value).toBe('text')
    expect(nhap.importActiveTab.value).toBe('files')
  })

  it('thả thêm lần nữa NỐI vào danh sách, bỏ đường dẫn đã có', async () => {
    const nhap = await freshModule()
    await fireDrop(['/tmp/a.txt'])

    await fireDrop(['/tmp/a.txt', '/tmp/b.txt'])

    expect(listedPaths(nhap)).toEqual(['/tmp/a.txt', '/tmp/b.txt'])
  })

  it('N đường dẫn đã thả sống sót tới `submitFilePath()`, gửi ĐÚNG cả N — không phải phần tử đầu', async () => {
    const nhap = await freshModule()
    await fireDrop(['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt'])
    previewFileMock.mockResolvedValue(batchFor(['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt']))

    await nhap.submitFilePath()

    expect(previewFileMock).toHaveBeenCalledWith(['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt'], 'auto', null, null)
  })

  it('thả `.pdf` ⇒ hàng bị đánh dấu không nhận, `submitFilePath()` no-op cho tới khi `Bỏ`', async () => {
    const nhap = await freshModule()
    await fireDrop(['/tmp/a.txt', '/tmp/x.pdf'])

    expect(nhap.importFilesVerdict.value).toBe('has_unsupported')
    expect(nhap.importFiles.value.map((f) => f.problem)).toEqual([null, 'unsupported'])
    await nhap.submitFilePath()
    expect(previewFileMock).not.toHaveBeenCalled()

    nhap.aimImportFile('/tmp/x.pdf')
    await nhap.removeAimedImportFile()
    expect(nhap.importFilesVerdict.value).toBe('text')
  })

  it('chọn tệp qua hộp thoại NỐI vào danh sách, rồi tới `submitFilePath()` với đủ tệp', async () => {
    const nhap = await freshModule()
    await fireDrop(['/tmp/dropped-a.txt'])
    pickFilesMock.mockResolvedValue({
      classification: fakeClassification(['/tmp/chon/a.txt', '/tmp/chon/b.md']),
      error: null,
    })

    await nhap.chooseImportFiles()

    expect(listedPaths(nhap)).toEqual(['/tmp/dropped-a.txt', '/tmp/chon/a.txt', '/tmp/chon/b.md'])
    const all = listedPaths(nhap)
    previewFileMock.mockResolvedValue(batchFor(all))
    await nhap.submitFilePath()
    expect(previewFileMock).toHaveBeenCalledWith(all, 'auto', null, null)
  })

  it('hộp thoại chọn tệp là nguồn đầu tiên: dùng nguyên phân loại Rust trả về', async () => {
    const nhap = await freshModule()
    pickFilesMock.mockResolvedValue({ classification: fakeClassification(['/tmp/chon/a.csv']), error: null })

    await nhap.chooseImportFiles()

    expect(nhap.importFilesVerdict.value).toBe('bilingual')
  })

  it('huỷ hộp thoại chọn tệp giữ nguyên danh sách đang có, không báo lỗi', async () => {
    const nhap = await freshModule()
    await fireDrop(['/tmp/dropped-a.txt'])
    pickFilesMock.mockResolvedValue({ classification: null, error: null })

    await nhap.chooseImportFiles()

    expect(listedPaths(nhap)).toEqual(['/tmp/dropped-a.txt'])
    expect(nhap.lastError.value).toBeNull()
  })

  it('hộp thoại chọn tệp trượt ⇒ lỗi lên màn hình, danh sách không đổi', async () => {
    const nhap = await freshModule()
    const error = { code: 'library.import_path_invalid', message_key: 'err.unknown', params: {}, retryable: false }
    pickFilesMock.mockResolvedValue({ classification: null, error })

    await nhap.chooseImportFiles()

    expect(nhap.lastError.value).toEqual(error)
    expect(listedPaths(nhap)).toEqual([])
  })

  it('danh sách rỗng ⇒ `submitFilePath()` no-op (0 lời gọi IPC)', async () => {
    const nhap = await freshModule()
    expect(listedPaths(nhap)).toEqual([])

    await nhap.submitFilePath()

    expect(previewFileMock).not.toHaveBeenCalled()
  })

  it('lượt tạo Tác phẩm THÀNH CÔNG xoá SẠCH danh sách tệp và dấu `Đọc như song ngữ`', async () => {
    const nhap = await freshModule()
    await fireDrop(['/tmp/a.txt', '/tmp/b.txt'])
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
      source_lang_mismatch: false,
    }
    nhap.finishImportSubmission(created, null)

    expect(listedPaths(nhap)).toEqual([])
  })

  it('thả khi màn nhập đang đóng: danh sách Tệp, tab và lỗi giữ nguyên', async () => {
    const nhap = await freshModule()
    const overlay = await import('../../src/modes/libraryImportOverlayState')
    overlay.closeLibraryImportOverlay()
    nhap.selectImportTab('urls')

    await fireDrop(['/tmp/a.txt', '/tmp/b.txt'])

    expect(listedPaths(nhap)).toEqual([])
    expect(nhap.importActiveTab.value).toBe('urls')
    expect(nhap.lastError.value).toBeNull()

    overlay.openLibraryImportOverlay()
    expect(listedPaths(nhap)).toEqual([])
  })

  it('thả khi màn nhập đang mở: tệp vào danh sách Tệp', async () => {
    const nhap = await freshModule()
    nhap.selectImportTab('urls')

    await fireDrop(['/tmp/a.txt', '/tmp/b.txt'])

    expect(listedPaths(nhap)).toEqual(['/tmp/a.txt', '/tmp/b.txt'])
    expect(nhap.importActiveTab.value).toBe('files')
  })

  it('bỏ qua phần tử thả rỗng một cách phòng thủ, không panic', async () => {
    const nhap = await freshModule()

    await fireDrop(['', '/tmp/that.txt', ''])

    expect(listedPaths(nhap)).toEqual(['/tmp/that.txt'])
  })

  it('sự kiện kéo-vào mang số tệp, kéo-ra xoá nó', async () => {
    const nhap = await freshModule()
    const enter = listenHandlers.get('aura://file-drag-enter')
    const leave = listenHandlers.get('aura://file-drag-leave')
    if (enter === undefined || leave === undefined) throw new Error('bo nghe keo chua duoc gan day')

    enter(3)
    expect(nhap.isDragOver.value).toBe(true)
    expect(nhap.dragFileCount.value).toBe(3)

    leave(undefined)
    expect(nhap.isDragOver.value).toBe(false)
    expect(nhap.dragFileCount.value).toBe(0)
  })
})
