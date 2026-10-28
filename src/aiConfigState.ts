/** The write tier follows `workIsOpen` (from `OpenWorkState` via `ai_config_get`), never `currentMode`. */
import { computed, readonly, ref } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import {
  aiConfigClearOverride,
  aiConfigDeleteKey,
  aiConfigGet,
  aiConfigSaveField,
  aiConfigSaveFields,
  aiConfigSaveKey,
  aiConfigTestConnection,
  aiConfigTestConnectionCancel,
} from './config/aiconfig'
import type { AiConfigField, AiConfigFieldWire, AiNotReadyReason } from './config/aiconfig'

const MAX_TOKENS_LIMIT = 4294967295
const PROVIDER_DEFAULT = 'default'

/** Sáu trường, ĐÚNG thứ tự hiển thị của form (`mockups/settings.html`). */
export const AI_CONFIG_FIELDS: readonly AiConfigField[] = [
  'provider',
  'endpoint',
  'model',
  'temperature',
  'max_tokens',
  'system_prompt',
]

export const AI_PROVIDERS = [
  'anthropic',
  'openai',
  'gemini',
  'openrouter',
  'mistral',
  'xai',
  'ollama',
  'lmstudio',
  'custom',
] as const
export type AiProvider = (typeof AI_PROVIDERS)[number]

type AiProviderPreset = {
  endpoint: string
  model: string
  temperature: string
  maxTokens: string
  keyPlaceholder: string
}

/**
 * Picking a provider writes `provider` plus each non-empty value here; an empty value is never written.
 * `temperature` / `maxTokens` hold a number or `default` (the field is then not sent to the provider).
 */
export const AI_PROVIDER_PRESETS: Record<AiProvider, AiProviderPreset> = {
  anthropic: {
    endpoint: 'https://api.anthropic.com/v1',
    model: 'claude-sonnet-5-5',
    temperature: 'default',
    maxTokens: '8192',
    keyPlaceholder: 'sk-ant-…',
  },
  openai: {
    endpoint: 'https://api.openai.com/v1',
    model: 'gpt-6.1-sol',
    temperature: 'default',
    maxTokens: 'default',
    keyPlaceholder: 'sk-…',
  },
  gemini: {
    endpoint: 'https://generativelanguage.googleapis.com/v1beta/openai',
    model: 'gemini-3.8-flash',
    temperature: 'default',
    maxTokens: 'default',
    keyPlaceholder: 'AIza…',
  },
  openrouter: {
    endpoint: 'https://openrouter.ai/api/v1',
    model: '',
    temperature: 'default',
    maxTokens: 'default',
    keyPlaceholder: 'sk-or-…',
  },
  mistral: {
    endpoint: 'https://api.mistral.ai/v1',
    model: 'mistral-large-latest',
    temperature: '0.2',
    maxTokens: 'default',
    keyPlaceholder: '',
  },
  xai: {
    endpoint: 'https://api.x.ai/v1',
    model: 'grok-4.7',
    temperature: 'default',
    maxTokens: 'default',
    keyPlaceholder: 'xai-…',
  },
  ollama: {
    endpoint: 'http://localhost:11434/v1',
    model: '',
    temperature: '0.2',
    maxTokens: 'default',
    keyPlaceholder: '',
  },
  lmstudio: {
    endpoint: 'http://localhost:1234/v1',
    model: '',
    temperature: '0.2',
    maxTokens: 'default',
    keyPlaceholder: '',
  },
  custom: { endpoint: '', model: '', temperature: '', maxTokens: '', keyPlaceholder: '' },
}

export const AI_CHIP_FIELDS = ['temperature', 'max_tokens'] as const
export type AiChipField = (typeof AI_CHIP_FIELDS)[number]
export const AI_CHIP_VALUES = {
  temperature: ['0.2', '0.5', '1'],
  max_tokens: ['4096', '8192', '16384'],
} as const satisfies Record<AiChipField, readonly string[]>
export type AiChipChoice = 'default' | 'custom' | (typeof AI_CHIP_VALUES)[AiChipField][number]

export function aiConfigChipChoices(field: AiChipField): readonly AiChipChoice[] {
  return ['default', ...AI_CHIP_VALUES[field], 'custom']
}

