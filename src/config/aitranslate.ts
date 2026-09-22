/**
 * Adapter IPC phía webview cho lượt **dịch một segment với kết quả chảy dần** — Story 4.8
 * (FR72/FR74, AD-22). Cùng khuôn `./aiprompt.ts`: một lời gọi `invoke`, một `try/catch`, kiểm
 * hình dạng LÚC CHẠY, không quy tắc nghiệp vụ nào ở đây — quy tắc sống ở Rust
 * (`commands/aitranslate.rs`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 PHẦN DUY NHẤT CỦA KHUÔN ADAPTER CHƯA CÓ TIỀN LỆ TRONG KHO — DỰNG `Channel`
 * ─────────────────────────────────────────────────────────────────────────────
 * `tauri::ipc::Channel<T>` (§Code Map spec 4.8) không giống mọi tham số khác của `invoke()`:
 * nó không phải dữ liệu JSON trần, nó là một đối tượng mang `onmessage` mà chính `invoke()`
 * biết tuần tự hoá (`[SERIALIZE_TO_IPC_FN]`). `runAiTranslateSegment` dựng NGAY một
 * `new Channel<string>(onToken)` rồi truyền nó làm tham số `channel` — mỗi token Rust gửi qua
 * `channel.send(text)` (`commands/aitranslate.rs::run_translate_call`) chạy thẳng `onToken`,
 * không một lượt `emit`/`listen` nào ở giữa (§Always spec 4.8: "grep tìm 0 emit/listen trên
 * đường này").
 *
 * ⚠️ `invoke()` gửi tham số ở dạng **camelCase** — `segment_id`/`prompt_set_name` phía Rust đi
 * trên dây thành `segmentId`/`promptSetName`, cùng luật `./aiprompt.ts`.
 *
 * ⚠️ Lệnh dịch trả về `Result<AiTranslateOutcomeWire, IpcError>` phía Rust — `generating`
 * **không có mặt** trong hình dạng này (§Code Map spec 4.8: nó là trạng thái webview tự giữ
 * TRONG LÚC lời gọi này chưa trả lời, xem `aiTranslateState.ts`), và `error` đi qua nhánh
 * `catch` của `invoke()`, không qua một biến thể nào ở đây.
 */
import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/**
 * Số liệu sử dụng + ước tính chi phí — Story 4.11. Khớp NGUYÊN VĂN `AiTranslateUsageWire`
 * phía Rust (`commands/aitranslate.rs`). `cost_usd: null` ⇔ `model_id` không có hàng trong
 * bảng giá bundled (`core::ai::pricing`) — mô hình cục bộ, hoặc một id bảng giá chưa được dạy;
 * đây LÀ quy tắc "mô hình cục bộ" (§Always spec 4.11), không một trường quên điền.
 */
export type AiTranslateUsageWire = {
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
  cost_usd: number | null
}

function isAiTranslateUsageWire(value: unknown): value is AiTranslateUsageWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<AiTranslateUsageWire>
  return (
    typeof v.prompt_tokens === 'number' &&
    typeof v.completion_tokens === 'number' &&
    typeof v.total_tokens === 'number' &&
    (v.cost_usd === null || typeof v.cost_usd === 'number')
  )
}

/**
 * Kết quả CUỐI của một lượt dịch — khớp NGUYÊN VĂN `AiTranslateOutcomeWire` phía Rust
 * (`#[serde(tag = "state", rename_all = "snake_case")]`). BA biến thể, đúng ba biến thể Rust
 * khai — `'generating'`/`'error'` không có mặt ở kiểu này, xem doc-comment đầu tệp.
 *
 * ⚠️ **SỬA (Story 4.11)** — `'done'` giờ mang `usage: AiTranslateUsageWire | null`: `null` ⇔
 * nhà cung cấp không trả về một khung `usage` nào cho lượt gọi này (I/O Matrix spec 4.11
 * "Provider sends no usage") — KHÔNG cùng nghĩa với `cost_usd: null` bên trong một `usage` CÓ
 * mặt (đó là "usage tới nhưng mô hình không có giá"). Hai lớp `null` khác nhau, cố ý.
 */
