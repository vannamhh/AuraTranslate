import { readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { cancelProofreadCall, runProofreadSegment } from './config/proofread'
import type { ProofreadFindingWire } from './config/proofread'
import type { AiTranslateUsageWire } from './config/aitranslate'
import type { IpcError } from './i18n'
import { flushEditorBeforeDiscreteWrite } from './panels/editorPanelState'

export type ProofreadState = 'idle' | 'scanning' | 'done' | 'error' | 'cancelled' | 'not_configured'

const state = ref<ProofreadState>('idle')
const runSegmentId = shallowRef<number | null>(null)
const scannedText = ref('')
const findings = shallowRef<readonly ProofreadFindingWire[]>([])
const unlocated = ref(0)
const usage = shallowRef<AiTranslateUsageWire | null>(null)
const error = shallowRef<IpcError | null>(null)

let sequence = 0
let flushingRun = 0

export const proofreadStateValue: DeepReadonly<Ref<ProofreadState>> = readonly(state)
export const proofreadRunSegmentId: DeepReadonly<Ref<number | null>> = readonly(runSegmentId)
export const proofreadScannedText: DeepReadonly<Ref<string>> = readonly(scannedText)
export const proofreadFindings: Readonly<Ref<readonly ProofreadFindingWire[]>> = findings
export const proofreadUnlocated: DeepReadonly<Ref<number>> = readonly(unlocated)
export const proofreadUsage: DeepReadonly<Ref<AiTranslateUsageWire | null>> = readonly(usage)
export const proofreadError: DeepReadonly<Ref<IpcError | null>> = readonly(error)

function clearResult(): void {
  scannedText.value = ''
  findings.value = []
  unlocated.value = 0
  usage.value = null
  error.value = null
}

/** Flushes the Editor first so the scan reads the same `target_text` the user sees. */
export async function runProofread(segmentId: number | null): Promise<void> {
  if (state.value === 'scanning') {
    console.warn('[proofread] not scanning: a scan is already running')
    return
  }
  if (segmentId === null) {
    console.warn('[proofread] not scanning: no segment has the caret (editorCaretSegmentId === null)')
    return
  }

  const mine = ++sequence
  runSegmentId.value = segmentId
  clearResult()
  state.value = 'scanning'

  flushingRun = mine
  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushingRun === mine) flushingRun = 0
  if (mine !== sequence) return
  if (flushed !== 'clean') {
    state.value = 'error'
    error.value = {
      code: 'proofread.flush_failed',
      message_key: flushed === 'failed' ? 'proofread.flush_failed' : 'proofread.flush_still_dirty',
      params: {},
      retryable: true,
    }
    return
  }

  const result = await runProofreadSegment(segmentId, () => {})
  if (mine !== sequence) return

  if (result.error !== null) {
    state.value = 'error'
    error.value = result.error
    return
  }
  if (result.value === null) {
    state.value = 'idle'
    return
  }
  if (result.value.state === 'done') {
    scannedText.value = result.value.scanned_text
    findings.value = result.value.findings
    unlocated.value = result.value.unlocated
    usage.value = result.value.usage
  }
  state.value = result.value.state
}

export function cancelProofread(): void {
  if (state.value !== 'scanning') return
  if (flushingRun === sequence) {
    // No Rust call is in flight yet; a cancel sent now would be lost to the next generation.
    sequence += 1
    state.value = 'cancelled'
    return
  }
  void cancelProofreadCall()
}

/** A typed edit in the scanned cell invalidates its underlines; other segments are untouched. */
export function clearProofreadFor(segmentId: number): void {
  if (runSegmentId.value !== segmentId) return
  if (state.value === 'scanning') {
    resetProofread()
    return
  }
  if (state.value !== 'done') return
  sequence += 1
  state.value = 'idle'
  runSegmentId.value = null
  clearResult()
}

export function resetProofread(): void {
  if (state.value === 'scanning' && flushingRun !== sequence) void cancelProofreadCall()
  sequence += 1
  flushingRun = 0
  state.value = 'idle'
  runSegmentId.value = null
  clearResult()
}
