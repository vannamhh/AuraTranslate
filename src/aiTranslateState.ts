/**
 * State của lượt **dịch một segment với kết quả chảy dần** — Story 4.8 (FR72/FR74, AD-22).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 NĂM TRẠNG THÁI, MỘT Ô NHỚ — §Always spec 4.8
 * ─────────────────────────────────────────────────────────────────────────────
 * "AI state is exactly one of five values at all times": [`AiTranslateState`] là **một** union
 * đóng, không hai cờ boolean lắp ghép (`isGenerating` + `isDone` + ...) mà một cặp giá trị nào
 * đó có thể mâu thuẫn nhau (`isGenerating && isDone` cùng `true` là chuyện KHÔNG kiểu nào ở đây
 * chặn được). `not_configured` là giá trị NGHỈ mặc định — không phải một lỗi (FR77,
 * `EXPERIENCE.md:136`), cùng đúng giá trị mà `commands/aitranslate.rs::AiTranslateOutcomeWire`
 * trả về khi chưa có `endpoint`/`model`/khoá nào.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 VÌ SAO CHỈ MỘT LƯỢT DỊCH TẠI MỘT THỜI ĐIỂM
 * ─────────────────────────────────────────────────────────────────────────────
 * [`runAiTranslate`] từ chối khởi một lượt mới khi [`aiTranslateStateValue`] đang `'generating'`
 * — Rust phía sau cũng chỉ giữ MỘT `AiTranslateGeneration` cho cả tiến trình
 * (`commands/aitranslate.rs`), nên hai lượt cùng bay sẽ đua nhau ghi [`accumulatedText`]. Muốn
 * dịch câu khác trong lúc một lượt đang chạy, người dùng huỷ lượt cũ trước ([`cancelAiTranslate`]).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 "CŨ" — CÙNG CƠ CHẾ MÀ `aiPromptInspectorState.ts` ĐÃ DỰNG CHO BẢN GHI PROMPT
 * ─────────────────────────────────────────────────────────────────────────────
 * §Code Map spec 4.8: *"Caret moves during generation → the call keeps running and lands
 * against the segment it started on; the panel marks the result stale, the way 4.7 already
 * marks a stale record"*. [`runSegmentId`] chở ĐỊNH DANH câu mà lượt dịch hiện tại/gần nhất
 * thuộc về — cùng vai `AssembledPromptWire.segment_id`, và [`isAiTranslateResultStale`] là hàm
 * thuần cùng hình dạng [`aiPromptRecordIsStale`] (`aiPromptInspectorState.ts`).
 *
 * ⚠️ Cùng luật mọi state Vue khác: tệp này KHÔNG được `import` vào `src/commands/index.ts`.
 */
import { readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { cancelAiTranslateCall, runAiTranslateSegment } from './config/aitranslate'
import type { IpcError } from './i18n'

/** Đúng năm giá trị của §Always spec 4.8 — không giá trị thứ sáu. */
export type AiTranslateState = 'not_configured' | 'generating' | 'done' | 'error' | 'cancelled'

const state = ref<AiTranslateState>('not_configured')
/** Văn bản đã nhận, CỘNG DỒN theo từng token qua `Channel` — KHÔNG một mảng token rời. */
const accumulatedText = ref('')
const error = shallowRef<IpcError | null>(null)
/** `segment.id` mà lượt dịch hiện tại/gần nhất được khởi cho — xem doc-comment đầu tệp. */
const runSegmentId = shallowRef<number | null>(null)

/**
 * Số thứ tự lượt dịch — cùng cơ chế và cùng lý do `sequence` của `aiPromptInspectorState.ts`:
 * vô hiệu hoá một token/kết quả TRỄ của một lượt đã bị [`resetAiTranslate`] vượt mặt (đổi Tác
 * phẩm giữa chừng), mà không cần huỷ lượt IPC đang bay — nó tự rơi vào im lặng vì `mine !==
 * sequence`.
 */
let sequence = 0

export const aiTranslateStateValue: DeepReadonly<Ref<AiTranslateState>> = readonly(state)
export const aiTranslateAccumulatedText: DeepReadonly<Ref<string>> = readonly(accumulatedText)
/** Lỗi của lượt dịch gần nhất — có nghĩa khi và chỉ khi [`aiTranslateStateValue`] là `'error'`. */
export const aiTranslateError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const aiTranslateRunSegmentId: DeepReadonly<Ref<number | null>> = readonly(runSegmentId)

/**
 * `true` ⇔ lượt dịch hiện tại/gần nhất được khởi cho một câu KHÁC câu đang có tiêu điểm bây
 * giờ — cùng khuôn [`aiPromptRecordIsStale`] (`aiPromptInspectorState.ts`). HÀM THUẦN — chỗ gọi
 * (`AiTranslationPanel.vue`) truyền `editorCaretSegmentId` vào, tệp này không tự `import` nó
 * (cùng lý lẽ `aiPromptInspectorState.ts` đã ghi).
 *
 * `runningSegmentId === null` (chưa lượt nào chạy) hoặc `focusedSegmentId === null` (không câu
 * nào đang chọn) ⇒ `false` — không tự xưng "cũ" khi không có gì để đối chiếu.
 */
export function isAiTranslateResultStale(
  runningSegmentId: number | null,
  focusedSegmentId: number | null,
): boolean {
  if (runningSegmentId === null || focusedSegmentId === null) return false
  return runningSegmentId !== focusedSegmentId
}

/**
 * Handler thật của `ai.translate.run` — dịch `segmentId` bằng bộ prompt hiệu lực
 * `promptSetName`, gọi 4.7's producer bên trong Rust (Decision 2 spec 4.8: KHÔNG một lượt lắp
 * ráp thứ hai ở đây, tệp này không biết gì về prompt).
 *
 * `segmentId: null` ⇔ không có câu nào đang được chọn — **không ném, chỉ kêu**, cùng luật
 * `assembleCurrentAiPrompt`. Một lượt đang `'generating'` cũng bị từ chối im lặng (kêu) —
 * xem doc-comment đầu tệp.
 */
export async function runAiTranslate(promptSetName: string | null, segmentId: number | null): Promise<void> {
  if (state.value === 'generating') {
    console.warn('[ai-translate] khong dich: mot luot dich khac dang chay')
    return
  }
  if (segmentId === null) {
    console.warn('[ai-translate] khong dich: khong co cau nao dang duoc chon (editorCaretSegmentId === null)')
    return
  }

  const mine = ++sequence
  runSegmentId.value = segmentId
  state.value = 'generating'
  accumulatedText.value = ''
  error.value = null

  const onToken = (text: string): void => {
    // Một lượt MỚI (hoặc một lượt reset) đã vượt mặt lượt này — token trễ không được cộng
    // dồn vào văn bản của lượt hiện hành.
    if (mine !== sequence) return
    accumulatedText.value += text
  }

  const result = await runAiTranslateSegment(segmentId, promptSetName, onToken)
  if (mine !== sequence) return // đã bị vượt mặt trong lúc lượt gọi này còn đang bay

  if (result.error !== null) {
    state.value = 'error'
    error.value = result.error
    return
  }

  // `value === null` cùng `error === null` ⇔ không gọi được IPC (chạy ngoài Tauri) — cùng quy
  // ước `aiPromptAssemble`. Không có gì xảy ra thật; `'not_configured'` là giá trị NGHỈ của
  // §Always spec 4.8, không phải một lời khai "provider chưa cấu hình" giả — chỗ gọi ngoài
  // Tauri không có provider nào để mà cấu hình.
  state.value = result.value === null ? 'not_configured' : result.value.state
}

/**
 * Handler thật của `ai.translate.cancel` — chỉ gửi lệnh huỷ khi thật sự có một lượt đang chạy.
 * KHÔNG tự đặt [`aiTranslateStateValue`] thành `'cancelled'` ở đây: lượt `await` trong
 * [`runAiTranslate`] mới là nơi ĐỌC kết quả thật Rust trả về
 * (`AiTranslateOutcomeWire::Cancelled`) sau khi backend xác nhận đã dừng — đặt trước ở đây là
 * một lời khai về một điều CHƯA XẢY RA.
 */
export function cancelAiTranslate(): void {
  if (state.value !== 'generating') return
  void cancelAiTranslateCall()
}

/**
 * Vứt toàn bộ state của mục — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`, cùng khuôn `resetAiPromptInspector`. Kết quả dịch mang danh tính THEO Tác phẩm
 * (`runSegmentId` chỉ có nghĩa trong `project.db` đang mở) — đổi Tác phẩm mà không vứt ô này
 * để lại đúng lớp lỗi mà `resetAiPromptInspector` đã ghi cho bản ghi prompt.
 *
 * 🔴 **SỬA — reset giữa lúc `'generating'` phải HUỶ lượt Rust đang bay, không chỉ dọn state
 * phía webview.** `sequence += 1` làm token/kết quả TRỄ của lượt đó rơi vào im lặng ở phía
 * webview (`mine !== sequence`), nhưng KHÔNG tự nó dừng vòng lặp đọc khung SSE phía Rust —
 * không gọi [`cancelAiTranslateCall`] ở đây thì đổi Tác phẩm/Chương giữa chừng một lượt dịch
 * (`modes/libraryChapters.ts`/`modes/libraryImport.ts`) bỏ rơi lượt gọi đó: Rust tiếp tục kéo
 * token từ provider mà không còn `Channel` nào hiển thị chúng — với BYOK đó là tiền người
 * dùng trả cho một kết quả không ai thấy được và không ai đưa sang bản dịch được nữa (AD-22).
 * Không có lượt nào đang chạy (`state.value !== 'generating'`) ⇒ không gửi gì — cùng khuôn
 * [`cancelAiTranslate`] ngay trên.
 */
export function resetAiTranslate(): void {
  if (state.value === 'generating') void cancelAiTranslateCall()

  sequence += 1
  state.value = 'not_configured'
  accumulatedText.value = ''
  error.value = null
  runSegmentId.value = null
}
