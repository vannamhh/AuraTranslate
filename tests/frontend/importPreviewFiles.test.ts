/**
 * Nhánh N TỆP của lớp phủ **Xem trước lượt nhập** — Story 6.6b, FR14 mở rộng.
 *
 * ⚠️ Khuôn `importPreviewUrls.test.ts` (N-nguồn gần nhất) — `config/project.ts` là biên IPC,
 * giả lập bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * 🔴 **Khác đường URL ở đúng MỘT điểm mà các ca dưới đây khoá tường minh**: một mục hỏng
 * KHOÁ TOÀN BỘ bốn tầng (không chỉ khoá nút xác nhận) — `encoding_preview: null` là điều
 * kiện ĐỦ (§Decisions spec 6.6b), khác `PipelineShape::Chapters` (URL) nơi vị từ XEM bỏ qua
 * mục hỏng để vẫn dựng bốn tầng cho các mục OK còn lại.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue` SAU
 * — `freshState()` tự `mockReset()` mọi mock.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterOriginWire, FileImportItemWire, ImportEncodingPreview } from '../../src/config/project'
import type { IpcError } from '../../src/i18n'

const ORIGIN_STUB: ChapterOriginWire = {
  author: null,
  site_name: null,
  url: null,
  published_at: null,
  author_confirmed: false,
  site_name_confirmed: false,
  url_confirmed: false,
  published_at_confirmed: false,
}

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
  }
})

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  const state = await import('../../src/importPreviewState')
  return { state }
}

async function freshOverlay() {
  const { state } = await freshState()
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

function fileItem(path: string, ok: boolean, error: IpcError | null = null): FileImportItemWire {
  return { path, ok, error: ok ? null : (error ?? sampleError(path)) }
}

function sampleError(path: string): IpcError {
  return { code: 'io.read_failed', message_key: 'err.io.read_failed', params: { path }, retryable: false }
}

/** Xem trước bảng mã tối giản, với năm ứng viên KHÁC NHỊCH (đủ để đổi ứng viên đổi được
 * `selected_encoding`) — mỗi ứng viên mang `chapters` riêng để `importPreviewSelectedChapters`
 * đọc đúng CHÍNH ứng viên đang chọn. */
function candidateChapters(chapterCount: number, needsReviewCount: number) {
  return {
    chapter_count: chapterCount,
    chapters: Array.from({ length: chapterCount }, (_, i) => ({
      ord: i + 1,
      title: null,
      length: 10,
      cleanup_match_count: 0,
      joined_line_count_in_chapter: null,
      needs_review: i < needsReviewCount,
      review_causes: i < needsReviewCount ? ['short_length' as const] : [],
      origin: ORIGIN_STUB,
      source_file: `tep-${i + 1}.txt`,
    })),
    broken_item_count: 0,
    needs_review_count: needsReviewCount,
    clean_count: chapterCount - needsReviewCount,
    any_signal_participated: needsReviewCount > 0,
  }
}

