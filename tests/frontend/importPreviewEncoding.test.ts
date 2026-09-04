/**
 * State của lớp phủ **Xem trước lượt nhập — bảng mã** (`src/importPreviewState.ts`) — Story
 * 6.3, FR126.
 *
 * ⚠️ Cùng khuôn `glossaryImportPreview.test.ts`: `config/project.ts` là biên IPC, giả lập
 * bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue`
 * SAU — `freshState()` tự `mockReset()` mọi mock.
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

/** Nạp lại module mỗi ca — state của lớp phủ là module-level singleton. */
async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  return import('../../src/importPreviewState')
}

/** Rust LUÔN cấp đủ năm ô khi có byte để dò, kể cả tin cậy CAO (I/O Matrix "Tệp thuần ASCII":
 * "năm bản dựng cho CÙNG một chuỗi") — chỉ khác `lowConfidencePreview()` ở việc cả năm ô
 * (trừ UTF-16) đọc ra CÙNG một chuỗi. */
function highConfidencePreview(): ImportEncodingPreview {
  return {
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [
      { label: 'UTF-8', encoding: 'UTF-8', preview: 'plain ascii' },
      { label: 'GB18030', encoding: 'gb18030', preview: 'plain ascii' },
      { label: 'GBK', encoding: 'GBK', preview: 'plain ascii' },
      { label: 'Big5', encoding: 'Big5', preview: 'plain ascii' },
      { label: 'UTF-16', encoding: 'UTF-16LE', preview: '灱慩⁮獡楣' },
    ],
  }
}

function selfDeclaredPreview(): ImportEncodingPreview {
  return { confidence: 'self_declared', selected_encoding: 'UTF-8', candidates: [] }
}

function lowConfidencePreview(): ImportEncodingPreview {
  return {
    confidence: 'low',
    selected_encoding: 'GBK',
    candidates: [
      { label: 'UTF-8', encoding: 'UTF-8', preview: null },
      { label: 'GB18030', encoding: 'gb18030', preview: '萧炎在东临' },
      { label: 'GBK', encoding: 'GBK', preview: '萧炎在东临' },
      { label: 'Big5', encoding: 'Big5', preview: '達鍁誗' },
      { label: 'UTF-16', encoding: 'UTF-16LE', preview: '扡摣捥' },
    ],
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewState — dải MỞ (hiển thị) đúng khi và chỉ khi tin cậy thấp', () => {
  it('tin cậy CAO ⇒ Rust vẫn cấp đủ năm ô (I/O Matrix ASCII), nhưng dải KHÔNG mở mặc định', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })

    await state.openImportPreviewFromText('Ten', 'en', '', 'plain ascii')

    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(state.importPreviewStatus.value).toBe('loaded')
    expect(state.importPreview.value?.confidence).toBe('high')
    expect(state.importPreview.value?.candidates).toHaveLength(5)
    // "Dải MỞ" ở đây nói về DẢI HIỂN THỊ, không phải mảng dữ liệu — Rust không giấu gì cả.
    expect(state.importPreviewStripIsOpen.value).toBe(false)
  })

  it('tin cậy THẤP ⇒ dải MỞ với đúng năm ô, theo thứ tự FR126', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })

    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    expect(state.importPreviewStatus.value).toBe('loaded')
    expect(state.importPreview.value?.confidence).toBe('low')
    expect(state.importPreviewStripIsOpen.value).toBe(true)
    expect(state.importPreview.value?.candidates.map((c) => c.label)).toEqual([
      'UTF-8',
      'GB18030',
      'GBK',
      'Big5',
      'UTF-16',
    ])
  })

  it('nguồn tự khai (BOM/dán tay) ⇒ `self_declared`, dải RỖNG THẬT (không có gì để mà mở)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: selfDeclaredPreview(), error: null })

    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay')

    expect(state.importPreview.value?.confidence).toBe('self_declared')
    expect(state.importPreview.value?.candidates).toHaveLength(0)
    expect(state.importPreviewStripIsOpen.value).toBe(false)
  })
})

