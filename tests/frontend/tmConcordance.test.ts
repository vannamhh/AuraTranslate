/**
 * Concordance tab states, tab order, the dictionary not-found pointer and the
 * command registration. Only the IPC boundary is faked; state, commands and the real
 * LookupPanel run.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import type { CommandDeps } from '../../src/commands'
import type { TmConcordance, TmConcordanceHit } from '../../src/config/segment'

const tmConcordanceMock = vi.fn()
const lookupDictionaryMock = vi.fn()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, tmConcordance: (...args: unknown[]) => tmConcordanceMock(...args) }
})

vi.mock('../../src/config/dict', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/dict')>()
  return { ...actual, lookupDictionary: (...args: unknown[]) => lookupDictionaryMock(...args) }
})

function hitOf(patch: Partial<TmConcordanceHit> = {}): TmConcordanceHit {
  return { tier: 'work', unit_id: 1, source_text: '他叫师父。', target_text: 'Hắn gọi sư phụ.', side: 'mine', created_at: '2026-08-03T00:00:00.000Z', ...patch }
}

function answerOf(query: string, patch: Partial<TmConcordance> = {}) {
  return { outcome: { query, tm_empty: false, total: 0, hits: [], ...patch }, error: null }
}

const NOT_FOUND = {
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
}

async function fresh(extra: Partial<CommandDeps> = {}) {
  vi.resetModules()
  tmConcordanceMock.mockReset()
  lookupDictionaryMock.mockReset()
  const commands = await import('../../src/commands')
  const concordance = await import('../../src/panels/concordanceState')
  const history = await import('../../src/panels/lookupHistoryState')
  const lookup = await import('../../src/panels/lookupPanelState')
  const LookupPanel = (await import('../../src/panels/LookupPanel.vue')).default
  const openTmConcordance = vi.fn()
  const keymap = commands.installCommands({ isMac: true, setMode: () => {}, openTmConcordance, ...extra } as CommandDeps)
  return { commands, keymap, concordance, history, lookup, LookupPanel, openTmConcordance }
}

function panelOf(LookupPanel: Awaited<ReturnType<typeof fresh>>['LookupPanel']) {
  return mount(LookupPanel, { props: { params: { params: {} } } })
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('tab strip', () => {
  it('lists Từ điển, Concordance, Lịch sử in that order', async () => {
    const { LookupPanel, history } = await fresh()
    expect([...history.LOOKUP_TABS]).toEqual(['record', 'concordance', 'history'])
    const w = panelOf(LookupPanel)
    expect(w.findAll('[role="tab"]').map((tab) => tab.text())).toEqual(['Từ điển', 'Concordance', 'Lịch sử'])
    w.unmount()
  })
})

describe('tab arrow keys', () => {
  const CASES: [string, string, string, string][] = [
    ['record', 'ArrowRight', 'concordance', 'lookup-tab-concordance'],
    ['record', 'ArrowLeft', 'history', 'lookup-tab-history'],
    ['concordance', 'ArrowRight', 'history', 'lookup-tab-history'],
    ['concordance', 'ArrowLeft', 'record', 'lookup-tab-record'],
    ['history', 'ArrowRight', 'record', 'lookup-tab-record'],
    ['history', 'ArrowLeft', 'concordance', 'lookup-tab-concordance'],
  ]
  for (const [from, key, to, focusId] of CASES) {
    it(`${key} on ${from} selects ${to} and moves focus`, async () => {
      const selectLookupTab = vi.fn()
      const { LookupPanel } = await fresh({ selectLookupTab })
      const w = mount(LookupPanel, { props: { params: { params: {} } }, attachTo: document.body })
      await w.get(`#lookup-tab-${from}`).trigger('keydown', { key })
      expect(selectLookupTab).toHaveBeenCalledWith(to)
      expect(document.activeElement?.id).toBe(focusId)
      w.unmount()
    })
  }
})

describe('concordance states', () => {
  async function tab(answer: unknown | null, query = '师父') {
    const ctx = await fresh()
    ctx.history.selectLookupTab('concordance')
    const w = panelOf(ctx.LookupPanel)
    if (answer !== null) {
      tmConcordanceMock.mockResolvedValue(answer)
      await ctx.concordance.runConcordance(query)
      await flushPromises()
    }
    return { ...ctx, w, text: () => w.get('#lookup-tabpanel').text() }
  }

  it('not searched yet: teaching line, no IPC', async () => {
    const { w, text } = await tab(null)
    expect(text()).toContain('phím tắt Concordance')
    expect(tmConcordanceMock).not.toHaveBeenCalled()
    w.unmount()
  })

  it('TM empty is its own state, not the no-hit state', async () => {
    const { w, text } = await tab(answerOf('师父', { tm_empty: true }))
    expect(text()).toContain('Bộ nhớ dịch còn trống')
    expect(text()).not.toContain('Không có cặp nào')
    w.unmount()
  })

  it('no hit names the phrase', async () => {
    const { w, text } = await tab(answerOf('师父'))
    expect(text()).toContain('Không có cặp nào trong bộ nhớ dịch chứa "师父"')
    expect(text()).not.toContain('Bộ nhớ dịch còn trống')
    w.unmount()
  })

  it('hits show source, target, side and tier in the given order', async () => {
    const { w } = await tab(
      answerOf('师父', {
        total: 2,
        hits: [
          hitOf({ tier: 'global', unit_id: 5, side: 'mine', source_text: 'A师父', target_text: 'A-vi' }),
          hitOf({ tier: 'work', unit_id: 9, side: 'others', source_text: 'B师父', target_text: 'B-vi' }),
        ],
      }),
    )
    const rows = w.findAll('.concordance-hit')
    expect(rows).toHaveLength(2)
    expect(rows[0]?.text()).toContain('A师父')
    expect(rows[0]?.text()).toContain('A-vi')
    expect(rows[0]?.text()).toContain('Của tôi · Toàn cục')
    expect(rows[1]?.text()).toContain('Của người khác · Tác phẩm này')
    expect(w.find('.lookup-banner').exists()).toBe(false)
    w.unmount()
  })

  it('each hit carries its date, rendered through historyTimeLabel', async () => {
    const { w } = await tab(
      answerOf('师父', { total: 1, hits: [hitOf({ created_at: '2020-01-02T03:04:05.000Z' })] }),
    )
    const { historyTimeLabel } = await import('../../src/panels/segmentHistoryTime')
    const { t } = await import('../../src/i18n')
    const { key, params } = historyTimeLabel('2020-01-02T03:04:05.000Z', Date.now())
    expect(w.get('.concordance-meta').text()).toContain(t(key, params))
    w.unmount()
  })

  it('capped result says it shows fewer than the total', async () => {
    const hits = Array.from({ length: 50 }, (_, i) => hitOf({ unit_id: i + 1 }))
    const { w } = await tab(answerOf('师父', { total: 120, hits }))
    expect(w.findAll('.concordance-hit')).toHaveLength(50)
    expect(w.get('.lookup-banner').text()).toContain('50 / 120')
    w.unmount()
  })

  it('a rejected search is the error state, not the no-hit state', async () => {
    const { w, text } = await tab({ outcome: null, error: { code: 'x', message_key: 'tm.lookup_failed', params: {}, retryable: false } })
    expect(text()).toContain('Không tìm được trong bộ nhớ dịch')
    expect(text()).not.toContain('Không có cặp nào')
    w.unmount()
  })

  it('a response for another query than the one sent is the error state', async () => {
    const { w, concordance } = await tab(answerOf('khác', { total: 1, hits: [hitOf()] }))
    expect(concordance.concordanceView.value).toBe('error')
    w.unmount()
  })

  it('a search in flight shows pending, not the previous answer', async () => {
    const ctx = await fresh()
    tmConcordanceMock.mockResolvedValueOnce(answerOf('旧', { total: 1, hits: [hitOf()] }))
    await ctx.concordance.runConcordance('旧')
    tmConcordanceMock.mockReturnValueOnce(new Promise(() => {}))
    void ctx.concordance.runConcordance('新')
    expect(ctx.concordance.concordanceView.value).toBe('pending')
  })

  it('no IPC bridge (outcome null, no error) is the error state', async () => {
    const ctx = await fresh()
    tmConcordanceMock.mockResolvedValue({ outcome: null, error: null })
    await ctx.concordance.runConcordance('师父')
    expect(ctx.concordance.concordanceView.value).toBe('error')
  })

  it('an older answer never overwrites a newer one', async () => {
    const ctx = await fresh()
    let releaseFirst: (v: unknown) => void = () => {}
    tmConcordanceMock
      .mockReturnValueOnce(new Promise((r) => (releaseFirst = r)))
      .mockResolvedValueOnce(answerOf('新', { total: 1, hits: [hitOf()] }))
    const first = ctx.concordance.runConcordance('旧')
    await ctx.concordance.runConcordance('新')
    releaseFirst(answerOf('旧', { tm_empty: true }))
    await first
    expect(ctx.concordance.concordanceResponse.value?.query).toBe('新')
    expect(ctx.concordance.concordanceView.value).toBe('hits')
  })

  it('no Glossary term for a Vietnamese selection is its own state', async () => {
    const ctx = await fresh()
    ctx.history.selectLookupTab('concordance')
    const w = panelOf(ctx.LookupPanel)
    ctx.concordance.showConcordanceNoGlossaryTerm('thầy')
    await flushPromises()
    expect(w.get('#lookup-tabpanel').text()).toContain('"thầy"')
    expect(ctx.concordance.concordanceView.value).toBe('no_glossary_term')
    w.unmount()
  })
})

describe('dictionary not-found pointer', () => {
  async function notFoundFor(query: string, total: number) {
    const ctx = await fresh()
    lookupDictionaryMock.mockResolvedValue(NOT_FOUND)
    tmConcordanceMock.mockResolvedValue(
      answerOf(query, { total, hits: total > 0 ? [hitOf()] : [] }),
    )
    const w = panelOf(ctx.LookupPanel)
    await ctx.lookup.runLookup(query)
    await flushPromises()
    return { ...ctx, w }
  }

  it('shows the background count and a click dispatches tm.concordance', async () => {
    const { w, openTmConcordance } = await notFoundFor('师父', 3)
    const pointer = w.get('.lookup-concordance-pointer')
    expect(pointer.text()).toBe('Concordance có 3 kết quả')
    await pointer.trigger('click')
    expect(openTmConcordance).toHaveBeenCalledTimes(1)
    w.unmount()
  })

  it('shows nothing extra when the count is zero', async () => {
    const { w } = await notFoundFor('师父', 0)
    expect(w.find('.lookup-concordance-pointer').exists()).toBe(false)
    expect(w.text()).toContain('Không tìm thấy trong từ điển')
    w.unmount()
  })

  it('never shows a count computed for an older query', async () => {
    const ctx = await fresh()
    lookupDictionaryMock.mockResolvedValue(NOT_FOUND)
    let releaseOld: (v: unknown) => void = () => {}
    tmConcordanceMock
      .mockReturnValueOnce(new Promise((r) => (releaseOld = r)))
      .mockResolvedValueOnce(answerOf('新', { total: 0 }))
    const w = panelOf(ctx.LookupPanel)
    await ctx.lookup.runLookup('旧')
    await flushPromises()
    await ctx.lookup.runLookup('新')
    await flushPromises()
    releaseOld(answerOf('旧', { total: 7, hits: [hitOf()] }))
    await flushPromises()
    expect(w.find('.lookup-concordance-pointer').exists()).toBe(false)
    w.unmount()
  })
})

describe('commands', () => {
  it('tm.concordance is registered with F3 and is rebindable', async () => {
    const { commands } = await fresh()
    expect(commands.defaultChordsFor('tm.concordance')).toEqual(['F3', 'Mod+Alt+N'])
    expect(commands.effectiveBindings().filter((b) => b.id === 'tm.concordance').map((b) => b.chord)).toEqual([
      'F3',
      'Mod+Alt+N',
    ])
  })

  it('both default chords reach the Settings › Shortcuts row and register as conflicts', async () => {
    const { commands } = await fresh()
    const shortcuts = await import('../../src/config/shortcutsState')
    const row = shortcuts.shortcutRows.value.find((r) => r.id === 'tm.concordance')
    expect(row?.chords).toEqual(['F3', 'Mod+Alt+N'])
    expect(row?.hasDefault).toBe(true)
    expect(row?.overridden).toBe(false)
    expect(commands.conflictFor('Mod+Alt+N', 'tm.fuzzy.hide')?.heldBy).toBe('tm.concordance')
    expect(commands.conflictFor('F3', 'tm.fuzzy.hide')?.heldBy).toBe('tm.concordance')

  })

  it('Mod+Alt+N dispatches from the Editor contenteditable cell; bare F3 does not', async () => {
    const { keymap, openTmConcordance } = await fresh()
    const cell = { tagName: 'SPAN', isContentEditable: true }
    const press = (code: string, mods: Record<string, boolean>) =>
      keymap.handle({ code, ...mods, target: cell, preventDefault: () => {} })

    expect(press('F3', {})).toBe(false)
    expect(openTmConcordance).not.toHaveBeenCalled()
    expect(press('KeyN', { metaKey: true, altKey: true })).toBe(true)
    expect(openTmConcordance).toHaveBeenCalledTimes(1)
  })

  it('a stored binding replaces the default chord', async () => {
    const { commands } = await fresh({ bindings: { 'tm.concordance': ['Mod+Alt+Y'] } })
    const mine = commands.effectiveBindings().filter((b) => b.id === 'tm.concordance')
    expect(mine.map((b) => b.chord)).toEqual(['Mod+Alt+Y'])
  })

  it('lookup.select_tab_concordance selects the tab', async () => {
    const selectLookupTab = vi.fn()
    const { commands } = await fresh({ selectLookupTab })
    commands.dispatch('lookup.select_tab_concordance')
    expect(selectLookupTab).toHaveBeenCalledWith('concordance')
  })
})
