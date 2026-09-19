/**
 * Adapter IPC phía webview cho bộ prompt theo thể loại — Story 4.4, FR69, `ScopeKind::Prompt
 * => "prompt" : Override` (`core/scope/kinds.rs:165`, Quyết định #1 spec 4.4).
 *
 * Cùng khuôn `./aiconfig.ts`: một lời gọi `invoke`, một `try/catch`, không quy tắc nghiệp vụ
 * nào ở đây — quy tắc sống ở Rust (`commands/promptset.rs`).
 *
 * ⚠️ **`invoke()` gửi tham số ở dạng camelCase** (`src/AGENTS.md:10`) — `new_name` phía Rust
 * (`prompt_set_rename`) đi trên dây thành `newName`; mọi tham số khác ở tệp này đã là một từ
 * nên camelCase và snake_case trùng nhau, đừng đọc điều đó thành "quy ước không áp dụng".
 *
 * ⚠️ **Cảnh báo dấu ngoặc đi trên dây như DỮ LIỆU, không bao giờ như một `IpcError`**
 * (Quyết định #3 spec 4.4: "A prompt body is never rejected for its markers"). Hai hàm ghi
 * thân (`promptSetCreate`/`promptSetUpdateBody`) trả `warnings` bên cạnh `error` — một thân
 * mang `{{glosary_terms}}` (lỗi chính tả) hay thiếu hẳn `{{glossary_terms}}` vẫn là một lượt
 * THÀNH CÔNG.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Hai tầng — khớp NGUYÊN VĂN `PromptSetTierWire` phía Rust (`commands/promptset.rs`). */
export type PromptSetTier = 'global' | 'work'

/**
 * Cảnh báo dấu ngoặc của MỘT thân prompt — khớp NGUYÊN VĂN `PromptSetWarningsWire` phía Rust.
 * `unknown_markers`: toàn văn (`"{{foo}}"`) mỗi dấu ngoặc lạ, thứ tự gặp lần đầu.
 * `glossary_terms_missing`: `true` khi thân KHÔNG mang `{{glossary_terms}}`.
 */
export type PromptSetWarningsWire = {
  unknown_markers: string[]
  glossary_terms_missing: boolean
}

/**
 * Một bộ prompt đã phân giải — khớp NGUYÊN VĂN `PromptSetWire` phía Rust, **`snake_case`,
 * đúng như trên dây** (`shadowed_body`, không `shadowedBody`).
 *
 * `shadowed_body`: thân của bộ Global cùng tên đang bị bộ này che (Quyết định #1) — `null`
 * khi bộ chỉ tồn tại ở một tầng, hoặc khi `tier === 'global'`.
 */
export type PromptSetWire = {
  id: number
  name: string
  body: string
  tier: PromptSetTier
  shadowed_body: string | null
  /**
   * Story 4.5, Quyết định #3. `id` THẬT của hàng Global bị che — cùng điều kiện `null`/số
   * với `shadowed_body`. Cho phép hàng đó được chọn, đổi tên, xoá, xuất qua chính `id` này,
   * ở tầng `"global"` — đóng `deferred-work.md §*Deferred from: 4-2-cau-hinh-nha-cung-cap-ai (2026-09-16)*`.
   */
  shadowed_id: number | null
}

/** Hình dạng `PromptSetListWire` phía Rust — phong bì, KHÔNG một `PromptSetWire[]` trần. */
type PromptSetListWire = {
  work_tier_available: boolean
  sets: PromptSetWire[]
  variables: string[]
}

