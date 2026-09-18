/**
 * Adapter IPC phía webview cho bản ghi prompt cuối cùng đã lắp — Story 4.7 (FR71, AD-14,
 * Decision 2 spec 4.7: hai nhịp riêng, LẮP RÁP+GHI và ĐỌC LẠI, không nhịp nào gọi nhịp kia).
 *
 * Cùng khuôn `./promptset.ts`: một lời gọi `invoke`, một `try/catch`, kiểm hình dạng LÚC
 * CHẠY trên MỌI tầng lồng nhau — dữ liệu qua IPC là một LỜI KHAI, không một bảo đảm của
 * trình biên dịch (`src/AGENTS.md`) — và không quy tắc nghiệp vụ nào ở đây; quy tắc sống ở
 * Rust (`commands/aiprompt.rs`).
 *
 * ⚠️ **`invoke()` gửi tham số ở dạng camelCase** (`src/AGENTS.md:10`) — `prompt_set_name`/
 * `segment_id` phía Rust (`ai_prompt_assemble`) đi trên dây thành `promptSetName`/`segmentId`.
 * Hình dạng TRẢ VỀ giữ `snake_case` nguyên văn, đúng mọi adapter khác.
 *
 * `prompt_set_tier` tái dùng NGUYÊN VĂN [`PromptSetTier`] của `./promptset.ts` (`AssembledPromptWire`
 * phía Rust ánh xạ tới `PromptSetTierWire`, đúng kiểu wire mà `./promptset.ts` đã mô hình —
 * không một bản chép Global/Work thứ ba). Tầng của mỗi thuật ngữ Glossary tái dùng NGUYÊN
 * VĂN [`GlossaryTierWire`] của `./glossary.ts`, cùng lý do.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import type { PromptSetTier } from './promptset'
import type { GlossaryTierWire as GlossaryTier } from './glossary'

/** Một thuật ngữ Glossary ĐÃ CHÈN — khớp NGUYÊN VĂN `InjectedGlossaryTermWire` phía Rust. Đủ
 * NĂM trường — bỏ sót `start`/`end`/`tier` là đúng khuyết tật §Always spec 4.7 cấm. */
export type InjectedGlossaryTermWire = {
  source_term: string
  translation: string
  start: number
  end: number
  tier: GlossaryTier
}

/** Một thuật ngữ Glossary ĐÃ CÂN NHẮC NHƯNG KHÔNG CHÈN — cùng năm trường, khớp NGUYÊN VĂN
 * `SuppressedGlossaryTermWire` phía Rust. Đọc-chỉ ở màn hình (Decision 3 spec 4.7): tệp này
 * không mang một hành động "sửa ngay" nào cho hình dạng này. */
export type SuppressedGlossaryTermWire = {
  source_term: string
  translation: string
  start: number
  end: number
  tier: GlossaryTier
}

/** Trạng thái Glossary trên dây — BA GIÁ TRỊ không được collapse (§Always spec 4.7).
 * `kind === 'not_asked'` ⇒ hai trường payload LUÔN `null`; `kind === 'asked'` ⇒ cả hai LUÔN
 * là mảng (có thể rỗng — "asked, nothing matched" vẫn là MỘT mảng rỗng, không phải thiếu). */
export type GlossaryInjectionStatusWire =
  | { kind: 'not_asked'; injected: null; suppressed_by_pending_overlap: null }
  | {
      kind: 'asked'
      injected: InjectedGlossaryTermWire[]
      suppressed_by_pending_overlap: SuppressedGlossaryTermWire[]
    }

/** Một câu tương tự tìm được trong TM — khớp NGUYÊN VĂN `SimilarSegmentWire` phía Rust.
 * KHÔNG có chỗ gọi nào của Story 4.7 tạo ra hình dạng `kind: 'searched'` mang mảng này (tham
 * số TM của `assemble_prompt` luôn `None` ở tầng Rust này) — kiểu vẫn khai để `switch` trên
 * `kind` không thiếu nhánh trước Epic 7. */
export type SimilarSegmentWire = {
  source_text: string
  target_text: string
}

/** Trạng thái TM trên dây — cùng khuôn tag `kind` với [`GlossaryInjectionStatusWire`].
 * §Never spec 4.7: không nội dung TM nào được hiện ngoài chính trạng thái này (Epic 7). */
export type TmInjectionStatusWire =
  | { kind: 'not_built_yet'; similar_segments: null }
  | { kind: 'searched'; similar_segments: SimilarSegmentWire[] }

/** Nhãn của MỘT mảnh `prompt` — khớp NGUYÊN VĂN `PromptPieceKindWire` phía Rust. Story 4.7
 * loop 1, finding B1: `'glossary'` là đúng khối `{{glossary_terms}}` mở rộng ra; `'authored'`
 * là thân do người soạn bộ prompt gõ; `'tm'` dành cho Epic 7 — không lệnh gọi nào của story 4.7
 * tạo mảnh `'tm'` mang văn bản. **SỬA loop 2, finding P7**: `'source_segment'` là biến thể MỚI
 * cho câu nguồn đã thay vào `{{source_segment}}` — TÁCH khỏi `'authored'` (bản loop 1 gộp hai
 * cái, khiến phần ĐỘNG NHẤT của prompt render giống hệt phần TĨNH NHẤT). */
