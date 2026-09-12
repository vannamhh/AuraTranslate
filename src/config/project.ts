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
 *
 * 🔵 SỬA 2026-09-09 (D8 vòng rà đối kháng 3 lớp) — thêm `images_saved`/`images_failed`
 * (Story 6.11, FR127): Rust nới `wire::CreatedWork` thêm hai trường này 2026-09-08, câu mô
 * tả cũ ("khớp ... phía Rust") đã hết đúng cho tới lúc sửa này vì kiểu TS chưa theo kịp.
 * `0` cho mọi lượt tạo KHÔNG đi qua đường URL (dán văn bản, tệp). `images_failed` chưa có
 * bề mặt hiển thị ở story này (nợ có chủ, `deferred-work.md`) — có mặt ở đây để không bị
 * bịa lại từ đầu khi bề mặt đó được dựng.
 */
export type CreatedWork = {
  meta: WorkMeta
  folder: string
  images_saved: number
  images_failed: number
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

/** Nguyên nhân *cần xem* — khớp `commands::project::ReviewCauseWire`, BỐN khoá literal ĐÓNG
 * (Story 6.10, §Always: "bốn nhãn nguyên nhân là bốn khoá literal riêng qua một `switch`
 * cạn"). */
export type ReviewCauseWire = 'short_length' | 'high_cleanup_matches' | 'high_joined_lines' | 'not_measured'

/** Bốn trường xuất xứ HIỆU LỰC (đã áp override, nếu có) của MỘT Chương — Story 6.15,
 * FR128/AD-43. Khớp `commands::project::ChapterOriginWire`. `*_confirmed` cùng triết lý
 * `BlockWire::confirmed`: `true` ⇔ NGƯỜI DÙNG đã chạm ô đó ở màn xem trước. */
export type ChapterOriginWire = {
  author: string | null
  site_name: string | null
  url: string | null
  published_at: string | null
  author_confirmed: boolean
  site_name_confirmed: boolean
  url_confirmed: boolean
  published_at_confirmed: boolean
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
  /** **THÊM Story 6.10a.** 🔵 **SỬA Story 6.10 — `number` → `number | null`.** `null` = *không
   * đo được cho Chương này*; một SỐ (kể cả `0`) = *luật thật sự khớp/không khớp gì, đã đo*.
   * Trục tóm tắt EAGER mà hàng rào Tukey của Story 6.10 đọc trực tiếp
   * (`core::segment::review::classify`). */
  cleanup_match_count: number | null
  /** **THÊM Story 6.10** — số LẦN bước chuẩn hoá đã nối hai dòng làm một, CỦA CHÍNH Chương
   * này (FR125). `null` = không đo được cho Chương này (trên đường `Blob`, con số đo được
   * TRƯỚC khi tách Chương thuộc về TOÀN TÀI LIỆU, không quy về Chương nào được — kể cả
   * Chương đầu). KHÔNG nhầm với `NormalizedPreviewWire.joined_lines` (nghĩa KHÁC: theo ứng
   * viên bảng mã, có cửa sổ). */
  joined_line_count_in_chapter: number | null
  /** **THÊM Story 6.10** — phán quyết hàng rào Tukey trên CHÍNH Chương này, tính LÚC CHẠY
   * (không lưu xuống đĩa). `needs_review === (review_causes.length > 0)` là một bất biến do
   * Rust giữ. */
  needs_review: boolean
  /** Danh mục nguyên nhân *cần xem* — RỖNG khi và chỉ khi `needs_review === false`. */
  review_causes: ReviewCauseWire[]
  /** **THÊM Story 6.15** — bốn trường xuất xứ HIỆU LỰC của CHÍNH Chương này. */
  origin: ChapterOriginWire
}

/** Thân một khối — khớp `commands::project::BlockBodyWire` (`#[serde(tag = "kind", rename_all
 * = "snake_case")]` trên `webimport::BlockBody`). Ba nhánh: đoạn văn/tiêu đề/mục danh sách/
 * trích dẫn, ảnh, chú thích ảnh — Story 6.9, FR123. */
export type BlockBodyWire =
  | { kind: 'paragraph'; text: string }
  | { kind: 'image'; src: string | null; alt: string | null }
  | { kind: 'caption'; text: string }

/** Một khối trên dây — khớp `commands::project::BlockWire`. `kept`/`confirmed` là giá trị
 * HIỆU LỰC (đã áp override), suy ra đúng ba vạch lề hiển thị:
 * `!kept` ⇒ "Đã loại"; `kept && !confirmed` ⇒ "Giữ · máy đoán"; `kept && confirmed` ⇒ "Giữ". */
export type BlockWire = {
  body: BlockBodyWire
  kept: boolean
  confirmed: boolean
}

/** Khối tầng 2 (ranh giới bóc) của MỘT ứng viên — Story 6.9, FR123. Khớp
 * `commands::project::ChapterBlocksPreviewWire`. */
export type ChapterBlocksPreviewWire = {
  blocks: BlockWire[]
}

/** Khối tách Chương của MỘT ứng viên/đường tự khai — tầng 4 (Story 6.6). Mang TOÀN BỘ N
 * Chương (không chỉ ba đầu/ba cuối) — tầng hiển thị tự co gọn khung nhìn mặc định và mở
 * rộng khi sắp xếp theo độ dài. Khớp `commands::project::ChapterSplitPreviewWire`. */
export type ChapterSplitPreviewWire = {
  chapter_count: number
  chapters: ChapterSplitPreviewEntryWire[]
  /** **THÊM Story 6.10** — số mục URL HỎNG của CẢ lượt nhập, `0` trên đường tệp/dán tay. Rust
   * cộng — KHÔNG tự cộng ở tầng hiển thị (AD-1). */
  broken_item_count: number
  /** **THÊM Story 6.10** — `N` của chip *"N cần xem · M sạch"*. BẰNG số Chương
   * `needs_review === true` CỘNG `broken_item_count` (một link hỏng LUÔN cần chú ý). */
  needs_review_count: number
  /** **THÊM Story 6.10** — `M` của chip — số Chương `needs_review === false`. KHÔNG BAO GIỜ
   * cộng `broken_item_count`. */
  clean_count: number
  /** **THÊM Story 6.10** — `true` khi ÍT NHẤT một trong ba tín hiệu so-tương-đối có hàng rào
   * tồn tại cho lượt nhập này. `false` ⇒ KHÔNG tín hiệu nào tham gia (dưới bốn Chương, hoặc
   * mọi hàng rào đều suy biến) — tầng hiển thị PHẢI nói *"chưa đủ Chương để so"* thay vì khai
   * `0 cần xem`. */
  any_signal_participated: boolean
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
  /** **THÊM Story 6.9** — khối tầng 2 (ranh giới bóc) của CHÍNH ứng viên này. `null` khi
   * `extract_main_content == false` (đường tệp/dán tay) — KHÔNG đồng bộ `null`/`Some` với
   * `cleanup`: một bảng mã "không ra chữ" cũng cho `blocks == null`, cùng lý do đó. */
  blocks: ChapterBlocksPreviewWire | null
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

function isReviewCauseWire(value: unknown): value is ReviewCauseWire {
  return (
    value === 'short_length' ||
    value === 'high_cleanup_matches' ||
    value === 'high_joined_lines' ||
    value === 'not_measured'
  )
}

function isNullableOriginField(value: unknown): value is string | null {
  return value === null || typeof value === 'string'
}

function isChapterOriginWire(value: unknown): value is ChapterOriginWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterOriginWire>
  return (
    isNullableOriginField(v.author) &&
    isNullableOriginField(v.site_name) &&
    isNullableOriginField(v.url) &&
    isNullableOriginField(v.published_at) &&
    typeof v.author_confirmed === 'boolean' &&
    typeof v.site_name_confirmed === 'boolean' &&
    typeof v.url_confirmed === 'boolean' &&
    typeof v.published_at_confirmed === 'boolean'
  )
}

function isChapterSplitPreviewEntryWire(value: unknown): value is ChapterSplitPreviewEntryWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterSplitPreviewEntryWire>
  return (
    typeof v.ord === 'number' &&
    (v.title === null || typeof v.title === 'string') &&
    typeof v.length === 'number' &&
    (v.cleanup_match_count === null || typeof v.cleanup_match_count === 'number') &&
    (v.joined_line_count_in_chapter === null || typeof v.joined_line_count_in_chapter === 'number') &&
    typeof v.needs_review === 'boolean' &&
    Array.isArray(v.review_causes) &&
    v.review_causes.every(isReviewCauseWire) &&
    isChapterOriginWire(v.origin)
  )
}

