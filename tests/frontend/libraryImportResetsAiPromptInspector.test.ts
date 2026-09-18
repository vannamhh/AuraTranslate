/**
 * `modes/libraryImport.ts::finishImportSubmission` — **đường đổi Tác phẩm dọn bản ghi prompt
 * cuối cùng đã lắp.** Story 4.7 loop 2, finding P4 (task 6-4).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI, VÀ NÓ ĐÓNG MỆNH ĐỀ NÀO KHÔNG AI CANH
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng khuôn `libraryImportResetsSegmentHistory.test.ts` (Story 2.12), một bậc khác:
 * `resetAiPromptInspector()` từng có **0 chỗ gọi sản phẩm** (bắt được ở lượt rà build,
 * Story 4.7 loop 2, finding P4 — cùng lớp lỗi finding V1 đã bắt ở tầng Rust một bậc thấp
 * hơn). `commands/aiprompt.rs::clear_last_assembled_prompt_on_work_close`'s doc-comment nói rõ
 * lý do: `segment_id`/`chapter_id` là khoá hàng của CHÍNH `project.db` sắp rời, và hai
 * `.atproj` khác nhau đều đánh số lại từ 1 — bản ghi của Tác phẩm CŨ sống sót qua một lượt đổi
 * Tác phẩm đọc nhầm thành "đúng câu/Chương đang mở" của Tác phẩm MỚI.
 *
 * ⚠️ **Hình dạng hỏng nếu ca này đỏ, và nó KHÔNG NÉM LỖI NÀO:** dòng tóm tắt LUÔN HIỆN của
 * `AiTranslationPanel.vue` ("Đã chèn N thuật ngữ Glossary") tiếp tục mô tả Tác phẩm VỪA RỜI —
 * không phải một lỗi báo ra, một CON SỐ SAI trông như bình thường (đúng hạng lỗi
 * `AGENTS.md::Known pitfalls` đặt lên hàng đầu).
 *
 * ⚠️ **Giới hạn thật, ghi ra thay vì để người sau tưởng đã được canh:** tệp này canh đúng MỘT
 * trong hai điểm nghẽn thật — `libraryImport.ts::finishImportSubmission` (đường Tác phẩm MỚI
 * NHẬP). `libraryChapters.ts::openWorkById` (đường mở lại một `.atproj` đã có) nhận CÙNG một
 * lời gọi `resetAiPromptInspector()`, cùng lý do, nhưng không có ca riêng ở đây — cùng giới
 * hạn `libraryImportResetsSegmentHistory.test.ts` đã tự ghi cho nhánh `submitFilePath` của
 * chính nó.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { readFixture } from './support/segmentFixture'

/** Tác phẩm mà lượt nhập giả lập trả về — chỉ cần khác `null` để `finishImportSubmission` đi
 * vào nhánh dọn. */
const TAC_PHAM_MOI = { work_id: 'w-moi', name: 'Tac pham B', chapter_count: 1 }

/** Bản ghi prompt giả lập của Tác phẩm CŨ — chỉ cần đúng hình dạng `AssembledPromptWire` để
 * `isAssembledPromptWire` (tầng adapter) chấp nhận nó. */
const BAN_GHI_CU = {
  prompt: 'Terms: dragon → rong',
  segment_id: 1,
  chapter_id: 10,
  prompt_set_name: 'Happy',
  prompt_set_tier: 'global' as const,
  ledger: {
    glossary: { kind: 'not_asked' as const, injected: null, suppressed_by_pending_overlap: null },
    tm: { kind: 'not_built_yet' as const, similar_segments: null },
    unknown_markers: [],
    source_segment_missing: false,
    pieces: [{ kind: 'authored' as const, text: 'Terms: dragon → rong' }],
  },
}

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

// Mock ĐÚNG BIÊN adapter — cùng khuôn `aiPromptInspector.test.ts` — để `assembleCurrentAiPrompt`
// có thể ghi một bản ghi THẬT vào `aiPromptInspectorState.ts::record` mà không cần cầu Tauri.
vi.mock('../../src/config/aiprompt', () => ({
  aiPromptAssemble: async () => ({ value: BAN_GHI_CU, error: null }),
  aiPromptReadRecord: async () => ({ value: null, error: null }),
}))

beforeEach(() => {
  vi.resetModules()
})

describe('🔴 đổi Tác phẩm (nhập mới) ⇒ bản ghi prompt cuối cùng đã lắp KHÔNG sống sót', () => {
  it('nộp rồi xác nhận gọi resetAiPromptInspector() — bản ghi của Tác phẩm CŨ bị vứt', async () => {
    const aiPrompt = await import('../../src/aiPromptInspectorState')
    const nhap = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')

    // Dựng đúng trạng thái nguy hiểm: một bản ghi prompt ĐÃ LẮP của Tác phẩm CŨ, lớp phủ Xem
    // prompt ĐANG MỞ.
    await aiPrompt.assembleCurrentAiPrompt('Happy', 1)
    aiPrompt.openAiPromptInspector()
    expect(aiPrompt.aiPromptRecord.value).toEqual(BAN_GHI_CU)
    expect(aiPrompt.aiPromptInspectorIsOpen.value).toBe(true)

    // Đúng đường sản phẩm, BƯỚC MỘT — nộp form chỉ MỞ màn xem trước, không tạo gì cả.
    nhap.pastedText.value = 'Cau nguyen van cua Tac pham moi.'
    await nhap.submitPastedText()
    expect(preview.importPreviewIsOpen.value).toBe(true)
    expect(nhap.createdWork.value).toBeNull()

    // Đúng đường sản phẩm, BƯỚC HAI — xác nhận, rồi đóng vòng nộp form đúng khuôn
    // `main.ts`'s handler của `import.preview.confirm`.
    const result = await preview.confirmImportPreview()
    nhap.finishImportSubmission(result.created, result.error)

    // Tiền đề của ca: lượt nhập phải THÀNH CÔNG, không thì `finishImportSubmission` không vào
    // nhánh dọn và ca sẽ xanh vì một lý do sai.
    expect(nhap.createdWork.value).not.toBeNull()

    expect(aiPrompt.aiPromptRecord.value).toBeNull()
    expect(aiPrompt.aiPromptInspectorIsOpen.value).toBe(false)
    expect(aiPrompt.aiPromptAssembleBusy.value).toBe(false)
    expect(aiPrompt.aiPromptAssembleError.value).toBeNull()
    expect(aiPrompt.aiPromptReadError.value).toBeNull()
  })
})
