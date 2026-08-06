/**
 * Adapter IPC phía webview cho âm Hán Việt — Story 1.16, AC4.
 *
 * Cùng khuôn `./chapter.ts`/`./project.ts`: một lời gọi `invoke`, một `try/catch`, ⛔
 * không quy tắc nghiệp vụ nào ở đây — quy tắc sống ở Rust (`commands/dict.rs`,
 * `core/dict/mod.rs::lookup_han_viet`).
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Hình dạng `HanVietReading` phía Rust — `snake_case`, đúng như trên dây. */
export type HanVietReading = {
  primary: string
  all: string[]
  source_code: string
}

/** Hình dạng `CharacterReading` phía Rust. */
export type CharacterReading = {
  character: string
  reading: HanVietReading | null
}

/**
 * Hình dạng `HanVietLookup` phía Rust — khớp `commands::dict::wire::read_han_viet`.
 *
 * ⚠️ `layers_loaded === false` là trạng thái BÌNH THƯỜNG có tên (⛔ không lớp từ điển nào
 * đang gắn — AD-25), KHÁC với "đã tra mà ký tự không có âm" (`reading === null` trong khi
 * `layers_loaded === true`). Ba trạng thái, ⛔ không phải hai — xem AC4.
 */
export type HanVietLookup = {
  characters: CharacterReading[]
  sources_used: string[]
  layers_loaded: boolean
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/dict.rs` (module `wire`). */
const CMD_READ_HAN_VIET = 'read_han_viet'

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/**
 * Kết quả của một lượt đọc âm Hán Việt. Ba trạng thái, cùng khuôn `ReadOpenChapterResult`.
 *
 * ⚠️ Đây ⛔ không phải `{ characters: [], ... }` khi lỗi — Rust cố ý ⛔ không có nhánh lỗi
 * cho command này (xem doc-comment `commands::dict::read_han_viet`), nên `error` chỉ khác
 * `null` khi Rust trượt bằng một thứ **không phải** hình dạng mong đợi (panic, sai tên
 * tham số) hoặc khi ⛔ không có cầu IPC nào cả.
 */
export type ReadHanVietResult = {
  lookup: HanVietLookup | null
  error: IpcError | null
}

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

/**
 * Đọc âm Hán Việt cho `chars`. ⛔ Không ném — cùng lý do `readOpenChapter` không ném.
 *
 * ⚠️ Chỗ gọi chịu trách nhiệm chỉ gọi hàm này khi `source_lang === 'zh'` (AC3) — adapter
 * ⛔ không tự phán xét, cùng nguyên tắc "hàm thuần không tự quyết chính sách sản phẩm".
 */
export async function readHanViet(chars: readonly string[]): Promise<ReadHanVietResult> {
  try {
    const lookup = await invoke<HanVietLookup>(CMD_READ_HAN_VIET, { chars })
    return { lookup, error: null }
  } catch (err) {
    if (isIpcError(err)) return { lookup: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[dict] \`${CMD_READ_HAN_VIET}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { lookup: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[dict] không gọi được \`${CMD_READ_HAN_VIET}\` — chạy ngoài Tauri? ${String(err)}`)
    return { lookup: null, error: null }
  }
}

// ─────────────────────────────────────────────────────────────────────────────────────
// Story 1.17 — Panel Lookup, đường tra cứu hai pha (`commands::dict::wire::lookup_dictionary`)
// ─────────────────────────────────────────────────────────────────────────────────────

/** Hình dạng `QueryRoute` phía Rust — Quyết định #2, chuỗi định danh máy. */
export type QueryRoute = 'zh' | 'en'

/** Hình dạng `QueryBranch` phía Rust — chuỗi định danh máy, ⛔ không số thứ tự biến thể. */
export type QueryBranch = 'exact_btree' | 'char_idx' | 'fts_trigram' | 'query_too_short'

/** Hình dạng `SourceInfo` phía Rust. */
export type SourceInfo = {
  code: string
  display_name: string
}

/** Hình dạng `EntryHit` phía Rust — MỘT hàng `dict_entry`, ⛔ không một nghĩa. */
export type EntryHit = {
  entry_id: number
  source_code: string
  lang: string
  headword: string
  headword_simp: string | null
}

/** Hình dạng `ExampleRecord` phía Rust. */
export type ExampleRecord = {
  text: string
  translation: string | null
  translation_lang: string | null
  /**
   * 🔴 **FR35 — bản dịch này CÓ PHẢI ngoại ngữ không.** Quyết định của **Rust**
   * (`is_foreign_lang`), ⛔ của webview (AD-1).
   *
   * ⚠️ ⛔ **Đồng nghĩa với `translation_lang !== null`** — `"vi"` có ngôn ngữ nhưng ⛔ là
   * ngoại ngữ. Đọc TRƯỜNG NÀY để bật dấu hiệu, ⛔ tự so `translation_lang`.
   */
  translation_is_foreign: boolean
  ord: number
}

/** Hình dạng `CitationRecord` phía Rust — bảng RIÊNG với ví dụ (mang xuất xứ, FR30). */
export type CitationRecord = {
  text: string
  work: string | null
  author: string | null
  ord: number
}

/**
 * Hình dạng `SenseRecord` phía Rust — sáu phần của FR28 (trừ nguồn, đọc ở `SourceGroup`).
 *
 * ⚠️ `pos_lang`/`translation_lang` (của `ExampleRecord`) là TRƯỜNG đọc thẳng, ⛔ không suy
 * từ nội dung `pos`/`translation` (FR35) — xem doc-comment Rust `SenseRecord::pos_lang`.
 */
