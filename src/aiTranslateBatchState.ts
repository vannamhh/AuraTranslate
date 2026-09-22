/**
 * State của lượt **dịch theo LÔ với huỷ giữa chừng** — Story 4.9, Phase 3 (FR73, AD-22,
 * Decision 1/2). Module RIÊNG với `aiTranslateState.ts` (lượt dịch MỘT segment, Story 4.8) —
 * xem §Design Notes spec 4.9 *"Why a separate batch state module instead of widening
 * `aiTranslateState.ts`"*: năm giá trị của module kia đã ĐÓNG BĂNG bởi AC của 4.8, và một lô
 * cần MỘT HÀNG cho MỖI câu, hình dạng khác chứ không phải hình dạng lớn hơn.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT `Channel` DUY NHẤT, N HÀNG — mỗi khung tự mang `segment_id` của nó
 * ─────────────────────────────────────────────────────────────────────────────
 * `runAiTranslateBatchCall` (`config/aitranslate.ts`) dựng MỘT `Channel<AiTranslateBatchEventWire>`
 * cho toàn lô (§Never spec 4.9: "no second `Channel` per sentence") — [`runAiTranslateBatch`]
 * tra `segment_id` của mỗi khung ra ĐÚNG hàng của nó qua [`rowIndexBySegmentId`] (một `Map`
 * dựng lại ở đầu mỗi lượt chạy, O(1) mỗi khung thay vì `findIndex` tuyến tính trên một lô có
 * thể tới 184 câu — Decision 1 spec 4.9: "there is no ceiling on N").
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 TRẠNG THÁI MỖI HÀNG — sáu giá trị, đúng sáu sự thật I/O Matrix spec 4.9 cần phân biệt
 * ─────────────────────────────────────────────────────────────────────────────
 * `pending` (chưa tới lượt — "never called and therefore never charged") → `running` (khung
 * `token` đầu tiên của câu đó đã tới) → `done` (khung `done`, kết quả CHỐT) hoặc `skipped`
 * (khung `skipped`, `is_omitted`, không lượt gọi provider nào). Hai giá trị còn lại chỉ được
 * GÁN LẠI ở [`runAiTranslateBatch`] SAU khi `Channel` đã đóng, không bao giờ qua một khung:
 * `cancelled` — hàng đang `running` lúc lệnh trả `Cancelled` (I/O Matrix "Cancel mid-batch":
 * *"the in-flight sentence's partial text is discarded"* — `text` bị XOÁ khi chuyển sang giá
 * trị này); `error` — hàng mà `segment_id` của `IpcError.params` nêu tên (I/O Matrix "Error
 * mid-batch": *"stops the batch and names the sentence"*), `text` GIỮ NGUYÊN — §Always spec
 * 4.9 chỉ hứa xoá văn bản dở dang cho CANCEL, và 4.8 đã kiểm đúng chiều ngược cho LỖI
 * (`a_provider_error_after_partial_tokens_leaves_earlier_tokens_visible_and_propagates_the_error`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 VÌ SAO CÓ [`aiTranslateBatchTextForSegment`] — cầu nối tới "Đưa sang bản dịch" ĐÃ CÓ
 * ─────────────────────────────────────────────────────────────────────────────
 * Decision 2 spec 4.9: "Promote stays per sentence", qua ĐÚNG `ai.translate.promote` mà 4.8
 * đã đăng ký — không một lệnh thứ hai. Nhưng lệnh đó (`main.ts`) trước Story 4.9 chỉ biết đọc
 * `aiTranslateState.ts::runSegmentId`/`accumulatedText` (kết quả của MỘT lượt đơn); một kết
 * quả đến từ lô không có mặt ở đó. Hàm thuần này là cầu nối: `main.ts` truyền
 * `editorCaretSegmentId.value` (Decision 3: "each promote targets the caret segment") vào,
 * nhận về văn bản CHỐT của đúng hàng đó nếu hàng đang ở trạng thái `done` — `null` cho MỌI
 * trạng thái khác, kể cả `cancelled`/`error`/`running` (chưa chốt, không đáng ghi vào Editor).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 CỔNG LOẠI TRỪ LẪN NHAU SỐNG Ở `main.ts`, KHÔNG Ở ĐÂY
 * ─────────────────────────────────────────────────────────────────────────────
 * §Tasks spec 4.9 Phase 3: *"the guard cannot live in either state module without a two-way
 * import cycle"*. [`runAiTranslateBatch`] chỉ tự chặn MỘT lô CHỒNG một lô khác
 * (`state.value === 'generating'`) — nó không biết gì về `aiTranslateState.ts`. Việc "một lô
 * từ chối trong khi một lượt đơn đang chạy, và ngược lại" là việc của `main.ts`'s handler của
 * `ai.translate.run`/`ai.translate.batch_run`, nơi CẢ HAI module đã được `import`.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { cancelAiTranslateCall, runAiTranslateBatchCall } from './config/aitranslate'
import type { AiTranslateBatchEventWire } from './config/aitranslate'
import type { IpcError } from './i18n'

/** Xem doc-comment đầu tệp §"Trạng thái mỗi hàng" cho ý nghĩa và thứ tự chuyển của sáu giá trị. */
export type AiTranslateBatchRowStatus = 'pending' | 'running' | 'done' | 'skipped' | 'cancelled' | 'error'

