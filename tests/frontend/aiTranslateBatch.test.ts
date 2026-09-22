/**
 * Panel `Đề xuất AI` — dịch theo LÔ với tiến độ và huỷ giữa chừng, Story 4.9 (FR73, AD-22,
 * Decision 1/2). Phase 4b's own file — `ai_translate_contract.rs`/`config_invariants.rs`
 * (Rust) và `entries_eligible_for_injection`/`deferred-work.md` (đo lường) là việc của các
 * agent khác trong cùng vòng Phase 4; tệp này chỉ canh phía webview của LÔ.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CÙNG KHUÔN `aiTranslate.test.ts::freshPanel()` — mount THẬT, dispatch qua registry THẬT,
 * mock ĐÚNG BIÊN adapter (`config/aitranslate`), KHÔNG mock `aiTranslateBatchState.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `deps.runAiTranslateBatch`/`cancelAiTranslate` trỏ THẲNG vào hàm SẢN PHẨM của
 * `aiTranslateBatchState.ts`/`aiTranslateState.ts` — đúng khuôn `main.ts::boot()`'s wiring
 * (chép nguyên văn cổng loại-trừ-lẫn-nhau mà `main.ts` cài, xem doc-comment ở đó).
 *
 * ⚠️ **KHÔNG thay `panels/editorPanelState.ts` bằng một module giả TOÀN BỘ** — khác
 * `aiTranslate.test.ts`. Vùng chọn (`segmentSelectionState.ts`) đọc THẬT `editorSegments`/
 * `editorCaretSegmentId` của module đó để neo/dời (Phase 1), nên một `editorPanelState` giả
 * TOÀN BỘ sẽ làm `segmentSelectionState.ts` không còn gì để tính. `config/segment.ts
 * ::readOpenChapterSegments` bị giả ở BIÊN IPC thay — cùng khuôn `segmentSelection.test.ts`,
 * tái dùng ĐÚNG fixture `FIXTURE_SEGMENTS` (id 11/12/13) của `support/segmentFixture.ts` —
 * không một fixture thứ hai chép tay.
 *
 * 🔵 **THÊM (rà xét lại) — MỘT override HẸP qua `importOriginal`, không phải một module giả.**
 * `promoteAiTranslationToEditor` bị thay bằng `promoteMock` (cùng khuôn `flushEditorBeforeDiscreteWrite`
 * của `segmentSelection.test.ts`'s hai cụm reset: `{ ...actual, promoteAiTranslationToEditor: … }`),
 * mọi export khác của module vẫn là bản THẬT. Cần vì `main.ts`'s handler `promoteAiTranslate`
 * (nhánh dự phòng LÔ, đọc caret rồi tra `aiTranslateBatchTextForSegment`) không có ca nào canh
 * trước bản sửa này — chép nguyên văn logic đó vào `installCommands` dưới đây, KHÔNG một stub
 * im lặng.
 *
 * `config/aitranslate.ts` là biên IPC bị `vi.mock` — mock `runAiTranslateBatchCall` trả một
 * `Promise` mà test tự điều khiển thời điểm trả lời VÀ tự giữ tham chiếu `onEvent` mà
 * `aiTranslateBatchState.ts::runAiTranslateBatch` truyền vào, mô phỏng ĐÚNG những gì một
 * `Channel` lô thật làm: gọi `onEvent` nhiều lần (mỗi khung của MỌI câu trong lô, không chỉ
 * một) TRƯỚC khi lời gọi `invoke` trả lời — "fake Channel" của lô, cùng vai `pendingRun()` đã
 * làm cho lượt đơn.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { Component } from 'vue'
import { FIXTURE_SEGMENTS, FIXTURE_CHAPTER_ID } from './support/segmentFixture'
import type { AiTranslateBatchEventWire, AiTranslateOutcomeWire } from '../../src/config/aitranslate'
import type { CommandDeps } from '../../src/commands'
import type { IpcError } from '../../src/i18n'

const runSegmentMock = vi.fn()
const runBatchMock = vi.fn()
const cancelMock = vi.fn()
const promoteMock = vi.fn()

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: (...args: unknown[]) => runSegmentMock(...args),
  cancelAiTranslateCall: (...args: unknown[]) => cancelMock(...args),
  runAiTranslateBatchCall: (...args: unknown[]) => runBatchMock(...args),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: async () => ({
      loaded: { chapter_id: FIXTURE_CHAPTER_ID, segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })) },
      error: null,
    }),
  }
})

// Override HẸP — xem khối doc-comment đầu tệp §"THÊM (rà xét lại)". Mọi export khác của
// `editorPanelState.ts` (`editorSegments`, `editorCaretSegmentId`, `resetEditorPanel`,
// `ensureSegmentsLoaded`, …) vẫn là bản THẬT qua `importOriginal`.
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, promoteAiTranslationToEditor: (...args: unknown[]) => promoteMock(...args) }
})

type RunResult = { value: AiTranslateOutcomeWire | null; error: IpcError | null }
type OnEvent = (event: AiTranslateBatchEventWire) => void
type OnToken = (text: string) => void

/**
 * 🔴 SỬA 2026-09-22 (Story 4.10, Phase 3) — trước bản sửa này hằng số này tên
 * `BATCH_STOPPED_ON_12`, mang `err.ai_translate.batch_stopped` (khoá GỘP RIÊNG cho đường batch
 * mà Quyết định 2 spec 4.10 đã gỡ — batch dùng CHUNG sáu khoá họ với lượt dịch MỘT segment, chỉ
 * mang thêm `segment_id`, xem `commands/aitranslate.rs::batch_stopped_error`). Cùng phát hiện
 * đã ghi ở `aiTranslate.test.ts::STREAM_ENDED_ERROR`: đo được (`npx vitest run` TRƯỚC khi xoá
 * hai dòng orphan khỏi `vi.json`, giữ NGUYÊN fixture) cả hai tệp vẫn XANH, vì dòng cũ so
 * `alert.text()` với `i18n.tError(BATCH_STOPPED_ON_12)` — gọi lại CHÍNH hàm đang được canh trên
 * CÙNG payload, nên hai vế luôn bằng nhau kể cả khi khoá đã biến mất. Đổi khoá THẬT
 * (`provider_unreachable`, retryable) VÀ đổi vế so sánh sang chuỗi LITERAL chép từ `vi.json`.
 *
 * 🔴 Khác MỘT điều nữa: khoá HỌ nguyên nhân (Quyết định 2) KHÔNG còn nội suy `{segment_id}` vào
 * CÂU hiển thị nữa — "the batch row list built in 4.9 carries WHICH row failed. No per-path
 * duplicate of the family." Câu cũ (`err.ai_translate.batch_stopped`) từng nêu tên "câu số 12"
 * NGAY TRONG câu; câu mới không nêu số câu nào -- danh tính câu 12 giờ chỉ đọc được qua HÀNG lô
 * (`data-ai-translate-batch-row-status`), không qua `.ai-translate-alert`. Ca dưới đây đã sửa
 * theo đúng thay đổi này (xem tiêu đề ca).
 */
