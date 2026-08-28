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

/**
 * **THÊM (2026-08-27, phán quyết Ice #3)** — một cặp `.atproj` cùng `work_id`, khớp
 * `commands::library::ConflictEntry` phía Rust, `snake_case`.
 */
export type ConflictEntry = {
  work_id: string
  /** Đường dẫn `.atproj` đang có mặt trong chỉ mục (mục ĐẦU, theo thứ tự quét đã sắp). */
  kept_path: string
  /** Đường dẫn `.atproj` trùng `work_id`, bị loại khỏi lượt ghi này. */
  duplicate_path: string
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
  /**
   * 🔵 SỬA (2026-08-27, phán quyết Ice #3) — đổi từ `number` sang `ConflictEntry[]`. AC4 nói
   * "phát hiện VÀ cảnh báo" — hai vế, và một con số nén không đủ dữ kiện cho vế "cảnh báo":
   * nó không nói được CHỖ NÀO trùng. `.length` vẫn cho con số cũ khi chỉ cần đếm (dòng
   * ba-con-số của màn hình).
   */
  conflicts: ConflictEntry[]
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
/**
 * 🔵 SỬA (2026-08-27, phán quyết Ice #3) — `conflicts` nay là một mảng, không một `number`.
 * Kiểm `Array.isArray` cộng hình dạng của MỤC ĐẦU (nếu có) — cùng mức chặt mà `orphans` đã
 * có ngay bên dưới (`Array.isArray`, không đào sâu từng phần tử): một mảng rỗng là hợp lệ
 * theo định nghĩa (không có xung đột nào), nên không có "mục đầu" nào để kiểm trong ca đó.
 */
function isConflictEntryArray(value: unknown): value is ConflictEntry[] {
  if (!Array.isArray(value)) return false
  if (value.length === 0) return true
  const first = value[0] as Partial<ConflictEntry>
  return (
    typeof first.work_id === 'string' &&
    typeof first.kept_path === 'string' &&
    typeof first.duplicate_path === 'string'
  )
}

function isRescanReport(value: unknown): value is RescanReport {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<RescanReport>
  return (
    typeof v.root === 'string' &&
    typeof v.root_missing === 'boolean' &&
    typeof v.indexed === 'number' &&
    isConflictEntryArray(v.conflicts) &&
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

/**
 * 🔵 **THÊM (2026-08-27, Story 5.4)** — một hàng của `library_work`, khớp
 * `commands::library::WorkRow` phía Rust, `snake_case`.
 */
export type WorkRow = {
  work_id: string
  atproj_path: string
  name: string
  source_lang: string
  genre: string
  created_at: string
  updated_at: string
  chapter_count: number
  status: string | null
  status_is_override: boolean
}

/** Kết quả một lượt liệt kê — khớp `commands::library::WorkListReport`. */
export type WorkListReport = {
  total: number
  matched: number
  works: WorkRow[]
}

/** Ba trạng thái cho `list_works`. */
export type WorkListResult = {
  report: WorkListReport | null
  error: IpcError | null
}

function isWorkRowArray(value: unknown): value is WorkRow[] {
  if (!Array.isArray(value)) return false
  if (value.length === 0) return true
  const first = value[0] as Partial<WorkRow>
  return (
    typeof first.work_id === 'string' &&
    typeof first.atproj_path === 'string' &&
    typeof first.name === 'string' &&
    typeof first.source_lang === 'string' &&
    typeof first.genre === 'string' &&
    typeof first.created_at === 'string' &&
    typeof first.updated_at === 'string' &&
    typeof first.chapter_count === 'number' &&
    (typeof first.status === 'string' || first.status === null) &&
    typeof first.status_is_override === 'boolean'
  )
}

function isWorkListReport(value: unknown): value is WorkListReport {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<WorkListReport>
  return typeof v.total === 'number' && typeof v.matched === 'number' && isWorkRowArray(v.works)
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/library.rs` (module `wire`). */
const CMD_RESCAN = 'library_rescan'
const CMD_CHOOSE_ROOT = 'library_choose_root'
const CMD_FORGET_ORPHAN = 'library_forget_orphan'
const CMD_LIST_WORKS = 'library_list_works'

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
    //
    // 🔴 `library_rescan` (Rust) trả `Result<RescanReport, IpcError>` -- KHÔNG `Option`.
    // `null` ở ĐÂY là một câu trả lời HỎNG (không phải "huỷ hộp thoại" -- lệnh này không mở
    // hộp thoại nào), nên `isRescanReport(null)` PHẢI trả `false` và rơi vào nhánh dưới. Vì
    // sao hàm này không dùng chung với `choose_root`: xem `callChooseRoot` ngay dưới.
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

/**
 * 🔵 THÊM (2026-08-27, vòng rà bốn lớp P1) — TÁCH khỏi `callRescan`, không dùng chung nữa.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT HÀM RIÊNG — BẢN TRƯỚC GHI SAI IM LẶNG CA "HUỶ HỘP THOẠI"
 * ─────────────────────────────────────────────────────────────────────────────
 * `library_choose_root` (Rust) trả `Result<Option<RescanReport>, IpcError>` -- `Ok(None)`
 * là ca HUỶ, đã ghi trong hợp đồng của `ChooseRootResult` NGAY TRÊN. Trên dây, `Ok(None)`
 * thành `null`. Bản trước gọi CHUNG `callRescan` cho cả `rescanLibrary` lẫn
 * `chooseLibraryRoot` -- `isRescanReport(null)` trả `false` (nó loại `null` ngay dòng đầu,
 * ĐÚNG cho `library_rescan` nơi `null` là một câu trả lời hỏng), nên nhánh "SAI HÌNH DẠNG"
 * chạy nhầm trên một `null` HỢP LỆ ⇒ người dùng bấm Huỷ nhận `err.unknown` -- một lỗi THẬT,
 * đi tới màn hình -- ngược đúng §I/O Matrix ("Huỷ hộp thoại ⇒ không một biến thể lỗi").
 *
 * ⇒ `null` được xử lý làm ca HUỶ TRƯỚC KHI chạm `isRescanReport` — hàm đó giữ nguyên vai
 * "hình dạng sai là hình dạng sai" cho `callRescan`, không bị nới để nuốt luôn ca hợp lệ.
 */
async function callChooseRoot(cmd: string, args?: Record<string, unknown>): Promise<ChooseRootResult> {
  try {
    const report = await invoke<RescanReport | null>(cmd, args)
    if (report === null) {
      // Huỷ hộp thoại -- Ok(None), KHÔNG một biến thể lỗi (§I/O Matrix).
      return { report: null, error: null }
    }
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
  return callChooseRoot(CMD_CHOOSE_ROOT)
}

/**
 * Gỡ một mục mồ côi khỏi chỉ mục — lệnh `library.forget_orphan`.
 *
 * 🔵 THÊM tham số `name` (2026-08-27, vòng rà THỨ HAI P9) — `err.library.not_orphaned` nay
 * cần cả `work_id` LẪN `name` để nói được TÊN mục cho người dùng, không chỉ một UUID trần.
 * `name` đã có sẵn ở CHỖ GỌI (đang hiển thị trên màn hình), nên gửi thẳng nó đi thay vì để
 * Rust phải tra lại.
 */
export async function forgetLibraryOrphan(workId: string, name: string): Promise<ForgetOrphanResult> {
  try {
    const orphans = await invoke<OrphanEntry[]>(CMD_FORGET_ORPHAN, { workId, name })
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

/**
 * 🔵 **THÊM (2026-08-27, Story 5.4)** — liệt kê + lọc Tác phẩm cho khối "Tác phẩm" của
 * Library — lệnh `library.list_works`. `filter` rỗng/`undefined` ⇒ không lọc (mọi hàng, kể
 * cả hàng `status` chưa biết).
 */
export async function listLibraryWorks(filter?: readonly string[]): Promise<WorkListResult> {
  try {
    const report = await invoke<WorkListReport>(CMD_LIST_WORKS, {
      filter: filter === undefined || filter.length === 0 ? null : filter,
    })
    if (!isWorkListReport(report)) {
      console.error(`[library] \`${CMD_LIST_WORKS}\` trả một WorkListReport SAI HÌNH DẠNG: ${JSON.stringify(report)}`)
      return { report: null, error: UNKNOWN_IPC_ERROR }
    }
    return { report, error: null }
  } catch (err) {
    if (isIpcError(err)) return { report: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[library] \`${CMD_LIST_WORKS}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { report: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[library] không gọi được \`${CMD_LIST_WORKS}\` — chạy ngoài Tauri? ${String(err)}`)
    return { report: null, error: null }
  }
}