export type SenseRecord = {
  entry_id: number
  sense_id: number
  pos: string | null
  pos_lang: string | null
  /**
   * 🔴 **FR35 — nhãn này CÓ PHẢI nhãn ngoại ngữ không.** Quyết định của **Rust**, ⛔ của
   * webview (AD-1). Bản đầu của 1.17 bật chip theo `pos_lang !== null` và dán `VI` lên
   * đúng những nhãn **tiếng Việt** (bắt ở code review 2026-08-07).
   */
  pos_is_foreign: boolean
  gloss: string
  note: string | null
  ord: number
  examples: ExampleRecord[]
  citations: CitationRecord[]
}

/** Hình dạng `SourceGroup` phía Rust — MỘT khối = MỘT nguồn (AD-19, ⛔ hợp nhất). */
export type SourceGroup = {
  layer: string
  source: SourceInfo
  entries: EntryHit[]
  /**
   * 🔴 Tổng số đầu mục khớp của nguồn này, **⛔ bị trần cắt** (Quyết định #4 §hệ quả ③).
   * `null` ⇔ trần ⛔ chạm lớp này ⇒ `entries.length` CHÍNH LÀ số thật.
   *
   * ⚠️ Thanh nhịp đọc trường này để ⛔ **khẳng định một con số nó ⛔ biết** (AC12).
   */
  total_entries: number | null
}

/**
 * Hình dạng `GroupedLookup` phía Rust — pha một.
 *
 * ⚠️ `skipped` là một MẢNG MÃ MÁY (Quyết định #2a.2), ⛔ không đường dẫn/lỗi thô — số
 * lượng đọc qua `skipped.length`. `truncated_layers` (Quyết định #4 hệ quả ②) là danh
 * sách LỚP mà trần `LIMIT` đã cắt bớt — panel phải nói "danh sách nguồn chưa đầy đủ", ⛔ im.
 */
export type GroupedLookup = {
  route: QueryRoute
  branch: QueryBranch
  groups: SourceGroup[]
  skipped: string[]
  truncated_layers: string[]
  /**
   * 🔴 **Các nguồn có đầu mục khớp mà trần đã cắt SẠCH khỏi `groups`** — Quyết định #4
   * §hệ quả ③. Mỗi mục là `[display_name, số đầu mục]`.
   *
   * ⚠️ Đây là câu trả lời cho đúng ca AC12 dựng ra: một tệp mang nhiều nguồn, và trần
   * cấp-tệp có thể lấy hết chỗ cho nguồn có `entry_id` nhỏ hơn. FR31 đòi *"mọi định nghĩa
   * hiển thị nguồn"*, nên một nguồn bị giấu TÊN là ⛔ hiển thị — panel gọi tên nó ra.
   */
  hidden_sources: [string, number][]
  /**
   * `false` ⇔ ⛔ không lớp từ điển nào đang gắn — trạng thái BÌNH THƯỜNG có tên (AD-25),
   * KHÁC với "đã tra mà ⛔ không khớp" (`groups` rỗng nhưng `layers_loaded === true`).
   * Cùng doctrine `HanVietLookup::layers_loaded` của Story 1.16.
   */
  layers_loaded: boolean
}

/**
 * Hình dạng `LookupResponse` phía Rust — pha một GOM cộng pha hai HYDRATE, một lượt IPC.
 *
 * ⚠️ `senses_by_layer` khoá theo LỚP (`group.layer`), ⛔ không theo `entry_id` phẳng —
 * `entry_id` chỉ duy nhất TRONG một tệp. Zip `senses_by_layer[group.layer]` với
 * `group.entries` theo `entry_id` ở chỗ tiêu thụ.
 */
export type LookupResponse = {
  grouped: GroupedLookup
  senses_by_layer: Record<string, SenseRecord[]>
  /**
   * 🔴 Truy vấn đã bị trần **độ dài** (`QUERY_LENGTH_CEILING = 200`) cắt trước khi vào
   * đường tra. Cùng nguyên tắc `truncated_layers`, áp cho trần độ dài thay vì trần số hàng.
   *
   * ⚠️ Panel PHẢI nói ra: một truy vấn bị cắt rồi tra `Exact` cho 0 kết quả, và câu
   * *"⛔ tìm thấy trong từ điển"* khi đó là một câu **SAI** — hệ thống ⛔ hề tra thứ người
   * dùng chọn (bắt ở code review 2026-08-07).
   */
  query_truncated: boolean
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/dict.rs` (module `wire`). */
const CMD_LOOKUP_DICTIONARY = 'lookup_dictionary'

/** Kết quả của một lượt tra cứu. Cùng khuôn `ReadHanVietResult`. */
export type LookupDictionaryResult = {
  response: LookupResponse | null
  error: IpcError | null
}

/**
 * Tra `query` qua Panel Lookup. ⛔ Không ném — cùng lý do `readHanViet` không ném.
 *
 * ⚠️ `LookupMode::Exact` cố định phía Rust (Quyết định #3) — adapter này ⛔ không có tham
 * số chế độ, cùng nguyên tắc "hàm thuần không tự quyết chính sách sản phẩm".
 */
export async function lookupDictionary(query: string): Promise<LookupDictionaryResult> {
  try {
    const response = await invoke<LookupResponse>(CMD_LOOKUP_DICTIONARY, { query })
    return { response, error: null }
  } catch (err) {
    if (isIpcError(err)) return { response: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[dict] \`${CMD_LOOKUP_DICTIONARY}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { response: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[dict] không gọi được \`${CMD_LOOKUP_DICTIONARY}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { response: null, error: null }
  }
}