const PRESET_KEY: Record<AiChipField, 'temperature' | 'maxTokens'> = {
  temperature: 'temperature',
  max_tokens: 'maxTokens',
}

const customOpen = ref<Record<AiChipField, boolean>>({ temperature: false, max_tokens: false })

function closeCustom(): void {
  customOpen.value = { temperature: false, max_tokens: false }
}

type FieldRecord<T> = Record<AiConfigField, T>

function emptyDrafts(): FieldRecord<string> {
  return { provider: '', endpoint: '', model: '', temperature: '', max_tokens: '', system_prompt: '' }
}

function emptyFlags(): FieldRecord<boolean> {
  return { provider: false, endpoint: false, model: false, temperature: false, max_tokens: false, system_prompt: false }
}

function emptyErrors(): FieldRecord<IpcError | null> {
  return { provider: null, endpoint: null, model: null, temperature: null, max_tokens: null, system_prompt: null }
}

export type AiConfigPresetChange = {
  field: 'endpoint' | 'model' | 'temperature' | 'max_tokens'
  oldValue: string
  newValue: string
}
export type AiConfigPresetConfirm = { provider: AiProvider; changes: AiConfigPresetChange[] }

const presetConfirm = ref<AiConfigPresetConfirm | null>(null)

const resolvedFields = ref<AiConfigFieldWire[]>([])
const defaultSystemPrompt = ref('')
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

export type AiConfigReadiness = 'unknown' | 'ready' | 'unconfigured' | 'key_unchecked'

const readiness = ref<AiConfigReadiness>('unknown')
const effectiveProvider = ref('')
const effectiveModel = ref('')
const notReady = ref<AiNotReadyReason[]>([])
const workOverrides = ref<AiConfigField[]>([])
const readinessWorkOpen = ref(false)

/** Số thứ tự lượt đọc — chỉ lượt MỚI NHẤT được quyền ghi kết quả (khuôn `settingsState.ts`). */
let sequence = 0
let readinessSequence = 0

export type AiConfigViewTier = 'global' | 'work'

const tierChoice = ref<AiConfigViewTier | null>(null)

function effectiveTier(): AiConfigViewTier {
  if (!workIsOpen.value) return 'global'
  return tierChoice.value ?? 'work'
}

export const aiConfigViewTier: ComputedRef<AiConfigViewTier> = computed(effectiveTier)

function valueAtTier(wire: AiConfigFieldWire, tier: AiConfigViewTier): string {
  if (tier === 'work' || wire.tier !== 'work') return wire.value
  return wire.shadowed ?? (wire.field === 'system_prompt' ? defaultSystemPrompt.value : '')
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
  closeCustom()
  drafts.value = draftsForTier(effectiveTier())
}

export function selectAiConfigViewTier(tier: AiConfigViewTier): void {
  presetConfirm.value = null
  if (tier === 'work' && !workIsOpen.value) return
  tierChoice.value = tier
  closeCustom()
  drafts.value = draftsForTier(effectiveTier())
  saveErrors.value = emptyErrors()
  if (testState.value !== 'running') resetAiConfigTest()
}

export const aiConfigLoading: DeepReadonly<Ref<boolean>> = readonly(loading)
export const aiConfigLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const aiConfigWorkIsOpen: DeepReadonly<Ref<boolean>> = readonly(workIsOpen)
export const aiConfigKeyConfigured: DeepReadonly<Ref<boolean | null>> = readonly(keyConfigured)
/** Rust's `ready` verdict: `unknown` = not read yet or the read failed; `key_unchecked` = endpoint and model set but the keychain cannot be asked. */
export const aiConfigReadiness: DeepReadonly<Ref<AiConfigReadiness>> = readonly(readiness)
/** Unmet conditions behind `unconfigured`, in Rust's order; empty otherwise. */
export const aiConfigNotReady: DeepReadonly<Ref<AiNotReadyReason[]>> = readonly(notReady)
/** Fields the open Work currently overrides; empty when no Work is open. */
export const aiConfigWorkOverrides: DeepReadonly<Ref<AiConfigField[]>> = readonly(workOverrides)
export const aiConfigReadinessWorkIsOpen: DeepReadonly<Ref<boolean>> = readonly(readinessWorkOpen)
export const aiConfigEffectiveProvider: DeepReadonly<Ref<string>> = readonly(effectiveProvider)
export const aiConfigEffectiveModel: DeepReadonly<Ref<string>> = readonly(effectiveModel)
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
      if (trimmed === PROVIDER_DEFAULT) return true
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
      if (trimmed === PROVIDER_DEFAULT) return true
      if (!/^\+?[0-9]+$/.test(trimmed)) return false
      const n = Number(trimmed)
      return Number.isInteger(n) && n > 0 && n <= MAX_TOKENS_LIMIT
    }
    case 'endpoint':
      return isAbsoluteHttpUrl(trimmed)
    case 'system_prompt':
      return true
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

