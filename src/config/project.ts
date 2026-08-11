/**
 * Adapter IPC phía webview cho việc tạo một Tác phẩm — Story 1.15, AC1/AC8.
 *
 * Cùng khuôn `./bootstrap.ts`: một lời gọi `invoke`, một `try/catch`, không quy tắc
 * nghiệp vụ nào ở đây — quy tắc sống ở Rust (`core/segment/import.rs`,
 * `commands/project.rs`).
 *
 * ⚠️ `invoke()` mặc định gửi tham số ở dạng **camelCase** dù hàm Rust nhận `snake_case`
 * (`tauri-macros` `ArgumentCase::Camel` — mặc định, `commands/project.rs` không đổi nó).
 * ⇒ `sourceLang` ở lời gọi, không `source_lang`.
 *
 * ⚠️ Hàm ở đây **không bao giờ ném** — cùng lý do `loadBootstrapConfig` không ném:
 * chỗ gọi (`LibraryMode.vue`) hiển thị lỗi bằng `tError()`, không bằng một khối
 * `try/catch` ở tầng UI.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Hình dạng `WorkMeta` phía Rust — `snake_case`, đúng như trên dây. */
export type WorkMeta = {
  meta_schema_version: number
  work_id: string
  name: string
  source_lang: string
  genre: string
  created_at: string
  updated_at: string
  chapter_count: number
}

/**
 * Thứ hai lệnh trả về — khớp `commands::project::wire::CreatedWork` phía Rust.
 *
 * `folder` là đường dẫn tuyệt đối tới `<Tên>.atproj/`. Nó **không** suy ra được từ
 * `meta.name`: Rust thay ký tự cấm và thêm hậu tố ` (2)` khi trùng tên — xem
 * `core::library::atproj::create_work_folder`. AC6 cần con số này để giao được lời hứa
 * *"copy thư mục là đủ để sao lưu"*.
 */
export type CreatedWork = {
  meta: WorkMeta
  folder: string
}

/** Ba trạng thái, cùng khuôn `BootstrapResult` — xem doc-comment ở đó về vì sao ba. */
export type CreateWorkResult = {
  created: CreatedWork | null
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
    // ⚠️ Hình dạng `IpcError` là một LỜI KHAI về dữ liệu đã qua dây IPC, không một bảo đảm của trình
    //    biên dịch. Rust có thể trả `null` cho `params` sau một lượt đổi lược đồ, và guard này là chỗ
    //    duy nhất biết điều đó.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/project.rs` (module `wire`). */
const CMD_CREATE_FROM_TEXT = 'create_work_from_text'
const CMD_CREATE_FROM_FILE = 'create_work_from_file'

/**
 * Có cầu IPC của Tauri trong window này không.
 *
 * 🔴 Phép phân biệt này là **bắt buộc**, không phải một tinh chỉnh — xem
 * [`callCreateWork`]. Đọc trạng thái THẬT của môi trường, không phải một cờ ứng dụng tự
 * giữ (bài học §Trí tuệ #4 của story).
 */
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

async function callCreateWork(cmd: string, args: Record<string, unknown>): Promise<CreateWorkResult> {
  try {
    const created = await invoke<CreatedWork>(cmd, args)
    return { created, error: null }
  } catch (err) {
    if (isIpcError(err)) return { created: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ đây là một lỗi
    // THẬT (sai tên tham số, command chưa đăng ký, một panic phía Rust), KHÔNG phải
    // "chạy ngoài Tauri". Nuốt nó thành `{ null, null }` cho ra đúng hạng lỗi tệ nhất:
    // người dùng bấm "Tạo Tác phẩm", không có gì xảy ra, không một dòng nào hiện
    // ra — thất bại im lặng ở đúng thao tác đầu tiên. Code review 2026-08-06.
    if (hasIpcBridge()) {
      console.error(`[project] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { created: null, error: UNKNOWN_IPC_ERROR }
    }

    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Không phải một
    // lỗi để hiện lên (cùng nhánh với `loadBootstrapConfig`).
    console.info(`[project] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
    return { created: null, error: null }
  }
}

/** Nhánh dán văn bản của AC1. */
export async function createWorkFromText(
  name: string,
  sourceLang: string,
  genre: string,
  text: string,
): Promise<CreateWorkResult> {
  return callCreateWork(CMD_CREATE_FROM_TEXT, { name, sourceLang, genre, text })
}

/** Nhánh tệp của AC1 — kéo-thả **hoặc** ô nhập đường dẫn, cả hai gọi cùng một hàm này. */
export async function createWorkFromFile(
  name: string,
  sourceLang: string,
  genre: string,
  path: string,
): Promise<CreateWorkResult> {
  return callCreateWork(CMD_CREATE_FROM_FILE, { name, sourceLang, genre, path })
}
