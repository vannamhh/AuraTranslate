import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import {
  addConventionRule,
  CONVENTION_WORK_TIER_UNAVAILABLE,
  deleteConventionRule,
  editConventionRule,
  listConventionRules,
  setConventionRuleEnabled,
} from './config/convention'
import type { ConventionRuleWire, ConventionTier, ConventionTierFilter, ConventionWriteAck } from './config/convention'
import { scanConventionNow } from './conventionState'
import type { IpcError } from './i18n'
import { editorChapterId, flushEditorBeforeDiscreteWrite } from './panels/editorPanelState'

const open = ref(false)
const globalRules = shallowRef<readonly ConventionRuleWire[]>([])
const workRules = shallowRef<readonly ConventionRuleWire[]>([])
const workTierAvailable = ref(false)
const hasLoaded = ref(false)
const busy = ref(false)
const failure = shallowRef<IpcError | null>(null)
const tierFilter = ref<ConventionTierFilter>('both')
const search = ref('')
const cursor = ref(0)
const editing = ref(false)
const deletePendingId = shallowRef<string | null>(null)

export const conventionAvoidDraft = ref('')
export const conventionReplacementDraft = ref('')
export const conventionAddTier = ref<ConventionTier>('global')
export const conventionEditAvoid = ref('')
export const conventionEditReplacement = ref('')

let session = 0

export const CONVENTION_TIER_ORDER: readonly ConventionTierFilter[] = ['global', 'work', 'both']

export const conventionManageIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)
export const conventionManageHasLoaded: DeepReadonly<Ref<boolean>> = readonly(hasLoaded)
export const conventionManageBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const conventionManageError: DeepReadonly<Ref<IpcError | null>> = readonly(failure)
export const conventionManageWorkTierAvailable: DeepReadonly<Ref<boolean>> = readonly(workTierAvailable)
export const conventionManageTierFilter: DeepReadonly<Ref<ConventionTierFilter>> = readonly(tierFilter)
export const conventionManageSearch: DeepReadonly<Ref<string>> = readonly(search)
export const conventionManageCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const conventionManageEditing: DeepReadonly<Ref<boolean>> = readonly(editing)

export function conventionRuleKey(rule: Pick<ConventionRuleWire, 'tier' | 'id'>): string {
  return `${rule.tier}:${rule.id}`
}

/** Both tiers by rule key, the Work row above the Global row it hides. */
export const conventionManageRules: ComputedRef<readonly ConventionRuleWire[]> = computed(() =>
  [...workRules.value, ...globalRules.value].sort((a, b) =>
    a.rule_key === b.rule_key ? 0 : a.rule_key < b.rule_key ? -1 : 1,
  ),
)

export const conventionManageCounts = computed(() => ({
  global: globalRules.value.length,
  work: workRules.value.length,
  both: globalRules.value.length + workRules.value.length,
}))

/**
 * The three numbers of the summary line. All of them count both tiers whole: the tier filter and
 * the search change which rows are shown, never what the sentence claims to have measured.
 */
export const conventionManageSummary = computed(() => ({
  rule_count: String(conventionManageCounts.value.both),
  disabled_count: String(conventionManageRules.value.filter((rule) => rule.status === 'disabled_in_work').length),
  shadowed_count: String(conventionManageRules.value.filter((rule) => rule.status === 'shadowed').length),
}))

function visibleTiers(): readonly ConventionRuleWire[] {
  if (tierFilter.value === 'global') return globalRules.value
  if (tierFilter.value === 'work') return workRules.value
  return conventionManageRules.value
}

/** The rows the table shows: the tier filter, then the search text over both phrases. */
export const conventionManageRows: ComputedRef<readonly ConventionRuleWire[]> = computed(() => {
  const needle = search.value.trim().toLowerCase()
  const tier = visibleTiers()
  if (needle === '') return tier
  return tier.filter(
    (rule) =>
      rule.avoid_phrase.toLowerCase().includes(needle) ||
      (rule.replacement_phrase ?? '').toLowerCase().includes(needle),
  )
})

