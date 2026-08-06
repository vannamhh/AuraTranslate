/**
 * Adapter IPC phía webview cho việc đọc Chương **đang mở** — Story 1.16, AC8.
 *
 * Cùng khuôn `./project.ts`: một lời gọi `invoke`, một `try/catch`, ⛔ không quy tắc
 * nghiệp vụ nào ở đây — quy tắc sống ở Rust (`commands/chapter.rs`).
 *
 * ⚠️ `invoke()` mặc định gửi tham số ở dạng **camelCase**; hàm này ⛔ không nhận tham số
 * nào nên điều đó không áp dụng ở đây, nhưng vẫn ghi ra để chỗ gọi sau (nếu Epic 2 thêm
 * tham số chọn Chương) không quên.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/**
 * Hình dạng `OpenChapter` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/chapter.rs::OpenChapter` cố ý KHÔNG đặt `#[serde(rename_all = "camelCase")]`
 * — cùng luật với `WorkMeta`/`CreatedWork` ở `./project.ts`.
 */
export type OpenChapter = {
  chapter_id: number
  source_text: string
  /** `"zh"` hoặc `"en"` — trường bất biến của Tác phẩm (AD-18), ⛔ không đoán từ nội dung. */
  source_lang: string
}

/** Ba trạng thái, cùng khuôn `CreateWorkResult`/`BootstrapResult` — xem doc-comment ở đó. */
export type ReadOpenChapterResult = {
  chapter: OpenChapter | null
  error: IpcError | null
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/chapter.rs` (module `wire`). */
const CMD_READ_OPEN_CHAPTER = 'read_open_chapter'

/**
 * Có cầu IPC của Tauri trong window này không — cùng khuôn `./project.ts::hasIpcBridge`.
 */
function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/** Lỗi hồi phòng khi Rust trượt bằng một thứ ⛔ không phải `IpcError`. */
const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

/**
 * Đọc Chương đang mở. ⛔ Không ném — cùng lý do `callCreateWork` không ném
 * (`./project.ts`): chỗ gọi hiển thị lỗi bằng `tError()`, ⛔ không bằng `try/catch` ở
 * tầng UI.
 */
export async function readOpenChapter(): Promise<ReadOpenChapterResult> {
  try {
    const chapter = await invoke<OpenChapter>(CMD_READ_OPEN_CHAPTER)
    return { chapter, error: null }
  } catch (err) {
    if (isIpcError(err)) return { chapter: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ lỗi THẬT (command
    // chưa đăng ký, một panic phía Rust), ⛔ KHÔNG phải "chạy ngoài Tauri" — cùng bài học
    // đã ghi ở `./project.ts::callCreateWork`.
    if (hasIpcBridge()) {
      console.error(
        `[chapter] \`${CMD_READ_OPEN_CHAPTER}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { chapter: null, error: UNKNOWN_IPC_ERROR }
    }

    // ⛔ Không có cầu IPC — `npm run dev` trong một trình duyệt thường. ⛔ Không phải một
    // lỗi để hiện lên.
    console.info(`[chapter] không gọi được \`${CMD_READ_OPEN_CHAPTER}\` — chạy ngoài Tauri? ${String(err)}`)
    return { chapter: null, error: null }
  }
}