type AiConfigGetResult = Awaited<ReturnType<typeof aiConfigGet>>

function applyReadiness(result: AiConfigGetResult): void {
  if (result.error !== null || result.fields === null) {
    readiness.value = 'unknown'
    notReady.value = []
    workOverrides.value = []
    readinessWorkOpen.value = false
    return
  }
  notReady.value = result.notReady
  workOverrides.value = result.fields.filter((f) => f.tier === 'work').map((f) => f.field)
  readinessWorkOpen.value = result.workTierAvailable
  effectiveProvider.value = result.fields.find((f) => f.field === 'provider')?.value ?? ''
  effectiveModel.value = result.fields.find((f) => f.field === 'model')?.value ?? ''
  readiness.value = result.ready === true ? 'ready' : result.ready === false ? 'unconfigured' : 'key_unchecked'
}

/** Reads only the readiness verdict and provider/model for the AI panel header; drafts and the Settings section state stay untouched. */
export async function refreshAiConfigReadiness(): Promise<void> {
  const mine = ++readinessSequence
  const result = await aiConfigGet()
  if (mine !== readinessSequence) return
  applyReadiness(result)
}

/**
 * Đọc lại năm trường — gọi mỗi lần mục `ai_and_model` trở thành mục ĐANG CHỌN (khuôn
 * `loadDomainLog` của `settingsState.ts`). **Không tham số**: `workIsOpen` lấy từ
 * `result.workTierAvailable` của chính lượt `aiConfigGet` này.
 */
export async function loadAiConfigSection(): Promise<void> {
  loading.value = true
  await reloadAiConfigSection(AI_CONFIG_FIELDS)
}

let writtenSinceRead = new Set<AiConfigField>()

/**
 * Re-reads without raising `loading`: the template swaps the whole section for a loading line on that flag.
 * Only `written` drafts are refreshed, so text typed in another box during the read survives.
 */
async function reloadAiConfigSection(written: readonly AiConfigField[]): Promise<void> {
  for (const field of written) writtenSinceRead.add(field)
  const mine = ++sequence
  const result = await aiConfigGet()
  if (mine !== sequence) return // một lượt tải MỚI đã vượt mặt lượt này
  loading.value = false
  readinessSequence += 1
  applyReadiness(result)

  if (result.error !== null) {
    loadError.value = result.error
    return
  }
  loadError.value = null
  if (result.fields === null) return

  workIsOpen.value = result.workTierAvailable
  keyConfigured.value = result.keyConfigured
  resolvedFields.value = result.fields
  defaultSystemPrompt.value = result.defaultSystemPrompt
  const stored = draftsForTier(effectiveTier())
  const nextDrafts = { ...drafts.value }
  const nextCustom = { ...customOpen.value }
  for (const field of writtenSinceRead) {
    nextDrafts[field] = stored[field]
    if (field === 'temperature' || field === 'max_tokens') nextCustom[field] = false
  }
  writtenSinceRead = new Set()
  drafts.value = nextDrafts
  customOpen.value = nextCustom
  if (testState.value !== 'running') resetAiConfigTest()
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

  const err = await aiConfigSaveField(effectiveTier(), field, field === 'system_prompt' ? raw : raw.trim())

  saving.value = { ...saving.value, [field]: false }
  if (err !== null) {
    saveErrors.value = { ...saveErrors.value, [field]: err }
    return
  }

  await reloadAiConfigSection([field])
}

