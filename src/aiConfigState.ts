/** The write tier follows `workIsOpen` (from `OpenWorkState` via `ai_config_get`), never `currentMode`. */
import { computed, readonly, ref } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import {
  aiConfigClearOverride,
  aiConfigDeleteKey,
  aiConfigGet,
  aiConfigSaveField,
  aiConfigSaveKey,
} from './config/aiconfig'
import type { AiConfigField, AiConfigFieldWire } from './config/aiconfig'

const MAX_TOKENS_LIMIT = 4294967295

/** Năm trường, ĐÚNG thứ tự hiển thị của form (`mockups/settings.html`). */
export const AI_CONFIG_FIELDS: readonly AiConfigField[] = [
  'provider',
  'endpoint',
  'model',
  'temperature',
  'max_tokens',
]

type FieldRecord<T> = Record<AiConfigField, T>

function emptyDrafts(): FieldRecord<string> {
  return { provider: '', endpoint: '', model: '', temperature: '', max_tokens: '' }
}

function emptyFlags(): FieldRecord<boolean> {
  return { provider: false, endpoint: false, model: false, temperature: false, max_tokens: false }
}

function emptyErrors(): FieldRecord<IpcError | null> {
  return { provider: null, endpoint: null, model: null, temperature: null, max_tokens: null }
}

const resolvedFields = ref<AiConfigFieldWire[]>([])
const drafts = ref<FieldRecord<string>>(emptyDrafts())
const saving = ref<FieldRecord<boolean>>(emptyFlags())
const saveErrors = ref<FieldRecord<IpcError | null>>(emptyErrors())
const loading = ref(false)
const loadError = ref<IpcError | null>(null)
const workIsOpen = ref(false)

/**
 * Trạng thái khoá API — Story 4.3, FR65/FR67, NFR11. BA giá trị, không hai:
 * `true` (đã cấu hình) · `false` (chưa cấu hình) · `null` (keychain từ chối trả lời thăm dò —
 * §Boundaries spec 4.3 "the latter surfaces as an `IpcError`... not as a permanently broken
 * settings screen", và `AiConfigGetWire.key_configured` phía Rust). `null` KHÔNG được vẽ như
 * `false` — hai trạng thái khác nhau, xem `config/aiconfig.ts::AiConfigGetWire`.
 */
const keyConfigured = ref<boolean | null>(null)
/** Giá trị THÔ đang gõ dở cho khoá — không bao giờ được nạp từ một lượt đọc (Rust không bao
 * giờ trả khoá qua IPC); chỉ tồn tại từ lúc người dùng gõ tới lúc lưu thành công (khi đó bị
 * vứt) hoặc tới lúc rời màn (bị `resetAiConfigSection` vứt). */
const keyDraft = ref('')
/** Một cờ dùng chung cho cả Lưu và Xoá — cùng khuôn `saving`/`aiConfigIsSaving`: chỉ một
 * trong hai thao tác chạy tại một thời điểm, và cả hai nút cùng khoá khi cờ này bật. */
const keyBusy = ref(false)
const keyError = ref<IpcError | null>(null)

/** Số thứ tự lượt đọc — chỉ lượt MỚI NHẤT được quyền ghi kết quả (khuôn `settingsState.ts`). */
let sequence = 0

export type AiConfigViewTier = 'global' | 'work'

const tierChoice = ref<AiConfigViewTier | null>(null)

function effectiveTier(): AiConfigViewTier {
  if (!workIsOpen.value) return 'global'
  return tierChoice.value ?? 'work'
}

export const aiConfigViewTier: ComputedRef<AiConfigViewTier> = computed(effectiveTier)

function valueAtTier(wire: AiConfigFieldWire, tier: AiConfigViewTier): string {
  return tier === 'work' || wire.tier !== 'work' ? wire.value : (wire.shadowed ?? '')
}

function draftsForTier(tier: AiConfigViewTier): FieldRecord<string> {
  const next = emptyDrafts()
  for (const wire of resolvedFields.value) next[wire.field] = valueAtTier(wire, tier)
  return next
}

export function aiConfigValueAtViewTier(field: AiConfigField): string {
  const wire = aiConfigFieldWire(field)
  return wire === null ? '' : valueAtTier(wire, effectiveTier())
}

export function resetAiConfigViewTier(): void {
  tierChoice.value = null
  drafts.value = draftsForTier(effectiveTier())
}

export function selectAiConfigViewTier(tier: AiConfigViewTier): void {
  if (tier === 'work' && !workIsOpen.value) return
  tierChoice.value = tier
  drafts.value = draftsForTier(effectiveTier())
  saveErrors.value = emptyErrors()
}

export const aiConfigLoading: DeepReadonly<Ref<boolean>> = readonly(loading)
export const aiConfigLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const aiConfigWorkIsOpen: DeepReadonly<Ref<boolean>> = readonly(workIsOpen)
export const aiConfigKeyConfigured: DeepReadonly<Ref<boolean | null>> = readonly(keyConfigured)
export const aiConfigKeyBusy: DeepReadonly<Ref<boolean>> = readonly(keyBusy)
export const aiConfigKeyError: DeepReadonly<Ref<IpcError | null>> = readonly(keyError)