const PROVIDER_UNREACHABLE_ON_12: IpcError = {
  code: 'ai_translate.provider_unreachable',
  message_key: 'err.ai_translate.provider_unreachable',
  params: { segment_id: '12' },
  retryable: true,
}

/** Cùng lý do trên: một lỗi KHÔNG retryable thật (Quyết định 2 spec 4.10), dùng cho cụm "nút
 * Thử lại LÔ chỉ hiện khi retryable". */
const PROVIDER_REFUSED_ON_13: IpcError = {
  code: 'ai_translate.provider_refused',
  message_key: 'err.ai_translate.provider_refused',
  params: { segment_id: '13', status: '500' },
  retryable: false,
}

/**
 * Dựng một lượt `runAiTranslateBatchCall` KHÔNG trả lời ngay — trả lại một hàm `emit(event)`
 * gọi thẳng `onEvent` mà `aiTranslateBatchState.ts::runAiTranslateBatch` đã đăng ký, cộng một
 * hàm `settle(result)` giả lập Rust trả lời sau cùng. "Fake Channel" của lô, cùng khuôn
 * `aiTranslate.test.ts::pendingRun()`.
 */
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
  return {
    emit: (event: AiTranslateBatchEventWire) => capturedOnEvent(event),
    settle: (result: RunResult) => resolve(result),
  }
}

/**
 * Cùng khuôn `aiTranslate.test.ts::pendingRun()` — một lượt ĐƠN (`runAiTranslateSegment`)
 * không trả lời ngay. Chỉ cần cho cụm "Cổng loại trừ lẫn nhau" và cụm "promote — nhánh LÔ"
 * dưới đây; ba `describe` gốc của tệp này (tiến độ/huỷ/lỗi) không đụng tới lượt ĐƠN.
 */
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
  return {
    token: (text: string) => capturedOnToken(text),
    settle: (result: RunResult) => resolve(result),
  }
}

/**
 * Cùng khuôn `aiTranslate.test.ts::freshPanel()`: `vi.resetModules()` rồi `import` lại mọi
 * thứ, để mỗi ca có một `aiTranslateBatchState.ts`/`segmentSelectionState.ts` NGUYÊN VẸN.
 *
 * `commands.installCommands({...})` chép NGUYÊN VĂN cách `main.ts::boot()` nối các dep của Story
 * 4.9 — xem `main.ts` dòng đăng ký `runAiTranslateBatch`/`cancelAiTranslate` (cổng loại-trừ-lẫn-nhau).
 */