export type PromptPieceKindWire = 'authored' | 'glossary' | 'source_segment' | 'tm'

/** MỘT mảnh liên tục của `prompt` đã lắp — khớp NGUYÊN VĂN `PromptPieceWire` phía Rust. Màn
 * hình vẽ TỪNG mảnh theo `kind` của nó thay vì tô cả `prompt` cùng một màu; nối `.text` của
 * TOÀN BỘ mảng theo đúng thứ tự phải cho lại `prompt` TỪNG BYTE — đối chứng ở
 * `tests/frontend/aiPromptInspector.test.ts`, không phải một lời hứa trong comment. */
export type PromptPieceWire = {
  kind: PromptPieceKindWire
  text: string
}

/** Toàn bộ sổ ghi của một lượt lắp — khớp NGUYÊN VĂN `InjectionLedgerWire` phía Rust. Không
 * trường nào bị bỏ. */
export type InjectionLedgerWire = {
  glossary: GlossaryInjectionStatusWire
  tm: TmInjectionStatusWire
  unknown_markers: string[]
  source_segment_missing: boolean
  pieces: PromptPieceWire[]
}

/**
 * Bản ghi DUY NHẤT của phiên — khớp NGUYÊN VĂN `AssembledPromptWire` phía Rust
 * (`commands/aiprompt.rs`). Mang ĐỊNH DANH của thứ đã tạo ra nó (`segment_id`/`chapter_id`/
 * `prompt_set_name`/`prompt_set_tier`) — §Always spec 4.7: "the record carries the identity
 * of what produced it... so a record from an earlier segment cannot be read as describing
 * the segment now focused". Màn hình đọc CHÍNH bản ghi này, không bao giờ lắp lại
 * (`assemble_prompt` không có mặt ở tệp này, và sẽ không bao giờ có).
 */
export type AssembledPromptWire = {
  prompt: string
  segment_id: number
  chapter_id: number
  prompt_set_name: string
  prompt_set_tier: PromptSetTier
  ledger: InjectionLedgerWire
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích `./pinned.ts`
    v.params !== null
  )
}

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function isGlossaryTier(value: unknown): value is GlossaryTier {
  return value === 'global' || value === 'work'
}

function isPromptSetTier(value: unknown): value is PromptSetTier {
  return value === 'global' || value === 'work'
}

function isInjectedGlossaryTermWire(value: unknown): value is InjectedGlossaryTermWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<InjectedGlossaryTermWire>
  return (
    typeof v.source_term === 'string' &&
    typeof v.translation === 'string' &&
    typeof v.start === 'number' &&
    typeof v.end === 'number' &&
    isGlossaryTier(v.tier)
  )
}

function isSuppressedGlossaryTermWire(value: unknown): value is SuppressedGlossaryTermWire {
  // Cùng năm trường, cùng luật — [`isInjectedGlossaryTermWire`] kiểm đúng hình dạng đó.
  return isInjectedGlossaryTermWire(value)
}

function isGlossaryInjectionStatusWire(value: unknown): value is GlossaryInjectionStatusWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as { kind?: unknown; injected?: unknown; suppressed_by_pending_overlap?: unknown }
  if (v.kind === 'not_asked') {
    return v.injected === null && v.suppressed_by_pending_overlap === null
  }
  if (v.kind === 'asked') {
    return (
      Array.isArray(v.injected) &&
      v.injected.every(isInjectedGlossaryTermWire) &&
      Array.isArray(v.suppressed_by_pending_overlap) &&
      v.suppressed_by_pending_overlap.every(isSuppressedGlossaryTermWire)
    )
  }
  return false
}

function isSimilarSegmentWire(value: unknown): value is SimilarSegmentWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<SimilarSegmentWire>
  return typeof v.source_text === 'string' && typeof v.target_text === 'string'
}

function isTmInjectionStatusWire(value: unknown): value is TmInjectionStatusWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as { kind?: unknown; similar_segments?: unknown }
  if (v.kind === 'not_built_yet') return v.similar_segments === null
  if (v.kind === 'searched') {
    return Array.isArray(v.similar_segments) && v.similar_segments.every(isSimilarSegmentWire)
  }
  return false
}

function isPromptPieceKindWire(value: unknown): value is PromptPieceKindWire {
  return value === 'authored' || value === 'glossary' || value === 'source_segment' || value === 'tm'
}

function isPromptPieceWire(value: unknown): value is PromptPieceWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<PromptPieceWire>
  return isPromptPieceKindWire(v.kind) && typeof v.text === 'string'
}

function isInjectionLedgerWire(value: unknown): value is InjectionLedgerWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<InjectionLedgerWire>
  return (
    isGlossaryInjectionStatusWire(v.glossary) &&
    isTmInjectionStatusWire(v.tm) &&
    Array.isArray(v.unknown_markers) &&
    v.unknown_markers.every((m) => typeof m === 'string') &&
    typeof v.source_segment_missing === 'boolean' &&
    Array.isArray(v.pieces) &&
    v.pieces.every(isPromptPieceWire)
  )
}