/** Trường đã phân giải, hoặc `null` nếu chưa đọc xong lần nào. */
export function aiConfigFieldWire(field: AiConfigField): AiConfigFieldWire | null {
  return resolvedFields.value.find((f) => f.field === field) ?? null
}

/** Giá trị THÔ đang hiển thị trong ô nhập của `field` — người dùng có thể đang gõ dở. */
export function aiConfigDraft(field: AiConfigField): string {
  return drafts.value[field]
}

export function aiConfigIsSaving(field: AiConfigField): boolean {
  return saving.value[field]
}

export function aiConfigSaveErrorFor(field: AiConfigField): IpcError | null {
  return saveErrors.value[field]
}

/** Handler của `@input` trên ô nhập của `field`. */
export function setAiConfigDraft(field: AiConfigField, value: string): void {
  drafts.value = { ...drafts.value, [field]: value }
}

/** Giá trị THÔ đang gõ dở của ô nhập khoá API. */
export function aiConfigKeyDraft(): string {
  return keyDraft.value
}

/** Handler của `@input` trên ô nhập khoá API. */
export function setAiConfigKeyDraft(value: string): void {
  keyDraft.value = value
}

/**
 * Kiểm tra hình dạng khoá API — **hàm thuần, xuất khẩu**, cùng luật
 * `core::aiconfig::validate_key` phía Rust: trim rồi từ chối rỗng/toàn khoảng trắng. Rust vẫn
 * là trọng tài cuối; hàm này chỉ tránh một vòng IPC vô ích cho một giá trị chắc chắn bị từ
 * chối (I/O Matrix spec 4.3 "Save an empty or whitespace-only key").
 */
export function isAiConfigKeyValueValid(raw: string): boolean {
  return raw.trim().length > 0
}

/**
 * Kiểm tra hình dạng một giá trị — **hàm thuần, xuất khẩu**, cùng luật
 * `core::aiconfig::validate_field` phía Rust (nhiệt độ `[0, 2]` · số token nguyên dương ·
 * endpoint là URL tuyệt đối `http`/`https` mang host · nhà cung cấp/mô hình không rỗng).
 * Rust vẫn là trọng tài cuối; hàm này chỉ tránh một vòng IPC vô ích cho một giá trị chắc
 * chắn bị từ chối.
 */
export function isAiConfigValueValid(field: AiConfigField, raw: string): boolean {
  const trimmed = raw.trim()
  switch (field) {
    case 'provider':
    case 'model':
      return trimmed.length > 0
    case 'temperature': {
      // Cùng lưới cú pháp mà `f64::from_str` phía Rust chấp nhận cho một số thập phân thường
      // (dấu tuỳ chọn, số nguyên/thập phân/mũ) — `.5`/`5.`/`1e-1`/`+0.5` đều phải qua được ở
      // đây vì `core::aiconfig::validate_temperature` sẽ nhận chúng. KHÔNG khớp `Infinity`/
      // `NaN`/hex (`0x..`): không nhánh nào của biểu thức cho phép chữ cái ngoài `eE`, nên cả
      // ba bị từ chối NGAY TẠI ĐÂY — không dựa vào khoảng `[0, 2]` phía dưới để chặn `NaN`
      // (`NaN >= 0` luôn `false`, nhưng đó là may rủi của phép so sánh, không phải một luật).
      if (!/^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/.test(trimmed)) return false
      const n = Number(trimmed)
      return Number.isFinite(n) && n >= 0 && n <= 2
    }
    case 'max_tokens': {
      if (!/^\+?[0-9]+$/.test(trimmed)) return false
      const n = Number(trimmed)
      return Number.isInteger(n) && n > 0 && n <= MAX_TOKENS_LIMIT
    }
    case 'endpoint':
      return isAbsoluteHttpUrl(trimmed)
  }
}

/** Cùng luật `core::aiconfig::validate_endpoint`: lược đồ `http`/`https` cộng một host không
 * rỗng, không khoảng trắng. `"localhost:11434"` (không lược đồ) bị từ chối. */
