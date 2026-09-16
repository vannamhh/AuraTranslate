/**
 * State của mục **AI và mô hình** trong lớp phủ Cài đặt — Story 4.2, FR68, `core/scope/
 * kinds.rs:175` (`ScopeKind::AiConfig`, ghi đè THEO TỪNG TRƯỜNG, Ice ký 2026-08-04).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 TẦNG ĐÍCH CỦA LƯỢT LƯU THEO TRẠNG THÁI Ứng dụng, KHÔNG MỘT THANH CHUYỂN PHẠM VI RIÊNG
 * ─────────────────────────────────────────────────────────────────────────────
 * Không có Tác phẩm nào đang mở ⇒ mọi lượt Lưu ghi tầng **Global** (không nơi nào khác để
 * ghi). Một Tác phẩm đang mở ⇒ mọi lượt Lưu ghi tầng **Tác phẩm** — đúng hành động "ghi đè
 * cấu hình cho Tác phẩm này đang mở". `workIsOpen` đọc `currentMode !== 'library'`
 * (`modes/modeState.ts`): `workspace`/`reading` chỉ vào được sau khi một Tác phẩm đã mở.
 * Sửa GIÁ TRỊ GLOBAL trong khi một Tác phẩm đang mở không có đường trong màn hình này ở
 * story này — đóng Tác phẩm rồi sửa. Ghi nợ có chủ, không phải một chỗ sót
 * (`deferred-work.md`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG THANH CHUYỂN PHẠM VI DẠNG NÚT — khác `CleanupRuleTier`/`GlossaryTier`
 * ─────────────────────────────────────────────────────────────────────────────
 * Mỗi trường hiện ĐÚNG MỘT giá trị đã phân giải (`AiConfigFieldWire.value`), kèm nhãn tầng
 * đọc được (`tier`) và giá trị Global bị che khi có (`shadowed`) — khuôn
 * `mockups/settings.html:172`. Không nút "Tác phẩm/Toàn cục" nào cho người dùng tự chọn
 * tầng ghi: tầng ghi LUÔN là tầng mà màn hình đang thao tác (xem trên).
 *
 * ⚠️ Cùng luật mọi state Vue khác: tệp này KHÔNG được `import` vào `src/commands/index.ts`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import {
  aiConfigClearOverride,
  aiConfigGet,
  aiConfigSaveField,
} from './config/aiconfig'
import type { AiConfigField, AiConfigFieldWire } from './config/aiconfig'

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

/** Số thứ tự lượt đọc — chỉ lượt MỚI NHẤT được quyền ghi kết quả (khuôn `settingsState.ts`). */
let sequence = 0

export const aiConfigLoading: DeepReadonly<Ref<boolean>> = readonly(loading)
export const aiConfigLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const aiConfigWorkIsOpen: DeepReadonly<Ref<boolean>> = readonly(workIsOpen)

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
      return Number.isInteger(n) && n > 0
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
 * `loadDomainLog` của `settingsState.ts`). `isWorkOpen` đến từ `currentMode !== 'library'`
 * ở nơi gọi (`SettingsOverlay.vue`) — tệp này không tự `import` `modes/modeState.ts` để giữ
 * nó độc lập với vòng đời chế độ, cùng lý lẽ `glossarySettingsState.ts` không tự đọc
 * `OpenWorkState`.
 */
export async function loadAiConfigSection(isWorkOpen: boolean): Promise<void> {
  const mine = ++sequence
  loading.value = true
  workIsOpen.value = isWorkOpen
  const result = await aiConfigGet()
  if (mine !== sequence) return // một lượt tải MỚI đã vượt mặt lượt này
  loading.value = false

  if (result.error !== null) {
    loadError.value = result.error
    return
  }
  loadError.value = null
  if (result.fields === null) return

  resolvedFields.value = result.fields
  const next = emptyDrafts()
  for (const wire of result.fields) next[wire.field] = wire.value
  drafts.value = next
}

/**
 * Lưu `field` — **hàm thuần re-validate**, không tin template: `Enter` trên ô nhập đi qua
 * `@submit` mà nút Lưu có thể đã khoá bằng chính [`isAiConfigValueValid`], nhưng handler
 * PHẢI tự kiểm lại (khuôn `saveGlossarySettings`).
 *
 * Tầng ghi là [`aiConfigWorkIsOpen`] tại THỜI ĐIỂM GỌI — xem §Tầng đích ở đầu tệp.
 */
export async function saveAiConfigField(field: AiConfigField): Promise<void> {
  if (saving.value[field]) return
  const raw = drafts.value[field]
  if (!isAiConfigValueValid(field, raw)) return

  saving.value = { ...saving.value, [field]: true }
  saveErrors.value = { ...saveErrors.value, [field]: null }

  const tier = workIsOpen.value ? 'work' : 'global'
  const err = await aiConfigSaveField(tier, field, raw.trim())

  saving.value = { ...saving.value, [field]: false }
  if (err !== null) {
    saveErrors.value = { ...saveErrors.value, [field]: err }
    return
  }

  await loadAiConfigSection(workIsOpen.value)
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

  await loadAiConfigSection(workIsOpen.value)
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
}
