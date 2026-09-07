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
// Story 6.4 (FR124/FR125) thêm `NormalizedPreviewWire` + trường `normalized` trên
// `EncodingCandidateWire` — không đổi ba lệnh dây, không thêm lệnh mới.
// ═══════════════════════════════════════════════════════════════════════════════

/** Ba trạng thái tin cậy — DỮ LIỆU (AD-21), Rust không gửi câu. `snake_case` đúng như trên
 * dây (`#[serde(rename_all = "snake_case")]` phía `ConfidenceWire`). */
export type ImportConfidence = 'self_declared' | 'high' | 'low'

/** Bản dựng đã chuẩn hoá của một ứng viên, cộng hai số đếm thiệt hại — khớp
 * `commands::project::NormalizedPreviewWire` (Story 6.4, FR124/FR125). */
export type NormalizedPreviewWire = {
  text: string
  joined_lines: number
  blank_lines_removed: number
  window_truncated: boolean
}

/** Nhãn tầng của một luật làm sạch — Story 6.5. Danh tính một luật trên dây là CẶP
 * `(tier, id)`, không phải `id` trần: hai tầng đánh số ĐỘC LẬP. */
export type CleanupRuleTierWire = 'global' | 'work'

/** Hai hình dạng mẫu — Story 6.5. */
export type CleanupRuleKindWire = 'literal' | 'regex'

/** Một luật kèm hai số đếm — khớp `commands::project::CleanupRuleReportWire`. */
export type CleanupRuleReportWire = {
  tier: CleanupRuleTierWire
  id: number
  pattern: string
  kind: CleanupRuleKindWire
  enabled: boolean
  count_in_chapter: number
  /** 🔵 Nợ có chủ (Story 6.6/6.7) — hôm nay LUÔN bằng `count_in_chapter`, xem doc-comment
   * Rust `CleanupRuleReportWire::count_in_import`. */
  count_in_import: number
}

/** Một chỗ khớp CỦA LUẬT ĐANG BẬT — điểm mã, nửa-mở `[start, end)`. Chỉ luật bật mới xuất
 * hiện ở đây (tắt một luật ⇒ chỗ vừa gạch ngang trở về nguyên trạng NGAY). */
export type CleanupSpanWire = {
  tier: CleanupRuleTierWire
  id: number
  start: number
  end: number
}

/** Hai hình dạng mẫu phân tách Chương — Story 6.6 (FR14). */
export type ChapterPatternKindWire = 'literal' | 'regex'

/** Mẫu phân tách Chương gửi lên Rust — tham số MỖI LƯỢT NHẬP (không lưu ở đâu cả giữa hai
 * lượt nhập, §Always spec 6.6). `null` ⇒ không mẫu, bước 5 no-op (N = 1). */
export type ChapterPatternInput = {
  pattern: string
  kind: ChapterPatternKindWire
}

/** Một Chương trong khối tách Chương (tầng 4) — khớp
 * `commands::project::ChapterSplitPreviewEntryWire`. */
export type ChapterSplitPreviewEntryWire = {
  ord: number
  /** Dòng khớp mẫu phân tách, `null` cho Chương lời tựa hoặc khi mẫu không khớp/chưa cấu
   * hình. */
  title: string | null
  /** Độ dài `source_text`, tính bằng ĐIỂM MÃ. */
  length: number
}

/** Khối tách Chương của MỘT ứng viên/đường tự khai — tầng 4 (Story 6.6). Mang TOÀN BỘ N
 * Chương (không chỉ ba đầu/ba cuối) — tầng hiển thị tự co gọn khung nhìn mặc định và mở
 * rộng khi sắp xếp theo độ dài. Khớp `commands::project::ChapterSplitPreviewWire`. */
export type ChapterSplitPreviewWire = {
  chapter_count: number
  chapters: ChapterSplitPreviewEntryWire[]
}

/** Khối làm sạch của MỘT ứng viên/đường tự khai — tầng 3 (Story 6.5). Khớp
 * `commands::project::CleanupPreviewWire`. */
export type CleanupPreviewWire = {
  /** Văn bản ĐÃ GIẢI MÃ, TRƯỚC khi luật xoá gì — thứ `spans` đánh dấu gạch ngang lên. */
  text: string
  spans: CleanupSpanWire[]
  rules: CleanupRuleReportWire[]
  /** `true` ⇒ `text` không phải TOÀN Chương. */
  window_truncated: boolean
  /** Văn bản CUỐI CÙNG (sau cả làm sạch VÀ chuẩn hoá) — khi `window_truncated === false`,
   * PHẢI giống hệt từng byte với `source_text` mà xác nhận ghi xuống (đóng nợ
   * `deferred-work.md:9359`). */
  final_text: string
}

