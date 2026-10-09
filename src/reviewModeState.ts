/**
 * Review Mode: a read-only layout state inside Workspace (AD-24), not a fourth mode. The data is
 * whatever `alignment_open` returned; nothing here writes. The layout it shows is never persisted.
 * Do NOT import this file from `src/commands/index.ts`; the handlers are injected through
 * `CommandDeps` (`reviewModeCommandDeps.ts`).
 */
import { computed, nextTick, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { enterFocus } from './commands'
import { alignmentOpen } from './config/alignment'
import type { AlignmentRow, AlignmentSegment } from './config/alignment'
import type { IpcError } from './i18n'
import { activeDockPanelId, resetDockSuspended, setDockSuspended } from './layout/dockController'
import { currentMode } from './modes/modeState'

export type ReviewModeStatus = 'idle' | 'loading' | 'open' | 'not_imported' | 'stale' | 'no_chapter' | 'error'

const FALLBACK_OWNER = 'panel.grid'

const status = ref<ReviewModeStatus>('idle')
const segments = ref<AlignmentSegment[]>([])
const rows = ref<AlignmentRow[]>([])
const loadError = ref<IpcError | null>(null)
let previousOwner: string | null = null
let sequence = 0

export const reviewModeStatus: DeepReadonly<Ref<ReviewModeStatus>> = readonly(status)
export const reviewModeIsOpen = computed(() => status.value === 'open')
export const reviewModeLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)

export const reviewModeSegments = computed<AlignmentSegment[]>(() => [...segments.value].sort((a, b) => a.ord - b.ord))
export const reviewModeRows = computed<AlignmentRow[]>(() => rows.value)

function clearData(): void {
  segments.value = []
  rows.value = []
  loadError.value = null
}

export async function openReviewMode(chapterId: number | null): Promise<void> {
  if (status.value === 'open' || status.value === 'loading') return

  sequence += 1
  const mySequence = sequence
  clearData()

  if (chapterId === null) {
    status.value = 'no_chapter'
    return
  }

  status.value = 'loading'

  const result = await alignmentOpen(chapterId)
  if (mySequence !== sequence) return

  if (result.alignment !== null) {
    segments.value = result.alignment.segments
    rows.value = result.alignment.rows
    previousOwner = activeDockPanelId()
    status.value = 'open'
    setDockSuspended(true)
    return
  }
  if (result.error?.code === 'export.alignment_not_imported') {
    status.value = 'not_imported'
    return
  }
  if (result.error?.code === 'export.alignment_stale') {
    status.value = 'stale'
    return
  }
  loadError.value = result.error
  status.value = 'error'
}

export async function closeReviewMode(): Promise<void> {
  const wasOpen = status.value === 'open'
  sequence += 1
  status.value = 'idle'
  clearData()
  setDockSuspended(false)
  const target = previousOwner ?? FALLBACK_OWNER
  previousOwner = null
  if (currentMode.value !== 'workspace') return

  await nextTick()
  if (!wasOpen) {
    const active = document.activeElement
    if (active === null || active === document.body) enterFocus('mode.workspace')
    return
  }
  if (!enterFocus(target)) enterFocus('mode.workspace')
}

export function resetReviewMode(): void {
  sequence += 1
  status.value = 'idle'
  clearData()
  previousOwner = null
  resetDockSuspended()
}