function isAssembledPromptWire(value: unknown): value is AssembledPromptWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<AssembledPromptWire>
  return (
    typeof v.prompt === 'string' &&
    typeof v.segment_id === 'number' &&
    typeof v.chapter_id === 'number' &&
    typeof v.prompt_set_name === 'string' &&
    isPromptSetTier(v.prompt_set_tier) &&
    isInjectionLedgerWire(v.ledger)
  )
}

const CMD_ASSEMBLE = 'ai_prompt_assemble'
const CMD_READ_RECORD = 'ai_prompt_read_record'

/**
 * LẮP RÁP một prompt cho `segmentId` bằng bộ prompt hiệu lực tên `promptSetName`, rồi GHI
 * bản ghi DUY NHẤT của phiên (đè bản ghi cũ, nếu có) — nhịp GHI của Decision 2. **Không bao
 * giờ ném.** `value: null` khi lượt gọi trượt — chỗ gọi PHẢI kiểm `error` trước khi dùng
 * `value`.
 */
export async function aiPromptAssemble(
  promptSetName: string | null,
  segmentId: number,
): Promise<{ value: AssembledPromptWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_ASSEMBLE, { promptSetName, segmentId })
    if (!isAssembledPromptWire(wire)) {
      console.error(`[aiprompt] \`${CMD_ASSEMBLE}\` tra ve mot hinh dang khong dung AssembledPromptWire`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { value: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[aiprompt] \`${CMD_ASSEMBLE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[aiprompt] không gọi được \`${CMD_ASSEMBLE}\` — chạy ngoài Tauri? ${String(err)}`)
    return { value: null, error: null }
  }
}

/**
 * ĐỌC lại bản ghi hiện tại của phiên — nhịp ĐỌC của Decision 2, KHÔNG BAO GIỜ lắp ráp gì
 * (đây là toàn bộ lý do tệp này không có một hàm "lắp rồi đọc" gộp lại). **Không bao giờ
 * ném.**
 *
 * 🔴 **SỬA — findings B4/E2/E3/E10 (loop 1): trả `{ value, error }`, không còn bare
 * `T | null`.** Bản trước gộp BA sự thật khác nhau vào cùng một `null`: (1) chưa có lượt
 * [`aiPromptAssemble`] nào chạy trong phiên (I/O Matrix "Nothing recorded yet ... this is a
 * state, not an error" — sự thật DUY NHẤT lệnh Rust này thật sự trả về `None`), (2) hình dạng
 * dây sai (lỗi CẤU HÌNH), (3) không gọi được IPC (bridge chưa sẵn/mất kết nối). Chỗ gọi
 * ([`aiPromptInspectorState.ts::refreshAiPromptRecord`]) cần phân biệt (1) — một `Option`
 * hợp lệ, ghi đè bản ghi cũ là ĐÚNG — khỏi (2)/(3), nơi ghi đè bằng `null` sẽ xoá một bản ghi
 * TỐT đã có chỉ vì MỘT lượt Đọc trượt.
 *
 * `{ value: T | null, error: null }` ⇔ đọc thành công — `value: null` LÀ chính I/O Matrix
 * "Nothing recorded yet", không phải một lỗi. `{ value: null, error: IpcError }` ⇔ đọc TRƯỢT —
 * kể cả nhánh không có cầu Tauri (KHÁC quy ước `{ value: null, error: null }` các adapter
 * `Result`-mang khác dùng cho ca đó: ở ĐÂY không có tín hiệu nào khác để chỗ gọi biết "lượt Đọc
 * này không đáng tin" — `UNKNOWN_IPC_ERROR` đóng vai đó).
 */
export async function aiPromptReadRecord(): Promise<{ value: AssembledPromptWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_READ_RECORD)
    if (wire === null) return { value: null, error: null }
    if (!isAssembledPromptWire(wire)) {
      console.error(`[aiprompt] \`${CMD_READ_RECORD}\` tra ve mot hinh dang khong dung AssembledPromptWire`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    // ⚠️ Lệnh này không mang `Result` phía Rust — một `IpcError` ở đây là một lỗi CẤU HÌNH
    // (state chưa được `app.manage`, xem `commands/aiprompt.rs::wire::ai_prompt_read_record`),
    // không một ca sản phẩm bình thường. Vẫn không ném: ghi rõ rồi trả lỗi, cùng luật mọi
    // adapter khác của tệp này.
    if (isIpcError(err)) {
      console.error(`[aiprompt] \`${CMD_READ_RECORD}\` (không mang Result phía Rust) ném một IpcError: ${err.code}`)
      return { value: null, error: err }
    }
    if (hasIpcBridge()) {
      console.error(`[aiprompt] \`${CMD_READ_RECORD}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[aiprompt] không gọi được \`${CMD_READ_RECORD}\` — chạy ngoài Tauri? ${String(err)}`)
    return { value: null, error: UNKNOWN_IPC_ERROR }
  }
}