/** Một ô trong dải năm ứng viên — khớp `commands::project::EncodingCandidateWire`. */
export type EncodingCandidateWire = {
  label: string
  encoding: string
  preview: string | null
  normalized: NormalizedPreviewWire | null
  /** Story 6.5 — khối làm sạch (tầng 3) của CHÍNH ứng viên này. `null` đồng bộ với
   * `preview`/`normalized` (bảng mã này "không ra chữ"). */
  cleanup: CleanupPreviewWire | null
  /** Story 6.6 — khối tách Chương (tầng 4) của CHÍNH ứng viên này. `null` đồng bộ với
   * `cleanup` (bảng mã này "không ra chữ"). */
  chapters: ChapterSplitPreviewWire | null
}

/** Kết quả một lượt xem trước bảng mã — khớp `commands::project::ImportEncodingPreview`. */
export type ImportEncodingPreview = {
  confidence: ImportConfidence
  selected_encoding: string
  candidates: EncodingCandidateWire[]
  /** Story 6.4, vá vòng rà 1 — bản chuẩn hoá cho nhánh TỰ KHAI (0 ứng viên). `null` khi
   * `candidates` không rỗng (đọc `.normalized` của ứng viên đang chọn thay). */
  self_declared_normalized: NormalizedPreviewWire | null
  /** Story 6.5 — khối làm sạch (tầng 3) cho nhánh TỰ KHAI, cùng điều kiện `null`/`Some` với
   * `self_declared_normalized`. */
  self_declared_cleanup: CleanupPreviewWire | null
  /** Story 6.6 — khối tách Chương (tầng 4) cho nhánh TỰ KHAI, cùng điều kiện `null`/`Some`
   * với `self_declared_normalized`. */
  self_declared_chapters: ChapterSplitPreviewWire | null
}

/** Ba trạng thái, cùng khuôn `CreateWorkResult`. */
export type ImportEncodingPreviewResult = {
  preview: ImportEncodingPreview | null
  error: IpcError | null
}

const CMD_PREVIEW_FROM_TEXT = 'preview_import_encoding_from_text'
const CMD_PREVIEW_FROM_FILE = 'preview_import_encoding_from_file'
const CMD_CONFIRM_WITH_ENCODING = 'confirm_import_with_encoding'

function isNormalizedPreviewWire(value: unknown): value is NormalizedPreviewWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<NormalizedPreviewWire>
  return (
    typeof v.text === 'string' &&
    typeof v.joined_lines === 'number' &&
    typeof v.blank_lines_removed === 'number' &&
    typeof v.window_truncated === 'boolean'
  )
}

function isCleanupRuleTierWire(value: unknown): value is CleanupRuleTierWire {
  return value === 'global' || value === 'work'
}

function isCleanupRuleKindWire(value: unknown): value is CleanupRuleKindWire {
  return value === 'literal' || value === 'regex'
}

function isCleanupSpanWire(value: unknown): value is CleanupSpanWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<CleanupSpanWire>
  return (
    isCleanupRuleTierWire(v.tier) &&
    typeof v.id === 'number' &&
    typeof v.start === 'number' &&
    typeof v.end === 'number'
  )
}

function isCleanupRuleReportWire(value: unknown): value is CleanupRuleReportWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<CleanupRuleReportWire>
  return (
    isCleanupRuleTierWire(v.tier) &&
    typeof v.id === 'number' &&
    typeof v.pattern === 'string' &&
    isCleanupRuleKindWire(v.kind) &&
    typeof v.enabled === 'boolean' &&
    typeof v.count_in_chapter === 'number' &&
    typeof v.count_in_import === 'number'
  )
}

function isCleanupPreviewWire(value: unknown): value is CleanupPreviewWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<CleanupPreviewWire>
  return (
    typeof v.text === 'string' &&
    Array.isArray(v.spans) &&
    v.spans.every(isCleanupSpanWire) &&
    Array.isArray(v.rules) &&
    v.rules.every(isCleanupRuleReportWire) &&
    typeof v.window_truncated === 'boolean' &&
    typeof v.final_text === 'string'
  )
}

