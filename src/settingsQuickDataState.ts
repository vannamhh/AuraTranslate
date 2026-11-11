import { computed, readonly, ref } from 'vue'
import type { IpcError } from './i18n'
import { t } from './i18n'
import {
  bootstrapQuickTranslationPurgeDays,
  KEY_QUICK_TRANSLATION_PURGE_DAYS,
  putConfig,
  SCOPE_APP_CONFIG,
} from './config/bootstrap'
import {
  quickTranslationCountOlder,
  quickTranslationPurge,
  quickTranslationStats,
} from './config/quickTranslationData'
import type { QuickTranslationStatsWire } from './config/quickTranslationData'
import { markQuickStorePurged } from './quickTranslateRun'

export type QuickDataResult =
  | { kind: 'done'; deleted: number; reclaimed: number }
  | { kind: 'not_shrunk'; deleted: number; detail: string }

const U32_MAX = 4294967295

const stats = ref<QuickTranslationStatsWire | null>(null)
const statsError = ref<IpcError | null>(null)
const statsLoaded = ref(false)
const daysInput = ref('')
const knownDays = ref<number | null>(null)
const olderCount = ref<number | null>(null)
const olderError = ref<IpcError | null>(null)
const busy = ref(false)
const confirmingAll = ref(false)
const result = ref<QuickDataResult | null>(null)
const purgeError = ref<IpcError | null>(null)
const daysSaveError = ref<IpcError | null>(null)
let countSequence = 0
let statsSequence = 0

export const quickDataStats = readonly(stats)
export const quickDataStatsError = readonly(statsError)
export const quickDataStatsLoaded = readonly(statsLoaded)
export const quickDataDaysInput = readonly(daysInput)
export const quickDataOlderCount = readonly(olderCount)
export const quickDataOlderError = readonly(olderError)
export const quickDataBusy = readonly(busy)
export const quickDataConfirmingAll = readonly(confirmingAll)
export const quickDataResult = readonly(result)
export const quickDataPurgeError = readonly(purgeError)
export const quickDataDaysSaveError = readonly(daysSaveError)

export function parsedQuickDataDays(raw: string): number | null {
  const trimmed = raw.trim()
  if (!/^\+?[0-9]+$/.test(trimmed)) return null
  const n = Number(trimmed)
  return Number.isInteger(n) && n >= 1 && n <= U32_MAX ? n : null
}

export const quickDataDaysValid = computed<boolean>(() => parsedQuickDataDays(daysInput.value) !== null)

/** Store readable and holding at least one row; the two buttons share this gate. */
export const quickDataStoreHasRows = computed<boolean>(() => stats.value !== null && stats.value.count > 0)

export const quickDataCanPurgeOlder = computed<boolean>(
  () => !busy.value && quickDataStoreHasRows.value && quickDataDaysValid.value,
)

export const quickDataCanPurgeAll = computed<boolean>(() => !busy.value && quickDataStoreHasRows.value)

export function formatQuickDataCount(count: number): string {
  return count.toLocaleString('vi-VN')
}

export function formatQuickDataBytes(bytes: number): string {
  const kb = 1024
  const mb = kb * 1024
  const gb = mb * 1024
  const fixed = (n: number): string => n.toLocaleString('vi-VN', { maximumFractionDigits: 1 })
  if (bytes >= gb) return `${fixed(bytes / gb)} GB`
  if (bytes >= mb) return `${fixed(bytes / mb)} MB`
  if (bytes >= kb) return `${fixed(bytes / kb)} KB`
  return `${fixed(bytes)} B`
}

export function quickDataResultText(value: QuickDataResult): string {
  if (value.kind === 'not_shrunk') {
    return t('settings.data.quick_not_shrunk', { count: formatQuickDataCount(value.deleted) })
  }
  return t('settings.data.quick_done', {
    count: formatQuickDataCount(value.deleted),
    size: formatQuickDataBytes(value.reclaimed),
  })
}

export const quickDataCompactError = computed<IpcError | null>(() => {
  const value = result.value
  if (value === null || value.kind !== 'not_shrunk') return null
  return {
    code: 'quick_translation.compact_failed',
    message_key: 'err.quick_translation.compact_failed',
    params: { detail: value.detail },
    retryable: false,
  }
})

async function refreshStats(): Promise<void> {
  const mine = ++statsSequence
  const read = await quickTranslationStats()
  if (mine !== statsSequence) return
  stats.value = read.value
  statsError.value = read.error
  statsLoaded.value = true
}

async function refreshOlderCount(): Promise<void> {
  const mine = ++countSequence
  const days = parsedQuickDataDays(daysInput.value)
  if (days === null || !quickDataStoreHasRows.value) {
    olderCount.value = null
    olderError.value = null
    return
  }
  const read = await quickTranslationCountOlder(days)
  if (mine !== countSequence) return
  olderCount.value = read.value
  olderError.value = read.error
}

/** Loads the row when the Settings section mounts. */
export async function loadQuickDataRow(): Promise<void> {
  if (!busy.value) {
    daysInput.value = String(knownDays.value ?? bootstrapQuickTranslationPurgeDays.value)
    confirmingAll.value = false
    result.value = null
    purgeError.value = null
    daysSaveError.value = null
  }
  await refreshStats()
  await refreshOlderCount()
}

export function setQuickDataDaysInput(value: string): void {
  daysInput.value = value
  void refreshOlderCount()
}

/** Remembers a valid number when the input is committed (blur or Enter). */
export async function commitQuickDataDays(): Promise<void> {
  const days = parsedQuickDataDays(daysInput.value)
  if (days === null || days === knownDays.value) return
  const err = await putConfig(SCOPE_APP_CONFIG, KEY_QUICK_TRANSLATION_PURGE_DAYS, String(days))
  daysSaveError.value = err
  if (err === null) knownDays.value = days
}

async function runPurge(scope: Parameters<typeof quickTranslationPurge>[0]): Promise<void> {
  if (busy.value) return
  busy.value = true
  confirmingAll.value = false
  result.value = null
  purgeError.value = null
  const done = await quickTranslationPurge(scope)
  markQuickStorePurged()
  if (done.value === null) {
    purgeError.value = done.error
  } else if (done.value.compact_error !== null) {
    result.value = { kind: 'not_shrunk', deleted: done.value.deleted, detail: done.value.compact_error }
  } else {
    result.value = {
      kind: 'done',
      deleted: done.value.deleted,
      reclaimed: Math.max(0, done.value.bytes_before - done.value.bytes_after),
    }
  }
  await refreshStats()
  await refreshOlderCount()
  busy.value = false
}

export async function purgeQuickDataOlder(): Promise<void> {
  const days = parsedQuickDataDays(daysInput.value)
  if (days === null || !quickDataCanPurgeOlder.value) return
  await commitQuickDataDays()
  await runPurge({ kind: 'older_than', days })
}

export function askPurgeQuickDataAll(): void {
  if (!quickDataCanPurgeAll.value) return
  confirmingAll.value = true
}

export function cancelPurgeQuickDataAll(): void {
  confirmingAll.value = false
}

export async function confirmPurgeQuickDataAll(): Promise<void> {
  if (!confirmingAll.value || !quickDataCanPurgeAll.value) return
  await runPurge({ kind: 'all' })
}