/** Hình dạng `PromptSetCreateWire` phía Rust. */
type PromptSetCreateWire = {
  id: number
  warnings: PromptSetWarningsWire
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

const CMD_LIST = 'prompt_set_list'
const CMD_CREATE = 'prompt_set_create'
const CMD_RENAME = 'prompt_set_rename'
const CMD_UPDATE_BODY = 'prompt_set_update_body'
const CMD_DELETE = 'prompt_set_delete'

/**
 * Liệt kê mọi bộ prompt, hai tầng đã phân giải (Quyết định #1: cả bộ, theo tên — bộ Global bị
 * che vẫn hiện qua `shadowed_body`). Không ném. `sets: null` khi lượt gọi trượt
 * (`workTierAvailable` khi đó là `false`, `variables` khi đó là `null` — không có gì để đọc,
 * chỗ gọi PHẢI kiểm `error`/`sets` trước khi dùng `workTierAvailable`/`variables`).
 *
 * `variables`: tên (không mang `{{`/`}}`) của mỗi biến số ratify, khớp NGUYÊN VĂN
 * `PromptSetListWire::variables` phía Rust — nguồn sự thật DUY NHẤT của danh sách biến chèn ở
 * màn soạn thảo (§Always spec 4.4). `PromptLibraryOverlay.vue` render trực tiếp từ trường này,
 * không một mảng gõ tay nào của riêng nó.
 */
export async function promptSetList(): Promise<{
  sets: PromptSetWire[] | null
  workTierAvailable: boolean
  variables: string[] | null
  error: IpcError | null
}> {
  try {
    const wire = await invoke<PromptSetListWire>(CMD_LIST)
    return { sets: wire.sets, workTierAvailable: wire.work_tier_available, variables: wire.variables, error: null }
  } catch (err) {
    if (isIpcError(err)) return { sets: null, workTierAvailable: false, variables: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_LIST}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { sets: null, workTierAvailable: false, variables: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[promptset] không gọi được \`${CMD_LIST}\` — chạy ngoài Tauri? ${String(err)}`)
    return { sets: null, workTierAvailable: false, variables: null, error: null }
  }
}

/**
 * Tạo một bộ mới ở tầng `tier`. Không ném. `id: null` khi lượt gọi trượt (`warnings` khi đó
 * cũng `null`) — chỗ gọi PHẢI kiểm `error` trước khi dùng `id`/`warnings`.
 */
export async function promptSetCreate(
  tier: PromptSetTier,
  name: string,
  body: string,
): Promise<{ id: number | null; warnings: PromptSetWarningsWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<PromptSetCreateWire>(CMD_CREATE, { tier, name, body })
    return { id: wire.id, warnings: wire.warnings, error: null }
  } catch (err) {
    if (isIpcError(err)) return { id: null, warnings: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_CREATE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { id: null, warnings: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[promptset] không gọi được \`${CMD_CREATE}\` — chạy ngoài Tauri? ${String(err)}`)
    return { id: null, warnings: null, error: null }
  }
}

/** Đổi TÊN của bộ `(tier, id)`. Không ném. */
export async function promptSetRename(tier: PromptSetTier, id: number, newName: string): Promise<IpcError | null> {
  try {
    await invoke(CMD_RENAME, { tier, id, newName })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_RENAME}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[promptset] không gọi được \`${CMD_RENAME}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}

/**
 * Sửa THÂN của bộ `(tier, id)`. Không ném. Trả về cảnh báo dấu ngoặc của thân MỚI (Quyết
 * định #3) — thân không bao giờ bị từ chối vì nội dung của nó, nên `warnings` chỉ `null` khi
 * chính lượt gọi trượt.
 */
export async function promptSetUpdateBody(
  tier: PromptSetTier,
  id: number,
  body: string,
): Promise<{ warnings: PromptSetWarningsWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<PromptSetWarningsWire>(CMD_UPDATE_BODY, { tier, id, body })
    return { warnings: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { warnings: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_UPDATE_BODY}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { warnings: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[promptset] không gọi được \`${CMD_UPDATE_BODY}\` — chạy ngoài Tauri? ${String(err)}`)
    return { warnings: null, error: null }
  }
}

/**
 * Xoá bộ `(tier, id)`. Không ném. Xoá một bộ Work đang che một bộ Global cùng tên làm bộ
 * Global đó hiệu lực trở lại ngay ở lượt [`promptSetList`] tiếp theo (I/O Matrix spec 4.4
 * "Delete a shadowing Work set").
 */
