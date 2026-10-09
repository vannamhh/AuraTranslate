/**
 * State of the reviewer-copy import overlay. The plan stays in Rust; the webview only holds
 * the preview. Do NOT import this file from `src/commands/index.ts`; the handlers are injected
 * through `CommandDeps` (`reviewerImportCommandDeps.ts`).
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { reviewerImportCancel, reviewerImportConfirm, reviewerImportOpenPreview } from './config/reviewerImport'
import type { ReviewerImportPreview, ReviewerImportSummary } from './config/reviewerImport'
import { glossaryExchangeBusy, resetGlossaryExchangeGate, setGlossaryExchangeBusy } from './glossaryExchangeGate'
import type { IpcError } from './i18n'

export type ReviewerImportStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded' | 'done'

export interface ReviewerImportHooks {
  afterImported?: () => void
  afterClosedWithHarvest?: () => void
}

let hooks: ReviewerImportHooks = {}

export function installReviewerImportHooks(next: ReviewerImportHooks): void {
  hooks = next
}

const NO_PENDING_CODE = 'export.reviewer_import_no_pending'

const overlayOpen = ref(false)
const status = ref<ReviewerImportStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const preview = ref<ReviewerImportPreview | null>(null)
const summary = ref<ReviewerImportSummary | null>(null)
const confirming = ref(false)
const confirmError = ref<IpcError | null>(null)
const confirmUnavailable = ref(false)
const opening = ref(false)
let sequence = 0

export const reviewerImportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const reviewerImportOpening: DeepReadonly<Ref<boolean>> = readonly(opening)
export const reviewerImportStatus: DeepReadonly<Ref<ReviewerImportStatus>> = readonly(status)
export const reviewerImportLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const reviewerImportPreview: DeepReadonly<Ref<ReviewerImportPreview | null>> = readonly(preview)
export const reviewerImportSummary: DeepReadonly<Ref<ReviewerImportSummary | null>> = readonly(summary)
export const reviewerImportConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const reviewerImportConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)
export const reviewerImportConfirmUnavailable: DeepReadonly<Ref<boolean>> = readonly(confirmUnavailable)

export async function openReviewerImportPreviewOverlay(): Promise<void> {
  if (opening.value || overlayOpen.value || glossaryExchangeBusy.value) return

  opening.value = true
  setGlossaryExchangeBusy(true)
  sequence += 1
  const mySequence = sequence
  confirming.value = false
  confirmError.value = null
  confirmUnavailable.value = false
  summary.value = null

  const result = await reviewerImportOpenPreview()
  if (mySequence !== sequence) return
  opening.value = false
  setGlossaryExchangeBusy(false)

  if (result.outcome === 'cancelled') return

  overlayOpen.value = true
  if (result.outcome === 'ipc_unavailable') {
    status.value = 'ipc_unavailable'
    loadError.value = null
    preview.value = null
    return
  }
  if (result.outcome === 'error') {
    status.value = 'error'
    loadError.value = result.error
    preview.value = null
    return
  }
  preview.value = result.preview
  status.value = 'loaded'
  loadError.value = null
}

export async function confirmReviewerImportPreview(): Promise<void> {
  if (confirming.value || preview.value === null || status.value !== 'loaded') return

  confirming.value = true
  confirmError.value = null
  confirmUnavailable.value = false
  const mySequence = sequence

  const result = await reviewerImportConfirm()
  if (mySequence !== sequence) return

  confirming.value = false
  if (result.summary === null) {
    if (result.error === null) {
      confirmUnavailable.value = true
      return
    }
    confirmError.value = result.error
    if (result.error.code === NO_PENDING_CODE) {
      status.value = 'error'
      loadError.value = result.error
      preview.value = null
    }
    return
  }

  summary.value = result.summary
  status.value = 'done'
  hooks.afterImported?.()
}

export async function cancelReviewerImportPreview(): Promise<void> {
  if (confirming.value || !overlayOpen.value) return

  const mySequence = sequence
  const wasDone = status.value === 'done'
  overlayOpen.value = false
  if (wasDone) {
    const harvested = summary.value
    const count = harvested?.harvest_candidate_count ?? null
    if (harvested !== null && harvested.harvest_error === null && count !== null && count > 0) {
      hooks.afterClosedWithHarvest?.()
    }
    return
  }

  const result = await reviewerImportCancel()
  if (mySequence !== sequence) return
  if (result.error !== null) {
    console.error(`[reviewer-import] \`reviewer_import_cancel\` failed: ${JSON.stringify(result.error)}`)
  }
}

export function resetReviewerImport(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  summary.value = null
  confirming.value = false
  confirmError.value = null
  confirmUnavailable.value = false
  opening.value = false
  resetGlossaryExchangeGate()
}
