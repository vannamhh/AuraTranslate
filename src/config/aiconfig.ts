/**
 * Adapter IPC phía webview cho cấu hình nhà cung cấp AI — Story 4.2, FR68.
 *
 * Cùng khuôn `./pinned.ts`: một lời gọi `invoke`, một `try/catch`, không quy tắc nghiệp vụ
 * nào ở đây — quy tắc sống ở Rust (`commands/aiconfig.rs`).
 *
 * ⚠️ **`invoke()` gửi tham số ở dạng camelCase** — `tier`/`field`/`value` là một từ, nên
 * camelCase và snake_case trùng nhau ở đây; đừng đọc điều đó thành "quy ước không áp dụng".
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/** Năm trường — khớp NGUYÊN VĂN `AiConfigField::as_str()` phía Rust. */
export type AiConfigField = 'provider' | 'endpoint' | 'model' | 'temperature' | 'max_tokens'

/** Hai tầng (AD-18) — khớp NGUYÊN VĂN `AiConfigTierWire` phía Rust. */
export type AiConfigTierWire = 'global' | 'work'

/**
 * Hình dạng `AiConfigFieldWire` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/aiconfig.rs::AiConfigFieldWire` cố ý KHÔNG đặt
 * `#[serde(rename_all = "camelCase")]` — cùng luật với `PinnedEntry`/`GlossaryQuickAddEntry`.
 */
export type AiConfigFieldWire = {
  field: AiConfigField
  value: string
  tier: AiConfigTierWire
  /** Giá trị Global bị che, khi `tier === 'work'`. `null` nếu không có gì bị che. */
  shadowed: string | null
}

/**
 * Hình dạng `AiConfigGetWire` phía Rust — phong bì, KHÔNG một `AiConfigFieldWire[]` trần.
 *
 * ⚠️ `work_tier_available` tính TRỰC TIẾP từ `OpenWorkState` trong CHÍNH lượt gọi này (cùng
 * lý do `commands/glossary.rs::QuickAddLookup`) — đây là nguồn thật DUY NHẤT cho "có Tác
 * phẩm đang mở không" ở dải AI và mô hình. `aiConfigState.ts` KHÔNG được suy trạng thái đó từ
 * `currentMode`/`modes/modeState.ts` (chế độ UI có thể quay về `'library'` trong khi
 * `OpenWorkState` phía Rust vẫn `Some` — không IPC nào đóng một Tác phẩm ngoài
 * `RunEvent::Exit`).
 */
type AiConfigGetWire = {
  work_tier_available: boolean
  fields: AiConfigFieldWire[]
  /** `Option<bool>` phía Rust — `true`/`false` khi keychain trả lời được thăm dò
   * `configured`, `null` khi keychain TỪ CHỐI trả lời. Xem doc-comment
   * `commands/aiconfig.rs::AiConfigGetWire.key_configured`: `null` là một trạng thái THỨ BA,
   * không phải "chưa cấu hình" (`false`). */
  key_configured: boolean | null
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

const CMD_GET = 'ai_config_get'
const CMD_SAVE_FIELD = 'ai_config_save_field'
const CMD_CLEAR_OVERRIDE = 'ai_config_clear_override'
const CMD_SAVE_KEY = 'ai_config_save_key'
const CMD_DELETE_KEY = 'ai_config_delete_key'

/**
 * Đọc năm trường, hai tầng đã phân giải, cộng `workTierAvailable` và `keyConfigured`. Không
 * ném. `fields: null` khi lượt gọi trượt (`workTierAvailable` khi đó là `false`, `keyConfigured`
 * là `null` — không có gì để đọc, chỗ gọi PHẢI kiểm `error`/`fields` trước khi dùng hai trường
 * kia). `keyConfigured: null` khi lượt gọi THÀNH CÔNG nhưng keychain từ chối trả lời thăm dò —
 * ba giá trị (`true`/`false`/`null`), không hai; xem `AiConfigGetWire.key_configured` ở trên.
 */
export async function aiConfigGet(): Promise<{
  fields: AiConfigFieldWire[] | null
  workTierAvailable: boolean
  keyConfigured: boolean | null
  error: IpcError | null
}> {
  try {
    const wire = await invoke<AiConfigGetWire>(CMD_GET)
    return {
      fields: wire.fields,
      workTierAvailable: wire.work_tier_available,
      keyConfigured: wire.key_configured,
      error: null,
    }
  } catch (err) {
    if (isIpcError(err)) return { fields: null, workTierAvailable: false, keyConfigured: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[aiconfig] \`${CMD_GET}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { fields: null, workTierAvailable: false, keyConfigured: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[aiconfig] không gọi được \`${CMD_GET}\` — chạy ngoài Tauri? ${String(err)}`)
    return { fields: null, workTierAvailable: false, keyConfigured: null, error: null }
  }
}

/** Ghi một trường ở tầng `tier`. Không ném. */
export async function aiConfigSaveField(
  tier: AiConfigTierWire,
  field: AiConfigField,
  value: string,
): Promise<IpcError | null> {
  try {
    await invoke(CMD_SAVE_FIELD, { tier, field, value })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[aiconfig] \`${CMD_SAVE_FIELD}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[aiconfig] không gọi được \`${CMD_SAVE_FIELD}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}

/** Trả một trường TẦNG TÁC PHẨM về kế thừa. Không ném. */
export async function aiConfigClearOverride(field: AiConfigField): Promise<IpcError | null> {
  try {
    await invoke(CMD_CLEAR_OVERRIDE, { field })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[aiconfig] \`${CMD_CLEAR_OVERRIDE}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[aiconfig] không gọi được \`${CMD_CLEAR_OVERRIDE}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}

/**
 * Ghi (hoặc thay) khoá API — **luôn tầng Global** (spec 4.3, Quyết định 2026-09-17: một
 * entry keychain cho toàn ứng dụng, không theo Tác phẩm). Không nhận tham số tầng: màn hình
 * không có nút chọn tầng cho khoá, nên không có gì để chuyển tiếp — `tier: 'global'` là
 * hằng số của lời gọi này, không phải một quyết định của adapter. `commands/aiconfig.rs`
 * vẫn từ chối `tier !== Global` tại tầng lệnh (phòng thủ theo lớp); adapter này chỉ không
 * bao giờ tạo ra yêu cầu tầng khác để mà bị từ chối. Giá trị KHÔNG BAO GIỜ được trả về —
 * lời gọi thành công chỉ trả `null`, giống mọi adapter ghi khác ở tệp này.
 */
export async function aiConfigSaveKey(value: string): Promise<IpcError | null> {
  try {
    await invoke(CMD_SAVE_KEY, { tier: 'global', value })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[aiconfig] \`${CMD_SAVE_KEY}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[aiconfig] không gọi được \`${CMD_SAVE_KEY}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}

/** Xoá khoá API — luôn tầng Global, cùng lý lẽ [`aiConfigSaveKey`]. Xoá khi không có entry
 * nào vẫn là một lượt thành công (I/O Matrix spec 4.3 "Delete when none exists"). */
export async function aiConfigDeleteKey(): Promise<IpcError | null> {
  try {
    await invoke(CMD_DELETE_KEY, { tier: 'global' })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    if (hasIpcBridge()) {
      console.error(`[aiconfig] \`${CMD_DELETE_KEY}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return UNKNOWN_IPC_ERROR
    }
    console.info(`[aiconfig] không gọi được \`${CMD_DELETE_KEY}\` — chạy ngoài Tauri? ${String(err)}`)
    return null
  }
}
