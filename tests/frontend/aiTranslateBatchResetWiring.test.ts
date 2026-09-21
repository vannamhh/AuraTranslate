/**
 * `resetAiTranslateBatch()`/`resetAiTranslate()` — canh DÂY NỐI ở BA điểm nghẽn thật, không
 * chỉ hàm thuần. Gap-closing pass của spec `4-9-dich-theo-lo-va-huy-giua-chung.md`, mục 1 và 2
 * của audit I/O & Edge-Case Matrix (2026-09-22).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — CÙNG CÁI BẪY `AGENTS.md` (root) ĐÃ GHI CHO STORY 6.15
 * ─────────────────────────────────────────────────────────────────────────────
 * `tests/frontend/aiTranslateBatch.test.ts`'s `describe('resetAiTranslateBatch huỷ lô đang
 * bay')` chỉ gọi `batchState.resetAiTranslateBatch()` TRỰC TIẾP — canh đúng HÀM, không canh
 * DÂY NỐI. Story 6.15 đã gỡ sạch SÁU chỗ gọi thật của một hàm reset khác mà 15/15 ca vẫn xanh,
 * vì không ca nào đi qua chỗ gọi SẢN PHẨM. Bốn ca dưới đây lái qua đúng BA điểm nghẽn thật mà
 * spec 4.9 đặt tên — `modes/libraryChapters.ts::openWorkById`, `modes/libraryImport.ts`'s cụm
 * nộp form (`finishImportSubmission`), và bước ③ của `switchChapter`
 * (`panels/editorPanelState.ts:1729-1730`) — cộng MỘT ca riêng cho Quyết định 4 (AC7 spec
 * 4.9): một lượt dịch MỘT segment đang chảy cũng phải bị huỷ ở đúng bước ③ đó.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * Khuôn mock — CHÉP TỪ hai tệp chị em, không dựng lại
 * ─────────────────────────────────────────────────────────────────────────────
 * `config/library`/`config/chapter`/`config/project`/`@tauri-apps/api/event`: cùng khuôn
 * `segmentSelection.test.ts`'s cụm reset #1/#2. `config/chapter::openAdjacentChapter` +
 * `readOpenChapter`: cùng khuôn `editorChapterSwitch.test.ts`'s `chuyenGia`/`docNguyenVanGia`,
 * thu gọn (không cần sổ thứ tự — mệnh đề ở đây là "đã huỷ", không phải "đúng thứ tự").
 * `panels/editorPanelState.ts` KHÔNG bị mock — cùng lý do `aiTranslateBatch.test.ts` đã ghi:
 * dây nối thật sống trong chính module đó (`switchChapter` bước ③), một bản giả sẽ xoá luôn
 * thứ ca này cần canh.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'
import type { AiTranslateBatchEventWire, AiTranslateOutcomeWire } from '../../src/config/aitranslate'
import type { ChapterDirection, ChapterSwitchOutcome } from '../../src/config/chapter'
import type { IpcError } from '../../src/i18n'

const runSegmentMock = vi.fn()
const runBatchMock = vi.fn()
const cancelMock = vi.fn()

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: (...args: unknown[]) => runSegmentMock(...args),
  cancelAiTranslateCall: (...args: unknown[]) => cancelMock(...args),
  runAiTranslateBatchCall: (...args: unknown[]) => runBatchMock(...args),
}))

/** Chương KẾ mà lượt chuyển giả trả về — id KHÁC `FIXTURE_CHAPTER_ID` nên "đã đổi Chương" là
 * một mệnh đề đo được, không phải trùng hợp. `segment.id` không tái dùng trong CÙNG một
 * `project.db` (cùng đo đã ghi ở `segmentSelection.test.ts`), nên hai id 31/32 dưới đây không
 * chồng lên 11/12/13. */
const CHUONG_KE_ID = 99
const CHUONG_KE_SEGMENTS: readonly ChapterSegment[] = [
  { ...FIXTURE_SEGMENTS[0], id: 31 },
  { ...FIXTURE_SEGMENTS[1], id: 32 },
]

/** Chương ĐANG "mở" trong bộ giả `readOpenChapterSegments`/`readOpenChapter` — CÙNG một ô nhớ
 * cho cả hai, đúng khuôn `editorChapterSwitch.test.ts::chuongDangMo`: `openAdjacentChapter`
 * (giả) dời con trỏ NGAY tại lượt gọi, trước khi `ensureSegmentsLoaded()`/`ensureChapterLoaded()`
 * đọc lại — nếu không đồng bộ, `editorChapterId` sẽ đứng nguyên ở Chương CŨ dù lượt chuyển đã
 * "thành công" theo `openAdjacentChapter`, và ca sẽ xanh vì lý do sai. */
const chuongDangMo: { value: number } = { value: FIXTURE_CHAPTER_ID }

