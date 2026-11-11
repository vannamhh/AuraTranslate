import { computed, readonly, ref } from 'vue'
import { aiConfigViewTier } from './aiConfigState'
import type { IpcError } from './i18n'
import { quickAiConfigGet, quickAiConfigSaveFields, quickAiConfigSaveKey, quickAiConfigDeleteKey, quickAiConfigTestConnection, quickAiConfigTestConnectionCancel } from './config/quickAiConfig'
import type { QuickAiConfigField, QuickAiConfigWire } from './config/quickAiConfig'

export const QUICK_AI_FIELDS = ['quick_provider', 'quick_endpoint', 'quick_model'] as const
const emptyDrafts = (): Record<QuickAiConfigField, string> => ({ quick_provider: '', quick_endpoint: '', quick_model: '' })
const drafts = ref(emptyDrafts())
const stored = ref(emptyDrafts())
const config = ref<QuickAiConfigWire | null>(null)
const loadError = ref<IpcError | null>(null)
const saveError = ref<IpcError | null>(null)
const keyDraft = ref('')
const keyError = ref<IpcError | null>(null)
const keyBusy = ref(false)
const saving = ref(false)
const readiness = ref<'unknown' | 'ready' | 'unconfigured' | 'key_unchecked'>('unknown')
const testStatus = ref<'idle' | 'running' | 'connected' | 'not_configured' | 'cancelled' | 'error'>('idle')
const testModelCount = ref(0)
const testError = ref<IpcError | null>(null)
let sequence = 0
let readSequence = 0
let readinessSequence = 0
let testSequence = 0
let inFlightSave: Promise<void> | null = null
let cancelRequested = false
let waitingForSave = false

export const quickAiConfigLocked = computed(() => aiConfigViewTier.value !== 'global')
export const quickAiConfigDrafts = readonly(drafts)
export const quickAiConfig = readonly(config)
export const quickAiConfigLoadError = readonly(loadError)
export const quickAiConfigSaveError = readonly(saveError)
export const quickAiConfigKeyDraft = readonly(keyDraft)
export const quickAiConfigKeyError = readonly(keyError)
export const quickAiConfigKeyBusy = readonly(keyBusy)
export const quickAiConfigSaving = readonly(saving)
export const quickAiConfigReadiness = readonly(readiness)
export const quickAiConfigTestStatus = readonly(testStatus)
export const quickAiConfigTestModelCount = readonly(testModelCount)
export const quickAiConfigTestError = readonly(testError)

function applyReadiness(value: QuickAiConfigWire | null): void {
  readiness.value = value === null ? 'unknown' : value.ready === true ? 'ready' : value.ready === false ? 'unconfigured' : 'key_unchecked'
}

export async function refreshQuickAiConfigReadiness(): Promise<void> {
  const mine = ++readinessSequence
  const result = await quickAiConfigGet()
  if (mine !== readinessSequence) return
  applyReadiness(result.config)
}

export async function loadQuickAiConfigSection(): Promise<void> {
  const mine = ++readSequence
  const before = { ...drafts.value }
  const dirty = new Set(QUICK_AI_FIELDS.filter((field) => drafts.value[field] !== stored.value[field]))
  const result = await quickAiConfigGet()
  if (mine !== readSequence) return
  readinessSequence += 1
  loadError.value = result.error
  applyReadiness(result.config)
  config.value = result.config
  if (result.config === null) return
  for (const field of result.config.fields) {
    stored.value[field.field] = field.value
    if (!dirty.has(field.field) && drafts.value[field.field] === before[field.field]) drafts.value[field.field] = field.value
  }
}

export function setQuickAiConfigDraft(field: QuickAiConfigField, value: string): void {
  if (quickAiConfigLocked.value) return
  drafts.value[field] = value
  saveError.value = null
}

export function setQuickAiConfigKeyDraft(value: string): void {
  if (quickAiConfigLocked.value) return
  keyDraft.value = value
  keyError.value = null
}

