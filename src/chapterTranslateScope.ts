import { computed } from 'vue'
import type { ComputedRef } from 'vue'
import { editorEditedShape, editorSegments } from './panels/editorPanelState'
import { textShapeOf } from './panels/editorSegments'

/** Mirrors what Rust keeps for a chapter run: not omitted and blank after trim, typing buffer included. */
export const chapterUntranslatedIds: ComputedRef<number[]> = computed(() => {
  const shapes = editorEditedShape.value
  return editorSegments.value
    .filter((s) => !s.is_omitted && (shapes.get(s.id) ?? textShapeOf(s.target_text)) !== 2)
    .map((s) => s.id)
})