function fiveCandidatePreview(chapterCount: number, needsReviewCount = 0): ImportEncodingPreview {
  return {
    confidence: 'low',
    selected_encoding: 'GBK',
    candidates: [
      { label: 'UTF-8', encoding: 'UTF-8', preview: null, normalized: null, cleanup: null, chapters: null, blocks: null },
      {
        label: 'GB18030',
        encoding: 'gb18030',
        preview: 'a',
        normalized: { text: 'a', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: null,
        chapters: candidateChapters(chapterCount, needsReviewCount),
        blocks: null,
      },
      {
        label: 'GBK',
        encoding: 'GBK',
        preview: 'a',
        normalized: { text: 'a', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: null,
        chapters: candidateChapters(chapterCount, needsReviewCount),
        blocks: null,
      },
      {
        label: 'Big5',
        encoding: 'Big5',
        preview: 'a',
        normalized: { text: 'a', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: null,
        chapters: candidateChapters(chapterCount, needsReviewCount),
        blocks: null,
      },
      {
        label: 'UTF-16',
        encoding: 'UTF-16LE',
        preview: 'a',
        normalized: { text: 'a', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: null,
        chapters: candidateChapters(chapterCount, needsReviewCount),
        blocks: null,
      },
    ],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

function batchAllOk(paths: string[]) {
  return { items: paths.map((p) => fileItem(p, true)), encoding_preview: fiveCandidatePreview(paths.length) }
}

function batchWithOneBroken(paths: string[], brokenIndex: number) {
  return {
    items: paths.map((p, i) => fileItem(p, i !== brokenIndex)),
    // §Decisions spec 6.6b — MỘT mục hỏng khoá TOÀN BỘ, không dựng xem trước cho phần còn lại
    // (khác URL) — `encoding_preview: null` LÀ điều kiện đủ.
    encoding_preview: null,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewState — openImportPreviewFromFile giữ đúng thứ tự N tệp (Story 6.6b)', () => {
  it('items khớp ĐÚNG thứ tự N tệp, cùng nội dung batch trả về', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.md']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })

    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)

    expect(state.importPreviewFileItems.value.map((it) => it.path)).toEqual(paths)
    expect(state.importPreviewFileItems.value.every((it) => it.ok)).toBe(true)
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')
    expect(previewFileMock).toHaveBeenCalledWith(paths, 'en', null, null)
  })

  it('N = 1 đi qua ĐÚNG cùng envelope batch (§Always: "one shape to reason about")', async () => {
    const { state } = await freshState()
    previewFileMock.mockResolvedValue({ batch: batchAllOk(['/tmp/mot.txt']), error: null })

    await state.openImportPreviewFromFile('Ten', 'en', '', ['/tmp/mot.txt'], null)

    expect(state.importPreviewFileItems.value).toEqual([{ path: '/tmp/mot.txt', ok: true, error: null }])
    expect(state.importPreview.value).not.toBeNull()
  })
})

describe('importPreviewState — một mục hỏng khoá TOÀN BỘ bốn tầng (khác URL)', () => {
  it('còn MỘT mục hỏng ⇒ importPreview null, importPreviewCanConfirm false, NHƯNG items vẫn hiện đủ N', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt']
    previewFileMock.mockResolvedValue({ batch: batchWithOneBroken(paths, 1), error: null })

    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)

    expect(state.importPreviewFileItems.value.length).toBe(3)
    expect(state.importPreviewFileItems.value[1]?.ok).toBe(false)
    expect(state.importPreview.value).toBeNull()
    expect(state.importPreviewCanConfirm.value).toBe(false)
    expect(state.importPreviewStatus.value).toBe('loaded') // KHÔNG 'error'/'ipc_unavailable' — đây là trạng thái BÌNH THƯỜNG, có lý do
  })
})

describe('importPreviewState — bỏ một tệp hỏng: đúng MỘT vòng IPC, mở khoá xác nhận', () => {
  it('removeImportPreviewFileItem gọi lại ĐÚNG một lần previewImportEncodingFromFile, với danh sách đã bỏ mục hỏng', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt']
    previewFileMock.mockResolvedValue({ batch: batchWithOneBroken(paths, 1), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    expect(state.importPreviewCanConfirm.value).toBe(false)
    previewFileMock.mockClear()

    const remaining = [paths[0]!, paths[2]!]
    previewFileMock.mockResolvedValue({ batch: batchAllOk(remaining), error: null })
    await state.removeImportPreviewFileItem(1)

    expect(previewFileMock).toHaveBeenCalledTimes(1)
    expect(previewFileMock).toHaveBeenCalledWith(remaining, 'en', null, null)
    expect(state.importPreviewFileItems.value.map((it) => it.path)).toEqual(remaining)
    expect(state.importPreviewFileItems.value.every((it) => it.ok)).toBe(true)
    expect(state.importPreview.value).not.toBeNull()
    expect(state.importPreviewCanConfirm.value).toBe(true)
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // Phản biện 2026-09-16 — lượt gọi lại của removeImportPreviewFileItem TRƯỢT: pendingPaths
  // (nguồn nội bộ mà lượt gọi KẾ TIẾP đọc) không được đổi ngắn hơn trong khi items/preview
  // hiển thị vẫn còn batch CŨ — nếu không, chỉ số của lượt bỏ KẾ TIẾP lệch khỏi danh sách
  // đang hiện, và xác nhận đọc một shape đã rút ngắn mà màn hình chưa từng cho thấy.
  // ─────────────────────────────────────────────────────────────────────────────

  it('lượt bỏ TRƯỢT (previewImportEncodingFromFile trả error) không đổi danh sách đang hiện, và lượt bỏ KẾ TIẾP vẫn dùng đúng danh sách CŨ', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    previewFileMock.mockClear()

    const infraError = { code: 'io.read_failed', message_key: 'err.io.read_failed', params: {}, retryable: false }
    previewFileMock.mockResolvedValue({ batch: null, error: infraError })
    await state.removeImportPreviewFileItem(1) // gọi lại bằng [a, c] -- lượt gọi NÀY trượt

    // Hiển thị PHẢI giữ nguyên batch CŨ (3 mục, preview cũ còn sống) -- lượt gọi vừa trượt
    // không được phép làm displayed/pending lệch nhau.
    expect(state.importPreviewFileImportError.value).toEqual(infraError)
    expect(state.importPreviewFileItems.value.map((it) => it.path)).toEqual(paths)
    expect(state.importPreview.value).not.toBeNull()

    // Lượt bỏ KẾ TIẾP (bỏ mục cuối, chỉ số 2) phải gọi lại bằng danh sách CŨ trừ mục đó --
    // [a, b] -- KHÔNG phải một danh sách đã lỡ rút ngắn từ lượt trượt ở trên.
    previewFileMock.mockClear()
    previewFileMock.mockResolvedValue({ batch: batchAllOk([paths[0]!, paths[1]!]), error: null })
    await state.removeImportPreviewFileItem(2)

    expect(previewFileMock).toHaveBeenCalledTimes(1)
    expect(previewFileMock).toHaveBeenCalledWith([paths[0], paths[1]], 'en', null, null)
  })

  it('bỏ TỆP CUỐI CÙNG (paths rỗng ⇒ import_files trả EmptyFileList) không xoá batch đang hiện, và lượt bỏ lại vẫn gửi đúng tệp cuối', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    previewFileMock.mockClear()

    // Bỏ mục 1 -- thanh cong, con lai dung mot tep.
    previewFileMock.mockResolvedValue({ batch: batchAllOk([paths[0]!]), error: null })
    await state.removeImportPreviewFileItem(1)
    expect(state.importPreviewFileItems.value.map((it) => it.path)).toEqual([paths[0]])
    previewFileMock.mockClear()

    // Bỏ tep CUOI CUNG -- goi lai bang paths = [] -- mo phong import_files([]) tra
    // EmptyFileList (Rust-side, khong phai mot loi TS).
    const emptyListError = {
      code: 'import.empty_file_list',
      message_key: 'err.import.empty_file_list',
      params: {},
      retryable: false,
    }
    previewFileMock.mockResolvedValue({ batch: null, error: emptyListError })
    await state.removeImportPreviewFileItem(0)

    expect(previewFileMock).toHaveBeenCalledWith([], 'en', null, null)
    // Man hinh KHONG duoc trong rong -- batch mot-tep TRUOC lượt bỏ này vẫn phải còn nguyên,
    // và preview van con song (khong bi wipe boi mot loi ma khong ai viet gi vao no).
    expect(state.importPreviewFileImportError.value).toEqual(emptyListError)
    expect(state.importPreviewFileItems.value.map((it) => it.path)).toEqual([paths[0]])
    expect(state.importPreview.value).not.toBeNull()
    expect(state.importPreviewCanConfirm.value).toBe(true)

    // Lượt bỏ LẠI (thử lại) phải vẫn gửi đúng tệp CUỐI CÙNG còn sống ([a]), không phải [].
    previewFileMock.mockClear()
    previewFileMock.mockResolvedValue({ batch: batchAllOk([]), error: null })
    await state.removeImportPreviewFileItem(0)
    expect(previewFileMock).toHaveBeenCalledWith([], 'en', null, null)
  })
})

describe('importPreviewState — phản biện 2026-09-16: xác nhận là no-op trong lúc một lượt bỏ tệp đang bay', () => {
  it('confirmImportPreview() không gọi confirmImportWithEncoding trong khi removeImportPreviewFileItem chưa xong', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    expect(state.importPreviewCanConfirm.value).toBe(true)

    let resolveRemove!: (value: { batch: { items: unknown[]; encoding_preview: unknown }; error: null }) => void
    const removePromise = new Promise<{ batch: { items: unknown[]; encoding_preview: unknown }; error: null }>(
      (resolve) => {
        resolveRemove = resolve
      },
    )
    previewFileMock.mockReturnValueOnce(removePromise)
    const removeCall = state.removeImportPreviewFileItem(1) // lượt bỏ CÒN BAY, chưa resolve

    expect(state.importPreviewFileImportBusy.value).toBe(true)

    const confirmResult = await state.confirmImportPreview()

    expect(confirmResult).toEqual({ created: null, error: null })
    expect(confirmMock).not.toHaveBeenCalled()

    resolveRemove({ batch: batchAllOk([paths[0]!]), error: null })
    await removeCall
    expect(state.importPreviewFileImportBusy.value).toBe(false)
  })
})

describe('importPreviewState — đổi ứng viên bảng mã trên N tệp: 0 lời gọi IPC thêm', () => {
  it('selectImportPreviewCandidate đổi NGAY importPreviewSelectedChapters, không gọi previewImportEncodingFromFile lần nữa', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')

    const before = previewFileMock.mock.calls.length
    state.selectImportPreviewCandidate('Big5')

    expect(state.importPreviewSelectedEncoding.value).toBe('Big5')
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(2)
    expect(previewFileMock.mock.calls.length).toBe(before)
  })
})

describe('importPreviewState — hai con số + ⌥W hoạt động đúng trên N tệp', () => {
  it('needs_review_count/clean_count đúng cho N tệp, bộ lọc ⌥W bật/tắt được', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt', '/tmp/c.txt', '/tmp/d.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    // fiveCandidatePreview mặc định needsReviewCount = 0 -- dựng lại thủ công một ứng viên có
    // Chương cần xem để bài kiểm có ý nghĩa.
    previewFileMock.mockResolvedValue({
      batch: { items: paths.map((p) => fileItem(p, true)), encoding_preview: fiveCandidatePreview(4, 2) },
      error: null,
    })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)

    expect(state.importPreviewSelectedChapters.value?.needs_review_count).toBe(2)
    expect(state.importPreviewSelectedChapters.value?.clean_count).toBe(2)

    expect(state.importPreviewChapterFilterActive.value).toBe(false)
    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(false)
  })
})