export async function promptSetDelete(tier: PromptSetTier, id: number): Promise<IpcError | null> {
  try {
    await invoke(CMD_DELETE, { tier, id })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_DELETE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[promptset] không gọi được \`${CMD_DELETE}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.5 (FR79, NFR9, AD-48) — adapter THỨ SÁU đến CHÍN: hộp thoại chọn tệp nối vào
// xuất/nhập một bộ prompt (`.prompt.md`). Cả bốn lệnh MỞ HỘP THOẠI TRONG RUST — adapter này
// không cầm một quyền `dialog:*`/`fs:*` nào, khuôn nguyên văn `config/glossary.ts`'s bốn
// adapter tương ứng (Story 3.10b).
// ═════════════════════════════════════════════════════════════════════════════════

const CMD_EXPORT = 'prompt_set_export'
const CMD_OPEN_IMPORT_PREVIEW = 'prompt_set_open_import_preview'
const CMD_CONFIRM_IMPORT = 'prompt_set_confirm_import'
const CMD_CANCEL_IMPORT = 'prompt_set_cancel_import'

/**
 * Kết quả một lượt XUẤT. **Không bao giờ ném.** Bốn nhánh phân biệt được, cùng khuôn
 * `GlossaryExportResult` (`config/glossary.ts`) — `'cancelled'` (huỷ hộp thoại, im lặng CÓ
 * CHỦ) tách khỏi `'ipc_unavailable'` (không có cầu IPC — PHẢI nói ra).
 */
export type PromptSetExportResult =
  | { outcome: 'done'; path: string }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

/** Mở hộp thoại LƯU rồi xuất bộ `(tier, id)`. **Không bao giờ ném.** */
export async function promptSetExport(tier: PromptSetTier, id: number): Promise<PromptSetExportResult> {
  try {
    const path = await invoke<unknown>(CMD_EXPORT, { tier, id })
    if (path === null) return { outcome: 'cancelled' }
    if (typeof path !== 'string' || path === '') {
      console.error(`[promptset] \`${CMD_EXPORT}\` tra ve mot duong dan khong hop le: ${JSON.stringify(path)}`)
      return { outcome: 'error', error: UNKNOWN_IPC_ERROR }
    }
    return { outcome: 'done', path }
  } catch (err) {
    if (isIpcError(err)) return { outcome: 'error', error: err }
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_EXPORT}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { outcome: 'error', error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[promptset] không gọi được \`${CMD_EXPORT}\` — chạy ngoài Tauri? ${String(err)}`)
    return { outcome: 'ipc_unavailable' }
  }
}

/** Phân loại một tầng cho màn hình xem trước — khớp NGUYÊN VĂN `PromptImportTierPreviewWire`
 * phía Rust. */
export type PromptSetImportTierPreview = {
  kind: 'new' | 'identical' | 'conflict'
  existing_body: string | null
}

/** Hình dạng `PromptImportPreviewWire` phía Rust — **`snake_case`, đúng như trên dây**.
 * `work: null` ⇔ không có Tác phẩm nào đang mở lúc xem trước (Work option VẮNG MẶT, không
 * một tuỳ chọn bị vô hiệu hoá rỗng). */
export type PromptSetImportPreview = {
  file_name: string
  name: string
  body: string
  warnings: PromptSetWarningsWire
  global: PromptSetImportTierPreview
  work: PromptSetImportTierPreview | null
}

function isPromptSetImportTierPreview(value: unknown): value is PromptSetImportTierPreview {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<PromptSetImportTierPreview>
  return (
    (v.kind === 'new' || v.kind === 'identical' || v.kind === 'conflict') &&
    (v.existing_body === null || typeof v.existing_body === 'string')
  )
}

/** 🔴 Type guard LÚC CHẠY — dữ liệu qua IPC là một LỜI KHAI, không một bảo đảm của trình
 * biên dịch, cùng lý do `isGlossaryImportPreview`. */
function isPromptSetImportPreview(value: unknown): value is PromptSetImportPreview {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<PromptSetImportPreview>
  return (
    typeof v.file_name === 'string' &&
    typeof v.name === 'string' &&
    typeof v.body === 'string' &&
    typeof v.warnings === 'object' &&
    // ⚠️ Hình dạng `PromptSetImportPreview` là một LỜI KHAI về dữ liệu đã qua dây IPC, không
    // một bảo đảm của trình biên dịch — Rust có thể trả `null` cho `warnings`, và
    // `typeof null === 'object'` không tự loại trường hợp đó (cùng lý do `isIpcError`'s
    // guard `params !== null`, `config/pinned.ts`).
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.warnings !== null &&
    Array.isArray(v.warnings.unknown_markers) &&
    v.warnings.unknown_markers.every((m) => typeof m === 'string') &&
    typeof v.warnings.glossary_terms_missing === 'boolean' &&
    isPromptSetImportTierPreview(v.global) &&
    (v.work === null || isPromptSetImportTierPreview(v.work))
  )
}

