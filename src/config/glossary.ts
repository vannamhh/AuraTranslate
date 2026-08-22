/**
 * Adapter IPC phía webview cho "Thêm nhanh thuật ngữ" — Story 3.3, FR48.
 *
 * Adapter thứ **BẢY** của `src/config/*.ts` (`project-context.md:184-187` viết "sáu tệp" —
 * hết đúng từ story này, xem mục 🔵 ở đó). Cùng khuôn `./pinned.ts`: một lời gọi `invoke`,
 * một `try/catch`, không quy tắc nghiệp vụ nào ở đây — quy tắc sống ở Rust
 * (`commands/glossary.rs`).
 *
 * ⚠️ **`invoke()` gửi tham số ở dạng camelCase**, Rust nhận `snake_case` — `sourceTerm` chứ
 * không `source_term`. Trường TRẢ VỀ giữ nguyên `snake_case` (`source_term`, `term_origin`)
 * — hai chiều khác nhau, xem `src/AGENTS.md`.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Bốn phân loại (FR46) — khớp NGUYÊN VĂN `Category::as_str()` phía Rust. */
export type GlossaryCategory = 'person' | 'place' | 'domain_term' | 'other'

/** Hai tầng (AD-18) — khớp NGUYÊN VĂN `GlossaryTier::as_str()` phía Rust. */
export type GlossaryTierWire = 'global' | 'work'

/**
 * Hình dạng `QuickAddTerm` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/glossary.rs::QuickAddTerm` cố ý KHÔNG đặt
 * `#[serde(rename_all = "camelCase")]` — cùng luật với `PinnedEntry`/`OpenChapter`. Đổi một
 * tên ở đây mà không đổi ở kia cho ra `undefined` mà TypeScript không hề biết.
 */
export type GlossaryQuickAddEntry = {
  /** `"global"` hoặc `"work"` — tầng đang GHIM cho chế độ SỬA (§Design Notes của spec). */
  tier: GlossaryTierWire
  /** `glossary_entry.id` — chỉ có nghĩa CÙNG VỚI `tier` ở trên (`id` không duy nhất xuyên
   * `Store`). */
  id: number
  source_term: string
  /** `null` == *chờ chốt*. Lượt tra KHÔNG lọc `is_confirmed` — một mục chờ chốt vẫn trả về. */
  translation: string | null
  note: string
  category: GlossaryCategory
  term_origin: string
  created_at: string
}

/**
 * Hình dạng `QuickAddLookup` phía Rust — **`snake_case`, đúng như trên dây**. Phong bì, KHÔNG
 * một `Option<GlossaryQuickAddEntry>` trần: `work_tier_available` đi cùng để dải hiện được
 * lý do tầng Tác phẩm chưa dùng được MÀ KHÔNG cần một vòng IPC riêng — xem doc-comment của
 * `commands/glossary.rs::QuickAddLookup`.
 */
type QuickAddLookupWire = {
  work_tier_available: boolean
  entry: GlossaryQuickAddEntry | null
}

/**
 * Ba trạng thái, cùng khuôn `PinnedResult`: phân biệt *"chưa nạp/ngoài Tauri"* (`found:
 * 'unknown'`) với *"đã tra, không tìm thấy"* (`found: 'none'`) — kết quả tra là MỘT mục hoặc
 * KHÔNG có mục nào (không phải một danh sách), nên hình dạng "không tìm thấy" và "chưa nạp"
 * phải phân biệt được RÕ hơn `null` chồng `null`.
 *
 * ⚠️ `found: 'unknown'` là trạng thái *"chưa tra được gì cả"* (chưa gọi/đang bay/ngoài Tauri
 * mà không có `error`); `found: 'none'` là *"đã tra, không có trong Glossary"* (chế độ
 * THÊM); `found: 'entry'` mang `entry` (chế độ SỬA). Đây là vế cấu trúc của "…HasLoaded" mà
 * spec đòi — một `null` trần không tự nói vì sao nó `null`.
 *
 * `workTierAvailable` có mặt ở **cả hai** ca đã tra (`'none'`/`'entry'`) — nó không phụ
 * thuộc việc có tìm thấy mục hay không, chỉ phụ thuộc có Tác phẩm nào đang mở hay không.
 */
export type GlossaryLookupResult =
  | { found: 'unknown'; error: IpcError | null }
  | { found: 'none'; workTierAvailable: boolean }
  | { found: 'entry'; entry: GlossaryQuickAddEntry; workTierAvailable: boolean }

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // ⚠️ Hình dạng `IpcError` là một LỜI KHAI về dữ liệu đã qua dây IPC, không một bảo đảm
    //    của trình biên dịch — cùng chú thích `./pinned.ts`.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/glossary.rs` (module `wire`). */