export type AiSystemPromptStatus = 'default' | 'edited' | 'off'

/** The app's built-in system prompt text, shown beside the editable box. */
export function aiConfigDefaultSystemPrompt(): string {
  return defaultSystemPrompt.value
}

/** Status of the viewed tier's system prompt: blank = off, the app text = default, anything else = edited. */
export function aiConfigSystemPromptStatus(): AiSystemPromptStatus {
  const value = aiConfigValueAtViewTier('system_prompt')
  if (value.trim() === '') return 'off'
  return value === defaultSystemPrompt.value ? 'default' : 'edited'
}

/** Whether "Restore default" has anything to do at the viewed tier: a Global that is not the app text, or a Work row. */
export function aiConfigSystemPromptCanRestore(): boolean {
  if (effectiveTier() === 'work') return aiConfigFieldWire('system_prompt')?.tier === 'work'
  return aiConfigSystemPromptStatus() !== 'default'
}

/** Deletes the viewed tier's `system_prompt` row: Global reads the app text again, Work inherits Global. */
export async function restoreAiConfigSystemPrompt(): Promise<void> {
  const field = 'system_prompt'
  if (saving.value[field]) return
  saving.value = { ...saving.value, [field]: true }
  saveErrors.value = { ...saveErrors.value, [field]: null }

  const err = await aiConfigClearOverride(field, effectiveTier())

  saving.value = { ...saving.value, [field]: false }
  if (err !== null) {
    saveErrors.value = { ...saveErrors.value, [field]: err }
    return
  }

  await reloadAiConfigSection([field])
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

  await reloadAiConfigSection([field])
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
  await reloadAiConfigSection([])
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

  await reloadAiConfigSection([])
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
  writtenSinceRead = new Set()
  readinessSequence += 1
  readiness.value = 'unknown'
  notReady.value = []
  workOverrides.value = []
  readinessWorkOpen.value = false
  effectiveProvider.value = ''
  effectiveModel.value = ''
  resolvedFields.value = []
  defaultSystemPrompt.value = ''
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
  inFlightSave = null
  presetConfirm.value = null
  closeCustom()
  resetAiConfigTest()
}

/** The provider chip that matches the displayed `provider` value, or `null` for an unknown value ("Endpoint khác"). */
export function aiConfigProviderChoice(): AiProvider | null {
  const current = drafts.value.provider.trim().toLowerCase()
  return AI_PROVIDERS.find((p) => p === current) ?? null
}

/** `true` when a non-empty provider value is outside the known list. */
export function aiConfigProviderIsOther(): boolean {
  return drafts.value.provider.trim() !== '' && aiConfigProviderChoice() === null
}

export function aiConfigEndpointPlaceholder(): string {
  const choice = aiConfigProviderChoice()
  return choice === null ? '' : AI_PROVIDER_PRESETS[choice].endpoint
}

export function aiConfigKeyPlaceholder(): string {
  const choice = aiConfigProviderChoice()
  return choice === null ? '' : AI_PROVIDER_PRESETS[choice].keyPlaceholder
}

function sameChipValue(a: string, b: string): boolean {
  const x = a.trim()
  const y = b.trim()
  if (x === y) return true
  if (x === '' || y === '' || x === PROVIDER_DEFAULT || y === PROVIDER_DEFAULT) return false
  return Number(x) === Number(y)
}

function isDefaultValue(stored: string): boolean {
  const v = stored.trim()
  return v === '' || v === PROVIDER_DEFAULT
}

function storedChip(field: AiChipField): AiChipChoice | null {
  const stored = aiConfigValueAtViewTier(field).trim()
  if (isDefaultValue(stored)) return 'default'
  const hit = (AI_CHIP_VALUES[field] as readonly AiChipChoice[]).find((v) => sameChipValue(v, stored))
  if (hit !== undefined) return hit
  return isAiConfigValueValid(field, stored) ? 'custom' : null
}

export function aiConfigChipIsOn(field: AiChipField, chip: AiChipChoice): boolean {
  if (customOpen.value[field]) return chip === 'custom'
  return storedChip(field) === chip
}

export function aiConfigChipInputIsOpen(field: AiChipField): boolean {
  return aiConfigChipIsOn(field, 'custom')
}

