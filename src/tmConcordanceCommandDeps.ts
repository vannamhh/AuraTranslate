import { watch } from 'vue'
import type { CommandDeps } from './commands'
import {
  concordanceSourceHighlight,
  runConcordance,
  setConcordanceSourceHighlight,
  showConcordanceNoGlossaryTerm,
  showConcordanceNotSearched,
} from './panels/concordanceState'
import { sliceByCodePoint, sourceSpanForVietnameseSelection } from './panels/concordanceSourceMapping'
import { editorCaretSegmentId, editorChapterId, editorSegments } from './panels/editorPanelState'
import { glossaryMarks } from './panels/glossaryMarksState'
import { glossaryMarksBySegment } from './panels/glossaryMarksMap'
import { currentQuery } from './panels/lookupPanelState'
import { lookupHasRetreated, openLookupDrawer } from './layout/lookupDrawerState'
import { setMode } from './modes/modeState'
import { resetReadingLookup } from './modes/readingLookupState'
import { selectLookupTab } from './panels/lookupHistoryState'
import {
  currentSelectionIsVietnameseForConcordance,
  currentSelectionSegmentIdForConcordance,
  currentSelectionTextForConcordance,
} from './panels/selectionContract'

watch(editorChapterId, () => {
  setConcordanceSourceHighlight(null)
})

watch(editorSegments, (segments) => {
  const hl = concordanceSourceHighlight.value
  if (hl === null) return
  const target = segments.find((s) => s.id === hl.segmentId)
  if (target === undefined || target.retired_at !== null) setConcordanceSourceHighlight(null)
})

/** Selection in a Vietnamese panel: map it through the active segment's confirmed Glossary marks. */
function searchFromVietnameseSelection(selection: string): void {
  const segmentId = currentSelectionSegmentIdForConcordance() ?? editorCaretSegmentId.value
  const segment = segmentId === null ? undefined : editorSegments.value.find((s) => s.id === segmentId)
  const spans =
    segment === undefined
      ? []
      : (glossaryMarksBySegment(editorSegments.value, glossaryMarks.value).get(segment.id)?.spans ?? [])
  const span = sourceSpanForVietnameseSelection(selection, spans)
  if (segment === undefined || span === null) {
    setConcordanceSourceHighlight(null)
    showConcordanceNoGlossaryTerm(selection)
    return
  }
  setConcordanceSourceHighlight({ segmentId: segment.id, start: span.start, end: span.end })
  void runConcordance(sliceByCodePoint(segment.source_text, span.start, span.end))
}

export function openConcordanceFromSelection(): void {
  selectLookupTab('concordance')
  const selection = currentSelectionTextForConcordance().trim()

  if (selection !== '') {
    if (currentSelectionIsVietnameseForConcordance()) {
      searchFromVietnameseSelection(selection)
      return
    }
    setConcordanceSourceHighlight(null)
    void runConcordance(selection)
    return
  }

  openConcordanceForQuery(currentQuery.value ?? '')
}

/** Concordance tab for a query already known, without reading the selection. */
function openConcordanceForQuery(query: string): void {
  selectLookupTab('concordance')
  setConcordanceSourceHighlight(null)
  const trimmed = query.trim()
  if (trimmed === '') {
    showConcordanceNotSearched()
    return
  }
  void runConcordance(trimmed)
}

/** Leaves Reading mode for the Workspace Concordance tab of the word the reading lookup popup shows. */
export function openReadingLookupConcordance(): void {
  resetReadingLookup()
  setMode('workspace')
  if (lookupHasRetreated.value) openLookupDrawer()
  openConcordanceForQuery(currentQuery.value ?? '')
}

/** The `CommandDeps` handler of `tm.concordance`; `main.ts` spreads this. */
export function tmConcordanceCommandDeps(): Pick<CommandDeps, 'openTmConcordance'> {
  return { openTmConcordance: openConcordanceFromSelection }
}