/** Một hàng của lô — `segmentId` cố định từ lúc [`runAiTranslateBatch`] khởi, `status`/`text`
 * đổi qua các khung `Channel` và qua kết quả cuối. */
export type AiTranslateBatchRow = {
  segmentId: number
  status: AiTranslateBatchRowStatus
  text: string
}

/** Cùng năm giá trị `AiTranslateState` (`aiTranslateState.ts`) — TÁI DÙNG hình dạng đó cho
 * TRẠNG THÁI TOÀN LÔ (không phải một hàng), vì `AiTranslateOutcomeWire` phía Rust dùng
 * chung cho cả lượt dịch một segment lẫn một lô (Code Map spec 4.9: "reuse it ... a second
 * enum would drift"). Module này KHÔNG mang cùng ràng buộc đóng băng — chỉ tình cờ khớp. */
export type AiTranslateBatchState = 'not_configured' | 'generating' | 'done' | 'error' | 'cancelled'

const state = ref<AiTranslateBatchState>('not_configured')
const rows = ref<AiTranslateBatchRow[]>([])
const error = shallowRef<IpcError | null>(null)

/** Cùng cơ chế `sequence` của `aiTranslateState.ts` — vô hiệu hoá khung/kết quả TRỄ của một
 * lượt đã bị [`resetAiTranslateBatch`] hoặc một lượt CHẠY MỚI vượt mặt. */
let sequence = 0
/** Tra `segment_id` của một khung ra CHỈ SỐ hàng của nó — dựng lại ở đầu mỗi
 * [`runAiTranslateBatch`], xem doc-comment đầu tệp. */
let rowIndexBySegmentId = new Map<number, number>()

export const aiTranslateBatchStateValue: DeepReadonly<Ref<AiTranslateBatchState>> = readonly(state)
export const aiTranslateBatchRows: DeepReadonly<Ref<AiTranslateBatchRow[]>> = readonly(rows)
/** Lỗi của lô gần nhất — có nghĩa khi và chỉ khi [`aiTranslateBatchStateValue`] là `'error'`. */
export const aiTranslateBatchError: DeepReadonly<Ref<IpcError | null>> = readonly(error)

/** Tổng số hàng của lô đang/đã chạy — Panel dùng để vẽ "N/Tổng". */
export const aiTranslateBatchTotalCount: ComputedRef<number> = computed(() => rows.value.length)
/** `done` VÀ `skipped` đều là "đã xong việc" — không còn gì để chờ ở hàng đó (AC2). */
export const aiTranslateBatchDoneCount: ComputedRef<number> = computed(
  () => rows.value.filter((r) => r.status === 'done' || r.status === 'skipped').length,
)
/** Số hàng CHƯA TỚI LƯỢT — đúng cột "remain" của AC2. */
export const aiTranslateBatchRemainingCount: ComputedRef<number> = computed(
  () => rows.value.filter((r) => r.status === 'pending').length,
)
/** `segmentId` của hàng ĐANG chảy — `null` khi không hàng nào `running` (giữa hai câu, hoặc lô
 * chưa/đã xong). Tối đa MỘT hàng `running` tại một thời điểm (§Always spec 4.9: một `Channel`
 * cho cả lô, `run_batch_call` phía Rust dịch tuần tự). */
export const aiTranslateBatchRunningSegmentId: ComputedRef<number | null> = computed(() => {
  const running = rows.value.find((r) => r.status === 'running')
  return running === undefined ? null : running.segmentId
})

/**
 * Văn bản CHỐT của `segmentId` trong `currentRows`, hoặc `null` nếu hàng đó không ở trạng
 * thái `done` (kể cả không tồn tại trong `currentRows`). HÀM THUẦN — chỗ gọi (`main.ts`'s
 * handler của `ai.translate.promote`) truyền `aiTranslateBatchRows.value` vào, xem doc-comment
 * đầu tệp §"Vì sao có hàm này".
 */