export const conventionManageCurrentRule: ComputedRef<ConventionRuleWire | null> = computed(
  () => conventionManageRows.value[cursor.value] ?? null,
)

export const conventionManageDeletePending: ComputedRef<boolean> = computed(() => {
  const rule = conventionManageCurrentRule.value
  return rule !== null && deletePendingId.value === conventionRuleKey(rule)
})

export function conventionRuleCanDisable(rule: ConventionRuleWire | null): boolean {
  return rule !== null && rule.tier === 'global' && rule.status === 'in_use' && workTierAvailable.value
}

export function conventionRuleCanEnable(rule: ConventionRuleWire | null): boolean {
  return rule !== null && rule.tier === 'global' && rule.status === 'disabled_in_work'
}

function clampCursor(): void {
  const last = conventionManageRows.value.length - 1
  cursor.value = Math.max(0, Math.min(cursor.value, last))
}

function cancelPending(): void {
  deletePendingId.value = null
}

async function load(mine: number): Promise<void> {
  const [global, work] = await Promise.all([listConventionRules('global'), listConventionRules('work')])
  if (mine !== session) return
  if (global.rules === null) {
    failure.value = global.error
    return
  }
  const keep = conventionManageCurrentRule.value === null ? null : conventionRuleKey(conventionManageCurrentRule.value)
  globalRules.value = global.rules
  if (work.rules !== null) {
    workRules.value = work.rules
    workTierAvailable.value = true
  } else if (work.error?.code === CONVENTION_WORK_TIER_UNAVAILABLE) {
    workRules.value = []
    workTierAvailable.value = false
  } else {
    failure.value = work.error
    return
  }
  if (!workTierAvailable.value && tierFilter.value === 'work') tierFilter.value = 'both'
  if (!workTierAvailable.value) conventionAddTier.value = 'global'
  hasLoaded.value = true
  const kept = keep === null ? -1 : conventionManageRows.value.findIndex((rule) => conventionRuleKey(rule) === keep)
  if (kept >= 0) cursor.value = kept
  clampCursor()
}

function clearCells(): void {
  globalRules.value = []
  workRules.value = []
  workTierAvailable.value = false
  hasLoaded.value = false
  failure.value = null
  tierFilter.value = 'both'
  search.value = ''
  cursor.value = 0
  editing.value = false
  deletePendingId.value = null
  conventionAvoidDraft.value = ''
  conventionReplacementDraft.value = ''
  conventionAddTier.value = 'global'
  conventionEditAvoid.value = ''
  conventionEditReplacement.value = ''
}

export async function openConventionManage(): Promise<void> {
  if (open.value) return
  session += 1
  const mine = session
  clearCells()
  open.value = true
  busy.value = true
  await load(mine)
  if (mine === session) {
    if (workTierAvailable.value) conventionAddTier.value = 'work'
    busy.value = false
  }
}

/**
 * Drops every cell of this module. The rule lists are re-read on open; the overlay's open flag,
 * in-flight error, filters, cursor, pending delete and half-typed drafts belong to the session
 * that opened it.
 */
export function resetConventionManage(): void {
  session += 1
  open.value = false
  busy.value = false
  clearCells()
}

export function cancelConventionManageDeleteConfirm(): void {
  cancelPending()
}

export function closeConventionManage(): void {
  session += 1
  open.value = false
  busy.value = false
  failure.value = null
}

export function setConventionManageTier(value: ConventionTierFilter): void {
  if (value === 'work' && !workTierAvailable.value) return
  if (editing.value) return
  tierFilter.value = value
  cancelPending()
  cursor.value = 0
}

export function nextConventionManageTier(): void {
  const usable = CONVENTION_TIER_ORDER.filter((tier) => tier !== 'work' || workTierAvailable.value)
  const at = usable.indexOf(tierFilter.value)
  setConventionManageTier(usable[(at + 1) % usable.length])
}