export type AiTranslateOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'done'; usage: AiTranslateUsageWire | null }
  | { state: 'cancelled' }

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích `./pinned.ts`
    v.params !== null
  )
}

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function isAiTranslateOutcomeWire(value: unknown): value is AiTranslateOutcomeWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as { state?: unknown; usage?: unknown }
  if (v.state === 'not_configured' || v.state === 'cancelled') return true
  if (v.state === 'done') return v.usage === null || isAiTranslateUsageWire(v.usage)
  return false
}

const CMD_TRANSLATE = 'ai_translate_segment'
const CMD_TRANSLATE_BATCH = 'ai_translate_batch'
const CMD_CANCEL = 'ai_translate_cancel'

/**
 * Chạy MỘT lượt dịch cho `segmentId` bằng bộ prompt hiệu lực `promptSetName`, gọi `onToken`
 * cho MỖI mảnh văn bản chảy tới qua `Channel` — không đợi lượt gọi trả lời mới thấy chữ đầu
 * tiên. **Không bao giờ ném.** `value: null` khi lượt gọi trượt bằng một thứ không phải
 * `IpcError` hoặc khi không có cầu Tauri — chỗ gọi PHẢI kiểm `error` trước khi dùng `value`,
 * cùng khuôn `aiPromptAssemble`.
 *
 * ⚠️ `onToken` có thể được gọi **không lần nào** (huỷ ngay lập tức, hoặc lượt gọi trượt trước
 * khung đầu tiên) — chỗ gọi không được giả định ít nhất một lần gọi.
 */
export async function runAiTranslateSegment(
  segmentId: number,
  promptSetName: string | null,
  onToken: (text: string) => void,
): Promise<{ value: AiTranslateOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<string>()
  channel.onmessage = onToken

  try {
    const wire = await invoke<unknown>(CMD_TRANSLATE, { segmentId, promptSetName, channel })
    if (!isAiTranslateOutcomeWire(wire)) {
      console.error(
        `[aitranslate] \`${CMD_TRANSLATE}\` tra ve mot hinh dang khong dung AiTranslateOutcomeWire`,
      )
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { value: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[aitranslate] \`${CMD_TRANSLATE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[aitranslate] không gọi được \`${CMD_TRANSLATE}\` — chạy ngoài Tauri? ${String(err)}`)
    return { value: null, error: null }
  }
}

/**
 * Một khung của `Channel` lô — Story 4.9, Phase 3 (FR73). Khớp NGUYÊN VĂN
 * `AiTranslateBatchEventWire` phía Rust (`commands/aitranslate.rs`,
 * `#[serde(tag = "kind", rename_all = "snake_case")]`) — BA biến thể, mỗi khung mang
 * `segment_id` của CHÍNH câu nó thuộc về, vì một `Channel` DUY NHẤT chở token của TOÀN lô
 * (§Never spec 4.9: "no second `Channel` per sentence").
 */
export type AiTranslateBatchEventWire =
  | { kind: 'token'; segment_id: number; text: string }
  | { kind: 'done'; segment_id: number; usage: AiTranslateUsageWire | null }
  | { kind: 'skipped'; segment_id: number }

/**
 * Kiểm hình dạng LÚC CHẠY — cùng luật `src/AGENTS.md`: "luôn kiểu-kiểm dữ liệu qua IPC lúc
 * chạy". Khác `onmessage` của lượt dịch MỘT segment (chỉ một `string` trần, không cần kiểm
 * hình dạng), mỗi khung ở đây là MỘT OBJECT — `Channel<T>` không tự kiểm `T` lúc chạy, nó chỉ
 * là một khai TypeScript phía webview.
 */
function isAiTranslateBatchEventWire(value: unknown): value is AiTranslateBatchEventWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as { kind?: unknown; segment_id?: unknown; text?: unknown; usage?: unknown }
  if (typeof v.segment_id !== 'number') return false
  if (v.kind === 'token') return typeof v.text === 'string'
  if (v.kind === 'done') return v.usage === null || isAiTranslateUsageWire(v.usage)
  return v.kind === 'skipped'
}