describe('importPreviewState — buộc mở dải bằng `E` (`openImportPreviewCandidatePicker`)', () => {
  it('tin cậy CAO ⇒ `E` mở được dải (dữ liệu đã có sẵn, không gọi IPC)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'plain ascii')
    expect(state.importPreviewStripIsOpen.value).toBe(false)

    const ipcCallsBefore = previewTextMock.mock.calls.length
    state.openImportPreviewCandidatePicker()

    expect(state.importPreviewStripIsOpen.value).toBe(true)
    expect(previewTextMock.mock.calls.length).toBe(ipcCallsBefore)
  })

  it('nguồn tự khai ⇒ `E` là no-op (0 ứng viên, không có gì để mở)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: selfDeclaredPreview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay')

    state.openImportPreviewCandidatePicker()

    expect(state.importPreviewStripIsOpen.value).toBe(false)
  })

  it('một lượt mở PREVIEW MỚI xoá cờ buộc-mở của lượt trước', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'plain ascii')
    state.openImportPreviewCandidatePicker()
    expect(state.importPreviewStripIsOpen.value).toBe(true)

    previewFileMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten2', 'en', '', '/tmp/x.txt')

    expect(state.importPreviewStripIsOpen.value).toBe(false)
  })
})

describe('importPreviewState — chọn một ứng viên khác', () => {
  it('cập nhật ứng viên đang chọn và bản dựng thật hiện NGAY, không đòi mở lại màn xem trước', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    // Mặc định là ứng viên Rust đã chọn (GBK).
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')
    expect(state.importPreviewSelectedCandidate.value?.preview).toBe('萧炎在东临')

    // Chọn Big5 — KHÔNG một lời gọi IPC nào thêm (bản dựng đã có sẵn từ lượt mở, cùng cửa
    // sổ bằng chứng mà Rust đã dò — §Always spec 6.3: "thấy kết quả đổi NGAY LẬP TỨC").
    const ipcCallsBefore = previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length
    state.selectImportPreviewCandidate('Big5')

    expect(state.importPreviewSelectedEncoding.value).toBe('Big5')
    expect(state.importPreviewSelectedCandidate.value?.preview).toBe('達鍁誗')
    expect(state.importPreviewSelectedCandidate.value?.preview).not.toBe('萧炎在东临')
    const ipcCallsAfter = previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length
    expect(ipcCallsAfter).toBe(ipcCallsBefore)
  })

  it('chọn một `encoding` không có trong dải là vô hiệu — giữ nguyên lựa chọn cũ', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    state.selectImportPreviewCandidate('shift_jis-khong-co-trong-dai')

    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')
  })

  it('🔴 vòng rà đối kháng 2, mục 11 — đổi lựa chọn TRONG LÚC một lượt xác nhận đang BAY là vô hiệu', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')

    let resolveConfirm!: (value: { created: unknown; error: null }) => void
    confirmMock.mockReturnValue(
      new Promise((resolve) => {
        resolveConfirm = resolve
      }),
    )
    const confirmPromise = state.confirmImportPreview()
    expect(state.importPreviewConfirming.value).toBe(true)

    // Đổi lựa chọn TRONG LÚC lượt trên còn đang bay — phải là no-op.
    state.selectImportPreviewCandidate('Big5')
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')

    resolveConfirm({ created: { meta: { work_id: 'w1', name: 'Ten' }, folder: '/tmp/Ten.atproj' }, error: null })
    await confirmPromise
  })
})

describe('importPreviewState — hai tầng rỗng nói ra lý do và tên chủ', () => {
  it('tầng 2 (ranh giới nội dung) rỗng vì Story 6.9', async () => {
    const state = await freshState()
    expect(state.importPreviewEmptyReasonForTier(2)).toBe('story_6_9')
  })

  it('tầng 3 (luật làm sạch) rỗng vì Story 6.5', async () => {
    const state = await freshState()
    expect(state.importPreviewEmptyReasonForTier(3)).toBe('story_6_5')
  })
})

