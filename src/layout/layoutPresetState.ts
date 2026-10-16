import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, ShallowRef } from 'vue'
import { deleteConfig, putConfig } from '../config/bootstrap'
import type { PresetId } from './workspaceLayout'

const active = shallowRef<PresetId | null>(null)

export const activeLayoutPreset: DeepReadonly<ShallowRef<PresetId | null>> = readonly(active)

export function setActiveLayoutPreset(id: PresetId | null): void {
  active.value = id
}

export function resetActiveLayoutPreset(): void {
  active.value = null
}

// ── User presets (named layouts, `ScopeKind::LayoutPreset`, Global) ────────────────────

export type UserLayoutPreset = { name: string; json: string }

export const LAYOUT_PRESET_SCOPE = 'layout_preset'
export const MAX_PRESET_NAME_LENGTH = 60

const userPresets = shallowRef<readonly UserLayoutPreset[]>([])
const presetErrorKey = shallowRef<string | null>(null)

export const userLayoutPresets: DeepReadonly<ShallowRef<readonly UserLayoutPreset[]>> = readonly(userPresets)
export const layoutPresetErrorKey: DeepReadonly<ShallowRef<string | null>> = readonly(presetErrorKey)

/** Called once from `main.ts` with `BootstrapConfig.layout_presets`. */
export function initUserLayoutPresets(raw: Readonly<Record<string, string>>): void {
  userPresets.value = Object.entries(raw)
    .map(([name, json]) => ({ name, json }))
    .sort((a, b) => a.name.localeCompare(b.name))
}

export function resetUserLayoutPresets(): void {
  userPresets.value = []
  presetErrorKey.value = null
}

function nameProblem(name: string, ignore: string | null): string | null {
  if (name === '') return 'settings.layout.error_name_empty'
  if (name.length > MAX_PRESET_NAME_LENGTH) return 'settings.layout.error_name_long'
  if (userPresets.value.some((p) => p.name === name && p.name !== ignore)) return 'settings.layout.error_name_taken'
  return null
}

function sortedWith(next: readonly UserLayoutPreset[]): readonly UserLayoutPreset[] {
  return [...next].sort((a, b) => a.name.localeCompare(b.name))
}

/** Saves `json` under `rawName`. Returns `true` when the preset was written. */
export async function saveUserLayoutPreset(rawName: string, json: string | null): Promise<boolean> {
  const name = rawName.trim()
  presetErrorKey.value = null
  const problem = nameProblem(name, null)
  if (problem !== null) {
    presetErrorKey.value = problem
    return false
  }
  if (json === null) {
    presetErrorKey.value = 'settings.layout.error_no_workspace'
    return false
  }
  const err = await putConfig(LAYOUT_PRESET_SCOPE, name, json)
  if (err !== null) {
    presetErrorKey.value = 'settings.layout.error_save_failed'
    return false
  }
  userPresets.value = sortedWith([...userPresets.value, { name, json }])
  return true
}

export async function renameUserLayoutPreset(oldName: string, rawName: string): Promise<boolean> {
  const name = rawName.trim()
  presetErrorKey.value = null
  const existing = userPresets.value.find((p) => p.name === oldName)
  if (existing === undefined || name === oldName) return false
  const problem = nameProblem(name, oldName)
  if (problem !== null) {
    presetErrorKey.value = problem
    return false
  }
  const writeErr = await putConfig(LAYOUT_PRESET_SCOPE, name, existing.json)
  if (writeErr !== null) {
    presetErrorKey.value = 'settings.layout.error_save_failed'
    return false
  }
  const deleteErr = await deleteConfig(LAYOUT_PRESET_SCOPE, oldName)
  if (deleteErr !== null) {
    presetErrorKey.value = 'settings.layout.error_save_failed'
    userPresets.value = sortedWith([...userPresets.value, { name, json: existing.json }])
    return false
  }
  userPresets.value = sortedWith(userPresets.value.map((p) => (p.name === oldName ? { name, json: p.json } : p)))
  return true
}

export async function deleteUserLayoutPreset(name: string): Promise<boolean> {
  presetErrorKey.value = null
  const err = await deleteConfig(LAYOUT_PRESET_SCOPE, name)
  if (err !== null) {
    presetErrorKey.value = 'settings.layout.error_save_failed'
    return false
  }
  userPresets.value = userPresets.value.filter((p) => p.name !== name)
  return true
}

/** Applies a user preset to the live dock; `false` + a message key when no dock is live. */
export function applyUserLayoutPreset(name: string, apply: (json: string) => boolean): boolean {
  presetErrorKey.value = null
  const preset = userPresets.value.find((p) => p.name === name)
  if (preset === undefined) return false
  try {
    JSON.parse(preset.json)
  } catch {
    presetErrorKey.value = 'settings.layout.error_preset_broken'
    return false
  }
  if (!apply(preset.json)) {
    presetErrorKey.value = 'settings.layout.error_preset_broken'
    return false
  }
  return true
}

export function setLayoutPresetError(key: string | null): void {
  presetErrorKey.value = key
}