async function docChuongDangMoSegments(): Promise<{
  loaded: { chapter_id: number; segments: ChapterSegment[] }
  error: null
}> {
  const segments = chuongDangMo.value === FIXTURE_CHAPTER_ID ? FIXTURE_SEGMENTS : CHUONG_KE_SEGMENTS
  return { loaded: { chapter_id: chuongDangMo.value, segments: segments.map((s) => ({ ...s })) }, error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: docChuongDangMoSegments }
})

async function chuyenChuongGia(
  _direction: ChapterDirection,
): Promise<{ switched: { outcome: ChapterSwitchOutcome; chapter: { chapter_id: number; source_text: string; source_lang: string } } | null; error: IpcError | null }> {
  chuongDangMo.value = CHUONG_KE_ID
  return {
    switched: {
      outcome: 'moved',
      chapter: { chapter_id: CHUONG_KE_ID, source_text: 'Chuong ke.', source_lang: 'zh' },
    },
    error: null,
  }
}

async function docNguyenVanGia(): Promise<{
  chapter: { chapter_id: number; source_text: string; source_lang: string } | null
  error: IpcError | null
}> {
  return { chapter: { chapter_id: chuongDangMo.value, source_text: 'Chuong ke.', source_lang: 'zh' }, error: null }
}

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return {
    ...actual,
    openAdjacentChapter: chuyenChuongGia,
    readOpenChapter: docNguyenVanGia,
    listChapters: async () => ({ chapters: [], error: null }),
  }
})

/** Tác phẩm mà `openWorkById('w-moi')` mở — cùng hằng số `segmentSelection.test.ts`. */
const TAC_PHAM_MOI = { meta: { work_id: 'w-moi' }, folder: '/tmp/w-moi.atproj', chapter_id: 1 }
/** Tác phẩm mà lượt NHẬP giả lập tạo ra — cùng hằng số `segmentSelection.test.ts`. */
const TAC_PHAM_MOI_NHAP = { work_id: 'w-moi-nhap', name: 'Tac pham nhap moi', chapter_count: 1 }

vi.mock('../../src/config/library', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/library')>()
  return { ...actual, openWork: async () => ({ opened: TAC_PHAM_MOI, error: null }) }
})

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: async () => ({
      preview: { confidence: 'self_declared', selected_encoding: 'UTF-8', candidates: [] },
      error: null,
    }),
    confirmImportWithEncoding: async () => ({ created: TAC_PHAM_MOI_NHAP, error: null }),
  }
})

// `libraryImport.ts` gọi `listen` ở cấp module; ngoài Tauri nó không phân giải được.
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

type RunResult = { value: AiTranslateOutcomeWire | null; error: IpcError | null }
type OnToken = (text: string) => void
type OnEvent = (event: AiTranslateBatchEventWire) => void

/** Cùng khuôn `aiTranslate.test.ts::pendingRun()` — lượt dịch MỘT segment không trả lời ngay. */
function pendingSegmentRun(): { token: OnToken; settle: (result: RunResult) => void } {
  let capturedOnToken: OnToken = () => {
    throw new Error('onToken chưa được gán — runSegmentMock chưa gọi tới')
  }
  let resolve: (result: RunResult) => void = () => {
    throw new Error('resolve chưa được gán — runSegmentMock chưa gọi tới')
  }
  runSegmentMock.mockImplementation((_segmentId: number, _promptSetName: string | null, onToken: OnToken) => {
    capturedOnToken = onToken
    return new Promise<RunResult>((res) => {
      resolve = res
    })
  })
  return { token: (text: string) => capturedOnToken(text), settle: (result: RunResult) => resolve(result) }
}

/** Cùng khuôn `aiTranslateBatch.test.ts::pendingBatchRun()` — lô không trả lời ngay. */
function pendingBatchRun(): { emit: OnEvent; settle: (result: RunResult) => void } {
  let capturedOnEvent: OnEvent = () => {
    throw new Error('onEvent chưa được gán — runBatchMock chưa gọi tới')
  }
  let resolve: (result: RunResult) => void = () => {
    throw new Error('resolve chưa được gán — runBatchMock chưa gọi tới')
  }
  runBatchMock.mockImplementation((_ids: number[], _promptSetName: string | null, onEvent: OnEvent) => {
    capturedOnEvent = onEvent
    return new Promise<RunResult>((res) => {
      resolve = res
    })
  })
  return { emit: (event: AiTranslateBatchEventWire) => capturedOnEvent(event), settle: (result: RunResult) => resolve(result) }
}

