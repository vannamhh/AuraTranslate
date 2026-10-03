/**
 * The `tm.concordance` handler. The selection reader, the active segment and the
 * Glossary marks are real module state faked at their boundary; the IPC wrapper is mocked.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref, shallowRef } from 'vue'
import type { GlossaryMark } from '../../src/config/glossary'

const tmConcordanceMock = vi.fn()
const dictionarySelectionSpy = vi.fn(() => '')

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, tmConcordance: (...args: unknown[]) => tmConcordanceMock(...args) }
})

vi.mock('../../src/panels/selectionContract', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/selectionContract')>()
  return { ...actual, currentSelectionText: () => dictionarySelectionSpy() }
})

const caret = ref<number | null>(7)
const chapterId = ref<number | null>(1)
const segments = shallowRef<{ id: number; ord: number; source_text: string; retired_at: string | null }[]>([
  { id: 7, ord: 1, source_text: '他叫师父，师父来了。', retired_at: null },
])
const marks = shallowRef<GlossaryMark[]>([])

vi.mock('../../src/panels/editorPanelState', () => ({
  editorCaretSegmentId: caret,
  editorChapterId: chapterId,
  editorSegments: segments,
}))
vi.mock('../../src/panels/glossaryMarksState', () => ({ glossaryMarks: marks }))

function markOf(patch: Partial<GlossaryMark>): GlossaryMark {
  return {
    start: 0,
    end: 1,
    tier: 'work',
    is_confirmed: true,
    translation: 'Hắn',
    id: 1,
    source_term: '他',
    han_viet_suggestion: null,
    han_viet_status: 'not_requested',
    occurrence_count: null,
    ...patch,
  }
}

function select(role: 'source' | 'display', text: string, registerSurface: typeof import('../../src/panels/selectionContract').registerSelectionSurface, vietnamese = false, segmentId: number | null = null): void {
  const box = document.createElement('div')
  if (segmentId !== null) box.setAttribute('data-segment-id', String(segmentId))
  const inner = document.createElement('span')
  inner.textContent = text
  box.append(inner)
  document.body.append(box)
  registerSurface(box, role, undefined, vietnamese)
  const node = inner.firstChild as Text
  const range = document.createRange()
  range.setStart(node, 0)
  range.setEnd(node, text.length)
  window.getSelection()?.removeAllRanges()
  window.getSelection()?.addRange(range)
}

async function fresh() {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
  tmConcordanceMock.mockReset()
  dictionarySelectionSpy.mockClear()
  tmConcordanceMock.mockImplementation(async (query: string) => ({
    outcome: { query, tm_empty: false, total: 1, hits: [] },
    error: null,
  }))
  vi.resetModules()
  const contract = await import('../../src/panels/selectionContract')
  const concordance = await import('../../src/panels/concordanceState')
  const history = await import('../../src/panels/lookupHistoryState')
  const lookup = await import('../../src/panels/lookupPanelState')
  const { openConcordanceFromSelection } = await import('../../src/tmConcordanceCommandDeps')
  const flush = async () => {
    await Promise.resolve()
    await Promise.resolve()
  }
  return {
    contract,
    concordance,
    history,
    lookup,
    run: async () => {
      openConcordanceFromSelection()
      await flush()
    },
  }
}

beforeEach(() => {
  caret.value = 7
  chapterId.value = 1
  segments.value = [{ id: 7, ord: 1, source_text: '他叫师父，师父来了。', retired_at: null }]
  marks.value = [
    markOf({ id: 1, start: 0, end: 1, translation: 'Hắn' }),
    markOf({ id: 2, start: 2, end: 4, translation: 'Sư phụ', source_term: '师父' }),
  ]
})

describe('tm.concordance handler', () => {
  it('opens the tab and searches a source-surface selection without touching the dictionary reader', async () => {
    const { contract, concordance, history, run } = await fresh()
    select('source', '师父', contract.registerSelectionSurface)
    await run()
    expect(history.lookupTab.value).toBe('concordance')
    expect(tmConcordanceMock).toHaveBeenCalledWith('师父')
    expect(dictionarySelectionSpy).not.toHaveBeenCalled()
    expect(concordance.concordanceSourceHighlight.value).toBeNull()
  })

  it('searches a selection inside a display surface (AI Translation) through the reverse mapping', async () => {
    const { contract, concordance, run } = await fresh()
    select('display', 'sư phụ', contract.registerSelectionSurface, true)
    await run()
    expect(tmConcordanceMock).toHaveBeenCalledWith('师父')
    expect(concordance.concordanceSourceHighlight.value).toEqual({ segmentId: 7, start: 2, end: 4 })
    expect(dictionarySelectionSpy).not.toHaveBeenCalled()
  })

  it('a Chinese selection on a display surface that is not Vietnamese (Lookup panel) is searched directly', async () => {
    const { contract, concordance, run } = await fresh()
    select('display', '师父', contract.registerSelectionSurface)
    await run()
    expect(tmConcordanceMock).toHaveBeenCalledWith('师父')
    expect(concordance.concordanceView.value).not.toBe('no_glossary_term')
    expect(concordance.concordanceSourceHighlight.value).toBeNull()
  })

  it('maps a Vietnamese selection against the row it sits in, not the caret row', async () => {
    const { contract, concordance, run } = await fresh()
    segments.value = [
      { id: 7, ord: 1, source_text: '他叫师父，师父来了。', retired_at: null },
      { id: 8, ord: 2, source_text: '他叫师兄', retired_at: null },
    ]
    marks.value = [
      markOf({ id: 1, start: 0, end: 1, translation: 'Hắn' }),
      markOf({ id: 5, start: 13, end: 15, translation: 'Sư huynh', source_term: '师兄' }),
    ]
    caret.value = 7
    select('display', 'sư huynh', contract.registerSelectionSurface, true, 8)
    await run()
    expect(tmConcordanceMock).toHaveBeenCalledWith('师兄')
    expect(concordance.concordanceSourceHighlight.value).toEqual({ segmentId: 8, start: 2, end: 4 })
  })

  it('a chapter switch clears the source highlight', async () => {
    const { contract, concordance, run } = await fresh()
    select('display', 'sư phụ', contract.registerSelectionSurface, true)
    await run()
    expect(concordance.concordanceSourceHighlight.value).not.toBeNull()
    chapterId.value = 2
    await Promise.resolve()
    expect(concordance.concordanceSourceHighlight.value).toBeNull()
  })

  it('replacing the segments (regroup) clears the highlight only when its segment is gone', async () => {
    const { contract, concordance, run } = await fresh()
    select('display', 'sư phụ', contract.registerSelectionSurface, true)
    await run()
    segments.value = [...segments.value, { id: 8, ord: 2, source_text: '他', retired_at: null }]
    await Promise.resolve()
    expect(concordance.concordanceSourceHighlight.value).not.toBeNull()
    segments.value = [{ id: 9, ord: 1, source_text: '他叫师父，师父来了。', retired_at: null }]
    await Promise.resolve()
    expect(concordance.concordanceSourceHighlight.value).toBeNull()
  })

  it('a Vietnamese selection with no matching mark shows the no-Glossary state and makes no IPC call', async () => {
    const { contract, concordance, run } = await fresh()
    marks.value = [markOf({ id: 3, start: 2, end: 4, translation: null, is_confirmed: false })]
    select('display', 'sư phụ', contract.registerSelectionSurface, true)
    await run()
    expect(tmConcordanceMock).not.toHaveBeenCalled()
    expect(concordance.concordanceView.value).toBe('no_glossary_term')
    expect(concordance.concordanceNoGlossarySelection.value).toBe('sư phụ')
    expect(concordance.concordanceSourceHighlight.value).toBeNull()
  })

  it('an empty selection falls back to the current Lookup query', async () => {
    const { lookup, run } = await fresh()
    const dict = await import('../../src/config/dict')
    vi.spyOn(dict, 'lookupDictionary').mockResolvedValue({
      response: {
        grouped: {
          route: 'zh',
          branch: 'exact_btree',
          groups: [],
          skipped: [],
          truncated_layers: [],
          hidden_sources: [],
          layers_loaded: true,
        },
        senses_by_layer: {},
        query_truncated: false,
        senses_failed: [],
      },
      error: null,
    })
    await lookup.runLookup('师兄')
    tmConcordanceMock.mockClear()
    await run()
    expect(tmConcordanceMock).toHaveBeenCalledWith('师兄')
  })

  it('empty selection and empty Lookup query is the not-searched state with no IPC call', async () => {
    const { concordance, run } = await fresh()
    await run()
    expect(tmConcordanceMock).not.toHaveBeenCalled()
    expect(concordance.concordanceView.value).toBe('not_searched')
  })
})
