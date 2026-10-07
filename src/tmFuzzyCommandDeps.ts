import type { CommandDeps } from './commands'
import { eligibleInlineStrips } from './inlineStripEligibility'
import { acceptTmFuzzyToEditor } from './panels/editorPanelState'
import { topmostStrip } from './panels/inlineStripPriority'
import {
  focusTmFuzzyStrip,
  hideTmFuzzyStrip,
  moveTmFuzzyAim,
  tmFuzzyAcceptingNow,
  tmFuzzyAimedExact,
  tmFuzzyAimedMatch,
  tmFuzzyPendingAccept,
  tmFuzzySegmentId,
} from './tmFuzzyStripState'
import { saveTmSettings } from './tmSettingsState'

/** The `CommandDeps` handlers of the fuzzy TM strip and its settings; `main.ts` spreads these. */
export function tmFuzzyCommandDeps(): Pick<
  CommandDeps,
  | 'focusTmFuzzyStrip'
  | 'nextTmFuzzyRow'
  | 'prevTmFuzzyRow'
  | 'acceptTmFuzzyRow'
  | 'confirmTmFuzzyOverwrite'
  | 'hideTmFuzzyStrip'
  | 'saveTmSettings'
> {
  return {
    focusTmFuzzyStrip: () => {
      focusTmFuzzyStrip(topmostStrip(eligibleInlineStrips.value) === 'tm_fuzzy')
    },
    nextTmFuzzyRow: () => {
      moveTmFuzzyAim(1)
    },
    prevTmFuzzyRow: () => {
      moveTmFuzzyAim(-1)
    },
    acceptTmFuzzyRow: () => {
      const segmentId = tmFuzzySegmentId.value
      const exact = tmFuzzyAimedExact()
      if (segmentId !== null && exact !== null && !tmFuzzyAcceptingNow()) {
        void acceptTmFuzzyToEditor(segmentId, exact.tier, exact.unit_id, exact.target_text, false, 'exact')
        return
      }
      const match = tmFuzzyAimedMatch()
      if (segmentId === null || match === null || tmFuzzyAcceptingNow()) return
      void acceptTmFuzzyToEditor(segmentId, match.tier, match.unit_id, match.target_text)
    },
    confirmTmFuzzyOverwrite: () => {
      const waiting = tmFuzzyPendingAccept.value
      if (waiting === null || tmFuzzyAcceptingNow()) return
      void acceptTmFuzzyToEditor(waiting.segmentId, waiting.tier, waiting.unitId, waiting.expectedTarget, true, waiting.kind)
    },
    hideTmFuzzyStrip,
    saveTmSettings: () => {
      void saveTmSettings()
    },
  }
}
