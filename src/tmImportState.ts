/**
 * State of the TMX import overlay (FR64). The plan stays in Rust; the webview only holds the
 * counts of the preview. Do NOT import this file from `src/commands/index.ts`; the handlers are
 * injected through `CommandDeps` (`tmManageCommandDeps.ts`).
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { tmCancelImport, tmConfirmImport, tmOpenImportPreview } from './config/tm'
import type { TmManageTier, TmxImportPreview } from './config/tm'
import { glossaryExchangeBusy, resetGlossaryExchangeGate, setGlossaryExchangeBusy } from './glossaryExchangeGate'
import type { IpcError } from './i18n'
import { noteTmImportDone, refreshTmManage } from './tmManageState'

export type TmImportStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

const overlayOpen = ref(false)
const status = ref<TmImportStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const preview = ref<TmxImportPreview | null>(null)
const confirming = ref(false)
const confirmError = ref<IpcError | null>(null)
const confirmUnavailable = ref(false)
const opening = ref(false)
let sequence = 0

export const tmImportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const tmImportOpening: DeepReadonly<Ref<boolean>> = readonly(opening)
export const tmImportStatus: DeepReadonly<Ref<TmImportStatus>> = readonly(status)
export const tmImportLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const tmImportPreview: DeepReadonly<Ref<TmxImportPreview | null>> = readonly(preview)
export const tmImportConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const tmImportConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)
export const tmImportConfirmUnavailable: DeepReadonly<Ref<boolean>> = readonly(confirmUnavailable)

export async function openTmImportPreviewOverlay(tier: TmManageTier): Promise<void> {
  if (opening.value || overlayOpen.value || glossaryExchangeBusy.value) return

  opening.value = true
  setGlossaryExchangeBusy(true)
  sequence += 1
  const mySequence = sequence
  confirming.value = false
  confirmError.value = null
  confirmUnavailable.value = false
  noteTmImportDone(null)

  const result = await tmOpenImportPreview(tier)
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

export async function confirmTmImportPreview(): Promise<void> {
  if (confirming.value || preview.value === null) return

  confirming.value = true
  confirmError.value = null
  confirmUnavailable.value = false
  const mySequence = sequence

  const result = await tmConfirmImport()
  if (mySequence !== sequence) return

  confirming.value = false
  if (result.summary === null) {
    if (result.error === null) confirmUnavailable.value = true
    else confirmError.value = result.error
    return
  }

  noteTmImportDone(result.summary)
  overlayOpen.value = false
  await refreshTmManage()
}

export async function cancelTmImportPreview(): Promise<void> {
  if (confirming.value || !overlayOpen.value) return

  const mySequence = sequence
  overlayOpen.value = false

  const result = await tmCancelImport()
  if (mySequence !== sequence) return
  if (result.error !== null) {
    console.error(`[tm-import] \`tm_cancel_import\` failed: ${JSON.stringify(result.error)}`)
  }
}

export function resetTmImport(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  confirming.value = false
  confirmError.value = null
  confirmUnavailable.value = false
  opening.value = false
  resetGlossaryExchangeGate()
}