function isBlockBodyWire(value: unknown): value is BlockBodyWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<BlockBodyWire> & { kind?: unknown }
  if (v.kind === 'paragraph' || v.kind === 'caption') {
    return typeof (v as { text?: unknown }).text === 'string'
  }
  if (v.kind === 'image') {
    const src = (v as { src?: unknown }).src
    const alt = (v as { alt?: unknown }).alt
    return (src === null || typeof src === 'string') && (alt === null || typeof alt === 'string')
  }
  return false
}

function isBlockWire(value: unknown): value is BlockWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<BlockWire>
  return isBlockBodyWire(v.body) && typeof v.kept === 'boolean' && typeof v.confirmed === 'boolean'
}

function isChapterBlocksPreviewWire(value: unknown): value is ChapterBlocksPreviewWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterBlocksPreviewWire>
  return Array.isArray(v.blocks) && v.blocks.every(isBlockWire)
}

function isChapterSplitPreviewWire(value: unknown): value is ChapterSplitPreviewWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterSplitPreviewWire>
  return (
    typeof v.chapter_count === 'number' &&
    Array.isArray(v.chapters) &&
    v.chapters.every(isChapterSplitPreviewEntryWire) &&
    typeof v.broken_item_count === 'number' &&
    typeof v.needs_review_count === 'number' &&
    typeof v.clean_count === 'number' &&
    typeof v.any_signal_participated === 'boolean'
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
    (v.chapters === null || isChapterSplitPreviewWire(v.chapters)) &&
    // 🔴 Story 6.9 — cùng lý do: thiếu vế này thì `undefined` lọt lên `.vue`.
    (v.blocks === null || isChapterBlocksPreviewWire(v.blocks))
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
// Story 6.16 — nhập tài liệu song ngữ hai cột (FR115, AD-39 · AD-37/46 · AD-47 ③). Khớp
// `commands::project::{BilingualMismatchWire, BilingualEncodingCandidateWire,
// BilingualImportEncodingPreview, wire::preview_bilingual_import_from_file,
// wire::confirm_bilingual_import}`.
//
// 🔴 Vai cột (`sourceColumn`/`targetColumn`) và cờ tiêu đề (`hasHeader`) là tham số MỖI LƯỢT
// gọi, cùng khuôn `chapterPattern`. Chỉ lượt MỞ gửi `path` (`previewBilingualImportFromFile`);
// mọi lượt đổi cột/tiêu đề/mẫu sau đó gọi `rebuildBilingualImportPreview` — KHÔNG `path`, Rust
// clone byte đã cất lúc mở, không đọc lại tệp.
// ═══════════════════════════════════════════════════════════════════════════════

/** Một hàng lệch cặp — khớp `commands::project::BilingualMismatchWire`. Mở rộng Story 6.17
 * (FR116): bốn trường sau đủ dữ kiện để dựng màn quy nhóm mà không cần hỏi lại Rust cho mỗi
 * lượt render — xem doc-comment cùng tên phía Rust (`core::segment::bilingual`). */
export type BilingualMismatchWire = {
  chapter_index: number
  row_number: number
  source_sentences: string[]
  target_line: string
  /** Số câu đích MÁY đã tách — Rust tính, KHÔNG suy từ `initial_cuts.length + 1` ở đây: hai
   * con số đó lệch nhau đúng ở hàng "Skip blank target" (0 câu đích cho `initial_cuts = []`,
   * và `0 + 1 = 1` là sai). Dùng trường này khi cần biết "còn bao nhiêu câu đích" của một
   * hàng, ví dụ tổng số câu bị bỏ khi hàng đó được đánh dấu Skip. */
  target_sentence_count: number
  candidate_positions: number[]
  initial_cuts: number[]
  proposed_cuts: number[]
}

/** Một lượt quy nhóm gửi lên Rust — khớp `commands::project::BilingualRegroupingWire`.
 * `source_sentences`/`target_line` là ẢNH CHỤP echo lại NGUYÊN VẸN từ `BilingualMismatchWire`
 * lúc nó được tạo — Rust so khớp lại với hàng THẬT trước khi áp (staleness check, Story
 * 6.17). `cuts` bị Rust bỏ qua khi `kind === 'skip'`. */
export type BilingualRegroupingInput = {
  row_number: number
  source_sentences: string[]
  target_line: string
  kind: 'cuts' | 'skip'
  cuts: number[]
}

/** Kết quả chạy TRỌN chuỗi bảy bước cho MỘT ứng viên bảng mã — khớp
 * `commands::project::BilingualEncodingCandidateWire`. */
export type BilingualEncodingCandidateWire = {
  label: string
  encoding: string
  preview: string | null
  row_count: number
  chapter_count: number
  pair_count: number
  /** Tổng câu đích của mọi hàng đã giải quyết bằng "Bỏ qua hàng này" (nguồn rỗng) khi ứng
   * viên này chạy TRỌN chuỗi — Rust tính lại MỖI LƯỢT (cạnh `pair_count`), SỐNG SÓT qua chính
   * hàng đã biến mất khỏi `mismatches` ngay khi được giải quyết. KHÔNG suy được từ
   * `mismatches` — hàng Skip không còn ở đó nữa. */
  skipped_target_sentence_count: number
  mismatches: BilingualMismatchWire[]
}

/** Dải năm ứng viên — khớp `commands::project::BilingualImportEncodingPreview`. */
export type BilingualImportEncodingPreview = {
  confidence: ImportConfidence
  selected_encoding: string
  candidates: BilingualEncodingCandidateWire[]
  /** Tối đa hai mươi hàng đầu tiên của bảng mã ĐANG CHỌN, MỌI cột — nguyên liệu cho thẻ chọn
   * cột nguồn/đích. */
  sample_rows: string[][]
  row_count: number
  column_count: number
}

/** Ba trạng thái, cùng khuôn `ImportEncodingPreviewResult`. */
export type BilingualImportEncodingPreviewResult = {
  preview: BilingualImportEncodingPreview | null
  error: IpcError | null
}

const CMD_PREVIEW_BILINGUAL_FROM_FILE = 'preview_bilingual_import_from_file'
const CMD_REBUILD_BILINGUAL_PREVIEW = 'rebuild_bilingual_import_preview'
const CMD_CONFIRM_BILINGUAL_IMPORT = 'confirm_bilingual_import'

function isNumberArray(value: unknown): value is number[] {
  return Array.isArray(value) && value.every((cell) => typeof cell === 'number')
}

function isBilingualMismatchWire(value: unknown): value is BilingualMismatchWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<BilingualMismatchWire>
  return (
    typeof v.chapter_index === 'number' &&
    typeof v.row_number === 'number' &&
    isStringRow(v.source_sentences) &&
    typeof v.target_line === 'string' &&
    typeof v.target_sentence_count === 'number' &&
    isNumberArray(v.candidate_positions) &&
    isNumberArray(v.initial_cuts) &&
    isNumberArray(v.proposed_cuts)
  )
}