export function aiTranslateBatchTextForSegment(
  currentRows: readonly AiTranslateBatchRow[],
  segmentId: number,
): string | null {
  const row = currentRows.find((r) => r.segmentId === segmentId)
  if (row === undefined || row.status !== 'done') return null
  return row.text
}

/**
 * ID những hàng một lượt RETRY sẽ chạy lại: hàng `error` cộng MỌI hàng `pending`, giữ ĐÚNG
 * thứ tự trong `currentRows` (thứ tự tài liệu đã đóng băng từ lúc [`runAiTranslateBatch`]
 * khởi lô — xem doc-comment đầu tệp §"Một `Channel` DUY NHẤT, N hàng"). HÀM THUẦN, cùng khuôn
 * [`aiTranslateBatchTextForSegment`] ngay trên — chỗ gọi (`main.ts`'s handler thật của
 * `ai.translate.batch_retry`) truyền `aiTranslateBatchRows.value` vào, tệp này không tự đọc gì.
 *
 * 🔴 `done` VÀ `skipped` VÀ `cancelled` không có mặt trong tập trả về — một retry không bao
 * giờ gọi lại một câu đã CHỐT kết quả (Task 6 spec 4.10: *"a retry must never re-send a `done`
 * sentence, which is the whole reason the user pressed stop rather than losing the work"*).
 * `running` cũng không có mặt: tối đa một hàng `running` tại một thời điểm và nó chỉ tồn tại
 * TRONG LÚC lô đang `generating`, khi nút retry vốn đã bị khoá (xem `canRetryAiTranslateBatch`,
 * `AiTranslationPanel.vue`).
 */
export function aiTranslateBatchRetryIds(currentRows: readonly AiTranslateBatchRow[]): number[] {
  return currentRows.filter((r) => r.status === 'error' || r.status === 'pending').map((r) => r.segmentId)
}

/** `index` luôn hợp lệ trong thực tế — nó đến từ [`rowIndexBySegmentId`], dựng ĐÚNG từ độ dài
 * `rows.value` ở đầu [`runAiTranslateBatch`] — nhưng vẫn kiểm biên PHÒNG THỦ (không đọc
 * `rows.value[index]` khi biên sai) thay vì tin tưởng lời hứa đó im lặng. */
function setRowAt(index: number, patch: Partial<AiTranslateBatchRow>): void {
  if (index < 0 || index >= rows.value.length) return
  rows.value[index] = { ...rows.value[index], ...patch }
}

/**
 * Handler thật của `ai.translate.batch_run` — dịch `segmentIds` (đúng thứ tự tài liệu, chỗ
 * gọi truyền `segmentSelectionIds.value`) bằng bộ prompt hiệu lực `promptSetName`.
 *
 * `segmentIds` rỗng ⇔ không có vùng chọn — **không ném, chỉ kêu** (I/O Matrix spec 4.9
 * "Empty selection": *"the batch command is disabled; no IPC call is made"* — cửa Panel đã
 * vô hiệu hoá nút ở chỗ BIẾT TRƯỚC, đây là lớp phòng thủ THỨ HAI, cùng khuôn `runAiTranslate`).
 * Một lô đang `'generating'` cũng bị từ chối im lặng — cổng LOẠI TRỪ một lượt ĐƠN đang chạy
 * sống ở `main.ts`, xem doc-comment đầu tệp.
 */