async function freshPanel() {
  vi.resetModules()
  runSegmentMock.mockReset()
  runBatchMock.mockReset()
  cancelMock.mockReset()
  promoteMock.mockReset()

  const commands = await import('../../src/commands')
  const editorState = await import('../../src/panels/editorPanelState')
  const selectionState = await import('../../src/panels/segmentSelectionState')
  const state = await import('../../src/aiTranslateState')
  const batchState = await import('../../src/aiTranslateBatchState')
  const i18n = await import('../../src/i18n')
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

  editorState.resetEditorPanel()
  selectionState.resetSegmentSelection()
  await editorState.ensureSegmentsLoaded()

  commands.installCommands({
    runAiTranslate: () => {
      if (batchState.aiTranslateBatchStateValue.value === 'generating') return
      void state.runAiTranslate(null, editorState.editorCaretSegmentId.value)
    },
    // Cổng ở ĐÂY, không ở hai module state — cùng lý lẽ `main.ts`: mỗi hàm `cancel*` tự gác
    // ĐÚNG module của nó, nên gọi CẢ HAI vô hại.
    cancelAiTranslate: () => {
      state.cancelAiTranslate()
      batchState.cancelAiTranslateBatch()
    },
    // Chép NGUYÊN VĂN `main.ts`'s handler thật của `ai.translate.promote` (Decision 2/3 spec
    // 4.9 — thử NHÁNH ĐƠN trước, trượt thì thử NHÁNH LÔ đọc kết quả của câu đang có TIÊU
    // ĐIỂM). `aiTranslate.test.ts` chỉ canh nhánh ĐƠN (không một tham chiếu LÔ nào trong tệp
    // đó) — nhánh LÔ chỉ có ca ở ĐÂY.
    promoteAiTranslate: () => {
      const s = state.aiTranslateStateValue.value
      const segmentId = state.aiTranslateRunSegmentId.value
      const text = state.aiTranslateAccumulatedText.value
      if ((s === 'done' || s === 'cancelled') && segmentId !== null && text !== '') {
        void editorState.promoteAiTranslationToEditor(segmentId, text)
        return
      }

      const caretId = editorState.editorCaretSegmentId.value
      const batchText =
        caretId === null ? null : batchState.aiTranslateBatchTextForSegment(batchState.aiTranslateBatchRows.value, caretId)
      if (batchText !== null && caretId !== null) {
        void editorState.promoteAiTranslationToEditor(caretId, batchText)
        return
      }

      console.warn(
        `[test] khong dua sang Editor: chua co ket qua hop le (state=${s}, segmentId=${String(segmentId)}, caretId=${String(caretId)})`,
      )
    },
    runAiTranslateBatch: () => {
      if (state.aiTranslateStateValue.value === 'generating') return
      void batchState.runAiTranslateBatch(null, selectionState.segmentSelectionIds.value)
    },
    // Story 4.10, Phase 3 — chép NGUYÊN VĂN hai lớp gác thật của `main.ts`'s handler thật
    // `retryAiTranslate`/`retryAiTranslateBatch` (Phase 2): cổng loại-trừ-lẫn-nhau ĐƠN/LÔ
    // trước (cùng khuôn `runAiTranslate`/`runAiTranslateBatch` ngay trên), rồi lớp gác riêng
    // của retry -- phải THẬT SỰ có một lỗi `retryable` đang chờ (§Always spec 4.10: "retryable
    // grants only the right to SHOW a button").
    retryAiTranslate: () => {
      if (batchState.aiTranslateBatchStateValue.value === 'generating') return
      if (state.aiTranslateStateValue.value === 'generating') return
      const err = state.aiTranslateError.value
      if (state.aiTranslateStateValue.value !== 'error' || err === null || err.retryable !== true) return
      const segmentId = state.aiTranslateRunSegmentId.value
      if (segmentId === null) return
      void state.runAiTranslate(null, segmentId)
    },
    // `aiTranslateBatchRetryIds` (Task 6 spec 4.10) đọc hàng `error` cộng mọi hàng `pending`
    // của LÔ vừa lỗi, giữ ĐÚNG thứ tự tài liệu -- không một câu `done`/`skipped`/`cancelled`
    // nào bị gọi lại.
    retryAiTranslateBatch: () => {
      if (state.aiTranslateStateValue.value === 'generating') return
      if (batchState.aiTranslateBatchStateValue.value === 'generating') return
      const err = batchState.aiTranslateBatchError.value
      if (batchState.aiTranslateBatchStateValue.value !== 'error' || err === null || err.retryable !== true) return
      const ids = batchState.aiTranslateBatchRetryIds(batchState.aiTranslateBatchRows.value)
      if (ids.length === 0) return
      void batchState.runAiTranslateBatch(null, ids)
    },
  } as CommandDeps)

  return { commands, editorState, selectionState, state, batchState, i18n, AiTranslationPanel }
}

function mountPanel(AiTranslationPanel: Component) {
  return mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
}

