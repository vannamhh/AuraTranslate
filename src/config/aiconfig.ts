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

/** Đọc năm trường, hai tầng đã phân giải. Không ném. `fields: null` khi lượt gọi trượt. */
export async function aiConfigGet(): Promise<{ fields: AiConfigFieldWire[] | null; error: IpcError | null }> {
  try {
    const fields = await invoke<AiConfigFieldWire[]>(CMD_GET)
    return { fields, error: null }
  } catch (err) {
    if (isIpcError(err)) return { fields: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[aiconfig] \`${CMD_GET}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { fields: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[aiconfig] không gọi được \`${CMD_GET}\` — chạy ngoài Tauri? ${String(err)}`)
    return { fields: null, error: null }
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