/** Nhường một vòng microtask cho lượt chuyển mà `goToNextChapter` phát bằng `void` — cùng
 * khuôn `editorChapterSwitch.test.ts::settle()`. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0))

beforeEach(() => {
  vi.resetModules()
  runSegmentMock.mockReset()
  runBatchMock.mockReset()
  cancelMock.mockReset()
  chuongDangMo.value = FIXTURE_CHAPTER_ID
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Mục 1a — `modes/libraryChapters.ts::openWorkById`
// ═══════════════════════════════════════════════════════════════════════════════════

describe('cụm reset #1 — `modes/libraryChapters.ts::openWorkById` huỷ MỘT LÔ đang bay', () => {
  it('🔴 gỡ `resetAiTranslateBatch()` khỏi `openWorkById` sẽ làm ca này ĐỎ: lô của Tác phẩm CŨ không bị huỷ khi mở Tác phẩm khác', async () => {
    const batchState = await import('../../src/aiTranslateBatchState')
    const chuong = await import('../../src/modes/libraryChapters')
    pendingBatchRun()

    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')

    cancelMock.mockClear()
    await chuong.openWorkById('w-moi')

    // Tiền đề của ca: lượt mở phải THÀNH CÔNG — cùng bẫy hai tệp chị em đã ghi.
    expect(chuong.libraryOpenWorkError.value).toBeNull()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(batchState.aiTranslateBatchStateValue.value).toBe('not_configured')
    expect(batchState.aiTranslateBatchRows.value).toEqual([])
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Mục 1b — `modes/libraryImport.ts`'s cụm nộp form (`finishImportSubmission`)
// ═══════════════════════════════════════════════════════════════════════════════════

describe('cụm reset #2 — `modes/libraryImport.ts::finishImportSubmission` huỷ MỘT LÔ đang bay', () => {
  it('🔴 gỡ `resetAiTranslateBatch()` khỏi `finishImportSubmission` sẽ làm ca này ĐỎ: lô của Tác phẩm CŨ không bị huỷ khi nhập Tác phẩm mới', async () => {
    const batchState = await import('../../src/aiTranslateBatchState')
    const nhap = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')
    pendingBatchRun()

    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')

    cancelMock.mockClear()
    nhap.pastedText.value = 'Cau nguyen van cua Tac pham moi.'
    await nhap.submitPastedText()
    expect(preview.importPreviewIsOpen.value).toBe(true)

    const result = await preview.confirmImportPreview()
    nhap.finishImportSubmission(result.created, result.error)

    // Tiền đề của ca: lượt nhập phải THÀNH CÔNG — cùng bẫy hai tệp chị em đã ghi.
    expect(nhap.createdWork.value).not.toBeNull()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(batchState.aiTranslateBatchStateValue.value).toBe('not_configured')
    expect(batchState.aiTranslateBatchRows.value).toEqual([])
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Mục 1c + Quyết định 4 (AC7 spec 4.9) — bước ③ của `switchChapter`
// (`panels/editorPanelState.ts:1729-1730`), MỘT lô VÀ MỘT lượt đơn
// ═══════════════════════════════════════════════════════════════════════════════════

describe('bước ③ `switchChapter` huỷ lượt AI đang bay của Chương VỪA RỜI (Quyết định 4, spec 4.9)', () => {
  it('🔴 gỡ `resetAiTranslateBatch()` khỏi bước ③ sẽ làm ca này ĐỎ: MỘT LÔ đang chảy không bị huỷ khi đổi Chương trong CÙNG Tác phẩm', async () => {
    const editorState = await import('../../src/panels/editorPanelState')
    const batchState = await import('../../src/aiTranslateBatchState')
    await editorState.ensureSegmentsLoaded()
    expect(editorState.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)
    pendingBatchRun()

    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')

    cancelMock.mockClear()
    editorState.goToNextChapter()
    await settle()

    // Tiền đề của ca: lượt chuyển phải THÀNH CÔNG, không thì `switchChapter` thoát sớm ở bước
    // ② và ca sẽ xanh vì một lý do sai.
    expect(editorState.editorChapterId.value).toBe(CHUONG_KE_ID)

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(batchState.aiTranslateBatchStateValue.value).toBe('not_configured')
    expect(batchState.aiTranslateBatchRows.value).toEqual([])
  })

  it('🔴 Quyết định 4 (AC7 spec 4.9) — gỡ `resetAiTranslate()` khỏi bước ③ sẽ làm ca này ĐỎ: MỘT LƯỢT ĐƠN đang chảy không bị huỷ và kết quả không bị vứt khi đổi Chương trong CÙNG Tác phẩm', async () => {
    const editorState = await import('../../src/panels/editorPanelState')
    const state = await import('../../src/aiTranslateState')
    await editorState.ensureSegmentsLoaded()
    expect(editorState.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)
    const fake = pendingSegmentRun()

    void state.runAiTranslate(null, 11)
    await flushPromises()
    expect(state.aiTranslateStateValue.value).toBe('generating')
    fake.token('Mot phan ket qua dang chay')
    await flushPromises()
    expect(state.aiTranslateAccumulatedText.value).toBe('Mot phan ket qua dang chay')

    cancelMock.mockClear()
    editorState.goToNextChapter()
    await settle()

    expect(editorState.editorChapterId.value).toBe(CHUONG_KE_ID)

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.aiTranslateStateValue.value).toBe('not_configured')
    // "…and its result cleared" (AC7 nguyên văn) — không chỉ trạng thái đổi, văn bản đã nhận
    // cũng phải bị vứt, không sống sót sang Chương người dùng vừa rời tới.
    expect(state.aiTranslateAccumulatedText.value).toBe('')
    expect(state.aiTranslateRunSegmentId.value).toBeNull()
  })
})
