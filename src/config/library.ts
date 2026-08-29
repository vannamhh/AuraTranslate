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
 *
 * 🔵 **THÊM `chapter_done_count` (2026-08-28, Story 5.5)** — `number | null`, KHÔNG
 * `number | undefined`: `null` là câu trả lời THẬT của Rust cho "chưa biết" (một `meta.json`
 * chưa từng qua `WorkMeta::rebuild_from_store`), và một hàng thiếu hẳn trường này là hình
 * dạng SAI phải bị `isWorkRowArray` từ chối, không phải một hàng hợp lệ đọc lên như đã biết.
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
  chapter_done_count: number | null
}

/**
 * 🔵 **THÊM (2026-08-28, Story 5.6)** — khoá sắp xếp, danh mục ĐÓNG, khớp
 * `core::library::indexer::WorkSortKey::as_str()` phía Rust. Union ĐÓNG, không `string` trần
 * — một giá trị lạ phải bị TypeScript từ chối LÚC BIÊN DỊCH ở mọi chỗ gọi tĩnh, đúng cùng
 * mức chặt mà Rust áp cho `WorkSortKey::from_wire`.
 */
export type WorkSortKey = 'updated_desc' | 'name_asc'

/**
 * Kết quả một lượt liệt kê — khớp `commands::library::WorkListReport`.
 *
 * 🔵 **THÊM `genres`/`source_langs` (2026-08-28, Story 5.6)** — hai tập giá trị CÓ THẬT, đã
 * `DISTINCT` ở Rust trên bảng CHƯA LỌC (AD-1). `LibraryMode.vue` dựng `<option>` từ ĐÂY,
 * KHÔNG BAO GIỜ tự suy từ `works` (mảng đó ĐÃ bị lọc — suy từ nó làm lựa chọn TEO DẦN theo
 * mỗi lượt lọc, đúng lỗi mà `AD-1` cấm).
 */
export type WorkListReport = {
  total: number
  matched: number
  works: WorkRow[]
  genres: string[]
  source_langs: string[]
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
    typeof first.status_is_override === 'boolean' &&
    // 🔵 THÊM (2026-08-28, Story 5.5) — chấp nhận `number` HOẶC `null`, từ chối `undefined`
    // (một trường VẮNG MẶT trên dây là hình dạng SAI, không phải "chưa biết" — "chưa biết" là
    // `null`, một câu trả lời TƯỜNG MINH từ Rust).
    (typeof first.chapter_done_count === 'number' || first.chapter_done_count === null)
  )
}

/**
 * 🔵 **THÊM (2026-08-28, Story 5.6)** — kiểm KIỂU LÚC CHẠY cho một mảng chuỗi trên dây. Không
 * đào sâu TỪNG phần tử (cùng mức chặt mà `isWorkRowArray` áp cho `orphans`/`conflicts` ở các
 * adapter khác của kho) — chỉ cần biết đây THẬT SỰ là một mảng chuỗi, không phải `undefined`
 * hay một hình dạng khác lọt qua.
 */
function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string')
}

function isWorkListReport(value: unknown): value is WorkListReport {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<WorkListReport>
  return (
    typeof v.total === 'number' &&
    typeof v.matched === 'number' &&
    isWorkRowArray(v.works) &&
    isStringArray(v.genres) &&
    isStringArray(v.source_langs)
  )
}

/**
 * 🔵 **THÊM Story 5.7.** Kết quả `open_work` — khớp `commands::project::wire::OpenedWork`
 * phía Rust, `snake_case`. Cùng hình dạng `CreatedWork` (`./project.ts`) cộng `chapter_id`:
 * mở lại một Tác phẩm đã có luôn kèm Chương nó sẽ mở (Chương đầu theo `(ord, id)`).
 */
export type OpenedWork = {
  meta: import('./project').WorkMeta
  folder: string
  chapter_id: number
}

/** Ba trạng thái, cùng khuôn [`WorkListResult`]. */
export type OpenWorkResult = {
  opened: OpenedWork | null
  error: IpcError | null
}

/**
 * 🔵 **THÊM Story 5.7.** Vị từ kiểm kiểu LÚC CHẠY cho `OpenedWork` — `IpcError` phía TS là
 * một lời khai về dữ liệu đã qua dây, không phải bảo đảm của trình biên dịch
 * (`src/AGENTS.md`). Không đào sâu `meta` (cùng mức chặt mà kho đã chấp nhận cho
 * `CreatedWork`/`WorkMeta` ở `./project.ts` — chưa adapter nào ở đó kiểm `WorkMeta` lúc
 * chạy) — chỉ kiểm hình dạng NGOÀI CÙNG mà story này thêm.
 */