/** Chọn cả BA câu fixture (id 11/12/13, cùng khuôn AC1 spec 4.9: caret + hai lần dời). */
function selectAllThreeFixtureSegments(
  editorState: Awaited<ReturnType<typeof freshPanel>>['editorState'],
  selectionState: Awaited<ReturnType<typeof freshPanel>>['selectionState'],
): void {
  editorState.setEditorCaret(11)
  selectionState.extendSegmentSelectionDown()
  selectionState.extendSegmentSelectionDown()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ═══════════════════════════════════════════════════════════════════════════════════
// AC1/AC2 — số đếm tiến độ (done/total/remaining) + hàng ĐANG chảy
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — dispatch("ai.translate.batch_run") qua "fake Channel", số đếm tiến độ', () => {
  it('3 câu đã chọn: câu 1 xong, câu 2 đang chảy ⇒ panel đọc đúng "1/3 · còn 1", hàng đang chảy nêu tên câu 2', async () => {
    const { i18n, editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12, 13])
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-batch-selection-count]').text()).toBe(
      i18n.t('panel.ai_translation.batch_selection_count', { count: '3' }),
    )
    expect(wrapper.get('[data-ai-translate-batch-run]').attributes('disabled')).toBeUndefined()

    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    expect(runBatchMock).toHaveBeenCalledTimes(1)
    expect(runBatchMock).toHaveBeenCalledWith([11, 12, 13], null, expect.any(Function))

    fake.emit({ kind: 'token', segment_id: 11, text: 'Mot.' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Dang chay.' })
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-batch-progress]').text()).toBe(
      i18n.t('panel.ai_translation.batch_progress', { done_count: '1', total_count: '3', remaining_count: '1' }),
    )
    expect(wrapper.get('[data-ai-translate-batch-running]').text()).toBe(
      i18n.t('panel.ai_translation.batch_running', { segment_id: '12' }),
    )

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows).toHaveLength(3)
    expect(rows[0]?.attributes('data-ai-translate-batch-row-status')).toBe('done')
    expect(rows[1]?.attributes('data-ai-translate-batch-row-status')).toBe('running')
    expect(rows[2]?.attributes('data-ai-translate-batch-row-status')).toBe('pending')

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Story 4.11 — tổng token + tổng ƯỚC TÍNH của LÔ (`epics.md:3811-3813`, AC ký: "hiển thị tổng
// token và tổng ước tính của cả lô"). Bốn hàng thật: "Batch completes" × {mô hình có giá / mô
// hình không có giá} và "Batch partly without usage" × {có giá / không có giá} — trong MỘT lô
// mọi câu dùng CHUNG một mô hình (`prepare_batch_call` phân giải cấu hình một lần cho cả lô),
// nên có-giá-hay-không là tính chất của LÔ, không trộn lẫn giữa các câu trong cùng một fixture
// (đúng lý lẽ `aiTranslateBatchUsageSummary`'s doc-comment). Cùng khuôn `aiTranslate.test.ts`:
// so với một chuỗi tiếng Việt LITERAL chép từ `vi.json`.
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — dispatch("ai.translate.batch_run"), tổng token + tổng ước tính của lô (Story 4.11)', () => {
  it('mọi câu đã dịch đều báo usage, mô hình CÓ giá ⇒ dòng tổng cộng CẢ token LẪN tiền', async () => {
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Mot.' })
    fake.emit({
      kind: 'done',
      segment_id: 11,
      usage: { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30, cost_usd: 0.0003 },
    })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Hai.' })
    fake.emit({
      kind: 'done',
      segment_id: 12,
      usage: { prompt_tokens: 5, completion_tokens: 5, total_tokens: 10, cost_usd: 0.0001 },
    })
    await wrapper.vm.$nextTick()

    // Chỉ 2/3 câu đã DỊCH XONG cho tới lúc này -- dòng tổng phải đọc đúng những gì ĐÃ có, và
    // vì cả hai câu đã xong đều báo usage, đây vẫn là dòng "đầy đủ" (không phải "một phần").
    expect(wrapper.get('[data-ai-translate-batch-usage]').text()).toBe(
      'Tổng 40 token · ước tính ~0,0004 USD cho 2 câu đã dịch',
    )

    fake.emit({ kind: 'token', segment_id: 13, text: 'Ba.' })
    fake.emit({
      kind: 'done',
      segment_id: 13,
      usage: { prompt_tokens: 8, completion_tokens: 12, total_tokens: 20, cost_usd: 0.0002 },
    })
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-batch-usage]').text()).toBe(
      'Tổng 60 token · ước tính ~0,0006 USD cho 3 câu đã dịch',
    )

    wrapper.unmount()
  })

  it('mọi câu đã dịch đều báo usage, mô hình KHÔNG có giá (cục bộ) ⇒ dòng tổng CHỈ token, không một số tiền nào', async () => {
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Mot.' })
    fake.emit({
      kind: 'done',
      segment_id: 11,
      usage: { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30, cost_usd: null },
    })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Hai.' })
    fake.emit({
      kind: 'done',
      segment_id: 12,
      usage: { prompt_tokens: 5, completion_tokens: 5, total_tokens: 10, cost_usd: null },
    })
    fake.emit({ kind: 'token', segment_id: 13, text: 'Ba.' })
    fake.emit({
      kind: 'done',
      segment_id: 13,
      usage: { prompt_tokens: 8, completion_tokens: 12, total_tokens: 20, cost_usd: null },
    })
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-batch-usage]').text()).toBe('Tổng 60 token cho 3 câu đã dịch')

    wrapper.unmount()
  })

  it('chỉ MỘT PHẦN câu đã dịch báo usage, mô hình CÓ giá ⇒ dòng tổng nêu rõ nó phủ bao nhiêu câu VÀ cộng đúng tiền của các câu đã báo', async () => {
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Mot.' })
    fake.emit({
      kind: 'done',
      segment_id: 11,
      usage: { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30, cost_usd: 0.0003 },
    })
    // Cau 12 dich xong nhung KHONG bao usage -- I/O Matrix "Provider sends no usage" ap dung
    // cho tung cau cua lo, khong chi lot don.
    fake.emit({ kind: 'token', segment_id: 12, text: 'Hai.' })
    fake.emit({ kind: 'done', segment_id: 12, usage: null })
    fake.emit({ kind: 'token', segment_id: 13, text: 'Ba.' })
    fake.emit({
      kind: 'done',
      segment_id: 13,
      usage: { prompt_tokens: 5, completion_tokens: 5, total_tokens: 10, cost_usd: 0.0001 },
    })
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-batch-usage]').text()).toBe(
      'Tổng 40 token · ước tính ~0,0004 USD — chỉ tính được cho 2/3 câu đã báo số liệu',
    )

    wrapper.unmount()
  })

  it('một hàng ĐÃ báo usage lại thiếu `cost_usd` trong khi hàng khác của CÙNG lô có ⇒ tổng lùi về CHỈ token, không một tổng thiếu-mất-một-phần', async () => {
    // Ca biên phòng thủ: trên lý thuyết một lô dùng chung một mô hình nên cost_usd của mọi
    // hàng ĐÃ báo phải cùng có mặt hoặc cùng vắng mặt -- nhưng `aiTranslateBatchUsageSummary`
    // không được TIN giả định đó im lặng, nó phải kiểm bằng `every(...)` thật (xem doc-comment
    // tại nguồn). Ca này gieo đúng hình dạng lệch để chứng minh nhánh an toàn chạy thật.
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Mot.' })
    fake.emit({
      kind: 'done',
      segment_id: 11,
      usage: { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30, cost_usd: 0.0003 },
    })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Hai.' })
    fake.emit({
      kind: 'done',
      segment_id: 12,
      usage: { prompt_tokens: 5, completion_tokens: 5, total_tokens: 10, cost_usd: null },
    })
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    // Cả hai câu đã DỊCH XONG đều CÓ mang usage (chỉ lệch ở `cost_usd`), nên phủ token vẫn
    // "đầy đủ" (2/2) -- điều bị lùi về là TIỀN, không phải nhãn đầy-đủ/một-phần: dòng tổng vẫn
    // là dạng "đầy đủ", chỉ KHÔNG mang một số tiền nào.
    expect(wrapper.get('[data-ai-translate-batch-usage]').text()).toBe('Tổng 40 token cho 2 câu đã dịch')

    wrapper.unmount()
  })

  it('chưa câu nào báo usage ⇒ KHÔNG một dòng tổng nào (không `0 token` giả)', async () => {
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Mot.' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-ai-translate-batch-usage]').exists()).toBe(false)

    fake.emit({ kind: 'token', segment_id: 12, text: 'Hai.' })
    fake.emit({ kind: 'done', segment_id: 12, usage: null })
    fake.emit({ kind: 'token', segment_id: 13, text: 'Ba.' })
    fake.emit({ kind: 'done', segment_id: 13, usage: null })
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-ai-translate-batch-usage]').exists()).toBe(false)

    wrapper.unmount()
  })
})

