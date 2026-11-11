import { computed, readonly, ref } from 'vue'

export type QuickSurface = 'glossary_queue' | 'import_preview'

export type QuickRun = { surface: QuickSurface; runId: number }

let stamp = 0
const current = ref<QuickRun | null>(null)

export const quickRunCurrent = readonly(current)

export function beginQuickRun(surface: QuickSurface): number {
  stamp += 1
  current.value = { surface, runId: stamp }
  return stamp
}

export function endQuickRun(runId: number): void {
  if (current.value?.runId === runId) current.value = null
}

export function quickRunBusyFor(surface: QuickSurface) {
  return computed<boolean>(() => current.value !== null && current.value.surface !== surface)
}

const purges = ref(0)

export const quickPurgeCount = readonly(purges)

export function markQuickStorePurged(): void {
  purges.value += 1
}