function isBilingualEncodingCandidateWire(value: unknown): value is BilingualEncodingCandidateWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<BilingualEncodingCandidateWire>
  return (
    typeof v.label === 'string' &&
    typeof v.encoding === 'string' &&
    (v.preview === null || typeof v.preview === 'string') &&
    typeof v.row_count === 'number' &&
    typeof v.chapter_count === 'number' &&
    typeof v.pair_count === 'number' &&
    typeof v.skipped_target_sentence_count === 'number' &&
    Array.isArray(v.mismatches) &&
    v.mismatches.every(isBilingualMismatchWire)
  )
}

function isStringRow(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((cell) => typeof cell === 'string')
}

function isBilingualImportEncodingPreview(value: unknown): value is BilingualImportEncodingPreview {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<BilingualImportEncodingPreview>
  return (
    (v.confidence === 'self_declared' || v.confidence === 'high' || v.confidence === 'low') &&
    typeof v.selected_encoding === 'string' &&
    Array.isArray(v.candidates) &&
    v.candidates.every(isBilingualEncodingCandidateWire) &&
    Array.isArray(v.sample_rows) &&
    v.sample_rows.every(isStringRow) &&
    typeof v.row_count === 'number' &&
    typeof v.column_count === 'number'
  )
}

/** Nhánh TỆP của màn xem trước song ngữ (Story 6.16, FR115) — chỉ lượt MỞ. Mọi lượt đổi
 * cột/tiêu đề/mẫu sau đó đi qua `rebuildBilingualImportPreview`, không gửi lại `path`. */
