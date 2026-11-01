import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { holdTargetCaret, restoreHeldSelection } from './commands/focus'

const open = ref(false)

export const toolsMenuIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)

export function toggleToolsMenu(): void {
  if (open.value) restoreHeldSelection()
  else holdTargetCaret()
  open.value = !open.value
}

export function closeToolsMenu(): void {
  if (open.value) restoreHeldSelection()
  open.value = false
}

export function resetToolsMenu(): void {
  open.value = false
}
