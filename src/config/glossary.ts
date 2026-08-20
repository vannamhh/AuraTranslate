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
