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

// 🔴 SỬA (vòng rà đối kháng 2, mục 5) — `createWorkFromText`/`createWorkFromFile` (adapter
// TS cho vỏ `wire::create_work_from_text`/`wire::create_work_from_file`) BỊ XOÁ khỏi đây.
// Từ Story 6.3, nộp form KHÔNG còn gọi thẳng chúng (`libraryImport.ts` đi qua màn xem
// trước bảng mã, `previewImportEncodingFromText`/`_FromFile` rồi `confirmImportWithEncoding`
// — xem `src/modes/libraryImport.ts::submitPastedText`/`submitFilePath`). Đo (2026-09-04):
// `grep` toàn `src/`, `tests/frontend/` cho 0 chỗ gọi PRODUCT nào của hai hàm này — chỉ hai
// chú thích còn nhắc TÊN chúng.
//
// ⚠️ **VỎ RUST (`wire::create_work_from_text`/`wire::create_work_from_file`) KHÔNG bị xoá**
// — chúng vẫn đăng ký trong `generate_handler!` (`lib.rs:638-639`) và vẫn là hạ tầng test
// SỐNG: 15+ tệp `e2e/specs/**` gọi thẳng `internals.invoke('create_work_from_text', {...})`
// để dựng fixture nhanh (đi ĐƯỜNG IPC trực tiếp, cố ý BỎ QUA UI — xem
// `e2e/support/workspace.mjs`). Xoá vỏ Rust sẽ phá TOÀN BỘ hạ tầng fixture đó.
//
// ⇒ Quyết định: adapter TS (lớp DUY NHẤT một dòng frontend sản phẩm tương lai có thể gọi
// mà không cố ý) bị xoá — thu hẹp bề mặt có thể bị lạm dụng xuống còn "gọi thẳng
// `invoke('create_work_from_text', …)` bằng tay", một hành động rõ ràng có chủ ý, không
// phải một lượt `import` tình cờ. Vỏ Rust ở lại, có tên, có lý do, không phải xác chết.
//
// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.3 — màn xem trước bảng mã (FR126). Khớp `commands::project::{
// ImportEncodingPreview, EncodingCandidateWire, ConfidenceWire, wire::preview_import_encoding_from_text,
// wire::preview_import_encoding_from_file, wire::confirm_import_with_encoding }`.
// ═══════════════════════════════════════════════════════════════════════════════

/** Ba trạng thái tin cậy — DỮ LIỆU (AD-21), Rust không gửi câu. `snake_case` đúng như trên
 * dây (`#[serde(rename_all = "snake_case")]` phía `ConfidenceWire`). */
export type ImportConfidence = 'self_declared' | 'high' | 'low'

/** Một ô trong dải năm ứng viên — khớp `commands::project::EncodingCandidateWire`. */
export type EncodingCandidateWire = {
  label: string
  encoding: string
  preview: string | null
}

/** Kết quả một lượt xem trước bảng mã — khớp `commands::project::ImportEncodingPreview`. */
export type ImportEncodingPreview = {
  confidence: ImportConfidence
  selected_encoding: string
  candidates: EncodingCandidateWire[]
}

/** Ba trạng thái, cùng khuôn `CreateWorkResult`. */
export type ImportEncodingPreviewResult = {
  preview: ImportEncodingPreview | null
  error: IpcError | null
}

const CMD_PREVIEW_FROM_TEXT = 'preview_import_encoding_from_text'
const CMD_PREVIEW_FROM_FILE = 'preview_import_encoding_from_file'
const CMD_CONFIRM_WITH_ENCODING = 'confirm_import_with_encoding'

function isEncodingCandidateWire(value: unknown): value is EncodingCandidateWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<EncodingCandidateWire>
  return (
    typeof v.label === 'string' &&
    typeof v.encoding === 'string' &&
    (v.preview === null || typeof v.preview === 'string')
  )
}

// 🔴 SỬA (vòng rà đối kháng 2, mục 23) — bản trước chỉ hỏi `Array.isArray(v.candidates)`,
// không hỏi PHẦN TỬ của mảng có đúng hình dạng không. Một mảng `[{}]` (hoặc bất kỳ rác nào)
// đi lọt Kiểm TYPE này rồi `undefined` hiện thẳng lên dải (`candidate.label`/`.encoding` đọc
// ra `undefined`, `ImportPreviewOverlay.vue` không có nhánh nào xử) — đúng lớp "kiểm kiểu
// LÚC CHẠY hờ hững" mà `src/AGENTS.md` cảnh báo.
function isImportEncodingPreview(value: unknown): value is ImportEncodingPreview {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ImportEncodingPreview>
  return (
    (v.confidence === 'self_declared' || v.confidence === 'high' || v.confidence === 'low') &&
    typeof v.selected_encoding === 'string' &&
    Array.isArray(v.candidates) &&
    v.candidates.every(isEncodingCandidateWire)
  )
}

async function callPreviewImportEncoding(
  cmd: string,
  args: Record<string, unknown>,
): Promise<ImportEncodingPreviewResult> {
  try {
    const preview = await invoke<ImportEncodingPreview>(cmd, args)
    if (!isImportEncodingPreview(preview)) {
      // 🔴 Kiểm kiểu LÚC CHẠY cho dữ liệu qua dây (`src/AGENTS.md`) — `IpcError` phía TS là
      // một lời khai, không phải bảo đảm của trình biên dịch.
      console.error(`[project] \`${cmd}\` tra ve mot hinh dang khong dung ImportEncodingPreview`)
      return { preview: null, error: UNKNOWN_IPC_ERROR }
    }
    return { preview, error: null }
  } catch (err) {
    if (isIpcError(err)) return { preview: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[project] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { preview: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
    return { preview: null, error: null }
  }
}

/** Nhánh DÁN VĂN BẢN của màn xem trước bảng mã (Story 6.3, FR126). */
export async function previewImportEncodingFromText(text: string): Promise<ImportEncodingPreviewResult> {
  return callPreviewImportEncoding(CMD_PREVIEW_FROM_TEXT, { text })
}

/** Nhánh TỆP của màn xem trước bảng mã (Story 6.3, FR126). */
export async function previewImportEncodingFromFile(path: string): Promise<ImportEncodingPreviewResult> {
  return callPreviewImportEncoding(CMD_PREVIEW_FROM_FILE, { path })
}

/** Xác nhận lượt nhập với bảng mã đã chọn — cùng hình dạng trả về `CreateWorkResult`
 * (`created`/`error`), vì lệnh này TẠO một Tác phẩm y hệt `create_work_from_text`/`_from_file`. */
export async function confirmImportWithEncoding(
  name: string,
  sourceLang: string,
  genre: string,
  encoding: string,
): Promise<CreateWorkResult> {
  return callCreateWork(CMD_CONFIRM_WITH_ENCODING, { name, sourceLang, genre, encoding })
}