export async function commitQuickAiConfigDrafts(): Promise<void> {
  if (inFlightSave !== null) await inFlightSave
  if (quickAiConfigLocked.value) return
  const fields = QUICK_AI_FIELDS.filter((field) => drafts.value[field] !== stored.value[field]).map((field) => ({ field, value: drafts.value[field] }))
  if (fields.length === 0) return
  const mine = sequence
  saving.value = true
  const save = async (): Promise<void> => {
    const error = await quickAiConfigSaveFields(fields)
    if (mine !== sequence) return
    saveError.value = error
    if (error === null) {
      invalidateQuickAiConfigTest()
      for (const field of fields) stored.value[field.field] = field.value
      await loadQuickAiConfigSection()
    }
    saving.value = false
  }
  const task = save()
  inFlightSave = task
  await task
  if (inFlightSave === task) inFlightSave = null
}

export async function saveQuickAiConfigKey(): Promise<void> {
  if (quickAiConfigLocked.value || keyBusy.value) return
  const mine = sequence
  const value = keyDraft.value
  keyDraft.value = ''
  keyBusy.value = true
  const error = await quickAiConfigSaveKey(value)
  if (mine !== sequence) return
  keyError.value = error
  if (error === null) invalidateQuickAiConfigTest()
  keyBusy.value = false
  await loadQuickAiConfigSection()
}

export async function deleteQuickAiConfigKey(): Promise<void> {
  if (quickAiConfigLocked.value || keyBusy.value) return
  const mine = sequence
  keyDraft.value = ''
  keyBusy.value = true
  const error = await quickAiConfigDeleteKey()
  if (mine !== sequence) return
  keyError.value = error
  if (error === null) invalidateQuickAiConfigTest()
  keyBusy.value = false
  await loadQuickAiConfigSection()
}

function invalidateQuickAiConfigTest(): void {
  if (testStatus.value === 'running' && waitingForSave) return
  if (testStatus.value === 'running') void quickAiConfigTestConnectionCancel()
  testSequence += 1
  testStatus.value = 'idle'
  testModelCount.value = 0
  testError.value = null
}

export async function runQuickAiConfigTest(): Promise<void> {
  if (quickAiConfigLocked.value || testStatus.value === 'running') return
  const mine = ++testSequence
  testStatus.value = 'running'
  testError.value = null
  cancelRequested = false
  waitingForSave = true
  await commitQuickAiConfigDrafts()
  waitingForSave = false
  if (mine !== testSequence) return
  if (wasCancelRequested()) { testStatus.value = 'cancelled'; return }
  if (saveError.value !== null) { testError.value = saveError.value; testStatus.value = 'error'; return }
  const result = await quickAiConfigTestConnection()
  if (mine !== testSequence) return
  testError.value = result.error
  testStatus.value = result.error !== null ? 'error' : result.outcome?.kind ?? 'idle'
  if (result.outcome?.kind === 'connected') testModelCount.value = result.outcome.model_count
}

function wasCancelRequested(): boolean { return cancelRequested }

export function discardQuickAiConfigKeyDraft(): void { keyDraft.value = '' }

export async function cancelQuickAiConfigTest(): Promise<void> {
  if (testStatus.value !== 'running') return
  cancelRequested = true
  if (waitingForSave) return
  const error = await quickAiConfigTestConnectionCancel()
  if (error !== null) { testError.value = error; testStatus.value = 'error' }
}

export function flushQuickAiConfigDrafts(): void {
  void commitQuickAiConfigDrafts()
  keyDraft.value = ''
  void cancelQuickAiConfigTest()
}

export function resetQuickAiConfig(): void {
  void cancelQuickAiConfigTest()
  sequence += 1
  readSequence += 1
  readinessSequence += 1
  testSequence += 1
  drafts.value = emptyDrafts()
  stored.value = emptyDrafts()
  config.value = null
  loadError.value = null
  saveError.value = null
  keyDraft.value = ''
  keyError.value = null
  keyBusy.value = false
  saving.value = false
  readiness.value = 'unknown'
  testStatus.value = 'idle'
  testModelCount.value = 0
  testError.value = null
  cancelRequested = false
  waitingForSave = false
  inFlightSave = null
}