export function setConventionManageSearch(value: string): void {
  search.value = value
  cancelPending()
  cursor.value = 0
}

export function setConventionManageCursor(index: number): void {
  if (editing.value) return
  const next = Math.max(0, Math.min(index, conventionManageRows.value.length - 1))
  if (next !== cursor.value) cancelPending()
  cursor.value = next
}

export function stepConventionManageCursor(delta: 1 | -1): void {
  setConventionManageCursor(cursor.value + delta)
}

export function beginConventionManageEdit(): void {
  const rule = conventionManageCurrentRule.value
  if (rule === null || editing.value || busy.value) return
  cancelPending()
  conventionEditAvoid.value = rule.avoid_phrase
  conventionEditReplacement.value = rule.replacement_phrase ?? ''
  editing.value = true
}

/** A write that succeeded reloads the list, then rescans the open Chapter once. */
async function afterWrite(mine: number, ack: ConventionWriteAck): Promise<boolean> {
  if (!ack.ok) {
    if (mine === session) {
      failure.value = ack.error
      if (ack.error?.code === 'convention.rule_missing') await load(mine)
    }
    return false
  }
  if (mine === session) await load(mine)
  if (editorChapterId.value !== null) {
    await flushEditorBeforeDiscreteWrite()
    void scanConventionNow()
  }
  return true
}

async function guarded(write: (mine: number) => Promise<void>): Promise<void> {
  if (!open.value || busy.value) return
  const mine = session
  busy.value = true
  failure.value = null
  try {
    await write(mine)
  } finally {
    if (mine === session) busy.value = false
  }
}

/** Adds the drafted rule to the chosen tier. */
export async function submitConventionRule(): Promise<void> {
  const avoid = conventionAvoidDraft.value.trim()
  const replacement = conventionReplacementDraft.value.trim()
  await guarded(async (mine) => {
    const ack = await addConventionRule(conventionAddTier.value, avoid, replacement === '' ? null : replacement)
    if ((await afterWrite(mine, ack)) && mine === session) {
      conventionAvoidDraft.value = ''
      conventionReplacementDraft.value = ''
    }
  })
}

export async function saveConventionManageEdit(): Promise<void> {
  const rule = conventionManageCurrentRule.value
  if (rule === null || !editing.value) return
  const replacement = conventionEditReplacement.value.trim()
  await guarded(async (mine) => {
    const ack = await editConventionRule(
      rule.tier,
      rule.id,
      conventionEditAvoid.value.trim(),
      replacement === '' ? null : replacement,
    )
    if ((await afterWrite(mine, ack)) && mine === session) editing.value = false
  })
}

export function cancelConventionManageEdit(): void {
  editing.value = false
}

/** First call arms the row, the second deletes it. Deleting a row that is already gone is harmless. */
export async function deleteConventionManageRule(): Promise<void> {
  const rule = conventionManageCurrentRule.value
  if (rule === null || editing.value || busy.value) return
  const key = conventionRuleKey(rule)
  if (deletePendingId.value !== key) {
    deletePendingId.value = key
    return
  }
  deletePendingId.value = null
  await guarded(async (mine) => {
    await afterWrite(mine, await deleteConventionRule(rule.tier, rule.id))
  })
}

export async function disableConventionManageRule(): Promise<void> {
  const rule = conventionManageCurrentRule.value
  if (!conventionRuleCanDisable(rule) || rule === null || editing.value) return
  cancelPending()
  await guarded(async (mine) => {
    await afterWrite(mine, await setConventionRuleEnabled(rule.rule_key, false))
  })
}

export async function enableConventionManageRule(): Promise<void> {
  const rule = conventionManageCurrentRule.value
  if (!conventionRuleCanEnable(rule) || rule === null || editing.value) return
  cancelPending()
  await guarded(async (mine) => {
    await afterWrite(mine, await setConventionRuleEnabled(rule.rule_key, true))
  })
}
