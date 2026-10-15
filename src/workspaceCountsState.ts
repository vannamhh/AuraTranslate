import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { countGlossaryEntries } from './config/glossary'
import { tmCountPairs } from './config/tm'
import type { TmPairCounts } from './config/tm'

const tmPairs = shallowRef<TmPairCounts | null>(null)
const glossaryEntries = shallowRef<number | null>(null)
let sequence = 0

/** `null` = not counted yet or the count failed; the status bar hides the item, never shows 0. */
export const workspaceTmPairCounts: DeepReadonly<Ref<TmPairCounts | null>> = readonly(tmPairs)
export const workspaceGlossaryEntryCount: DeepReadonly<Ref<number | null>> = readonly(glossaryEntries)

/** Re-reads both counts; a slower earlier call never overwrites a newer one. */
export async function refreshWorkspaceCounts(): Promise<void> {
  const mine = ++sequence
  const [pairs, entries] = await Promise.all([tmCountPairs(), countGlossaryEntries()])
  if (mine !== sequence) return
  tmPairs.value = pairs
  glossaryEntries.value = entries
}

/** Drops both counts (back to unknown) and discards any call still in flight. */
export function resetWorkspaceCounts(): void {
  sequence += 1
  tmPairs.value = null
  glossaryEntries.value = null
}
