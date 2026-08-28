/**
 * Adapter IPC phía webview cho "Bốn trạng thái vòng đời" — Story 5.4, FR5/FR6. Adapter thứ
 * CHÍN (`bootstrap` · `chapter` · `dict` · `glossary` · `pinned` · `project` · `segment` ·
 * `library` · `lifecycle`).
 *
 * Cùng khuôn `./chapter.ts`: một `invoke`, một `try/catch`, hình dạng BA TRẠNG THÁI
 * `{ <giá trị> | null, error: IpcError | null }` — KHÔNG BAO GIỜ ném. Tầng UI
 * (`src/modes/libraryWorks.ts`) hiển thị lỗi bằng `tError()`, không bằng `try/catch`.
 *
 * ⚠️ `invoke()` gửi tham số dạng **camelCase** dù hàm Rust nhận `snake_case`
 * (`commands/lifecycle.rs` không đổi `ArgumentCase` mặc định) ⇒ `chapterId`, không
 * `chapter_id`. **Nhưng trường của struct TRẢ VỀ giữ nguyên `snake_case`**
 * (`status_is_override`) — hai chiều khác nhau, chỗ dễ sai nhất trên dây (`src/AGENTS.md`).
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Hình dạng `WorkLifecycle` phía Rust — khớp `commands::lifecycle::WorkLifecycle`, `snake_case`. */
export type WorkLifecycle = {
  status: string | null
  status_is_override: boolean
}

/** Ba trạng thái, cùng khuôn `ReadOpenChapterResult` (`./chapter.ts`). */
export type WorkLifecycleResult = {
  lifecycle: WorkLifecycle | null
  error: IpcError | null
}

function isWorkLifecycle(value: unknown): value is WorkLifecycle {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<WorkLifecycle>
  return (
    (typeof v.status === 'string' || v.status === null) && typeof v.status_is_override === 'boolean'
  )
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // ⚠️ `IpcError` là một LỜI KHAI về dữ liệu qua IPC, không một bảo đảm của trình biên
    // dịch — cùng chú thích ở mọi adapter khác của kho.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/lifecycle.rs` (module `wire`). */
const CMD_READ = 'read_work_lifecycle'
const CMD_SET_CHAPTER_STATUS = 'set_chapter_status'
const CMD_SET_WORK_OVERRIDE = 'set_work_status_override'

/** Cùng khuôn `config/chapter.ts::hasIpcBridge`. */
function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/** Lỗi hồi phòng khi Rust trượt bằng một thứ không phải `IpcError`. */
const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

async function callLifecycle(cmd: string, args?: Record<string, unknown>): Promise<WorkLifecycleResult> {
  try {
    const lifecycle = await invoke<WorkLifecycle>(cmd, args)
    // Không tin thẳng kiểu generic của `invoke<T>()` — kiểm KIỂU LÚC CHẠY, cùng luật chung
    // của kho (`src/AGENTS.md`): `IpcError` phía TS là một lời khai, không một bảo đảm.
    if (!isWorkLifecycle(lifecycle)) {
      console.error(`[lifecycle] \`${cmd}\` trả một WorkLifecycle SAI HÌNH DẠNG: ${JSON.stringify(lifecycle)}`)
      return { lifecycle: null, error: UNKNOWN_IPC_ERROR }
    }
    return { lifecycle, error: null }
  } catch (err) {
    if (isIpcError(err)) return { lifecycle: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[lifecycle] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { lifecycle: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[lifecycle] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
    return { lifecycle: null, error: null }
  }
}

/** Đọc trạng thái vòng đời của Tác phẩm đang mở — lệnh `read_work_lifecycle`. */
export async function readWorkLifecycle(): Promise<WorkLifecycleResult> {
  return callLifecycle(CMD_READ)
}

/**
 * Đặt trạng thái vòng đời của MỘT Chương — lệnh `set_chapter_status`. `status` là một trong
 * bốn giá trị trên dây (`"not_started"` · `"in_progress"` · `"paused"` · `"done"`).
 */
export async function setChapterStatus(chapterId: number, status: string): Promise<WorkLifecycleResult> {
  return callLifecycle(CMD_SET_CHAPTER_STATUS, { chapterId, status })
}

/**
 * Ghi đè (hoặc bỏ ghi đè) trạng thái vòng đời của Tác phẩm đang mở — lệnh
 * `set_work_status_override`. `status = null` ⇒ bỏ ghi đè, đọc lại giá trị SUY RA hiện thời.
 */
export async function setWorkStatusOverride(status: string | null): Promise<WorkLifecycleResult> {
  return callLifecycle(CMD_SET_WORK_OVERRIDE, { status })
}
