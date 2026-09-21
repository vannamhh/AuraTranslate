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
 * Kết quả CUỐI của một lượt dịch — khớp NGUYÊN VĂN `AiTranslateOutcomeWire` phía Rust
 * (`#[serde(tag = "state", rename_all = "snake_case")]` trên một enum không mang trường nào).
 * BA giá trị, đúng ba biến thể Rust khai — `'generating'`/`'error'` không có mặt ở kiểu này,
 * xem doc-comment đầu tệp.
 */
export type AiTranslateOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'done' }
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
  const v = value as { state?: unknown }
  return v.state === 'not_configured' || v.state === 'done' || v.state === 'cancelled'
}

const CMD_TRANSLATE = 'ai_translate_segment'
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
 * Huỷ lượt dịch đang chạy — bơm bộ đếm thế hệ phía Rust lên một (`AiTranslateGeneration::next`),
 * làm thế hệ đang bay (nếu có) không còn là thế hệ hiện hành. Lệnh KHÔNG mang `Result` phía
 * Rust (`commands/aitranslate.rs::wire::ai_translate_cancel` trả `()`), nên không có nhánh
 * `IpcError` nào để phân biệt — **không bao giờ ném**, chỉ ghi chẩn đoán khi trượt.
 *
 * ⚠️ Không tham số nào đi trên dây: lệnh huỷ đúng lượt đang bay của TIẾN TRÌNH, không một
 * `segmentId` cụ thể — chỉ một lượt dịch chạy tại một thời điểm (`aiTranslateState.ts` từ
 * chối khởi một lượt mới trong khi một lượt khác đang `generating`).
 */
export async function cancelAiTranslateCall(): Promise<void> {
  try {
    await invoke<void>(CMD_CANCEL)
  } catch (err) {
    console.error(`[aitranslate] \`${CMD_CANCEL}\` trượt: ${String(err)}`)
  }
}
