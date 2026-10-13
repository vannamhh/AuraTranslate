import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, ShallowRef } from 'vue'
import type { PresetId } from './workspaceLayout'

const active = shallowRef<PresetId | null>(null)

export const activeLayoutPreset: DeepReadonly<ShallowRef<PresetId | null>> = readonly(active)

export function setActiveLayoutPreset(id: PresetId | null): void {
  active.value = id
}

export function resetActiveLayoutPreset(): void {
  active.value = null
}
