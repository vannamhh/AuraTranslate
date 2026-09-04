/**
 * Tầng chuẩn hoá xuống dòng & khoảng trắng của lớp phủ **Xem trước lượt nhập**
 * (`src/importPreviewState.ts::importPreviewSelectedNormalized`) — Story 6.4, FR124/FR125.
 *
 * ⚠️ Khuôn `importPreviewEncoding.test.ts:18-36`: `config/project.ts` là biên IPC, giả lập
 * bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue`
 * SAU — `freshState()` tự `mockReset()` mọi mock.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { EncodingCandidateWire, ImportEncodingPreview, NormalizedPreviewWire } from '../../src/config/project'

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string, sourceLang: string) => previewTextMock(text, sourceLang),
    previewImportEncodingFromFile: (path: string, sourceLang: string) => previewFileMock(path, sourceLang),
    confirmImportWithEncoding: (name: string, sourceLang: string, genre: string, encoding: string) =>
      confirmMock(name, sourceLang, genre, encoding),
  }
})

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  return import('../../src/importPreviewState')
}

function normalized(
  text: string,
  joinedLines: number,
  blankLinesRemoved: number,
  windowTruncated: boolean,
): NormalizedPreviewWire {
  return { text, joined_lines: joinedLines, blank_lines_removed: blankLinesRemoved, window_truncated: windowTruncated }
}

function candidate(
  label: string,
  encoding: string,
  preview: string | null,
  n: NormalizedPreviewWire | null,
): EncodingCandidateWire {
  return { label, encoding, preview, normalized: n }
}

/** Dải năm ô, tin cậy thấp — mỗi ô mang một bản chuẩn hoá KHÁC NHAU rõ rệt để phép chọn
 * ứng viên khác đổi được kết quả đọc ra. Ô `UTF-8` cố ý "không ra chữ" (`normalized: null`)
 * để canh nhánh "undecodable" của tầng mới. */
function fivecandidatePreview(): ImportEncodingPreview {
  return {
    confidence: 'low',
    selected_encoding: 'GBK',
    candidates: [
      candidate('UTF-8', 'UTF-8', null, null),
      candidate('GB18030', 'gb18030', '萧炎', normalized('萧炎đã nối', 2, 1, false)),
      candidate('GBK', 'GBK', '萧炎', normalized('萧炎bản GBK', 3, 0, false)),
      candidate('Big5', 'Big5', '達鍁', normalized('達鍁bản Big5', 0, 0, true)),
      candidate('UTF-16', 'UTF-16LE', '扡摣', normalized('扡摣bản UTF-16', 1, 2, false)),
    ],
    // candidates khong rong -- doc .normalized cua ung vien dang chon, khong doc truong nay.
    self_declared_normalized: null,
  }
}

/** Vá vòng rà 1, mục 1 — nhánh TỰ KHAI (0 ứng viên, đường DÁN VĂN BẢN TAY). */
function selfDeclaredPreview(n: NormalizedPreviewWire): ImportEncodingPreview {
  return {
    confidence: 'self_declared',
    selected_encoding: 'UTF-8',
    candidates: [],
    self_declared_normalized: n,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewState — importPreviewSelectedNormalized hiện đúng bản dựng của ứng viên đang chọn', () => {
  it('mặc định là bản chuẩn hoá của ứng viên Rust đã chọn (GBK)', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: fivecandidatePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    expect(state.importPreviewSelectedNormalized.value).toEqual(normalized('萧炎bản GBK', 3, 0, false))
  })

  it('đổi ứng viên đổi NGAY bản dựng + hai số đếm, 0 lời gọi IPC thêm', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: fivecandidatePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    const ipcCallsBefore =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length

    state.selectImportPreviewCandidate('Big5')

    expect(state.importPreviewSelectedNormalized.value).toEqual(normalized('達鍁bản Big5', 0, 0, true))
    expect(state.importPreviewSelectedNormalized.value?.joined_lines).toBe(0)
    expect(state.importPreviewSelectedNormalized.value?.blank_lines_removed).toBe(0)

    const ipcCallsAfter =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length
    expect(ipcCallsAfter).toBe(ipcCallsBefore)
  })

  it('phạm vi cửa sổ (`window_truncated`) đổi theo đúng ứng viên đang chọn', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: fivecandidatePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    // GBK (mặc định): cửa sổ KHÔNG bị cắt.
    expect(state.importPreviewSelectedNormalized.value?.window_truncated).toBe(false)

    // Big5: cửa sổ CÓ bị cắt — tầng hiển thị phải nói ra phạm vi bằng chữ.
    state.selectImportPreviewCandidate('Big5')
    expect(state.importPreviewSelectedNormalized.value?.window_truncated).toBe(true)

    // UTF-16: quay lại KHÔNG bị cắt.
    state.selectImportPreviewCandidate('UTF-16LE')
    expect(state.importPreviewSelectedNormalized.value?.window_truncated).toBe(false)
  })

  it('ứng viên "không ra chữ" (`normalized: null`) đọc ra `null`, không panic/undefined ngầm', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: fivecandidatePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    state.selectImportPreviewCandidate('UTF-8')

    expect(state.importPreviewSelectedCandidate.value?.preview).toBeNull()
    expect(state.importPreviewSelectedNormalized.value).toBeNull()
  })

  it('chưa có `preview` nào (chưa mở màn xem trước) ⇒ `null`, không ném lỗi', async () => {
    const state = await freshState()
    expect(state.importPreviewSelectedNormalized.value).toBeNull()
  })

  // 🔴 Vá vòng rà 1, mục 1 — bản trước của ca này khẳng định `null`, đúng CÁI LỖI mà spec
  // 6.4 phát hiện ở vòng rà 1: đường DÁN VĂN BẢN TAY (0 ứng viên) rơi vào tầng rỗng dù luật
  // gộp dòng vẫn chạy và AD-4 đóng băng kết quả. Cơ chế đã sửa: đọc
  // `preview.self_declared_normalized` khi không có ứng viên nào, KHÔNG còn `null`.
  it('nguồn tự khai (dán tay, 0 ứng viên) ⇒ đọc `self_declared_normalized`, KHÔNG rơi về null', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: selfDeclaredPreview(normalized('van ban da dan roi chuan hoa', 2, 1, false)),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay')

    expect(state.importPreviewSelectedCandidate.value).toBeNull()
    expect(state.importPreviewSelectedNormalized.value).toEqual(
      normalized('van ban da dan roi chuan hoa', 2, 1, false),
    )
  })

  it('nguồn tự khai — 0 lời gọi IPC nào thêm khi đọc lại computed (cùng lý lẽ đổi ứng viên)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: selfDeclaredPreview(normalized('x', 0, 0, false)),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const ipcCallsBefore =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length
    // Đọc lại nhiều lần — computed, không phải một lượt gọi.
    void state.importPreviewSelectedNormalized.value
    void state.importPreviewSelectedNormalized.value
    const ipcCallsAfter =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length

    expect(ipcCallsAfter).toBe(ipcCallsBefore)
  })
})
