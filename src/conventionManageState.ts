import { readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { addConventionRule, listConventionRules } from './config/convention'
import type { ConventionRuleWire } from './config/convention'
import { scanConventionNow } from './conventionState'
import type { IpcError } from './i18n'
import { editorChapterId, flushEditorBeforeDiscreteWrite } from './panels/editorPanelState'

const open = ref(false)
const rules = shallowRef<readonly ConventionRuleWire[]>([])
const hasLoaded = ref(false)
const busy = ref(false)
const failure = shallowRef<IpcError | null>(null)

export const conventionAvoidDraft = ref('')
export const conventionReplacementDraft = ref('')

let session = 0

export const conventionManageIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)
export const conventionManageRules: Readonly<Ref<readonly ConventionRuleWire[]>> = rules
export const conventionManageHasLoaded: DeepReadonly<Ref<boolean>> = readonly(hasLoaded)
export const conventionManageBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const conventionManageError: DeepReadonly<Ref<IpcError | null>> = readonly(failure)

async function load(mine: number): Promise<void> {
  const result = await listConventionRules()
  if (mine !== session) return
  if (result.rules === null) {
    failure.value = result.error
    return
  }
  rules.value = result.rules
  hasLoaded.value = true
}

export async function openConventionManage(): Promise<void> {
  if (open.value) return
  session += 1
  const mine = session
  open.value = true
  rules.value = []
  hasLoaded.value = false
  failure.value = null
  conventionAvoidDraft.value = ''
  conventionReplacementDraft.value = ''
  busy.value = true
  await load(mine)
  if (mine === session) busy.value = false
}

export function closeConventionManage(): void {
  session += 1
  open.value = false
  busy.value = false
  failure.value = null
}

/** Adds the drafted rule, reloads the list, then rescans the open Chapter so the new rule marks it. */
export async function submitConventionRule(): Promise<void> {
  if (!open.value || busy.value) return
  const avoid = conventionAvoidDraft.value.trim()
  const replacement = conventionReplacementDraft.value.trim()
  const mine = session
  busy.value = true
  failure.value = null
  try {
    const added = await addConventionRule(avoid, replacement === '' ? null : replacement)
    if (!added.ok) {
      if (mine === session) failure.value = added.error
      return
    }
    if (mine === session) {
      conventionAvoidDraft.value = ''
      conventionReplacementDraft.value = ''
      await load(mine)
    }
    if (editorChapterId.value !== null) {
      await flushEditorBeforeDiscreteWrite()
      void scanConventionNow()
    }
  } finally {
    if (mine === session) busy.value = false
  }
}