/**
 * Chạy MỘT lượt dịch theo LÔ cho `segmentIds` (đúng thứ tự tài liệu — chỗ gọi truyền
 * `segmentSelectionIds.value`, tệp này không sắp lại gì) bằng bộ prompt hiệu lực
 * `promptSetName`, gọi `onEvent` cho MỖI khung chảy tới qua `Channel` — cùng khuôn
 * [`runAiTranslateSegment`] (một `invoke`, không bao giờ ném, `{ value, error }`), chỉ khác
 * hình dạng khung mang thêm `segment_id`/`kind`.
 *
 * ⚠️ Một khung sai hình dạng bị BỎ QUA (chẩn đoán ra console), không làm sập lượt gọi —
 * cùng mức độ nghiêm khắc mà [`isAiTranslateOutcomeWire`] áp cho kết quả cuối, nhưng ở đây
 * bỏ một khung không ném vì các khung KHÁC của cùng lô vẫn còn ý nghĩa.
 */
export async function runAiTranslateBatchCall(
  segmentIds: number[],
  promptSetName: string | null,
  onEvent: (event: AiTranslateBatchEventWire) => void,
): Promise<{ value: AiTranslateOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<unknown>()
  channel.onmessage = (raw) => {
    if (!isAiTranslateBatchEventWire(raw)) {
      console.error(
        `[aitranslate] \`${CMD_TRANSLATE_BATCH}\` gui mot khung Channel khong dung AiTranslateBatchEventWire`,
      )
      return
    }
    onEvent(raw)
  }

  try {
    const wire = await invoke<unknown>(CMD_TRANSLATE_BATCH, { segmentIds, promptSetName, channel })
    if (!isAiTranslateOutcomeWire(wire)) {
      console.error(
        `[aitranslate] \`${CMD_TRANSLATE_BATCH}\` tra ve mot hinh dang khong dung AiTranslateOutcomeWire`,
      )
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { value: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[aitranslate] \`${CMD_TRANSLATE_BATCH}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[aitranslate] không gọi được \`${CMD_TRANSLATE_BATCH}\` — chạy ngoài Tauri? ${String(err)}`)
    return { value: null, error: null }
  }
}

/**
 * Huỷ lượt dịch đang chạy — bơm bộ đếm thế hệ phía Rust lên một (`AiTranslateGeneration::next`),
 * làm thế hệ đang bay (nếu có) không còn là thế hệ hiện hành. Lệnh KHÔNG mang `Result` phía
 * Rust (`commands/aitranslate.rs::wire::ai_translate_cancel` trả `()`), nên không có nhánh
 * `IpcError` nào để phân biệt — **không bao giờ ném**, chỉ ghi chẩn đoán khi trượt.
 *
 * ⚠️ Không tham số nào đi trên dây: lệnh huỷ đúng lượt đang bay của TIẾN TRÌNH, không một
 * `segmentId` cụ thể — chỉ một lượt dịch chạy tại một thời điểm, dù đó là MỘT segment
 * (`aiTranslateState.ts`) hay một LÔ (`aiTranslateBatchState.ts`, Story 4.9). 🔵 SỬA
 * (Story 4.9, Phase 3) — hàm này dùng CHUNG cho cả hai module state; chỗ gọi thật sự
 * (`main.ts`'s handler của `ai.translate.cancel`) đọc trạng thái CỦA CẢ HAI rồi chỉ gửi lệnh
 * huỷ khi ít nhất một trong hai đang `generating` — xem doc-comment tại đó.
 */
export async function cancelAiTranslateCall(): Promise<void> {
  try {
    await invoke<void>(CMD_CANCEL)
  } catch (err) {
    console.error(`[aitranslate] \`${CMD_CANCEL}\` trượt: ${String(err)}`)
  }
}