describe('ImportPreviewOverlay.vue — nhánh TỆP dựng được không vỡ, nút xác nhận khoá đúng lúc', () => {
  it('toàn bộ mục OK: danh sách tệp hiện, tầng 1-4 hiện, tên tệp nguồn hiện trên mỗi Chương, nút xác nhận KHÔNG bị khoá', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // Danh sách mục-theo-tệp dùng lại đúng khuôn CSS của danh sách URL (`.ip-url-list`/`.ip-url-item`).
    expect(wrapper.find('.ip-url-list').exists()).toBe(true)
    expect(wrapper.findAll('.ip-url-item').length).toBe(2)
    expect(wrapper.find('.ip-tier-1').exists()).toBe(true)
    expect(wrapper.findAll('.ip-chapters-source-file').length).toBeGreaterThan(0)
    const confirmButton = wrapper.find('.ip-act-primary')
    expect(confirmButton.exists()).toBe(true)
    expect((confirmButton.element as HTMLButtonElement).disabled).toBe(false)

    wrapper.unmount()
  })

  it('còn một mục hỏng: danh sách tệp VẪN hiện đủ N, nhưng bốn tầng biến mất (rỗng có lý do) và nút xác nhận khoá', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchWithOneBroken(paths, 1), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ip-url-list').exists()).toBe(true)
    expect(wrapper.findAll('.ip-url-item').length).toBe(2)
    // Khác URL (Story 6.10a) — ĐÂY bốn tầng biến mất hoàn toàn, cùng khuôn "MỌI mục đều hỏng"
    // của đường URL (§Decisions spec 6.6b: một mục hỏng ĐÃ ĐỦ, không cần MỌI mục đều hỏng).
    expect(wrapper.find('.ip-tier-1').exists()).toBe(false)
    expect(wrapper.find('.ip-url-locked-reason').exists()).toBe(true)
    const confirmButton = wrapper.find('.ip-act-primary')
    expect((confirmButton.element as HTMLButtonElement).disabled).toBe(true)

    wrapper.unmount()
  })

  it('bấm "Bỏ" trên mục hỏng gọi removeImportPreviewFileItem đúng chỉ số', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchWithOneBroken(paths, 1), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    previewFileMock.mockClear()
    previewFileMock.mockResolvedValue({ batch: batchAllOk([paths[0]!]), error: null })

    // Cùng khuôn đường URL — nút "Bỏ" có mặt trên MỌI mục (kể cả mục OK, người dùng có thể
    // muốn bỏ một tệp lành lặn khỏi batch), không chỉ mục hỏng.
    const removeForms = wrapper.findAll('.ip-url-action-form')
    expect(removeForms.length).toBe(2)
    await removeForms[1]!.trigger('submit') // mục thứ hai (chỉ số 1) là mục HỎNG
    await wrapper.vm.$nextTick()
    await Promise.resolve()
    await wrapper.vm.$nextTick()

    expect(previewFileMock).toHaveBeenCalledTimes(1)
    expect(previewFileMock).toHaveBeenCalledWith([paths[0]], 'en', null, null)

    wrapper.unmount()
  })

  it('vòng rà đối kháng 2026-09-16, mục 9/G8: cửa sổ HỞ giữa một lượt MỞ MỚI (đã reset danh sách, `status` của lượt TRƯỚC còn "loaded") không hiện "(0)" giả', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()

    // Lượt MỞ ĐẦU — thành công, `status` lật "loaded", danh sách có 1 tệp.
    const paths1 = ['/tmp/a.txt']
    previewFileMock.mockResolvedValueOnce({ batch: batchAllOk(paths1), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths1, null)
    expect(state.importPreviewStatus.value).toBe('loaded')

    // Lượt MỞ THỨ HAI — `openImportPreviewFromFile` reset `fileImportItems` về [] ĐỒNG BỘ
    // (trước khi `await` lệnh IPC), trong khi `status` của lượt TRƯỚC vẫn còn "loaded" cho
    // tới khi lượt IPC MỚI giải quyết. Giữ Promise CHƯA giải quyết để đứng đúng giữa cửa sổ đó.
    let resolveSecond!: (v: { batch: ReturnType<typeof batchAllOk>; error: null }) => void
    const secondPromise = new Promise<{ batch: ReturnType<typeof batchAllOk>; error: null }>((resolve) => {
      resolveSecond = resolve
    })
    previewFileMock.mockReturnValueOnce(secondPromise)
    const paths2 = ['/tmp/b.txt', '/tmp/c.txt']
    const openPromise = state.openImportPreviewFromFile('Ten', 'en', '', paths2, null)

    // NGAY LÚC NÀY: `lastSubmittedFrom === 'file'` (đã set đồng bộ), `fileImportItems === []`
    // (đã reset đồng bộ), `status` VẪN "loaded" (của lượt TRƯỚC) — đúng cửa sổ hở G8 canh: chỉ
    // `lastSubmittedFrom === 'file'` còn đứng cản, đúng câu G8 "gated ... alone".
    expect(state.importPreviewFileItems.value).toEqual([])
    expect(state.importPreviewStatus.value).toBe('loaded')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const title = wrapper.find('#ip-file-list-title')
    expect(title.exists()).toBe(true)
    expect(title.text()).not.toContain('(0)')

    resolveSecond({ batch: batchAllOk(paths2), error: null })
    await openPromise
    wrapper.unmount()
  })
})

describe('importPreviewState — resetImportPreview vứt sạch state của nhánh TỆP', () => {
  it('huỷ lớp phủ xoá fileImportItems và trả lastSubmittedFrom về null', async () => {
    const { state } = await freshState()
    const paths = ['/tmp/a.txt', '/tmp/b.txt']
    previewFileMock.mockResolvedValue({ batch: batchAllOk(paths), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', paths, null)
    expect(state.importPreviewFileItems.value.length).toBe(2)

    state.cancelImportPreview()

    expect(state.importPreviewFileItems.value).toEqual([])
    expect(state.importPreviewLastSubmittedFrom.value).toBeNull()
    expect(state.importPreviewIsOpen.value).toBe(false)
  })
})