/** The draft shown in the free-entry input; the stored `default` is shown as an empty box. */
export function aiConfigChipInputValue(field: AiChipField): string {
  const draft = drafts.value[field]
  return draft.trim() === PROVIDER_DEFAULT ? '' : draft
}

/** Picks a chip: `custom` only opens the input; any other chip saves its value at the viewed tier. */
export async function selectAiConfigChip(field: AiChipField, chip: AiChipChoice): Promise<void> {
  if (inFlightSave !== null) await inFlightSave
  if (chip === 'custom') {
    customOpen.value = { ...customOpen.value, [field]: true }
    return
  }
  customOpen.value = { ...customOpen.value, [field]: false }
  drafts.value = { ...drafts.value, [field]: aiConfigValueAtViewTier(field) }
  if (aiConfigChipIsOn(field, chip)) return
  drafts.value = { ...drafts.value, [field]: chip }
  return trackSave(saveAiConfigField(field))
}

let inFlightSave: Promise<void> | null = null

function trackSave(job: Promise<void>): Promise<void> {
  const tracked = job.finally(() => {
    if (inFlightSave === tracked) inFlightSave = null
  })
  inFlightSave = tracked
  return tracked
}

/** Saves `field` when the user leaves the box: a valid draft that differs from the displayed stored value. */
export function commitAiConfigField(field: AiConfigField): Promise<void> {
  if (field === 'system_prompt') {
    if (drafts.value[field] === aiConfigValueAtViewTier(field)) return Promise.resolve()
    return trackSave(saveAiConfigField(field))
  }
  const raw = drafts.value[field].trim()
  if (raw === '') {
    drafts.value = { ...drafts.value, [field]: aiConfigValueAtViewTier(field) }
    return Promise.resolve()
  }
  if (!isAiConfigValueValid(field, raw)) return Promise.resolve()
  if (raw === aiConfigValueAtViewTier(field)) return Promise.resolve()
  return trackSave(saveAiConfigField(field))
}

/** Non-null while the in-page overwrite confirmation is open; nothing is written until it is applied. */
export const aiConfigPresetConfirm: DeepReadonly<Ref<AiConfigPresetConfirm | null>> = readonly(presetConfirm)

async function applyAiConfigPreset(provider: AiProvider): Promise<void> {
  const preset = AI_PROVIDER_PRESETS[provider]
  const fields: { field: AiConfigField; value: string }[] = [{ field: 'provider', value: provider }]
  if (preset.endpoint !== '') fields.push({ field: 'endpoint', value: preset.endpoint })
  if (preset.model !== '') fields.push({ field: 'model', value: preset.model })
  for (const field of AI_CHIP_FIELDS) {
    const value = preset[PRESET_KEY[field]]
    if (value !== '') fields.push({ field, value })
  }

  for (const { field } of fields) {
    saving.value = { ...saving.value, [field]: true }
    saveErrors.value = { ...saveErrors.value, [field]: null }
  }
  const err = await aiConfigSaveFields(effectiveTier(), fields)
  for (const { field } of fields) saving.value = { ...saving.value, [field]: false }
  if (err !== null) {
    saveErrors.value = { ...saveErrors.value, provider: err }
    return
  }
  await reloadAiConfigSection(fields.map((f) => f.field))
}

/**
 * Picks a provider chip: writes `provider`, `endpoint`, `model`, `temperature` and `max_tokens` of its preset in one call.
 * When a stored value of those four differs from the preset, opens the confirmation instead.
 */
export async function selectAiConfigProvider(provider: AiProvider): Promise<void> {
  // Clicking a pill blurs the input first: its save must land before the overwrite check reads stored values.
  if (inFlightSave !== null) await inFlightSave
  const preset = AI_PROVIDER_PRESETS[provider]
  const changes: AiConfigPresetChange[] = []
  for (const field of ['endpoint', 'model'] as const) {
    const newValue = preset[field]
    const oldValue = aiConfigValueAtViewTier(field).trim()
    if (newValue !== '' && oldValue !== '' && oldValue !== newValue) changes.push({ field, oldValue, newValue })
  }
  for (const field of AI_CHIP_FIELDS) {
    const newValue = preset[PRESET_KEY[field]]
    const oldValue = aiConfigValueAtViewTier(field).trim()
    if (newValue !== '' && oldValue !== '' && !sameChipValue(oldValue, newValue)) {
      changes.push({ field, oldValue, newValue })
    }
  }
  if (changes.length > 0) {
    presetConfirm.value = { provider, changes }
    return
  }
  presetConfirm.value = null
  return trackSave(applyAiConfigPreset(provider))
}

