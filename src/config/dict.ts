/**
 * Adapter IPC phía webview cho âm Hán Việt — Story 1.16, AC4.
 *
 * Cùng khuôn `./chapter.ts`/`./project.ts`: một lời gọi `invoke`, một `try/catch`, ⛔
 * không quy tắc nghiệp vụ nào ở đây — quy tắc sống ở Rust (`commands/dict.rs`,
 * `core/dict/mod.rs::lookup_han_viet`).
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Hình dạng `HanVietReading` phía Rust — `snake_case`, đúng như trên dây. */
export type HanVietReading = {
  primary: string
  all: string[]
  source_code: string
}

/** Hình dạng `CharacterReading` phía Rust. */
export type CharacterReading = {
  character: string
  reading: HanVietReading | null
}

/**
 * Hình dạng `HanVietLookup` phía Rust — khớp `commands::dict::wire::read_han_viet`.
 *
 * ⚠️ `layers_loaded === false` là trạng thái BÌNH THƯỜNG có tên (⛔ không lớp từ điển nào
 * đang gắn — AD-25), KHÁC với "đã tra mà ký tự không có âm" (`reading === null` trong khi
 * `layers_loaded === true`). Ba trạng thái, ⛔ không phải hai — xem AC4.
 */
export type HanVietLookup = {
  characters: CharacterReading[]
  sources_used: string[]
  layers_loaded: boolean
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

/** Tên command trên dây. Khớp `src-tauri/src/commands/dict.rs` (module `wire`). */
const CMD_READ_HAN_VIET = 'read_han_viet'

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/**
 * Kết quả của một lượt đọc âm Hán Việt. Ba trạng thái, cùng khuôn `ReadOpenChapterResult`.
 *
 * ⚠️ Đây ⛔ không phải `{ characters: [], ... }` khi lỗi — Rust cố ý ⛔ không có nhánh lỗi
 * cho command này (xem doc-comment `commands::dict::read_han_viet`), nên `error` chỉ khác
 * `null` khi Rust trượt bằng một thứ **không phải** hình dạng mong đợi (panic, sai tên
 * tham số) hoặc khi ⛔ không có cầu IPC nào cả.
 */
export type ReadHanVietResult = {
  lookup: HanVietLookup | null
  error: IpcError | null
}

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

/**
 * Đọc âm Hán Việt cho `chars`. ⛔ Không ném — cùng lý do `readOpenChapter` không ném.
 *
 * ⚠️ Chỗ gọi chịu trách nhiệm chỉ gọi hàm này khi `source_lang === 'zh'` (AC3) — adapter
 * ⛔ không tự phán xét, cùng nguyên tắc "hàm thuần không tự quyết chính sách sản phẩm".
 */
export async function readHanViet(chars: readonly string[]): Promise<ReadHanVietResult> {
  try {
    const lookup = await invoke<HanVietLookup>(CMD_READ_HAN_VIET, { chars })
    return { lookup, error: null }
  } catch (err) {
    if (isIpcError(err)) return { lookup: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[dict] \`${CMD_READ_HAN_VIET}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { lookup: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[dict] không gọi được \`${CMD_READ_HAN_VIET}\` — chạy ngoài Tauri? ${String(err)}`)
    return { lookup: null, error: null }
  }
}