function isOpenedWork(value: unknown): value is OpenedWork {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<OpenedWork>
  return (
    typeof v.meta === 'object' &&
    // ⚠️ Hình dạng qua IPC là một LỜI KHAI, không một bảo đảm của trình biên dịch: Rust có
    // thể trả `null` cho `meta` dù kiểu tĩnh nói `WorkMeta | undefined` — cùng lý do dòng
    // `v.params !== null` của `isIpcError` ở trên đã cần cùng một miễn trừ.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.meta !== null &&
    typeof v.folder === 'string' &&
    typeof v.chapter_id === 'number'
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/library.rs` (module `wire`). */
const CMD_RESCAN = 'library_rescan'
const CMD_CHOOSE_ROOT = 'library_choose_root'
const CMD_FORGET_ORPHAN = 'library_forget_orphan'
const CMD_LIST_WORKS = 'library_list_works'
/** 🔵 **THÊM Story 5.7.** Khớp `commands::project::wire::open_work`, KHÔNG `commands::library::*`
 * — lệnh này sống ở `commands/project.rs` vì nó ghi vào `OpenWorkState`. */
const CMD_OPEN_WORK = 'open_work'
/** 🔵 **THÊM Story 5.9.** Khớp `commands::library::wire::library_search`. */
const CMD_SEARCH = 'library_search'

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
 *
 * 🔵 **THÊM `genre`/`sourceLang`/`sort` (2026-08-28, Story 5.6)** — `genre`/`sourceLang`
 * `undefined` ⇒ không lọc lĩnh vực/ngôn ngữ nguồn tương ứng; `sort` `undefined` ⇒ mặc định
 * `updated_desc` phía Rust. `sourceLang` gửi camelCase trên dây (`src/AGENTS.md`) — vỏ Rust
 * nhận `source_lang`, Tauri tự ánh xạ.
 */
export async function listLibraryWorks(
  filter?: readonly string[],
  genre?: string,
  sourceLang?: string,
  sort?: WorkSortKey,
): Promise<WorkListResult> {
  try {
    const report = await invoke<WorkListReport>(CMD_LIST_WORKS, {
      filter: filter === undefined || filter.length === 0 ? null : filter,
      genre: genre ?? null,
      sourceLang: sourceLang ?? null,
      sort: sort ?? null,
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

/**
 * 🔵 **THÊM Story 5.7.** Mở lại một `.atproj` **đã có trên đĩa** (FR12). Không ném — cùng
 * lý do và cùng khuôn [`listLibraryWorks`].
 *
 * 🔴 Tham số là `workId` — KHÔNG một đường dẫn hệ tệp (§Never của story): `atproj_path`
 * phân giải Ở RUST, từ `library-index.db`.
 */
export async function openWork(workId: string): Promise<OpenWorkResult> {
  try {
    const opened = await invoke<unknown>(CMD_OPEN_WORK, { workId })
    if (!isOpenedWork(opened)) {
      console.error(`[library] \`${CMD_OPEN_WORK}\` trả một OpenedWork SAI HÌNH DẠNG: ${JSON.stringify(opened)}`)
      return { opened: null, error: UNKNOWN_IPC_ERROR }
    }
    return { opened, error: null }
  } catch (err) {
    if (isIpcError(err)) return { opened: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[library] \`${CMD_OPEN_WORK}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { opened: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[library] không gọi được \`${CMD_OPEN_WORK}\` — chạy ngoài Tauri? ${String(err)}`)
    return { opened: null, error: null }
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.9 — "TÌM KIẾM FULL-TEXT XUYÊN LIBRARY" (FR8)
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Nửa nào của một segment một hit khớp — khớp `core::library::indexer::SearchField::as_str()`
 * phía Rust. Union ĐÓNG, không `string` trần (cùng mức chặt `WorkSortKey`).
 */
export type SearchField = 'target' | 'source'

/**
 * Một kết quả tìm kiếm — khớp `commands::library::SearchHit` phía Rust, `snake_case`.
 *
 * `segment_id: null` ⇒ hit CẤP CHƯƠNG (Chương chưa tách segment sống nào) — lượt mở kết quả
 * (`src/modes/librarySearch.ts::openSearchHit`) truyền `null` xuống thành `undefined` cho
 * `openChapterById`, để Rust quyết con trỏ như mọi lượt mở Chương bình thường.
 */
export type SearchHit = {
  work_id: string
  work_name: string
  chapter_id: number
  chapter_ord: number
  chapter_title: string | null
  segment_id: number | null
  field: SearchField
  /** Đoạn trích văn bản THUẦN, cặp dấu `‹…›` bao quanh phần khớp — KHÔNG một thẻ HTML nào
   * (AD-16). Render bằng nội suy Vue thường (`{{ }}`), không bao giờ `v-html`. */
  snippet: string
}

/** Kết quả một lượt tìm kiếm — khớp `commands::library::SearchReport`. */
export type SearchReport = {
  hits: SearchHit[]
  /** `hits.length` — trường TƯỜNG MINH từ Rust, không suy từ `.length` phía TypeScript (AD-1). */
  total: number
  /** Tổng số hàng CÓ THẬT trong `library_segment`, KHÔNG phụ thuộc truy vấn — phân biệt "chỉ
   * mục chưa có dòng nào" (`0`) với "có dòng mà không khớp" (`> 0`). */
  indexed_segments: number
  /** `true` ⇔ truy vấn dưới 3 ký tự — nửa nguyên văn (trigram) KHÔNG chạy ở lượt này. */
  short_query: boolean
  /** 🔴 `true` ⇔ danh sách đã bị trần CẮT — `total` là *"số hàng đang hiện"*, không phải *"số
   * hàng khớp"*. Giao diện PHẢI nói ra điều đó: một trần cắt trong im lặng biến một con số
   * thành một lời khai sai. */
  truncated: boolean
}

/** Ba trạng thái cho [`searchLibrary`]. */
export type SearchLibraryResult = {
  report: SearchReport | null
  error: IpcError | null
}

function isSearchField(value: unknown): value is SearchField {
  return value === 'target' || value === 'source'
}

function isSearchHit(value: unknown): value is SearchHit {
  if (typeof value !== 'object' || value === null) return false
  const hit = value as Partial<SearchHit>
  return (
    typeof hit.work_id === 'string' &&
    typeof hit.work_name === 'string' &&
    typeof hit.chapter_id === 'number' &&
    typeof hit.chapter_ord === 'number' &&
    (typeof hit.chapter_title === 'string' || hit.chapter_title === null) &&
    (typeof hit.segment_id === 'number' || hit.segment_id === null) &&
    isSearchField(hit.field) &&
    typeof hit.snippet === 'string'
  )
}

/**
 * 🔴 **Kiểm MỌI phần tử, không chỉ phần tử đầu.** Bản đầu đọc `value[0]` rồi kết luận cho cả
 * mảng — một hàng thứ hai sai hình dạng (một lượt đổi lược đồ phía Rust chỉ chạm một nhánh, một
 * cột mới trả `null` ngoài dự kiến) đi thẳng qua cửa và vào `v-for` của `LibraryMode.vue`. Chi
 * phí là tuyến tính trên tối đa `MAX_SEARCH_LIMIT` hàng, tức không đáng kể — còn cái giá của
 * cửa hở là đúng thứ `src/AGENTS.md` gọi tên: *"`IpcError` phía TS là một LỜI KHAI về dữ liệu
 * đã đi qua IPC, không phải bảo đảm của trình biên dịch"*.
 */
function isSearchHitArray(value: unknown): value is SearchHit[] {
  return Array.isArray(value) && value.every(isSearchHit)
}

function isSearchReport(value: unknown): value is SearchReport {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<SearchReport>
  return (
    isSearchHitArray(v.hits) &&
    typeof v.total === 'number' &&
    typeof v.indexed_segments === 'number' &&
    typeof v.short_query === 'boolean' &&
    typeof v.truncated === 'boolean'
  )
}

/**
 * Tìm kiếm full-text xuyên TOÀN BỘ Library — lệnh `library.search` (FR8). Không ném — cùng lý
 * do và cùng khuôn [`listLibraryWorks`]. `query` KHÔNG được `trim()` ở đây — chỗ gọi
 * (`src/modes/librarySearch.ts`) quyết định khi nào PHÁT lượt IPC này (§I/O Matrix: "truy vấn
 * rỗng ⇒ 0 lượt IPC").
 */
export async function searchLibrary(query: string, limit?: number): Promise<SearchLibraryResult> {
  try {
    const report = await invoke<unknown>(CMD_SEARCH, { query, limit: limit ?? null })
    if (!isSearchReport(report)) {
      console.error(`[library] \`${CMD_SEARCH}\` trả một SearchReport SAI HÌNH DẠNG: ${JSON.stringify(report)}`)
      return { report: null, error: UNKNOWN_IPC_ERROR }
    }
    return { report, error: null }
  } catch (err) {
    if (isIpcError(err)) return { report: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[library] \`${CMD_SEARCH}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { report: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[library] không gọi được \`${CMD_SEARCH}\` — chạy ngoài Tauri? ${String(err)}`)
    return { report: null, error: null }
  }
}
