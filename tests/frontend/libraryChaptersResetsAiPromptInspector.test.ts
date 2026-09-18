/**
 * `modes/libraryChapters.ts::openWorkById` — **đường MỞ LẠI một `.atproj` đã có cũng dọn bản
 * ghi prompt cuối cùng đã lắp.** Story 4.7 loop 2, finding P4 — nửa thứ hai.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI: MỘT PHÉP GỠ GỘP ĐÃ SUÝT CHỨNG NHẬN MỘT CHỖ NỐI KHÔNG AI CANH
 * ─────────────────────────────────────────────────────────────────────────────
 * `libraryImportResetsAiPromptInspector.test.ts` canh đúng MỘT trong hai điểm nghẽn —
 * `libraryImport.ts::finishImportSubmission` — và tự ghi giới hạn đó ra. Ở lượt nghiệm thu
 * finding P4, gỡ CẢ HAI lời gọi `resetAiPromptInspector()` cùng lúc cho **một** ca đỏ, trông
 * như cả hai đường đều có người canh. Đo lại bằng cách gỡ RIÊNG lời gọi ở `libraryChapters.ts`:
 * **toàn bộ 86 tệp / 1260 ca vẫn xanh** — đường này không có gì canh cả. Đó đúng bài học
 * `AGENTS.md` ghi cho ma trận assert: một phép gỡ gộp không nói gì về từng hàng, phải gỡ N lần.
 *
 * ⚠️ **Hình dạng hỏng nếu ca này đỏ, và nó KHÔNG NÉM LỖI NÀO:** `chapter.id`/`segment.id` là
 * `AUTOINCREMENT` **cục bộ trong từng `project.db`** (đo được ở bàn đo e2e của chính
 * `libraryChapters.ts`: hai Tác phẩm khác nhau **đều** có `chapter_id = 1`). Một bản ghi prompt
 * của Tác phẩm CŨ sống sót qua lượt mở Tác phẩm MỚI vì thế **không** bị `aiPromptRecordIsStale`
 * bắt — hai id trùng nhau một cách ngẫu nhiên — nên dòng tóm tắt LUÔN HIỆN của
 * `AiTranslationPanel.vue` mô tả Tác phẩm vừa rời như thể nó là Tác phẩm đang mở.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

/** Bản ghi prompt giả lập của Tác phẩm CŨ — đúng hình dạng `AssembledPromptWire` để
 * `isAssembledPromptWire` (tầng adapter) chấp nhận nó. Cùng hằng số với tệp chị em. */
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

/** Tác phẩm MỚI mà `open_work` trả về — chỉ cần qua được `isOpenedWork`. */
const TAC_PHAM_MOI = { meta: { work_id: 'w-moi' }, folder: '/tmp/w-moi.atproj', chapter_id: 1 }

vi.mock('../../src/config/library', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/library')>()
  return { ...actual, openWork: async () => ({ opened: TAC_PHAM_MOI, error: null }) }
})

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, listChapters: async () => ({ chapters: [], error: null }) }
})

// Lượt flush là TIỀN ĐỀ của `openWorkById`, không phải thứ ca này nghiệm thu: một `'clean'`
// giả lập giữ hàm đi tới nhánh vứt state. Mock TỪNG HÀM qua `importOriginal` — `resetEditorPanel`
// và `ensureSegmentsLoaded` thật vẫn chạy.
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return {
    ...actual,
    flushEditorBeforeDiscreteWrite: async () => 'clean' as const,
    flushChapterPositionNow: async () => {},
  }
})

// Mock ĐÚNG BIÊN adapter — cùng khuôn tệp chị em — để `assembleCurrentAiPrompt` ghi được một
// bản ghi THẬT vào `aiPromptInspectorState.ts::record` mà không cần cầu Tauri.
vi.mock('../../src/config/aiprompt', () => ({
  aiPromptAssemble: async () => ({ value: BAN_GHI_CU, error: null }),
  aiPromptReadRecord: async () => ({ value: null, error: null }),
}))

beforeEach(() => {
  vi.resetModules()
})

describe('🔴 mở lại một Tác phẩm đã có ⇒ bản ghi prompt cuối cùng đã lắp KHÔNG sống sót', () => {
  it('openWorkById() gọi resetAiPromptInspector() — bản ghi của Tác phẩm CŨ bị vứt', async () => {
    const aiPrompt = await import('../../src/aiPromptInspectorState')
    const chuong = await import('../../src/modes/libraryChapters')

    // Dựng đúng trạng thái nguy hiểm: một bản ghi ĐÃ LẮP của Tác phẩm CŨ, lớp phủ ĐANG MỞ.
    await aiPrompt.assembleCurrentAiPrompt('Happy', 1)
    aiPrompt.openAiPromptInspector()
    expect(aiPrompt.aiPromptRecord.value).toEqual(BAN_GHI_CU)
    expect(aiPrompt.aiPromptInspectorIsOpen.value).toBe(true)

    await chuong.openWorkById('w-moi')

    // Tiền đề của ca: lượt mở phải THÀNH CÔNG, không thì hàm thoát sớm và ca sẽ xanh vì một
    // lý do sai (đúng bẫy `libraryImportResetsAiPromptInspector.test.ts` đã ghi cho chính nó).
    expect(chuong.libraryOpenWorkError.value).toBeNull()
    expect(chuong.libraryOpenWorkNotice.value).toBeNull()

    expect(aiPrompt.aiPromptRecord.value).toBeNull()
    expect(aiPrompt.aiPromptInspectorIsOpen.value).toBe(false)
    expect(aiPrompt.aiPromptAssembleBusy.value).toBe(false)
    expect(aiPrompt.aiPromptAssembleError.value).toBeNull()
    expect(aiPrompt.aiPromptReadError.value).toBeNull()
  })
})
