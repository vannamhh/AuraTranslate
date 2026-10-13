import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'

const open = ref(false)

export const toolsMenuIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)

export function toggleToolsMenu(): void {
  open.value = !open.value
}

export function closeToolsMenu(): void {
  open.value = false
}

export function resetToolsMenu(): void {
  open.value = false
}
