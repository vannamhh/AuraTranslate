import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, ShallowRef } from 'vue'
import type { PanelId } from './workspaceLayout'

const visible = shallowRef<ReadonlySet<PanelId>>(new Set())

export const visibleDockPanels: DeepReadonly<ShallowRef<ReadonlySet<PanelId>>> = readonly(visible)

export function setVisibleDockPanels(ids: Iterable<PanelId>): void {
  visible.value = new Set(ids)
}

export function resetVisibleDockPanels(): void {
  visible.value = new Set()
}