function isAbsoluteHttpUrl(trimmed: string): boolean {
  if (trimmed === '') return false
  const rest = trimmed.startsWith('https://')
    ? trimmed.slice('https://'.length)
    : trimmed.startsWith('http://')
      ? trimmed.slice('http://'.length)
      : null
  if (rest === null) return false

  const hostEnd = /[/?#]/.exec(rest)
  const host = hostEnd === null ? rest : rest.slice(0, hostEnd.index)
  return host.length > 0 && !/\s/.test(host)
}

/**
 * Đọc lại năm trường — gọi mỗi lần mục `ai_and_model` trở thành mục ĐANG CHỌN (khuôn
 * `loadDomainLog` của `settingsState.ts`). **Không tham số**: `workIsOpen` lấy từ
 * `result.workTierAvailable` của chính lượt `aiConfigGet` này.
 */
export async function loadAiConfigSection(): Promise<void> {
  const mine = ++sequence
  loading.value = true
  const result = await aiConfigGet()
  if (mine !== sequence) return // một lượt tải MỚI đã vượt mặt lượt này
  loading.value = false

  if (result.error !== null) {
    loadError.value = result.error
    return
  }
  loadError.value = null
  if (result.fields === null) return

  workIsOpen.value = result.workTierAvailable
  keyConfigured.value = result.keyConfigured
  resolvedFields.value = result.fields
  drafts.value = draftsForTier(effectiveTier())
}

/**
 * Lưu `field` — **hàm thuần re-validate**, không tin template: `Enter` trên ô nhập đi qua
 * `@submit` mà nút Lưu có thể đã khoá bằng chính [`isAiConfigValueValid`], nhưng handler
 * PHẢI tự kiểm lại (khuôn `saveGlossarySettings`).
 *
 */
export async function saveAiConfigField(field: AiConfigField): Promise<void> {
  if (saving.value[field]) return
  const raw = drafts.value[field]
  if (!isAiConfigValueValid(field, raw)) return

  saving.value = { ...saving.value, [field]: true }
  saveErrors.value = { ...saveErrors.value, [field]: null }

  const err = await aiConfigSaveField(effectiveTier(), field, raw.trim())

  saving.value = { ...saving.value, [field]: false }
  if (err !== null) {
    saveErrors.value = { ...saveErrors.value, [field]: err }
    return
  }

  await loadAiConfigSection()
}

/**
 * Trả `field` TẦNG TÁC PHẨM về kế thừa — chỉ có tác dụng khi trường đang ghi đè ở tầng
 * Tác phẩm (`aiConfigFieldWire(field)?.tier === 'work'`); nút gọi hàm này chỉ hiện đúng lúc
 * đó (`SettingsOverlay.vue`).
 */
export async function clearAiConfigOverride(field: AiConfigField): Promise<void> {
  if (saving.value[field]) return
  saving.value = { ...saving.value, [field]: true }
  saveErrors.value = { ...saveErrors.value, [field]: null }

  const err = await aiConfigClearOverride(field)

  saving.value = { ...saving.value, [field]: false }
  if (err !== null) {
    saveErrors.value = { ...saveErrors.value, [field]: err }
    return
  }

  await loadAiConfigSection()
}

/**
 * Lưu (hoặc thay) khoá API — **hàm thuần re-validate**, cùng khuôn [`saveAiConfigField`]:
 * `Enter` trên ô nhập đi qua `@submit` mà nút Lưu có thể đã khoá, handler PHẢI tự kiểm lại.
 * Luôn ghi tầng Global ([`aiConfigSaveKey`] không nhận tham số tầng — spec 4.3, Quyết định
 * 2026-09-17). Lượt thành công vứt draft ngay — giá trị không có lý do gì để tiếp tục sống
 * trong state sau khi đã vào keychain.
 */
export async function saveAiConfigKey(): Promise<void> {
  if (keyBusy.value) return
  const raw = keyDraft.value
  if (!isAiConfigKeyValueValid(raw)) return

  keyBusy.value = true
  keyError.value = null

  const err = await aiConfigSaveKey(raw.trim())

  keyBusy.value = false
  if (err !== null) {
    keyError.value = err
    return
  }

  keyDraft.value = ''
  await loadAiConfigSection()
}

/**
 * Xoá khoá API — luôn tầng Global, cùng lý lẽ [`saveAiConfigKey`]. Xoá khi không có entry nào
 * vẫn đi qua đường thành công (I/O Matrix spec 4.3 "Delete when none exists") — handler này
 * không tự phán đoán trạng thái trước khi gọi, Rust là trọng tài.
 */
export async function deleteAiConfigKey(): Promise<void> {
  if (keyBusy.value) return
  keyBusy.value = true
  keyError.value = null

  const err = await aiConfigDeleteKey()

  keyBusy.value = false
  if (err !== null) {
    keyError.value = err
    return
  }

  await loadAiConfigSection()
}

/**
 * Vứt toàn bộ state của mục — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Chưa có chỗ gọi SẢN PHẨM nào hôm nay (mục này sống trong lớp phủ Cài đặt,
 * không theo vòng đời một Tác phẩm cụ thể — đóng/mở Tác phẩm chỉ đổi TẦNG ghi, không có lý
 * do xoá draft đang gõ dở) — cùng khuôn `resetSettings()`/`resetGlossarySettings` (từng có,
 * bị gỡ vì 0 chỗ gọi — ở đây hàm vẫn giữ lại làm chỗ cắm sẵn, đúng luật của cổng).
 */
export function resetAiConfigSection(): void {
  sequence += 1
  resolvedFields.value = []
  drafts.value = emptyDrafts()
  saving.value = emptyFlags()
  saveErrors.value = emptyErrors()
  loading.value = false
  loadError.value = null
  workIsOpen.value = false
  tierChoice.value = null
  keyConfigured.value = null
  keyDraft.value = ''
  keyBusy.value = false
  keyError.value = null
}