export function confirmAiConfigPreset(): Promise<void> {
  const pending = presetConfirm.value
  if (pending === null) return Promise.resolve()
  presetConfirm.value = null
  return trackSave(applyAiConfigPreset(pending.provider))
}

export function cancelAiConfigPreset(): void {
  presetConfirm.value = null
}

/** Commits every dirty text draft; called when Settings closes, before the inputs unmount. */
export function flushAiConfigDrafts(): Promise<void> {
  const jobs = AI_CONFIG_FIELDS.filter((f) => f !== 'provider').map((f) => commitAiConfigField(f))
  return Promise.all(jobs).then(() => undefined)
}

/** Number of AI fields the open Work overrides. */
export const aiConfigOverrideCount: ComputedRef<number> = computed(
  () => resolvedFields.value.filter((f) => f.tier === 'work').length,
)

/** Returns every Work-tier override to inheritance (`⌘⇧R` and the button share this). */
export async function clearAllAiConfigOverrides(): Promise<void> {
  if (!workIsOpen.value) return
  const overridden = resolvedFields.value.filter((f) => f.tier === 'work').map((f) => f.field)
  if (overridden.length === 0) return
  for (const field of overridden) {
    saving.value = { ...saving.value, [field]: true }
    saveErrors.value = { ...saveErrors.value, [field]: null }
  }
  for (const field of overridden) {
    const err = await aiConfigClearOverride(field)
    saving.value = { ...saving.value, [field]: false }
    if (err !== null) saveErrors.value = { ...saveErrors.value, [field]: err }
  }
  await reloadAiConfigSection(overridden)
}

export type AiConfigTestState = 'idle' | 'running' | 'connected' | 'not_configured' | 'cancelled' | 'error'

const testState = ref<AiConfigTestState>('idle')
const testModelCount = ref(0)
const testError = ref<IpcError | null>(null)
let testSequence = 0
let testWaitingForSave = false
let testCancelRequested = false

export const aiConfigTestStatus: DeepReadonly<Ref<AiConfigTestState>> = readonly(testState)
export const aiConfigTestModelCount: DeepReadonly<Ref<number>> = readonly(testModelCount)
export const aiConfigTestError: DeepReadonly<Ref<IpcError | null>> = readonly(testError)

/** Tests the SAVED configuration of the tier being viewed; waits for a blur save still in flight. */
export async function runAiConfigTest(): Promise<void> {
  if (testState.value === 'running') return
  const mine = ++testSequence
  testState.value = 'running'
  testError.value = null
  testCancelRequested = false
  if (inFlightSave !== null) {
    testWaitingForSave = true
    await inFlightSave
    testWaitingForSave = false
    if (mine !== testSequence) return
    if (cancelWasRequested()) {
      testCancelRequested = false
      testState.value = 'cancelled'
      return
    }
  }
  const result = await aiConfigTestConnection(effectiveTier())
  if (mine !== testSequence) return
  if (result.error !== null) {
    testError.value = result.error
    testState.value = 'error'
    return
  }
  if (result.outcome === null) {
    testState.value = 'idle'
    return
  }
  if (result.outcome.kind === 'connected') testModelCount.value = result.outcome.model_count
  testState.value = result.outcome.kind
}

function cancelWasRequested(): boolean {
  return testCancelRequested
}

export async function cancelAiConfigTest(): Promise<void> {
  if (testState.value !== 'running') return
  if (testWaitingForSave) {
    testCancelRequested = true
    return
  }
  await aiConfigTestConnectionCancel()
}

export function resetAiConfigTest(): void {
  testSequence += 1
  testWaitingForSave = false
  testCancelRequested = false
  testState.value = 'idle'
  testModelCount.value = 0
  testError.value = null
}
