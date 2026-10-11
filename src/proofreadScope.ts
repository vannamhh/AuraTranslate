import { computed, ref, watch } from 'vue'
import type { ComputedRef } from 'vue'
import { editorCaretSegmentId, editorSegments } from './panels/editorPanelState'
import { segmentSelectionIds } from './panels/segmentSelectionState'

const lastCaret = ref<number | null>(null)

// A pointer-opened chooser blurs the grid first, which nulls the caret.
watch(
  editorCaretSegmentId,
  (id) => {
    if (id !== null) lastCaret.value = id
  },
  { immediate: true },
)

export function resetProofreadScopeCaret(): void {
  lastCaret.value = null
}

export type ProofreadScopeKind = 'segment' | 'selection' | 'chapter'

/** Mirrors what Rust drops before a scan: omitted or blank-translation segments. */
function scannableIds(ids: readonly number[]): number[] {
  const byId = new Map(editorSegments.value.map((s) => [s.id, s]))
  return ids.filter((id) => {
    const segment = byId.get(id)
    return segment !== undefined && !segment.is_omitted && segment.target_text.trim() !== ''
  })
}

export const proofreadScopeIds: ComputedRef<Record<ProofreadScopeKind, number[]>> = computed(() => ({
  segment: (() => {
    const id = editorCaretSegmentId.value ?? lastCaret.value
    return id !== null && editorSegments.value.some((s) => s.id === id) ? [id] : []
  })(),
  selection: [...segmentSelectionIds.value],
  chapter: editorSegments.value.map((s) => s.id),
}))

export const proofreadScopeCounts: ComputedRef<Record<ProofreadScopeKind, number>> = computed(() => ({
  segment: scannableIds(proofreadScopeIds.value.segment).length,
  selection: scannableIds(proofreadScopeIds.value.selection).length,
  chapter: scannableIds(proofreadScopeIds.value.chapter).length,
}))