describe('importPreviewState — xác nhận', () => {
  it('thành công đóng lớp phủ và trả về Tác phẩm vừa tạo', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'plain ascii')

    const created = { meta: { work_id: 'w1', name: 'Ten' }, folder: '/tmp/Ten.atproj' }
    confirmMock.mockResolvedValue({ created, error: null })

    const result = await state.confirmImportPreview()

    expect(confirmMock).toHaveBeenCalledWith('Ten', 'en', '', 'UTF-8')
    expect(result.created).toEqual(created)
    expect(state.importPreviewIsOpen.value).toBe(false)
    // 🔴 Vòng rà đối kháng 2, mục 8 — thành công KHÔNG chỉ đóng lớp phủ, còn phải DỌN state
    // đã tải (`preview`/lựa chọn/tên-ngôn ngữ-thể loại đang chờ) NGAY, không để nó sống tới
    // lượt `openImportPreview*` kế tiếp mới bị ghi đè.
    expect(state.importPreview.value).toBeNull()
    expect(state.importPreviewSelectedEncoding.value).toBeNull()
  })

  it('trượt (bảng mã sai) giữ lớp phủ MỞ và hiện lỗi — không đóng vòng nộp form', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')

    const err = { code: 'import.undecodable_bytes', message_key: 'err.import.undecodable_bytes', params: {}, retryable: false }
    confirmMock.mockResolvedValue({ created: null, error: err })

    const result = await state.confirmImportPreview()

    expect(result.error).toEqual(err)
    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(state.importPreviewConfirmError.value).toEqual(err)
  })

  it('`importPreviewLastSubmittedFrom` SỐNG SÓT qua một lượt xác nhận TRƯỢT — điều kiện để lượt xác nhận lại (thành công) còn biết xoá ô nào', async () => {
    // 🔴 Đối chứng đỏ cho một lỗi THẬT đã bị bắt (phản biện Ice, 2026-09-04): bản đặt
    // `lastSubmittedFrom` ở `libraryImport.ts` xoá ô này VÔ ĐIỀU KIỆN ở cuối
    // `finishImportSubmission`, kể cả khi `created === null` (nhánh TRƯỢT) — nên một lượt
    // xác nhận lại (chọn đúng bảng mã) sau đó không còn biết xoá `pastedText`/`filePath`.
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')

    // Lượt 1 — chọn nhầm, TRƯỢT.
    const err = { code: 'import.undecodable_bytes', message_key: 'err.import.undecodable_bytes', params: {}, retryable: false }
    confirmMock.mockResolvedValue({ created: null, error: err })
    await state.confirmImportPreview()

    // Vé "đã nộp từ đâu" PHẢI còn nguyên sau lượt trượt — đây là dòng đỏ nếu bị xoá sớm.
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')

    // Lượt 2 — chọn lại đúng, THÀNH CÔNG.
    const created = { meta: { work_id: 'w1', name: 'Ten' }, folder: '/tmp/Ten.atproj' }
    confirmMock.mockResolvedValue({ created, error: null })
    const result = await state.confirmImportPreview()

    expect(result.created).toEqual(created)
    // Vé vẫn còn NGAY SAU thành công — `finishImportSubmission` (ở `libraryImport.ts`) đọc
    // nó SAU khi `confirmImportPreview()` trả về, không phải bên trong nó.
    expect(state.importPreviewLastSubmittedFrom.value).toBe('file')
  })
})

