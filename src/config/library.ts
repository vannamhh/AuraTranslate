/**
 * Adapter IPC phía webview cho "Quét lại thư mục" — Story 5.3, FR99. Adapter thứ TÁM
 * (`bootstrap` · `chapter` · `dict` · `glossary` · `pinned` · `project` · `segment` · `library`).
 *
 * Cùng khuôn `./project.ts`: một `invoke`, một `try/catch`, hình dạng BA TRẠNG THÁI
 * `{ <giá trị> | null, error: IpcError | null }` — KHÔNG BAO GIỜ ném. Tầng UI
 * (`src/modes/libraryRescan.ts`) hiển thị lỗi bằng `tError()`, không bằng `try/catch`.
 *
 * ⚠️ `invoke()` gửi tham số dạng **camelCase** dù hàm Rust nhận `snake_case`
 * (`commands/library.rs` không đổi `ArgumentCase` mặc định) ⇒ `workId`, không `work_id`.
 * **Nhưng trường của struct TRẢ VỀ giữ nguyên `snake_case`** (`work_id`, `atproj_path`) — hai
 * chiều khác nhau, chỗ dễ sai nhất trên dây (`src/AGENTS.md`).
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Một hàng mồ côi — khớp `commands::library::OrphanEntry` phía Rust, `snake_case`. */
export type OrphanEntry = {
  work_id: string
  name: string
  atproj_path: string
}

/** Kết quả một lượt quét — khớp `commands::library::RescanReport`. */
export type RescanReport = {
  /** Thư mục gốc VỪA quét — chỉ Rust biết bộ phân giải (móc e2e ⇒ cấu hình ⇒ mặc định) đã
   * chọn đường nào. */
  root: string
  /**
   * 🔵 THÊM (2026-08-27, vòng rà bốn lớp P1) — `true` ⇒ `root` KHÔNG tồn tại trên đĩa ở lượt
   * quét này. Phân biệt với `indexed === 0 && root_missing === false` (gốc CÓ tồn tại nhưng
   * rỗng thật) — hai trạng thái khác nhau mà một con số `0` một mình không nói được
   * (`AGENTS.md::Known pitfalls` — "danh sách rỗng phải nói vì sao rỗng").
   */
  root_missing: boolean
  indexed: number
  conflicts: number
  skipped: number
  orphans: OrphanEntry[]
}

/** Ba trạng thái, cùng khuôn `CreateWorkResult`. */
export type RescanResult = {
  report: RescanReport | null
  error: IpcError | null
}

/** Ba trạng thái cho một lệnh trả về danh sách mồ côi còn lại (`forget_orphan`). */
export type ForgetOrphanResult = {
  orphans: OrphanEntry[] | null
  error: IpcError | null
}

/**
 * Ba trạng thái cho `choose_root` — `report: null, error: null` gộp CẢ HAI ca "huỷ hộp
 * thoại" (§I/O Matrix: "Huỷ hộp thoại là `Ok(None)`, không một biến thể lỗi") VÀ "không có
 * cầu IPC", đúng khuôn `CreateWorkResult` đã chọn cho `project.ts` — không một hạng thứ ba.
 */
export type ChooseRootResult = RescanResult

/**
 * 🔵 THÊM (2026-08-27, vòng rà bốn lớp P1) — kiểm KIỂU LÚC CHẠY cho `RescanReport` qua dây,
 * cùng luật chung của kho ("Luôn kiểm kiểu LÚC CHẠY cho dữ liệu qua dây", `src/AGENTS.md`):
 * `IpcError` phía TS là một LỜI KHAI về dữ liệu đã qua IPC, không phải bảo đảm của trình
 * biên dịch -- Rust có thể trả thiếu một trường sau một lượt đổi lược đồ mà không tệp
 * `.ts` nào ở đây biết trước, và guard là chỗ DUY NHẤT phát hiện điều đó.
 */
function isRescanReport(value: unknown): value is RescanReport {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<RescanReport>
  return (
    typeof v.root === 'string' &&
    typeof v.root_missing === 'boolean' &&
    typeof v.indexed === 'number' &&
    typeof v.conflicts === 'number' &&
    typeof v.skipped === 'number' &&
    Array.isArray(v.orphans)
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
    // ⚠️ Cùng lý do `config/project.ts`: `IpcError` là một LỜI KHAI về dữ liệu qua IPC,
    // không một bảo đảm của trình biên dịch.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/library.rs` (module `wire`). */
const CMD_RESCAN = 'library_rescan'
const CMD_CHOOSE_ROOT = 'library_choose_root'
const CMD_FORGET_ORPHAN = 'library_forget_orphan'

/** Cùng khuôn `config/project.ts::hasIpcBridge`. */
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

async function callRescan(cmd: string, args?: Record<string, unknown>): Promise<RescanResult> {
  try {
    const report = await invoke<RescanReport>(cmd, args)
    // P1 -- không tin thẳng kiểu generic của `invoke<T>()`: đó là một khai báo phía TS,
    // không một bảo đảm từ Rust. Hình dạng sai (thiếu `root_missing`, kiểu lệch) ⇒ hồi
    // phòng bằng lỗi THẬT, không chuyển tiếp một giá trị đã biết là sai hình dạng.
    if (!isRescanReport(report)) {
      console.error(`[library] \`${cmd}\` trả một RescanReport SAI HÌNH DẠNG: ${JSON.stringify(report)}`)
      return { report: null, error: UNKNOWN_IPC_ERROR }
    }
    return { report, error: null }
  } catch (err) {
    if (isIpcError(err)) return { report: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[library] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { report: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[library] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
    return { report: null, error: null }
  }
}

/** Quét lại thư mục gốc đang cấu hình — lệnh `library.rescan`. */
export async function rescanLibrary(): Promise<RescanResult> {
  return callRescan(CMD_RESCAN)
}

/**
 * Mở hộp thoại chọn thư mục, đổi thư mục gốc rồi quét lại ngay trên đó — lệnh
 * `library.choose_root`. `report: null, error: null` == huỷ hộp thoại HOẶC không cầu IPC —
 * hai ca cùng một hình dạng ba trạng thái (§I/O Matrix của story: huỷ hộp thoại không phải
 * một biến thể lỗi).
 */
export async function chooseLibraryRoot(): Promise<ChooseRootResult> {
  return callRescan(CMD_CHOOSE_ROOT)
}

/** Gỡ một mục mồ côi khỏi chỉ mục — lệnh `library.forget_orphan`. */
export async function forgetLibraryOrphan(workId: string): Promise<ForgetOrphanResult> {
  try {
    const orphans = await invoke<OrphanEntry[]>(CMD_FORGET_ORPHAN, { workId })
    return { orphans, error: null }
  } catch (err) {
    if (isIpcError(err)) return { orphans: null, error: err }
    if (hasIpcBridge()) {
      console.error(
        `[library] \`${CMD_FORGET_ORPHAN}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { orphans: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[library] không gọi được \`${CMD_FORGET_ORPHAN}\` — chạy ngoài Tauri? ${String(err)}`)
    return { orphans: null, error: null }
  }
}