export async function previewBilingualImportFromFile(
  path: string,
  sourceLang: string,
  chapterPattern: ChapterPatternInput | null,
  sourceColumn: number,
  targetColumn: number,
  hasHeader: boolean,
): Promise<BilingualImportEncodingPreviewResult> {
  try {
    const preview = await invoke<BilingualImportEncodingPreview>(CMD_PREVIEW_BILINGUAL_FROM_FILE, {
      path,
      sourceLang,
      chapterPattern,
      sourceColumn,
      targetColumn,
      hasHeader,
      // Lượt MỞ luôn bắt đầu 0 quy nhóm — chưa hàng lệch cặp nào từng hiện ra để mà sửa.
      regroupings: [] as BilingualRegroupingInput[],
    })
    if (!isBilingualImportEncodingPreview(preview)) {
      console.error(
        `[project] \`${CMD_PREVIEW_BILINGUAL_FROM_FILE}\` tra ve mot hinh dang khong dung BilingualImportEncodingPreview`,
      )
      return { preview: null, error: UNKNOWN_IPC_ERROR }
    }
    return { preview, error: null }
  } catch (err) {
    if (isIpcError(err)) return { preview: null, error: err }
    if (hasIpcBridge()) {
      console.error(
        `[project] \`${CMD_PREVIEW_BILINGUAL_FROM_FILE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { preview: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${CMD_PREVIEW_BILINGUAL_FROM_FILE}\` — chạy ngoài Tauri? ${String(err)}`)
    return { preview: null, error: null }
  }
}

/** Dựng lại màn xem trước song ngữ trên nguồn ĐANG CHỜ phía Rust (Story 6.16) — cho mọi lượt
 * đổi cột, đảo vai, bật/tắt tiêu đề, sửa mẫu phân tách. KHÔNG gửi `path`: Rust clone byte đã
 * cất lúc mở (`PendingImportSourceState`), không đọc lại tệp (Quyết định Ice 2026-09-11:
 * "toggling rebuilds the preview in memory"). */
export async function rebuildBilingualImportPreview(
  sourceLang: string,
  chapterPattern: ChapterPatternInput | null,
  sourceColumn: number,
  targetColumn: number,
  hasHeader: boolean,
  regroupings: BilingualRegroupingInput[],
): Promise<BilingualImportEncodingPreviewResult> {
  try {
    const preview = await invoke<BilingualImportEncodingPreview>(CMD_REBUILD_BILINGUAL_PREVIEW, {
      sourceLang,
      chapterPattern,
      sourceColumn,
      targetColumn,
      hasHeader,
      regroupings,
    })
    if (!isBilingualImportEncodingPreview(preview)) {
      console.error(
        `[project] \`${CMD_REBUILD_BILINGUAL_PREVIEW}\` tra ve mot hinh dang khong dung BilingualImportEncodingPreview`,
      )
      return { preview: null, error: UNKNOWN_IPC_ERROR }
    }
    return { preview, error: null }
  } catch (err) {
    if (isIpcError(err)) return { preview: null, error: err }
    if (hasIpcBridge()) {
      console.error(
        `[project] \`${CMD_REBUILD_BILINGUAL_PREVIEW}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { preview: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${CMD_REBUILD_BILINGUAL_PREVIEW}\` — chạy ngoài Tauri? ${String(err)}`)
    return { preview: null, error: null }
  }
}

/** Xác nhận lượt nhập song ngữ (Story 6.16, FR115) — cùng hình dạng trả về `CreateWorkResult`.
 * `sourceColumn`/`targetColumn`/`hasHeader` PHẢI là bộ ba mà lượt xem trước gần nhất vừa hiện. */
export async function confirmBilingualImport(
  name: string,
  sourceLang: string,
  genre: string,
  encoding: string,
  chapterPattern: ChapterPatternInput | null,
  sourceColumn: number,
  targetColumn: number,
  hasHeader: boolean,
  regroupings: BilingualRegroupingInput[],
): Promise<CreateWorkResult> {
  return callCreateWork(CMD_CONFIRM_BILINGUAL_IMPORT, {
    name,
    sourceLang,
    genre,
    encoding,
    chapterPattern,
    sourceColumn,
    targetColumn,
    hasHeader,
    regroupings,
  })
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
 *
 * 🔵 **SỬA 2026-09-08 (Story 6.10a) — "`encoding_preview === null` là điều kiện ĐỦ để biết nút
 * xác nhận phải khoá" đã HẾT ĐÚNG.** Vị từ XEM phía Rust (`chapters_shape_for_view`) nay bỏ
 * qua mục hỏng để vẫn dựng `encoding_preview` từ các mục OK còn lại — nó khác `null` NGAY CẢ
 * KHI còn mục hỏng. Nút xác nhận khoá theo vị từ GHI RIÊNG
 * (`importPreviewCanConfirm`/`importPreviewUrlListHasBrokenItem`, tính CỤC BỘ trên
 * `items[].ok`, `src/importPreviewState.ts`) — KHÔNG còn đọc trường này. `encoding_preview
 * === null` giờ chỉ còn nghĩa "không có gì để mà xem" (danh sách rỗng, hoặc KHÔNG mục OK
 * nào). */
export type UrlImportBatchWire = {
  items: UrlImportItemWire[]
  encoding_preview: ImportEncodingPreview | null
  /** 🔵 **THÊM Story 6.8 (NFR19)** — số domain PHÂN BIỆT trong nhật ký của CẢ PHIÊN CHẠY tại
   * thời điểm trả lời, không chỉ lượt gọi vừa rồi. Chân màn xem trước đọc trực tiếp trường
   * này cho dòng *"Đã gọi N domain · xem"* — không một lệnh IPC thứ hai chỉ để có một số. */
  domain_log_domain_count: number
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
    (v.encoding_preview === null || isImportEncodingPreview(v.encoding_preview)) &&
    typeof v.domain_log_domain_count === 'number'
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

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.9 — sửa ranh giới bóc bằng bàn phím (FR123). Khớp
// `commands::project::wire::{tier2_block_set_kept, tier2_block_confirm_range}` — cả hai trả
// LẠI `UrlImportBatchWire` TƯƠI (cùng hình dạng ba lệnh URL ở trên, dùng lại `callUrlImportBatch`),
// vì tầng 2 chỉ có nghĩa trên đường URL (§Always spec 6.7/6.9 — `extract_main_content` chỉ
// `true` ở đó).
// ═══════════════════════════════════════════════════════════════════════════════

const CMD_TIER2_BLOCK_SET_KEPT = 'tier2_block_set_kept'
const CMD_TIER2_BLOCK_CONFIRM_RANGE = 'tier2_block_confirm_range'

/** Đổi trạng thái giữ/loại của khối `index` (`Space`) — 0 lời gọi mạng, chỉ đổi
 * `Tier2BlockOverridesState` trong bộ nhớ Rust rồi dựng lại xem trước. */
export async function tier2BlockSetKept(
  index: number,
  kept: boolean,
  sourceLang: string,
): Promise<UrlImportBatchResult> {
  return callUrlImportBatch(CMD_TIER2_BLOCK_SET_KEPT, { index, kept, sourceLang })
}

/** Đặt dải `[start, end]` thành giữ, mọi khối NGOÀI dải thành loại — một lượt (`]`). `total`
 * đến từ CHÍNH mảng khối frontend đang hiện (xem doc-comment
 * `commands::project::block_overrides_for_range`). */
export async function tier2BlockConfirmRange(
  start: number,
  end: number,
  total: number,
  sourceLang: string,
): Promise<UrlImportBatchResult> {
  return callUrlImportBatch(CMD_TIER2_BLOCK_CONFIRM_RANGE, { start, end, total, sourceLang })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.15 (FR128/AD-43) — ghi một lượt sửa tay xuất xứ vào
// `ChapterOriginOverridesState` (Rust), theo CHỈ SỐ Chương. Khớp
// `commands::project::wire::set_chapter_origin_override`.
//
// 🔴 KHÁC hai lệnh tier2 ngay trên — lệnh này KHÔNG trả lại một `UrlImportBatchWire` tươi.
// Khối `ChapterOrigin.vue` là bốn Ô NHẬP VĂN BẢN đơn giản, không cần Rust tính lại gì để
// hiện (khác khối tầng 2, nơi `kept`/`confirmed` là kết quả một phép TÍNH trên `machine_kept`
// mà chỉ Rust biết). `importPreviewState.ts` giữ draft NGAY Ở PHÍA CLIENT (sống qua lượt đổi
// con trỏ/đổi bảng mã — không phụ thuộc một round-trip IPC nào); lệnh này chỉ ĐỒNG BỘ bản ghi
// xuống Rust để `confirm_import_with_encoding` đọc được LÚC XÁC NHẬN.
// ═══════════════════════════════════════════════════════════════════════════════

const CMD_SET_CHAPTER_ORIGIN_OVERRIDE = 'set_chapter_origin_override'

/** Bốn trường xuất xứ người dùng vừa gõ — chuỗi tự do, cùng khuôn `ChapterOriginEdit` của
 * `config/chapter.ts` (chuỗi rỗng = "để trống"). */
export type ChapterOriginEditFields = {
  author: string
  siteName: string
  url: string
  publishedAt: string
}

/** Hai trạng thái — lệnh này không trả dữ liệu, chỉ `ok`/`error` (cùng khuôn
 * `ChapterOrganiseResult` của `config/chapter.ts`). */
export type SetChapterOriginOverrideResult = {
  ok: true | null
  error: IpcError | null
}

/**
 * Ghi một lượt sửa tay xuất xứ cho Chương thứ `chapterIndex` (0-based, vị trí trong danh
 * sách Chương của lượt nhập ĐANG XEM TRƯỚC) vào state Rust — không ném.
 *
 * ⚠️ `invoke()` gửi tham số dạng camelCase: `chapterIndex`/`author`/`siteName`/`url`/
 * `publishedAt`.
 */
export async function setChapterOriginOverride(
  chapterIndex: number,
  fields: ChapterOriginEditFields,
): Promise<SetChapterOriginOverrideResult> {
  try {
    await invoke<void>(CMD_SET_CHAPTER_ORIGIN_OVERRIDE, {
      chapterIndex,
      author: fields.author,
      siteName: fields.siteName,
      url: fields.url,
      publishedAt: fields.publishedAt,
    })
    return { ok: true, error: null }
  } catch (err) {
    if (isIpcError(err)) return { ok: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[project] \`${CMD_SET_CHAPTER_ORIGIN_OVERRIDE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { ok: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[project] không gọi được \`${CMD_SET_CHAPTER_ORIGIN_OVERRIDE}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { ok: null, error: null }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.10a — con trỏ *Chương đang chọn*: chi tiết LAZY (tầng 2/3) cho Chương thứ k khi con
// trỏ dời (`⌥←`/`⌥→`). Khớp `commands::project::{ChapterDetailWire, wire::preview_chapter_detail}`.
// Chỉ có nghĩa trên đường URL (`PipelineShape::Chapters`, §Always spec 6.7/6.9/6.10a).
// ═══════════════════════════════════════════════════════════════════════════════

/** Chi tiết tầng 2/3 của MỘT Chương — khớp `commands::project::ChapterDetailWire`. */
export type ChapterDetailWire = {
  cleanup: CleanupPreviewWire
  blocks: ChapterBlocksPreviewWire | null
}

function isChapterDetailWire(value: unknown): value is ChapterDetailWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ChapterDetailWire>
  return (
    v.cleanup !== undefined &&
    isCleanupPreviewWire(v.cleanup) &&
    (v.blocks === null || isChapterBlocksPreviewWire(v.blocks))
  )
}

/** Tên command trên dây. Khớp `commands::project::wire::preview_chapter_detail`. */
const CMD_PREVIEW_CHAPTER_DETAIL = 'preview_chapter_detail'

/** Ba trạng thái, cùng khuôn `UrlImportBatchResult`. `detail === null` mà `error === null`
 * nghĩa là `chapterIndex` không còn khớp trạng thái hiện hành (mẫu phân tách vừa đổi làm N
 * đổi, hoặc bảng mã đã chọn "không ra chữ" cho Chương này) — chỗ gọi coi đó là CŨ, không
 * đoán. */
export type ChapterDetailResult = {
  detail: ChapterDetailWire | null
  error: IpcError | null
}

/** Dựng lại tầng 2/3 cho Chương thứ `chapterIndex` (0-based) — với bảng mã ĐÃ CHỌN (không dò
 * lại, không lặp năm ứng viên). `chapterPattern` gửi lại HIỆN HÀNH, cùng quy ước mọi lệnh
 * `preview_import_encoding_from_*`/`confirm_import_with_encoding` (§Always spec 6.6: mẫu là
 * tham số MỖI LƯỢT NHẬP, không lưu ở đâu cả giữa hai lượt gọi). */
export async function previewChapterDetail(
  chapterIndex: number,
  encoding: string,
  sourceLang: string,
  chapterPattern: ChapterPatternInput | null,
): Promise<ChapterDetailResult> {
  try {
    const detail = await invoke<ChapterDetailWire>(CMD_PREVIEW_CHAPTER_DETAIL, {
      chapterIndex,
      encoding,
      sourceLang,
      chapterPattern,
    })
    if (!isChapterDetailWire(detail)) {
      console.error(`[project] \`${CMD_PREVIEW_CHAPTER_DETAIL}\` tra ve mot hinh dang khong dung ChapterDetailWire`)
      return { detail: null, error: UNKNOWN_IPC_ERROR }
    }
    return { detail, error: null }
  } catch (err) {
    if (isIpcError(err)) return { detail: null, error: err }
    if (hasIpcBridge()) {
      console.error(
        `[project] \`${CMD_PREVIEW_CHAPTER_DETAIL}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { detail: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${CMD_PREVIEW_CHAPTER_DETAIL}\` — chạy ngoài Tauri? ${String(err)}`)
    return { detail: null, error: null }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.8 — Allowlist mạng hai tầng và nhật ký domain (NFR19, AD-41). Khớp
// `commands::project::{DomainLogEntryWire, wire::list_domain_log}`.
// ═══════════════════════════════════════════════════════════════════════════════

/** Danh mục ĐÓNG — khớp `webimport::DomainLogOutcome` (`serde(rename_all = "snake_case")`).
 * `null` khi `allowed === false` (0 kết nối, không có kết quả mạng nào để mà báo).
 *
 * 🔵 THÊM 2026-09-09 (Story 6.11, mục A vòng rà đối kháng 3 lớp, Ice ký) — trường "và rồi
 * SAO" cho một chặng ĐÃ được phép: `tier`/`allowed` một mình không phân biệt được một chặng
 * ĐÃ CHO PHÉP rồi tải xong với một chặng ĐÃ CHO PHÉP rồi trượt mạng/MIME/quá cỡ — đúng ô
 * "Error Handling" thứ hai của I/O Matrix spec 6.11 đòi. */
export type DomainLogOutcomeWire =
  | 'fetched'
  | 'redirected'
  | 'http_status'
  | 'timeout'
  | 'connect_failed'
  | 'too_large'
  | 'mime_rejected'
  | 'other'

const DOMAIN_LOG_OUTCOMES: readonly DomainLogOutcomeWire[] = [
  'fetched',
  'redirected',
  'http_status',
  'timeout',
  'connect_failed',
  'too_large',
  'mime_rejected',
  'other',
]

/** Một bản ghi THÔ — khớp `commands::project::DomainLogEntryWire`. `kind`/`tier`/`outcome` đi
 * qua như DỮ LIỆU (chuỗi định danh máy, AD-21) — `domainLogKindLabelKey`/`domainLogReasonKey`/
 * `domainLogOutcomeLabelKey` (`settingsState.ts`) ánh xạ sang câu, cùng khuôn `cleanupTierLabelKey`. */
export type DomainLogEntryWire = {
  at_epoch_ms: number
  domain: string
  kind: 'page' | 'image'
  allowed: boolean
  tier: 'tier1' | 'tier2' | 'denied'
  /** 🔵 THÊM 2026-09-09 (Story 6.11, mục A) — xem [`DomainLogOutcomeWire`]. */
  outcome: DomainLogOutcomeWire | null
}

/** Hình dạng CỐT LÕI của một bản ghi — mọi trường TRỪ `outcome`. Tách riêng khỏi
 * `sanitizeDomainLogEntry` (ngay dưới) cho đúng lý do đó: một `outcome` LẠ (một
 * biến thể Rust MỚI mà bản TS này chưa biết) không được phép làm SAI LỆCH tính hợp lệ của
 * CHÍNH bản ghi đó — nó chỉ là MỘT TRƯỜNG không đọc được, không phải cả bản ghi hỏng. */
function isDomainLogEntryWireCoreShape(value: unknown): value is Omit<DomainLogEntryWire, 'outcome'> {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<DomainLogEntryWire>
  return (
    typeof v.at_epoch_ms === 'number' &&
    typeof v.domain === 'string' &&
    (v.kind === 'page' || v.kind === 'image') &&
    typeof v.allowed === 'boolean' &&
    (v.tier === 'tier1' || v.tier === 'tier2' || v.tier === 'denied')
  )
}

/** 🔵 THÊM 2026-09-09 (vòng rà đối kháng 3, mục R3) — HẠ một `outcome` LẠ xuống `null` cho
 * RIÊNG bản ghi đó, không loại BỎ cả mảng. Trước sửa này, `listDomainLog` gọi
 * `entries.every(...)` kiểm CẢ `outcome` trong CÙNG một vị từ — MỘT bản ghi mang một biến thể `outcome` mà bản TS
 * này CHƯA BIẾT (ví dụ Rust thêm biến thể thứ chín mà bản build frontend chưa cập nhật danh
 * mục `DOMAIN_LOG_OUTCOMES`) làm `every()` trả `false`, và `listDomainLog` vứt TOÀN BỘ mảng
 * (`entries: null`) — cả màn Quyền riêng tư trống trơn vì đúng MỘT trường của đúng MỘT bản
 * ghi. Đây là "rỗng không có lý do" đúng lớp mà kho cấm: những bản ghi HOÀN TOÀN HỢP LỆ khác
 * biến mất theo. Hạ `outcome` lạ về `null` — cùng ý nghĩa "không biết/không áp dụng" mà
 * `null` đã mang cho một chặng bị từ chối — giữ lại được TOÀN BỘ dữ liệu THẬT, chỉ mất đúng
 * MỘT trường của đúng MỘT bản ghi. Trả `null` (không phải loại bỏ) nếu hình dạng CỐT LÕI
 * (ngoài `outcome`) cũng hỏng — đó vẫn là một bản ghi không đọc được, không phải chỉ một
 * trường lạ.
 */
function sanitizeDomainLogEntry(value: unknown): DomainLogEntryWire | null {
  if (!isDomainLogEntryWireCoreShape(value)) return null
  const v = value as Omit<DomainLogEntryWire, 'outcome'> & { outcome?: unknown }
  const outcome =
    v.outcome === null || v.outcome === undefined
      ? null
      : DOMAIN_LOG_OUTCOMES.includes(v.outcome as DomainLogOutcomeWire)
        ? (v.outcome as DomainLogOutcomeWire)
        : null
  if (outcome === null && v.outcome !== null && v.outcome !== undefined) {
    console.error(`[project] một bản ghi nhật ký domain mang outcome lạ (${String(v.outcome)}) -- ha ve null cho RIENG ban ghi nay`)
  }
  return { ...v, outcome }
}

/** Tên command trên dây. Khớp `commands::project::wire::list_domain_log`. */
const CMD_LIST_DOMAIN_LOG = 'list_domain_log'

/** Kết quả một lượt đọc nhật ký domain. `entries === null` chỉ khi Rust trả một hình dạng
 * KHÔNG đúng `DomainLogEntryWire[]` hoặc không có cầu IPC — lệnh này phía Rust không có
 * nhánh lỗi (đọc thẳng một `Mutex<Vec<_>>`), cùng khuôn `ReadHanVietResult`. */
export type ListDomainLogResult = {
  entries: DomainLogEntryWire[] | null
  error: IpcError | null
}

/** Đọc TOÀN BỘ nhật ký domain THÔ của phiên chạy hiện tại (§Always spec 6.8: "không phân
 * trang, không xem thêm che bớt hàng") — gộp theo domain là việc của tầng trình bày
 * (`settingsState.ts`), không của adapter này. Không ném. */
export async function listDomainLog(): Promise<ListDomainLogResult> {
  try {
    const raw = await invoke<unknown[]>(CMD_LIST_DOMAIN_LOG)
    if (!Array.isArray(raw)) {
      console.error(`[project] \`${CMD_LIST_DOMAIN_LOG}\` tra ve mot hinh dang khong phai mang`)
      return { entries: null, error: UNKNOWN_IPC_ERROR }
    }
    // R3 (vòng rà đối kháng 3, lớp 3) — sanitize TỪNG bản ghi (một `outcome` lạ chỉ hạ
    // trường đó về `null`), rồi CHỈ từ chối TOÀN BỘ mảng nếu hình dạng CỐT LÕI của MỘT bản
    // ghi cũng hỏng (không phải chỉ `outcome`).
    const sanitized = raw.map(sanitizeDomainLogEntry)
    if (sanitized.some((e) => e === null)) {
      console.error(`[project] \`${CMD_LIST_DOMAIN_LOG}\` tra ve mot hinh dang khong dung DomainLogEntryWire[]`)
      return { entries: null, error: UNKNOWN_IPC_ERROR }
    }
    const entries = sanitized as DomainLogEntryWire[]
    return { entries, error: null }
  } catch (err) {
    if (isIpcError(err)) return { entries: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[project] \`${CMD_LIST_DOMAIN_LOG}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { entries: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[project] không gọi được \`${CMD_LIST_DOMAIN_LOG}\` — chạy ngoài Tauri? ${String(err)}`)
    return { entries: null, error: null }
  }
}