/** Kết quả nhịp MỘT. **Không bao giờ ném.** Cùng bốn nhánh với [`PromptSetExportResult`]. */
export type PromptSetImportPreviewResult =
  | { outcome: 'loaded'; preview: PromptSetImportPreview }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

/**
 * Mở hộp thoại CHỌN rồi đọc/phân tích/phân loại — nhịp MỘT của lượt nhập. **Không bao giờ
 * ném.** **KHÔNG nhận `tier`** — tầng được chọn Ở MÀN XEM TRƯỚC (I/O Matrix spec 4.5 "Import
 * picks the tier"), sau khi tệp đã đọc, khác Glossary (nơi tầng chọn TRƯỚC khi mở hộp
 * thoại).
 */
export async function promptSetOpenImportPreview(): Promise<PromptSetImportPreviewResult> {
  try {
    const wire = await invoke<unknown>(CMD_OPEN_IMPORT_PREVIEW)
    if (wire === null) return { outcome: 'cancelled' }
    if (!isPromptSetImportPreview(wire)) {
      console.error(
        `[promptset] \`${CMD_OPEN_IMPORT_PREVIEW}\` tra ve mot hinh dang khong dung PromptSetImportPreview`,
      )
      return { outcome: 'error', error: UNKNOWN_IPC_ERROR }
    }
    return { outcome: 'loaded', preview: wire }
  } catch (err) {
    if (isIpcError(err)) return { outcome: 'error', error: err }
    if (hasIpcBridge()) {
      console.error(
        `[promptset] \`${CMD_OPEN_IMPORT_PREVIEW}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { outcome: 'error', error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[promptset] không gọi được \`${CMD_OPEN_IMPORT_PREVIEW}\` — chạy ngoài Tauri? ${String(err)}`)
    return { outcome: 'ipc_unavailable' }
  }
}

/** Quyết định của người dùng cho va chạm tên — khớp NGUYÊN VĂN `ConflictDecision::…`
 * (`#[serde(rename = …)]`) phía Rust. */
export type PromptSetConflictDecision = 'keep_mine' | 'take_theirs'

/** Hình dạng `PromptImportOutcomeWire` phía Rust — `#[serde(rename_all = "snake_case")]`. */
export type PromptSetImportOutcome = 'inserted' | 'updated' | 'skipped'

function isPromptSetImportOutcome(value: unknown): value is PromptSetImportOutcome {
  return value === 'inserted' || value === 'updated' || value === 'skipped'
}

/** Ba trạng thái, cùng khuôn [`GlossaryConfirmImportResult`]. */
export type PromptSetConfirmImportResult = { outcome: PromptSetImportOutcome | null; error: IpcError | null }

/**
 * Xác nhận lượt nhập — nhịp HAI. **Không bao giờ ném.** `tier` là tầng người dùng chọn Ở MÀN
 * XEM TRƯỚC; `decision` chỉ có ý nghĩa khi tầng đó phân loại `'conflict'` — vắng mặt (hoặc
 * `null`) ⇒ `'keep_mine'` (mặc định, §Always).
 */
export async function promptSetConfirmImport(
  tier: PromptSetTier,
  decision: PromptSetConflictDecision | null,
): Promise<PromptSetConfirmImportResult> {
  try {
    const wire = await invoke<unknown>(CMD_CONFIRM_IMPORT, { tier, decision })
    if (!isPromptSetImportOutcome(wire)) {
      console.error(`[promptset] \`${CMD_CONFIRM_IMPORT}\` tra ve mot hinh dang khong dung PromptSetImportOutcome`)
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }
    return { outcome: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { outcome: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_CONFIRM_IMPORT}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[promptset] không gọi được \`${CMD_CONFIRM_IMPORT}\` — chạy ngoài Tauri? ${String(err)}`)
    return { outcome: null, error: null }
  }
}

/** Huỷ lô đang treo. **Không bao giờ ném.** Vô hại khi không có lô nào. */
export async function promptSetCancelImport(): Promise<IpcError | null> {
  try {
    await invoke(CMD_CANCEL_IMPORT)
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[promptset] \`${CMD_CANCEL_IMPORT}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[promptset] không gọi được \`${CMD_CANCEL_IMPORT}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}