const CMD_LOOKUP = 'glossary_lookup_term'
const CMD_ADD = 'glossary_add_term'
const CMD_UPDATE = 'glossary_update_term'

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

/**
 * Tra một `source_term` qua hai tầng. **Không bao giờ ném.**
 *
 * 🔴 Adapter này KHÔNG lọc `is_confirmed` — nó chỉ chuyển tiếp nguyên hình dạng Rust trả về.
 * Vế "chờ chốt vẫn mở SỬA" của spec là hành vi của `resolve_term_for_quick_add`
 * (`core/glossary/store.rs`), không phải một điều gì frontend phải tự làm lại.
 */
export async function lookupGlossaryTerm(sourceTerm: string): Promise<GlossaryLookupResult> {
  try {
    const wire = await invoke<QuickAddLookupWire>(CMD_LOOKUP, { sourceTerm })
    return wire.entry === null
      ? { found: 'none', workTierAvailable: wire.work_tier_available }
      : { found: 'entry', entry: wire.entry, workTierAvailable: wire.work_tier_available }
  } catch (err) {
    if (isIpcError(err)) return { found: 'unknown', error: err }

    if (hasIpcBridge()) {
      console.error(`[glossary] \`${CMD_LOOKUP}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { found: 'unknown', error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[glossary] không gọi được \`${CMD_LOOKUP}\` — chạy ngoài Tauri? ${String(err)}`)
    return { found: 'unknown', error: null }
  }
}

/** Kết quả một lượt ghi (thêm hoặc sửa) — ba trạng thái, cùng khuôn mọi adapter khác. */
export type GlossaryWriteResult<T> = { value: T | null; error: IpcError | null }

/**
 * Thêm một mục **nhập tay** mới ở tầng `tier`. **Không bao giờ ném.** Trả về `id` mục vừa
 * tạo khi thành công.
 *
 * ⚠️ Tham số `invoke` viết camelCase — xem doc-comment đầu tệp.
 */
export async function addGlossaryTerm(
  tier: GlossaryTierWire,
  sourceTerm: string,
  translation: string | null,
  note: string,
  category: GlossaryCategory,
): Promise<GlossaryWriteResult<number>> {
  try {
    const id = await invoke<number>(CMD_ADD, {
      tier,
      sourceTerm,
      translation,
      note,
      category,
    })
    return { value: id, error: null }
  } catch (err) {
    if (isIpcError(err)) return { value: null, error: err }

    if (hasIpcBridge()) {
      console.error(`[glossary] \`${CMD_ADD}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[glossary] không gọi được \`${CMD_ADD}\` — chạy ngoài Tauri? ${String(err)}`)
    return { value: null, error: null }
  }
}

/**
 * Sửa `translation`/`note`/`category` của mục `(tier, id)`. **Không bao giờ ném.**
 *
 * ⚠️ Tham số `invoke` viết camelCase — xem doc-comment đầu tệp.
 */
export async function updateGlossaryTerm(
  tier: GlossaryTierWire,
  id: number,
  translation: string | null,
  note: string,
  category: GlossaryCategory,
): Promise<GlossaryWriteResult<true>> {
  try {
    await invoke(CMD_UPDATE, { tier, id, translation, note, category })
    return { value: true, error: null }
  } catch (err) {
    if (isIpcError(err)) return { value: null, error: err }

    if (hasIpcBridge()) {
      console.error(`[glossary] \`${CMD_UPDATE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[glossary] không gọi được \`${CMD_UPDATE}\` — chạy ngoài Tauri? ${String(err)}`)
    return { value: null, error: null }
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.4b — adapter THỨ TƯ: dấu khớp thuật ngữ cho TOÀN VĂN một Chương (FR50/FR51)
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Hình dạng `GlossaryMarkWire` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/glossary.rs::GlossaryMarkWire` cố ý KHÔNG đặt
 * `#[serde(rename_all = "camelCase")]` — cùng luật với mọi struct qua biên IPC (`is_confirmed`,
 * không `isConfirmed`).
 *
 * 🔴 **`start`/`end` là ĐIỂM MÃ, không byte, không UTF-16** — Rust đã quy đổi byte → điểm mã
 * một lần, một chỗ (`core/glossary/store.rs`). Frontend KHÔNG quy đổi lại — xem
 * `3-4b-…md` §Design Notes "Ba đơn vị đo".
 *
 * ⚠️ Cố ý KHÔNG mang `id`/`source_term` — bốn trường này đủ để VẼ một dấu, không đủ để
 * correlate hai dấu về cùng một mục Glossary (`deferred-work.md:5925-5940`). Đừng đúc thêm
 * trường ở đây để "cho tiện" — đó là một quyết định thiết kế tương tác chưa ai mở.
 */
export type GlossaryMark = {
  start: number
  end: number
  tier: GlossaryTierWire
  /** `false` == mục *chờ chốt* — dấu vẫn phải vẽ, chỉ khác KIỂU (gạch chân, không opacity). */
  is_confirmed: boolean
  /** `null` khi và chỉ khi `is_confirmed === false`. */
  translation: string | null
}

/**
 * 🔴 **SIẾT 2026-08-21 (rà ba lớp, P6 + P10)** — bản đầu chỉ hỏi `typeof`, không hỏi GIÁ TRỊ.
 * `IpcError`/`GlossaryMark` là một LỜI KHAI về dữ liệu đã qua dây, không một bảo đảm của trình
 * biên dịch (`src/AGENTS.md`); một Rust build hỏng hoặc một lượt đổi lược đồ không đồng bộ có
 * thể gửi `NaN`/số âm/phân số cho `start`/`end`, hoặc phá đúng bất biến mà
 * `commands/glossary.rs::GlossaryMarkWire` doc-comment khai (`translation` chỉ `null` KHI VÀ
 * CHỈ KHI `is_confirmed === false`). Type guard là chỗ DUY NHẤT biết — bỏ qua ở đây thì một
 * mark hỏng đi thẳng lên `StatusBar` thành *"Bản dịch: "* RỖNG, đúng lớp *"rỗng im lặng"* mà
 * kho cấm.
 */
function isGlossaryMark(value: unknown): value is GlossaryMark {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<GlossaryMark>
  return (
    typeof v.start === 'number' &&
    Number.isInteger(v.start) &&
    v.start >= 0 &&
    typeof v.end === 'number' &&
    Number.isInteger(v.end) &&
    // 🔴 `>` nghiêm ngặt, không `>=`: một span rỗng (`start === end`) không phủ ký tự nào —
    // `find_terms`/`marks_for_source_text` phía Rust không sinh được ca đó (mọi thuật ngữ
    // rỗng bị chặn TRƯỚC khi vào lượt khớp), nên nó chỉ có thể là dữ liệu hỏng.
    v.end > v.start &&
    (v.tier === 'global' || v.tier === 'work') &&
    typeof v.is_confirmed === 'boolean' &&
    // 🔴 Bất biến CHÉO trường — đây là vế P6: `is_confirmed` và `translation` phải KHỚP nhau,
    // không chỉ đúng KIỂU từng trường riêng lẻ.
    (v.is_confirmed ? typeof v.translation === 'string' : v.translation === null)
  )
}

/**
 * 🔴 **Type guard LÚC CHẠY cho cả MẢNG** — Rust có thể trả `null` cho `translation` của một
 * mục chờ chốt, và đây là chỗ DUY NHẤT biết hình dạng đó còn hợp lệ hay không (`src/AGENTS.md`
 * §"Luôn kiểm kiểu LÚC CHẠY cho dữ liệu qua dây").
 */
function isGlossaryMarkArray(value: unknown): value is GlossaryMark[] {
  return Array.isArray(value) && value.every(isGlossaryMark)
}

/** Ba trạng thái, cùng khuôn [`GlossaryWriteResult`]. */
export type GlossaryMarksResult = { marks: GlossaryMark[] | null; error: IpcError | null }

/** Tên command trên dây. Khớp `src-tauri/src/commands/glossary.rs` (module `wire`). */
const CMD_MARKS_FOR_CHAPTER = 'glossary_marks_for_chapter'

/**
 * Tìm mọi dấu khớp thuật ngữ trong `text`. **Không bao giờ ném.**
 *
 * 🔴 **Đúng MỘT lượt mỗi lần mở Chương, cộng một lượt làm mới sau gộp/tách và sau thêm nhanh
 * 3.3 — KHÔNG một lượt nào trên đường gõ** (Ice ký 2026-08-21, `3-4b-…md` §Intent). Chỗ gọi
 * (`src/panels/glossaryMarksState.ts`) chịu trách nhiệm giữ kỷ luật đó; adapter này không tự
 * giới hạn được tần suất gọi của nó.
 *
 * ⚠️ `text` phải là văn bản đã NỐI theo đúng phép cộng dồn mà `glossaryMarksMap.ts` dùng để
 * chia mark tuyệt đối về từng segment (`segment.source_text` nối bằng `\n`) — KHÔNG phải
 * `chapter.source_text` thô, thứ không cho một phép cộng dồn nghịch được (xem Design Notes
 * của story: `push_segment` `trim()` mỗi câu và bỏ câu rỗng).
 *
 * ⚠️ Tham số `invoke` viết camelCase — `sourceLang`, không `source_lang`.
 */
export async function glossaryMarksForChapter(text: string, sourceLang: string): Promise<GlossaryMarksResult> {
  try {
    const wire = await invoke<unknown>(CMD_MARKS_FOR_CHAPTER, { text, sourceLang })
    if (!isGlossaryMarkArray(wire)) {
      console.error(
        `[glossary] \`${CMD_MARKS_FOR_CHAPTER}\` tra ve mot hinh dang khong dung GlossaryMark[]`,
      )
      return { marks: null, error: UNKNOWN_IPC_ERROR }
    }
    return { marks: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { marks: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[glossary] \`${CMD_MARKS_FOR_CHAPTER}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { marks: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[glossary] không gọi được \`${CMD_MARKS_FOR_CHAPTER}\` — chạy ngoài Tauri? ${String(err)}`)
    return { marks: null, error: null }
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.5 — adapter THỨ NĂM: bảng chờ ứng viên, vỏ CHỈ-ĐỌC
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Hình dạng một hàng `glossary_candidate` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/glossary.rs::GlossaryCandidateWire` cố ý KHÔNG đặt
 * `#[serde(rename_all = "camelCase")]` — cùng luật với mọi struct qua biên IPC.
 */
export type GlossaryCandidate = {
  id: number
  source_term: string
  candidate_origin: string
  /** `null` == chờ duyệt — vị từ DUY NHẤT của `GlossaryCandidate::is_pending` phía Rust. */
  resolution: string | null
  created_at: string
  occurrence_count: number
  /** `null` cho hàng KHÔNG tới từ một lượt quét (nhập tay trước Story 3.5, hoặc thu hoạch
   * từ bản review — Epic 8, chưa gán `context_example`). */
  context_example: string | null
}

function isGlossaryCandidate(value: unknown): value is GlossaryCandidate {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<GlossaryCandidate>
  return (
    typeof v.id === 'number' &&
    Number.isInteger(v.id) &&
    typeof v.source_term === 'string' &&
    typeof v.candidate_origin === 'string' &&
    (v.resolution === null || typeof v.resolution === 'string') &&
    typeof v.created_at === 'string' &&
    typeof v.occurrence_count === 'number' &&
    Number.isInteger(v.occurrence_count) &&
    (v.context_example === null || typeof v.context_example === 'string')
  )
}

/**
 * 🔴 **Type guard LÚC CHẠY cho cả MẢNG** — cùng lý do `isGlossaryMarkArray`: dữ liệu qua
 * IPC là một LỜI KHAI, không một bảo đảm của trình biên dịch.
 */
function isGlossaryCandidateArray(value: unknown): value is GlossaryCandidate[] {
  return Array.isArray(value) && value.every(isGlossaryCandidate)
}

/** Ba trạng thái, cùng khuôn [`GlossaryMarksResult`]. */
export type GlossaryPendingCandidatesResult = {
  candidates: GlossaryCandidate[] | null
  error: IpcError | null
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/glossary.rs` (module `wire`). */
const CMD_PENDING_CANDIDATES = 'glossary_pending_candidates'

/**
 * Mọi ứng viên **chờ duyệt** của Tác phẩm đang mở. **Không bao giờ ném.**
 *
 * ⚠️ **Vỏ CHỈ-ĐỌC** — Story 3.5 không dựng component nào duyệt bảng chờ (Story 3.8). Bề
 * mặt này tồn tại để lượt quét khi nhập nghiệm thu được BẰNG MẮT (`§Intent` của story).
 */
export async function pendingGlossaryCandidates(): Promise<GlossaryPendingCandidatesResult> {
  try {
    const wire = await invoke<unknown>(CMD_PENDING_CANDIDATES)
    if (!isGlossaryCandidateArray(wire)) {
      console.error(
        `[glossary] \`${CMD_PENDING_CANDIDATES}\` tra ve mot hinh dang khong dung GlossaryCandidate[]`,
      )
      return { candidates: null, error: UNKNOWN_IPC_ERROR }
    }
    return { candidates: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { candidates: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[glossary] \`${CMD_PENDING_CANDIDATES}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { candidates: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[glossary] không gọi được \`${CMD_PENDING_CANDIDATES}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { candidates: null, error: null }
  }
}