describe('AiTranslationPanel.vue — dispatch("ai.translate.batch_run") qua "fake Channel", số đếm tiến độ (tiếp)', () => {
  it('segment mang is_omitted bên trong vùng chọn ⇒ khung `skipped` đưa hàng đó thẳng sang "xong việc", đếm done tăng mà KHÔNG chờ token nào', async () => {
    const { i18n, editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'skipped', segment_id: 12 })
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-batch-progress]').text()).toBe(
      i18n.t('panel.ai_translation.batch_progress', { done_count: '1', total_count: '3', remaining_count: '2' }),
    )
    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[1]?.attributes('data-ai-translate-batch-row-status')).toBe('skipped')
    // Không hàng nào ĐANG chảy — "skipped" không phải "running".
    expect(wrapper.find('[data-ai-translate-batch-running]').exists()).toBe(false)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// I/O Matrix "Cancel mid-batch" — sentence đang chảy mất văn bản dở dang, 1..N-1 giữ nguyên
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — huỷ giữa lô (I/O Matrix "Cancel mid-batch")', () => {
  it('huỷ trong lúc câu 2 đang chảy ⇒ câu 1 giữ kết quả, câu 2 chuyển "cancelled" và MẤT văn bản dở dang, câu 3 chưa từng được gọi', async () => {
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Mot phan cau 2' })
    await wrapper.vm.$nextTick()

    await wrapper.get('[data-ai-translate-cancel]').trigger('click')
    expect(cancelMock).toHaveBeenCalledTimes(1)
    // Doc-comment `cancelAiTranslateBatch` (`aiTranslateBatchState.ts`): KHÔNG tự đặt
    // 'cancelled' ở đây — vẫn 'generating' cho tới khi Rust xác nhận.
    const rowsBeforeSettle = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rowsBeforeSettle[1]?.attributes('data-ai-translate-batch-row-status')).toBe('running')

    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[0]?.attributes('data-ai-translate-batch-row-status')).toBe('done')
    expect(rows[1]?.attributes('data-ai-translate-batch-row-status')).toBe('cancelled')
    expect(rows[2]?.attributes('data-ai-translate-batch-row-status')).toBe('pending')
    // Câu 3 KHÔNG BAO GIỜ được gọi — chỉ MỘT lần `runBatchMock`, không một lời gọi thứ hai.
    expect(runBatchMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// I/O Matrix "Error mid-batch" — lỗi dừng LÔ, nêu tên đúng câu, 1..N-1 giữ nguyên
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — lỗi giữa lô (I/O Matrix "Error mid-batch")', () => {
  it('provider trượt ở câu 2 ⇒ `.ai-translate-alert` hiện đúng bản dịch của họ nguyên nhân (KHÔNG nêu số câu — hàng lô mới là nơi biết CÂU NÀO, Quyết định 2 spec 4.10), câu 2 chuyển "error" VÀ GIỮ NGUYÊN văn bản đã nhận, câu 1 vẫn "done", câu 3 chưa từng được gọi', async () => {
    const { editorState, selectionState, batchState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Mot phan truoc khi loi' })
    await wrapper.vm.$nextTick()

    fake.settle({ value: null, error: PROVIDER_UNREACHABLE_ON_12 })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const alert = wrapper.get('[data-ai-translate-batch-alert]')
    expect(alert.attributes('role')).toBe('alert')
    // 🔴 CHUỖI LITERAL, KHÔNG gọi lại `i18n.tError(...)` — xem doc-comment
    // `PROVIDER_UNREACHABLE_ON_12` đầu tệp: so hai lời gọi `tError` giống nhau là một phép đối
    // chứng RỖNG. Chép NGUYÊN VĂN từ `src/i18n/vi.json` (`err.ai_translate.provider_unreachable`)
    // — câu KHÔNG nêu số câu 12, đúng Quyết định 2 ("No per-path duplicate of the family").
    expect(alert.text()).toBe(
      'Không kết nối được tới nhà cung cấp AI — chưa có kết quả mới nào. Những đoạn đã nhận trước đó vẫn còn trên màn hình.',
    )

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[0]?.attributes('data-ai-translate-batch-row-status')).toBe('done')
    expect(rows[1]?.attributes('data-ai-translate-batch-row-status')).toBe('error')
    expect(rows[2]?.attributes('data-ai-translate-batch-row-status')).toBe('pending')

    // 🔴 Khác nhánh `cancelled`: §Always spec 4.9 chỉ hứa xoá văn bản dở dang cho CANCEL —
    // văn bản của câu lỗi phải còn nguyên. Không một `data-*` riêng cho text của hàng trong
    // template (chỉ segmentId + nhãn trạng thái) — kiểm qua state của module thay vì DOM.
    const errorRow = batchState.aiTranslateBatchRows.value.find((r) => r.segmentId === 12)
    expect(errorRow?.text).toBe('Mot phan truoc khi loi')

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Story 4.10, Phase 3 — Task 10: nút "Thử lại" LÔ. `[data-ai-translate-batch-retry]` chỉ hiện
// khi `retryable`, và một click dispatch ĐÚNG tập id chưa chạy (`error` + mọi `pending`), không
// một câu `done` nào bị gọi lại -- assert TẬP ID THẬT, không chỉ "có gọi lại".
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — nút "Thử lại" LÔ (Story 4.10, §Always: "retryable grants only the right to SHOW a button")', () => {
  it('lỗi retryable ở câu 12/3 ⇒ nút Thử lại LÔ hiện, `aiTranslateBatchRetryIds` trả ĐÚNG [12, 13], click dispatch lại ĐÚNG hai id đó (không câu 11 đã "done")', async () => {
    const { editorState, selectionState, batchState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Mot phan truoc khi loi' })
    await wrapper.vm.$nextTick()

    fake.settle({ value: null, error: PROVIDER_UNREACHABLE_ON_12 })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[0]?.attributes('data-ai-translate-batch-row-status')).toBe('done')
    expect(rows[1]?.attributes('data-ai-translate-batch-row-status')).toBe('error')
    expect(rows[2]?.attributes('data-ai-translate-batch-row-status')).toBe('pending')

    expect(wrapper.get('[data-ai-translate-batch-retry]').text()).toBe('Thử lại các câu chưa xong')
    // Hàm THUẦN trực tiếp (Left for Phase 3, `## Phase notes` §"Phase 2") -- không suy tập id
    // từ dispatch, đọc thẳng từ module.
    expect(batchState.aiTranslateBatchRetryIds(batchState.aiTranslateBatchRows.value)).toEqual([12, 13])

    runBatchMock.mockClear()
    const fake2 = pendingBatchRun()
    await wrapper.get('[data-ai-translate-batch-retry]').trigger('click')
    await flushPromises()

    // 🔴 Mệnh đề trung tâm: ĐÚNG [12, 13], KHÔNG [11, 12, 13] -- câu 11 đã "done" không bao giờ
    // được gọi lại.
    expect(runBatchMock).toHaveBeenCalledTimes(1)
    expect(runBatchMock).toHaveBeenCalledWith([12, 13], null, expect.any(Function))
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')
    expect(batchState.aiTranslateBatchRows.value.map((r) => r.segmentId)).toEqual([12, 13])

    fake2.emit({ kind: 'token', segment_id: 12, text: 'z' })
    fake2.emit({ kind: 'done', segment_id: 12, usage: null })
    fake2.emit({ kind: 'token', segment_id: 13, text: 'z' })
    fake2.emit({ kind: 'done', segment_id: 13, usage: null })
    fake2.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()

    wrapper.unmount()
  })

  it('lỗi KHÔNG retryable ⇒ không có `[data-ai-translate-batch-retry]` nào trong DOM, và dispatch trực tiếp (chord) không gọi lại provider', async () => {
    const { commands, editorState, selectionState, batchState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Ket qua 2' })
    fake.emit({ kind: 'done', segment_id: 12, usage: null })
    await wrapper.vm.$nextTick()

    fake.settle({ value: null, error: PROVIDER_REFUSED_ON_13 })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(batchState.aiTranslateBatchStateValue.value).toBe('error')
    expect(wrapper.find('[data-ai-translate-batch-retry]').exists()).toBe(false)

    runBatchMock.mockClear()
    commands.dispatch('ai.translate.batch_retry')
    await flushPromises()

    expect(runBatchMock).not.toHaveBeenCalled()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('error')

    wrapper.unmount()
  })

  it('huỷ giữa lúc câu 12 đang chảy ⇒ KHÔNG có nút Thử lại LÔ nào (frozen row đã SỬA, §Spec Change Log spec 4.10: "a cancel produces no `IpcError`"), và `aiTranslateBatchRetryIds` chỉ trả câu CHƯA CHẠY (13), KHÔNG gồm câu vừa bị huỷ dở (12)', async () => {
    const { commands, editorState, selectionState, batchState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState)
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'Mot phan cau 2' })
    await wrapper.vm.$nextTick()

    await wrapper.get('[data-ai-translate-cancel]').trigger('click')
    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[0]?.attributes('data-ai-translate-batch-row-status')).toBe('done')
    expect(rows[1]?.attributes('data-ai-translate-batch-row-status')).toBe('cancelled')
    expect(rows[2]?.attributes('data-ai-translate-batch-row-status')).toBe('pending')
    expect(batchState.aiTranslateBatchError.value).toBeNull()

    expect(wrapper.find('[data-ai-translate-batch-retry]').exists()).toBe(false)
    // 🔴 KEEP của §Spec Change Log: "aiTranslateBatchRetryIds returns the `error` row plus every
    // `pending` row and excludes `done`, `skipped` and `cancelled`" -- câu 12 (cancelled) KHÔNG
    // có mặt, chỉ câu 13 (pending, chưa từng được gọi).
    expect(batchState.aiTranslateBatchRetryIds(batchState.aiTranslateBatchRows.value)).toEqual([13])

    runBatchMock.mockClear()
    commands.dispatch('ai.translate.batch_retry')
    await flushPromises()

    // Không `IpcError` nào đang chờ (`aiTranslateBatchError` vẫn `null`) ⇒ lớp gác thứ hai từ
    // chối, đúng cách nó từ chối ca "KHÔNG retryable" ngay trên.
    expect(runBatchMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// `resetAiTranslateBatch()` huỷ lượt LÔ đang bay — cùng khuôn `aiTranslate.test.ts`'s
// `resetAiTranslate huỷ lượt dịch đang bay`
// ═══════════════════════════════════════════════════════════════════════════════════

describe('resetAiTranslateBatch huỷ lô đang bay', () => {
  it('reset giữa lúc generating ⇒ cancelAiTranslateCall đúng MỘT lần, state về not_configured, rows rỗng', async () => {
    const { batchState } = await freshPanel()
    pendingBatchRun()

    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')

    cancelMock.mockClear()
    batchState.resetAiTranslateBatch()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(batchState.aiTranslateBatchStateValue.value).toBe('not_configured')
    expect(batchState.aiTranslateBatchRows.value).toEqual([])
  })

  it('reset khi KHÔNG có lô nào đang bay ⇒ không gửi gì — đối chứng âm, kẻo ca trên xanh vì reset luôn huỷ', async () => {
    const { batchState } = await freshPanel()
    cancelMock.mockClear()

    batchState.resetAiTranslateBatch()

    expect(cancelMock).not.toHaveBeenCalled()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// I/O Matrix "Selection changes while a batch runs" — lô GIỮ NGUYÊN danh sách đã đóng
// băng lúc khởi; vùng chọn đổi TRONG LÚC lô chảy chỉ có nghĩa cho LƯỢT SAU
// ═══════════════════════════════════════════════════════════════════════════════════

describe('I/O Matrix "Selection changes while a batch runs" — lô đang chạy giữ nguyên danh sách đã đóng băng lúc khởi', () => {
  it('mở rộng vùng chọn TRONG LÚC lô đang chảy ⇒ hàng của lô đứng nguyên; vùng chọn mới chỉ đổi cho LƯỢT SAU', async () => {
    const { i18n, editorState, selectionState, batchState, AiTranslationPanel } = await freshPanel()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12])
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    // `runAiTranslateBatch` chụp một BẢN SAO (`ids = segmentIds.slice()`) NGAY tại lời gọi —
    // đây là mệnh đề trung tâm của ca này, không phải một chi tiết cài đặt.
    expect(runBatchMock).toHaveBeenCalledWith([11, 12], null, expect.any(Function))
    expect(batchState.aiTranslateBatchRows.value.map((r) => r.segmentId)).toEqual([11, 12])

    // Mở rộng vùng chọn thêm MỘT hàng — cùng hợp âm AC1 — trong khi lô vẫn đang chảy.
    selectionState.extendSegmentSelectionDown()
    await wrapper.vm.$nextTick()

    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12, 13])
    expect(wrapper.get('[data-ai-translate-batch-selection-count]').text()).toBe(
      i18n.t('panel.ai_translation.batch_selection_count', { count: '3' }),
    )

    // 🔴 Mệnh đề trung tâm: lô ĐANG CHẠY không hay biết gì về vùng chọn vừa đổi — vẫn đúng
    // HAI hàng ban đầu, không một hàng thứ ba nào xuất hiện, và KHÔNG một lượt `runBatchMock`
    // thứ hai nào tự kích hoạt.
    expect(batchState.aiTranslateBatchRows.value.map((r) => r.segmentId)).toEqual([11, 12])
    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows).toHaveLength(2)
    expect(runBatchMock).toHaveBeenCalledTimes(1)

    // Cho lô hiện tại kết thúc bình thường — chỉ để không rò một Promise treo sang ca sau.
    fake.emit({ kind: 'token', segment_id: 11, text: 'x' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    fake.emit({ kind: 'token', segment_id: 12, text: 'y' })
    fake.emit({ kind: 'done', segment_id: 12, usage: null })
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// I/O Matrix "Empty selection" — không vùng chọn, hoặc caret `null` ⇒ nút khoá, 0 lời gọi IPC
// ═══════════════════════════════════════════════════════════════════════════════════

describe('I/O Matrix "Empty selection" — nút Dịch LÔ khoá, và dispatch KHÔNG gửi gì', () => {
  it('caret null, chưa từng chọn gì ⇒ dòng đếm vùng chọn không hiện, nút khoá, click KHÔNG gọi runAiTranslateBatchCall', async () => {
    const { editorState, selectionState, AiTranslationPanel } = await freshPanel()
    expect(editorState.editorCaretSegmentId.value).toBeNull()
    expect(selectionState.segmentSelectionCount.value).toBe(0)

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-ai-translate-batch-selection-count]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-translate-batch-run]').attributes('disabled')).toBeDefined()

    // Đo được (script riêng, happy-dom): một `<button disabled>` KHÔNG phát sự kiện `click` —
    // cùng hành vi trình duyệt thật, nên `.trigger('click')` ở đây là phép đo, không phải hình
    // thức.
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    expect(runBatchMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })

  it('lớp phòng thủ THỨ HAI (`aiTranslateBatchState.ts::runAiTranslateBatch`): gọi thẳng với vùng chọn rỗng vẫn không gửi gì — kêu, không ném', async () => {
    const { batchState } = await freshPanel()

    await batchState.runAiTranslateBatch(null, [])

    expect(runBatchMock).not.toHaveBeenCalled()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('not_configured')
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Cổng loại trừ lẫn nhau (`main.ts`) — một LÔ đang chạy chặn một lượt ĐƠN, và ngược lại.
// `installCommands` ở `freshPanel()` đã chép nguyên văn cả hai cửa chéo; các ca dưới đây là
// thứ còn thiếu để CHÍNH chúng được dispatch thật, không chỉ đứng đó chưa ai gọi tới.
// ═══════════════════════════════════════════════════════════════════════════════════

describe('Cổng loại trừ lẫn nhau giữa một lượt ĐƠN và một LÔ', () => {
  it('LÔ đang "generating" ⇒ dispatch("ai.translate.run") KHÔNG gọi runAiTranslateSegment', async () => {
    const { commands, editorState, selectionState, batchState } = await freshPanel()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    pendingBatchRun()

    commands.dispatch('ai.translate.batch_run')
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')

    commands.dispatch('ai.translate.run')
    await flushPromises()

    expect(runSegmentMock).not.toHaveBeenCalled()
  })

  it('lượt ĐƠN đang "generating" ⇒ dispatch("ai.translate.batch_run") KHÔNG gọi runAiTranslateBatchCall', async () => {
    const { commands, editorState, selectionState, state } = await freshPanel()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    pendingSegmentRun()

    commands.dispatch('ai.translate.run')
    await flushPromises()
    expect(state.aiTranslateStateValue.value).toBe('generating')

    commands.dispatch('ai.translate.batch_run')
    await flushPromises()

    expect(runBatchMock).not.toHaveBeenCalled()
  })

  // 🔴 THÊM (rà soát) — cùng cổng loại-trừ-lẫn-nhau ngay trên, áp cho HAI command retry
  // (`main.ts:1048/1052` và `:1075/1079`): mỗi handler retry hỏi module KIA trước khi hỏi lỗi
  // `retryable` của chính nó. Hai ca trên chỉ canh `run`/`batch_run` — gỡ cổng loại-trừ khỏi
  // `retryAiTranslate`/`retryAiTranslateBatch` vẫn để cả bộ này xanh, nên hai ca dưới đây canh
  // riêng nhánh retry.
  it('LÔ đang "generating" ⇒ dispatch("ai.translate.retry") KHÔNG gọi lại runAiTranslateSegment dù lượt ĐƠN đang "error" với một lỗi retryable đang chờ', async () => {
    const { commands, editorState, selectionState, state, batchState } = await freshPanel()
    editorState.setEditorCaret(11)
    const single = pendingSegmentRun()
    commands.dispatch('ai.translate.run')
    await flushPromises()
    single.settle({
      value: null,
      error: {
        code: 'ai_translate.stream_ended_without_done',
        message_key: 'err.ai_translate.stream_ended_without_done',
        params: {},
        retryable: true,
      },
    })
    await flushPromises()
    expect(state.aiTranslateStateValue.value).toBe('error')

    selectionState.extendSegmentSelectionDown()
    pendingBatchRun()
    commands.dispatch('ai.translate.batch_run')
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('generating')

    runSegmentMock.mockClear()
    commands.dispatch('ai.translate.retry')
    await flushPromises()

    expect(runSegmentMock).not.toHaveBeenCalled()
  })

  it('lượt ĐƠN đang "generating" ⇒ dispatch("ai.translate.batch_retry") KHÔNG gọi lại runAiTranslateBatchCall dù LÔ đang "error" với một lỗi retryable đang chờ', async () => {
    const { commands, editorState, selectionState, state, batchState } = await freshPanel()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    const batch = pendingBatchRun()
    commands.dispatch('ai.translate.batch_run')
    await flushPromises()
    batch.settle({ value: null, error: PROVIDER_UNREACHABLE_ON_12 })
    await flushPromises()
    expect(batchState.aiTranslateBatchStateValue.value).toBe('error')

    pendingSegmentRun()
    commands.dispatch('ai.translate.run')
    await flushPromises()
    expect(state.aiTranslateStateValue.value).toBe('generating')

    runBatchMock.mockClear()
    commands.dispatch('ai.translate.batch_retry')
    await flushPromises()

    expect(runBatchMock).not.toHaveBeenCalled()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// dispatch("ai.translate.promote") — nhánh LÔ (Decision 2/3 spec 4.9). `main.ts` thử nhánh
// ĐƠN trước (nguyên vẹn, canh ở `aiTranslate.test.ts`); nhánh LÔ — đọc kết quả "done" của câu
// đang có TIÊU ĐIỂM, tra qua `aiTranslateBatchTextForSegment` — không có ca nào cho tới bản
// sửa này, dù `main.ts` đã có dây thật.
// ═══════════════════════════════════════════════════════════════════════════════════

describe('dispatch("ai.translate.promote") — nhánh dự phòng LÔ của main.ts', () => {
  it('lượt ĐƠN chưa từng chạy, hàng LÔ tại đúng câu TIÊU ĐIỂM đã "done" ⇒ promoteAiTranslationToEditor nhận ĐÚNG caret id và văn bản CHỐT của hàng đó', async () => {
    const { commands, editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState) // chọn 11/12/13, caret ở 11
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua cau 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    await wrapper.vm.$nextTick()
    expect(editorState.editorCaretSegmentId.value).toBe(11)

    commands.dispatch('ai.translate.promote')
    await flushPromises()

    expect(promoteMock).toHaveBeenCalledTimes(1)
    expect(promoteMock).toHaveBeenCalledWith(11, 'Ket qua cau 1')

    wrapper.unmount()
  })

  it('caret NGOÀI lô (một segment_id không nằm trong vùng chọn đã chạy) ⇒ promoteAiTranslationToEditor KHÔNG được gọi — kêu, không ghi', async () => {
    const { commands, editorState, selectionState, AiTranslationPanel } = await freshPanel()
    selectAllThreeFixtureSegments(editorState, selectionState) // chọn 11/12/13
    const fake = pendingBatchRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-batch-run]').trigger('click')
    await flushPromises()

    fake.emit({ kind: 'token', segment_id: 11, text: 'Ket qua cau 1' })
    fake.emit({ kind: 'done', segment_id: 11, usage: null })
    await wrapper.vm.$nextTick()

    // Tiêu điểm dời sang một `segment_id` KHÔNG nằm trong ba hàng của lô đang chạy —
    // `aiTranslateBatchTextForSegment` trả `null` cho một hàng không tồn tại trong `currentRows`.
    editorState.setEditorCaret(999)
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => undefined)

    commands.dispatch('ai.translate.promote')
    await flushPromises()

    expect(promoteMock).not.toHaveBeenCalled()
    expect(warnSpy).toHaveBeenCalled()

    wrapper.unmount()
  })
})
