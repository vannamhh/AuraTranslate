/**
 * `modes/libraryImport.ts::finishSubmit` — **đường đổi Tác phẩm dọn state lịch sử phiên bản.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI, VÀ NÓ ĐÓNG MỆNH ĐỀ NÀO KHÔNG AI CANH
 * ─────────────────────────────────────────────────────────────────────────────
 * Lượt rà ba tầng 2026-08-19 trên Story 2.12 bắt được: `resetSegmentHistory()` được viết,
 * được ghi là AC5 đã đóng, và **không một lời gọi nào trong `src/**`**. Chỗ gọi duy nhất
 * trong kho là bảng fixture `e2e/support/panelReset.mjs` — tức **bộ đo dọn state mà sản phẩm
 * thì không**.
 *
 * ⚠️ **Và cổng `check:panel-refs` KHÔNG BAO GIỜ bắt được lớp đó.** Nó hỏi *"tệp có một hàm
 * `reset*` gán ô nhớ này không"*; nó **không** hỏi *"ai gọi hàm đó"*. Một hàm reset đúng đắn
 * mà không có chỗ gọi đi qua cổng ấy **sạch sẽ** — nên mệnh đề *"lượt đổi Tác phẩm THẬT SỰ
 * gọi nó"* phải có chủ ở một đường khác, và bốn đường nghiệm thu (AC25) chỉ cho một đường:
 * đây, `tests/frontend/**`.
 *
 * 🔴 Hình dạng hỏng nếu ca này đỏ, và nó **không ném lỗi nào**: hộp thoại lịch sử còn mở qua
 * một lượt đổi Tác phẩm thì `segmentId` còn trỏ vào một `segment.id` của Tác phẩm CŨ.
 * `restoreVersion()` đọc thẳng ô ấy *(`segmentHistoryState.ts:274`)* và **không** đối chiếu
 * Tác phẩm đang mở. Hai kho đánh số `segment.id` **ĐỘC LẬP**, nên id đó tồn tại thật ở kho
 * mới và trỏ vào một câu khác hẳn ⇒ một lượt khôi phục FR101 ghi bản dịch của câu này đè lên
 * câu kia. *"Chỗ hỏng là VĨNH VIỄN."*
 *
 * ⚠️ **Giới hạn thật, ghi ra thay vì để người sau tưởng đã được canh:** tệp này canh vế *"có
 * gọi không"* trên đường `submitPastedText`. Nó **không** canh nhánh `submitFilePath` — hai
 * nhánh đi qua **cùng** một `finishImportSubmission`, và đó là điểm nghẽn mà chú thích tại
 * chỗ gọi là *"điểm nghẽn DUY NHẤT mà cả hai nhánh nhập đều đi qua"*. Một mệnh đề, một đường.
 *
 * 🔵 **SỬA 2026-09-04 (Story 6.3, FR126) — `submitPastedText()` không còn tự tạo Tác phẩm.**
 * Trước story này, gọi `submitPastedText()` MỘT LẦN là đủ để `finishSubmit` chạy (nhập
 * thẳng qua `createWorkFromText`). Từ story này, nộp form chỉ MỞ màn xem trước bảng mã
 * (`importPreviewState.ts::openImportPreviewFromText`) — việc TẠO Tác phẩm dời sang
 * `confirmImportPreview()`, và `finishImportSubmission` (đổi tên từ `finishSubmit`, nay
 * EXPORT) chỉ chạy SAU lượt xác nhận đó. Ca dưới đây lái qua ĐÚNG hai bước sản phẩm, khớp
 * đúng cách `main.ts` nối `import.preview.confirm` (`confirmImportPreview()` rồi
 * `finishImportSubmission(created, error)` — xem `src/main.ts`).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { readFixture } from './support/segmentFixture'

/** Tác phẩm mà lượt nhập giả lập trả về — chỉ cần khác `null` để `finishImportSubmission` đi
 * vào nhánh dọn. */
const TAC_PHAM_MOI = { work_id: 'w-moi', name: 'Tac pham B', chapter_count: 1 }

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: async () => ({
      preview: { confidence: 'self_declared', selected_encoding: 'UTF-8', candidates: [] },
      error: null,
    }),
    confirmImportWithEncoding: async () => ({ created: TAC_PHAM_MOI, error: null }),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture }
})

// `libraryImport` `import` `listen` ở cấp module; ngoài Tauri nó không phân giải được.
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

beforeEach(() => {
  vi.resetModules()
})

describe('🔴 đổi Tác phẩm ⇒ state lịch sử phiên bản KHÔNG sống sót', () => {
  it('nộp rồi xác nhận gọi `resetSegmentHistory()` — hộp thoại đóng và `segmentId` về `null`', async () => {
    const history = await import('../../src/panels/segmentHistoryState')
    const editor = await import('../../src/panels/editorPanelState')
    const nhap = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')

    // Dựng đúng trạng thái nguy hiểm: một câu đã nhắm, hộp thoại lịch sử ĐANG MỞ.
    await editor.ensureSegmentsLoaded()
    editor.setEditorCaret(0)
    history.openSegmentHistory()
    expect(history.historyIsOpen.value).toBe(true)
    expect(history.historySegmentId.value).not.toBeNull()

    // Đúng đường sản phẩm, BƯỚC MỘT — nộp form chỉ MỞ màn xem trước, không tạo gì cả.
    nhap.pastedText.value = 'Cau nguyen van cua Tac pham moi.'
    await nhap.submitPastedText()
    expect(preview.importPreviewIsOpen.value).toBe(true)
    expect(nhap.createdWork.value).toBeNull()

    // Đúng đường sản phẩm, BƯỚC HAI — xác nhận, rồi đóng vòng nộp form đúng khuôn
    // `main.ts`'s handler của `import.preview.confirm`.
    const result = await preview.confirmImportPreview()
    nhap.finishImportSubmission(result.created, result.error)

    // Tiền đề của ca: lượt nhập phải THÀNH CÔNG, không thì `finishImportSubmission` không
    // vào nhánh dọn và ca sẽ xanh vì một lý do sai.
    expect(nhap.createdWork.value).not.toBeNull()

    expect(history.historyIsOpen.value).toBe(false)
    expect(history.historySegmentId.value).toBeNull()
    expect(history.historyVersions.value).toEqual([])
    expect(history.historyPendingRestore.value).toBeNull()
    expect(history.historyRestoreNotice.value).toBeNull()
    expect(history.historyAimedVersionId.value).toBeNull()
  })
})