export async function runAiTranslateBatch(
  promptSetName: string | null,
  segmentIds: readonly number[],
): Promise<void> {
  if (state.value === 'generating') {
    console.warn('[ai-translate-batch] khong dich: mot lo khac dang chay')
    return
  }
  if (segmentIds.length === 0) {
    console.warn('[ai-translate-batch] khong dich: vung chon rong')
    return
  }

  const mine = ++sequence
  const ids = segmentIds.slice()
  rowIndexBySegmentId = new Map(ids.map((id, index) => [id, index]))
  rows.value = ids.map((id) => ({ segmentId: id, status: 'pending', text: '' }))
  state.value = 'generating'
  error.value = null

  const onEvent = (event: AiTranslateBatchEventWire): void => {
    // Một lượt MỚI (hoặc một lượt reset) đã vượt mặt lượt này — khung trễ không được ghi vào
    // hàng của lượt hiện hành, cùng luật `onToken` của `aiTranslateState.ts`.
    if (mine !== sequence) return
    const index = rowIndexBySegmentId.get(event.segment_id)
    if (index === undefined) return

    if (event.kind === 'token') {
      setRowAt(index, { status: 'running', text: rows.value[index].text + event.text })
      return
    }
    if (event.kind === 'done') {
      setRowAt(index, { status: 'done' })
      return
    }
    setRowAt(index, { status: 'skipped' })
  }

  const result = await runAiTranslateBatchCall(ids, promptSetName, onEvent)
  if (mine !== sequence) return // đã bị vượt mặt trong lúc lượt gọi này còn đang bay

  if (result.error !== null) {
    state.value = 'error'
    error.value = result.error
    // 🔵 SỬA 2026-09-22 (Story 4.10) — `batch_stopped_error` (`commands/aitranslate.rs`) không
    // còn mang một nhãn gộp `ai_translate.batch_stopped` nữa; nó trả về MỘT trong sáu `code` họ
    // nguyên nhân (Quyết định 2 spec 4.10 — `ai_translate.provider_unreachable`/
    // `provider_refused`/`stream_ended_without_done`/`reply_unreadable`/`client_build_failed`/
    // `api_key_header_invalid`), chỉ khác lượt dịch MỘT segment ở chỗ `params` LUÔN mang thêm
    // `segment_id` của ĐÚNG câu batch dừng ở đó — điểm mà đoạn dưới đây vẫn dựa vào không đổi.
    // Nhánh này chỉ ĐỔI `status` của HÀNG (`setRowAt(..., { status: 'error' })`), KHÔNG xoá `text`.
    // 🔴 Khác nhánh `cancelled` ngay dưới: §Always spec 4.9 chỉ hứa xoá văn bản dở dang cho
    // CANCEL ("the in-flight sentence's partial text is discarded"), không cho LỖI — đúng
    // tiền lệ đã kiểm của 4.8
    // (`a_provider_error_after_partial_tokens_leaves_earlier_tokens_visible_and_propagates_the_error`,
    // `ai_translate_contract.rs`): đoạn đã nhận trước khi provider trượt vẫn ở lại màn hình.
    const failedSegmentId = Number(result.error.params.segment_id)
    const index = Number.isNaN(failedSegmentId) ? undefined : rowIndexBySegmentId.get(failedSegmentId)
    if (index !== undefined) setRowAt(index, { status: 'error' })
    return
  }

  // `value === null` cùng `error === null` ⇔ không gọi được IPC (chạy ngoài Tauri) — cùng quy
  // ước `aiTranslateState.ts::runAiTranslate`.
  if (result.value === null) {
    state.value = 'not_configured'
    return
  }

  if (result.value.state === 'cancelled') {
    // I/O Matrix "Cancel mid-batch": "the in-flight sentence's partial text is discarded" —
    // tối đa MỘT hàng `running` tại một thời điểm (xem doc-comment `aiTranslateBatchRunningSegmentId`).
    const runningIndex = rows.value.findIndex((r) => r.status === 'running')
    if (runningIndex !== -1) setRowAt(runningIndex, { status: 'cancelled', text: '' })
  }
  state.value = result.value.state
}

/**
 * Handler thật của `ai.translate.cancel` KHI mục tiêu là một LÔ — chỉ gửi lệnh huỷ khi thật
 * sự có một lô đang chạy. Cùng khuôn `aiTranslateState.ts::cancelAiTranslate`; `main.ts`'s
 * handler của `ai.translate.cancel` gọi CẢ HAI hàm cùng tên ở hai module, mỗi hàm tự gác
 * đúng module của nó (xem doc-comment đầu tệp).
 */
export function cancelAiTranslateBatch(): void {
  if (state.value !== 'generating') return
  void cancelAiTranslateCall()
}

/**
 * Vứt toàn bộ state của module — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`, cùng khuôn `resetAiTranslate`. Kết quả của lô mang danh tính THEO Tác phẩm
 * (mỗi `segmentId` chỉ có nghĩa trong `project.db` đang mở) — nối vào CẢ BA cụm reset
 * (`modes/libraryChapters.ts`, `modes/libraryImport.ts`, và lượt đổi CHƯƠNG ở
 * `panels/editorPanelState.ts`, Decision 4 spec 4.9).
 *
 * 🔴 Reset giữa lúc `'generating'` phải HUỶ lượt Rust đang bay, không chỉ dọn state phía
 * webview — cùng lý lẽ `resetAiTranslate` đã ghi cho lượt dịch MỘT segment: không gọi
 * `cancelAiTranslateCall` ở đây thì đổi Tác phẩm/Chương giữa chừng MỘT LÔ bỏ rơi lượt gọi đó,
 * và với BYOK đó là tiền người dùng trả cho những câu không ai còn thấy được.
 */
export function resetAiTranslateBatch(): void {
  if (state.value === 'generating') void cancelAiTranslateCall()

  sequence += 1
  state.value = 'not_configured'
  rows.value = []
  error.value = null
  rowIndexBySegmentId = new Map()
}