describe('importPreviewState — huỷ (defect #5, vòng rà 1)', () => {
  it('huỷ xoá SẠCH `preview`/lựa chọn — xác nhận SAU đó là no-op, KHÔNG gọi IPC', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: lowConfidencePreview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'zh', '', '/tmp/gbk.txt')
    expect(state.importPreviewIsOpen.value).toBe(true)

    state.cancelImportPreview()

    expect(state.importPreviewIsOpen.value).toBe(false)
    expect(state.importPreview.value).toBeNull()
    expect(state.importPreviewSelectedEncoding.value).toBeNull()

    // "Huỷ rồi xác nhận ⇒ 0 Tác phẩm được tạo" — không một lời gọi Rust nào xảy ra, vì
    // `confirmImportPreview()` chặn NGAY DÒNG ĐẦU khi `preview.value === null`.
    const result = await state.confirmImportPreview()
    expect(confirmMock).not.toHaveBeenCalled()
    expect(result.created).toBeNull()
    expect(result.error).toBeNull()
  })

  it('🔴 vòng rà đối kháng 2, mục 4 — huỷ TRONG LÚC một lượt xác nhận đang BAY là no-op, không tạo Tác phẩm mồ côi', async () => {
    // Kịch bản đo được: Rust đã CHẠY XONG create_work + replace_open_work (kết quả THÀNH
    // CÔNG đang trên đường về), nhưng người dùng bấm Esc/đóng đúng lúc đó. Bản lỗi: huỷ
    // bump `sequence`, lượt confirm về sau đó rơi vào nhánh "lượt cũ" (`{created: null,
    // error: null}`), main.ts không gọi `finishImportSubmission`, panel không reset —
    // trong khi `.atproj` đã nằm thật trên đĩa và `OpenWorkState` đã trỏ vào nó.
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'plain ascii')

    // Lượt xác nhận BẮT ĐẦU nhưng CHƯA VỀ — mô phỏng "Rust đang chạy create_work".
    let resolveConfirm!: (value: { created: unknown; error: null }) => void
    confirmMock.mockReturnValue(
      new Promise((resolve) => {
        resolveConfirm = resolve
      }),
    )
    const confirmPromise = state.confirmImportPreview()
    expect(state.importPreviewConfirming.value).toBe(true)

    // Bấm huỷ TRONG LÚC lượt trên còn đang bay.
    state.cancelImportPreview()

    // 🔴 Đối chứng: huỷ PHẢI là no-op khi đang confirm — lớp phủ VẪN MỞ, `preview` VẪN CÒN.
    // Bản lỗi (không guard) sẽ làm hai khẳng định này ĐỎ (resetImportPreview đã chạy).
    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(state.importPreview.value).not.toBeNull()

    // Rust "trả lời" — lượt xác nhận PHẢI giao đúng kết quả THẬT của nó (không rơi vào
    // nhánh "lượt cũ", vì `sequence` không hề đổi — huỷ đã bị chặn TRƯỚC khi bump nó).
    const created = { meta: { work_id: 'w1', name: 'Ten' }, folder: '/tmp/Ten.atproj' }
    resolveConfirm({ created, error: null })
    const result = await confirmPromise

    expect(result.created).toEqual(created)
  })

  it('`resetImportPreview()` xoá mọi ô nhớ cấp module (check:panel-refs Kiểm A)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: highConfidencePreview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    state.openImportPreviewCandidatePicker()
    expect(state.importPreviewStripIsOpen.value).toBe(true)

    state.resetImportPreview()

    expect(state.importPreviewIsOpen.value).toBe(false)
    expect(state.importPreviewStatus.value).toBe('unknown')
    expect(state.importPreviewLoadError.value).toBeNull()
    expect(state.importPreview.value).toBeNull()
    expect(state.importPreviewSelectedEncoding.value).toBeNull()
    expect(state.importPreviewConfirming.value).toBe(false)
    expect(state.importPreviewConfirmError.value).toBeNull()
    expect(state.importPreviewOpening.value).toBe(false)
    expect(state.importPreviewStripForcedOpen.value).toBe(false)
  })
})

describe('importPreviewState — ba trạng thái tải, khuôn glossaryImportState', () => {
  it('không có cầu IPC (`preview: null`, `error: null`) ⇒ MỞ lớp phủ và NÓI RA', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(state.importPreviewStatus.value).toBe('ipc_unavailable')
  })

  it('lượt mở trượt THẬT ⇒ MỞ lớp phủ và hiện IpcError, KHÁC `ipc_unavailable`', async () => {
    const state = await freshState()
    const err = { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
    previewTextMock.mockResolvedValue({ preview: null, error: err })

    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    expect(state.importPreviewStatus.value).toBe('error')
    expect(state.importPreviewLoadError.value).toEqual(err)
  })
})