function isChapterSplitPreviewEntryWire(value: unknown): value is ChapterSplitPreviewEntryWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterSplitPreviewEntryWire>
  return (
    typeof v.ord === 'number' &&
    (v.title === null || typeof v.title === 'string') &&
    typeof v.length === 'number'
  )
}

function isChapterSplitPreviewWire(value: unknown): value is ChapterSplitPreviewWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterSplitPreviewWire>
  return (
    typeof v.chapter_count === 'number' &&
    Array.isArray(v.chapters) &&
    v.chapters.every(isChapterSplitPreviewEntryWire)
  )
}

function isEncodingCandidateWire(value: unknown): value is EncodingCandidateWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<EncodingCandidateWire>
  return (
    typeof v.label === 'string' &&
    typeof v.encoding === 'string' &&
    (v.preview === null || typeof v.preview === 'string') &&
    // 🔴 Story 6.4 — thiếu vế NÀY thì `undefined` (trường vắng mặt, ví dụ một backend cũ
    // chưa nâng cấp) lọt qua Kiểm TYPE này y hệt bẫy vòng rà đối kháng 2 mục 23 đã vá cho
    // `candidates`: `.vue` đọc `candidate.normalized.text` trên `undefined` rồi vỡ trắng
    // màn hình thay vì hiện lý do rỗng.
    (v.normalized === null || isNormalizedPreviewWire(v.normalized)) &&
    // 🔴 Story 6.5 — cùng lý do: thiếu vế này thì `undefined` lọt lên `.vue`.
    (v.cleanup === null || isCleanupPreviewWire(v.cleanup)) &&
    // 🔴 Story 6.6 — cùng lý do.
    (v.chapters === null || isChapterSplitPreviewWire(v.chapters))
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
    v.candidates.every(isEncodingCandidateWire) &&
    // Story 6.4, vá vòng rà 1 — thiếu vế này thì `undefined` (backend cũ chưa nâng cấp) lọt
    // qua Kiểm TYPE rồi `.vue` đọc `preview.self_declared_normalized.text` trên `undefined`.
    (v.self_declared_normalized === null || isNormalizedPreviewWire(v.self_declared_normalized)) &&
    // Story 6.5 — cùng lý do.
    (v.self_declared_cleanup === null || isCleanupPreviewWire(v.self_declared_cleanup)) &&
    // Story 6.6 — cùng lý do.
    (v.self_declared_chapters === null || isChapterSplitPreviewWire(v.self_declared_chapters))
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

/** Nhánh DÁN VĂN BẢN của màn xem trước bảng mã (Story 6.3, FR126).
 *
 * 🔵 THÊM 2026-09-04 (Story 6.4) — tham số `sourceLang`: bảng dựng chuẩn hoá của mỗi ứng
 * viên rẽ nhánh Trung/Anh (`Encoding::render_candidates` phía Rust). KHÔNG phải một lệnh
 * mới, không một lượt gọi thêm — `sourceLang` đã có sẵn ở form TRƯỚC khi lệnh này chạy.
 *
 * 🔵 THÊM 2026-09-05 (Story 6.6) — tham số `chapterPattern`: mẫu phân tách Chương là tham số
 * MỖI LƯỢT NHẬP (§Always spec 6.6) — KHÔNG lưu ở đâu cả giữa hai lượt nhập, gửi lại `null`
 * khi người dùng chưa gõ mẫu nào. */
export async function previewImportEncodingFromText(
  text: string,
  sourceLang: string,
  chapterPattern: ChapterPatternInput | null,
): Promise<ImportEncodingPreviewResult> {
  return callPreviewImportEncoding(CMD_PREVIEW_FROM_TEXT, { text, sourceLang, chapterPattern })
}

/** Nhánh TỆP của màn xem trước bảng mã (Story 6.3, FR126). Tham số `sourceLang`/`chapterPattern`
 * — xem doc-comment [`previewImportEncodingFromText`]. */
export async function previewImportEncodingFromFile(
  path: string,
  sourceLang: string,
  chapterPattern: ChapterPatternInput | null,
): Promise<ImportEncodingPreviewResult> {
  return callPreviewImportEncoding(CMD_PREVIEW_FROM_FILE, { path, sourceLang, chapterPattern })
}

/** Xác nhận lượt nhập với bảng mã đã chọn — cùng hình dạng trả về `CreateWorkResult`
 * (`created`/`error`), vì lệnh này TẠO một Tác phẩm y hệt `create_work_from_text`/`_from_file`.
 *
 * 🔵 THÊM 2026-09-05 (Story 6.6) — tham số `chapterPattern`: PHẢI là CÙNG mẫu mà lượt xem
 * trước gần nhất vừa hiện (§Always spec 6.6: xem trước và xác nhận phải trùng từng byte). */
export async function confirmImportWithEncoding(
  name: string,
  sourceLang: string,
  genre: string,
  encoding: string,
  chapterPattern: ChapterPatternInput | null,
): Promise<CreateWorkResult> {
  return callCreateWork(CMD_CONFIRM_WITH_ENCODING, { name, sourceLang, genre, encoding, chapterPattern })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.5 — luật làm sạch lúc nhập (FR124, AD-18). Khớp `commands::cleanup::{
// wire::cleanup_add_rule, wire::cleanup_edit_rule, wire::cleanup_delete_rule,
// wire::cleanup_set_enabled }`.
//
// 🔴 **KHÔNG adapter cho `wire::cleanup_list_rules`** — vòng rà 2026-09-06 gỡ
// `cleanupListRules`/`CleanupRuleWire`/`CleanupRuleListResult`: lệnh Rust CÒN đăng ký
// (`commands::cleanup::wire::cleanup_list_rules`, `lib.rs::generate_handler!`), nhưng adapter
// JS của nó không một chỗ gọi nào trong `src/**` (màn xem trước đọc danh sách luật qua
// `EncodingCandidateWire.cleanup.rules`/`ImportEncodingPreview.self_declared_cleanup.rules` —
// dữ liệu đã ĐI KÈM lượt xem trước, không cần một lệnh liệt kê riêng). Nếu một bề mặt MỚI
// (ví dụ màn quản lý luật độc lập, ngoài lượt nhập) cần liệt kê hai tầng mà KHÔNG đi qua xem
// trước, thêm lại adapter này KÈM chỗ gọi đó trong CÙNG một lượt.
// ═══════════════════════════════════════════════════════════════════════════════

const CMD_CLEANUP_ADD_RULE = 'cleanup_add_rule'
const CMD_CLEANUP_EDIT_RULE = 'cleanup_edit_rule'
const CMD_CLEANUP_DELETE_RULE = 'cleanup_delete_rule'
const CMD_CLEANUP_SET_ENABLED = 'cleanup_set_enabled'

/** Ba trạng thái — `ok: true` chính xác khi `error === null`. */
export type CleanupWriteResult = {
  ok: boolean
  error: IpcError | null
}

async function callCleanupWrite(cmd: string, args: Record<string, unknown>): Promise<CleanupWriteResult> {
  try {
    await invoke(cmd, args)
    return { ok: true, error: null }
  } catch (err) {
    if (isIpcError(err)) return { ok: false, error: err }
    if (hasIpcBridge()) {
      console.error(`[project] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { ok: false, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${cmd}\` — chạy ngoài Tauri?`)
    return { ok: false, error: null }
  }
}

/** Thêm một luật mới vào tầng `tier`. */
export async function cleanupAddRule(
  tier: CleanupRuleTierWire,
  pattern: string,
  kind: CleanupRuleKindWire,
): Promise<CleanupWriteResult> {
  return callCleanupWrite(CMD_CLEANUP_ADD_RULE, { tier, pattern, kind })
}

/** Sửa mẫu/hình dạng của luật `(tier, id)`. */
export async function cleanupEditRule(
  tier: CleanupRuleTierWire,
  id: number,
  pattern: string,
  kind: CleanupRuleKindWire,
): Promise<CleanupWriteResult> {
  return callCleanupWrite(CMD_CLEANUP_EDIT_RULE, { tier, id, pattern, kind })
}

/** Xoá luật `(tier, id)`. */
export async function cleanupDeleteRule(tier: CleanupRuleTierWire, id: number): Promise<CleanupWriteResult> {
  return callCleanupWrite(CMD_CLEANUP_DELETE_RULE, { tier, id })
}

/** Bật/tắt luật `(tier, id)` — MỘT lượt ghi thật (§Always spec 6.5), không trạng thái chỉ
 * sống trong bộ nhớ frontend. */
export async function cleanupSetEnabled(
  tier: CleanupRuleTierWire,
  id: number,
  enabled: boolean,
): Promise<CleanupWriteResult> {
  return callCleanupWrite(CMD_CLEANUP_SET_ENABLED, { tier, id, enabled })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.7 — Nhập từ URL bằng danh sách link (AD-15 · AD-40 · AD-41 · FR122). Khớp
// `commands::project::{UrlImportItemWire, UrlImportBatchWire, wire::start_url_import,
// wire::reload_url_import_item, wire::remove_url_import_item}`.
//
// 🔴 Hai con số *"N link · sẽ tạo N Chương"* KHÔNG đi qua đây — chúng là computed CỤC BỘ
// trên `pastedUrls` (đếm dòng non-empty, JS thuần, 0 IPC). Ba lệnh dưới đây chỉ chạy SAU
// khi người dùng đã bấm nút tải.
// ═══════════════════════════════════════════════════════════════════════════════

/** Một mục trong danh sách URL — khớp `commands::project::UrlImportItemWire`. */
export type UrlImportItemWire = {
  url: string
  ok: boolean
  error: IpcError | null
}

/** Kết quả CẢ BA lệnh (tải/tải lại/bỏ một mục) — khớp `commands::project::UrlImportBatchWire`.
 * `encoding_preview === null` là điều kiện ĐỦ để biết nút xác nhận phải khoá — cùng điều
 * kiện mà Rust dùng để đồng bộ `PendingImportSourceState` (không suy luận riêng ở đây). */
export type UrlImportBatchWire = {
  items: UrlImportItemWire[]
  encoding_preview: ImportEncodingPreview | null
}

/** Ba trạng thái, cùng khuôn `ImportEncodingPreviewResult`. */
export type UrlImportBatchResult = {
  batch: UrlImportBatchWire | null
  error: IpcError | null
}

const CMD_START_URL_IMPORT = 'start_url_import'
const CMD_RELOAD_URL_IMPORT_ITEM = 'reload_url_import_item'
const CMD_REMOVE_URL_IMPORT_ITEM = 'remove_url_import_item'

function isUrlImportItemWire(value: unknown): value is UrlImportItemWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<UrlImportItemWire>
  return (
    typeof v.url === 'string' &&
    typeof v.ok === 'boolean' &&
    // Thiếu vế `null` thì một mục lỗi mang `error: undefined` lọt qua Kiểm TYPE này, đúng
    // bẫy mà mọi trường "tuỳ chọn qua dây" khác trong tệp này đã bị bắt.
    (v.error === null || isIpcError(v.error))
  )
}

function isUrlImportBatchWire(value: unknown): value is UrlImportBatchWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<UrlImportBatchWire>
  return (
    Array.isArray(v.items) &&
    v.items.every(isUrlImportItemWire) &&
    (v.encoding_preview === null || isImportEncodingPreview(v.encoding_preview))
  )
}

async function callUrlImportBatch(cmd: string, args: Record<string, unknown>): Promise<UrlImportBatchResult> {
  try {
    const batch = await invoke<UrlImportBatchWire>(cmd, args)
    if (!isUrlImportBatchWire(batch)) {
      console.error(`[project] \`${cmd}\` tra ve mot hinh dang khong dung UrlImportBatchWire`)
      return { batch: null, error: UNKNOWN_IPC_ERROR }
    }
    return { batch, error: null }
  } catch (err) {
    if (isIpcError(err)) return { batch: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[project] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { batch: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
    return { batch: null, error: null }
  }
}

/** Tải TUẦN TỰ đúng thứ tự đã dán — `urls` đã TRIM/lọc dòng rỗng ở tầng gọi
 * (`libraryImport.ts::submitPastedUrls`), Rust lọc lại lần nữa cho chắc (phòng thủ kép,
 * không phải kỳ vọng trùng lặp công việc). */
export async function startUrlImport(urls: string[], sourceLang: string): Promise<UrlImportBatchResult> {
  return callUrlImportBatch(CMD_START_URL_IMPORT, { urls, sourceLang })
}

/** Tải lại ĐÚNG MỘT mục hỏng ở vị trí `index` — đúng 1 lời gọi mạng. */
export async function reloadUrlImportItem(index: number, sourceLang: string): Promise<UrlImportBatchResult> {
  return callUrlImportBatch(CMD_RELOAD_URL_IMPORT_ITEM, { index, sourceLang })
}

/** Bỏ một mục ở vị trí `index` — 0 lời gọi mạng. */
export async function removeUrlImportItem(index: number, sourceLang: string): Promise<UrlImportBatchResult> {
  return callUrlImportBatch(CMD_REMOVE_URL_IMPORT_ITEM, { index, sourceLang })
}
