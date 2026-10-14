/**
 * History chips `Phiên này · Chương này`: every lookup records the Chapter that was open, and
 * the chip filters on it. Pins are never filtered.
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import LookupPanel from '../../src/panels/LookupPanel.vue'
import { installCommands } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import { loadDictSources, resetDictSources } from '../../src/panels/dictSourcesState'
import { resetLookupPanel, runLookup } from '../../src/panels/lookupPanelState'
import {
  lookupHistory,
  lookupHistoryScope,
  resetLookupHistory,
  selectLookupHistoryScope,
  selectLookupTab,
  setLookupChapter,
  visibleLookupHistory,
} from '../../src/panels/lookupHistoryState'
import { flushPromises } from './support/flushMicrotasks'
import { DA, SOURCES } from './support/lookupFixtures'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'list_dict_sources') return { sources: SOURCES, skipped: [] }
    if (cmd === 'lookup_dictionary') return DA
    if (cmd === 'list_pinned_entries') return []
    if (cmd === 'read_han_viet') return { lookup: null, error: null }
    throw new Error(`lookupHistoryChapter.test.ts: unknown command "${cmd}"`)
  }),
}))

beforeAll(() => {
  installCommands({
    isMac: true,
    setMode: () => {},
    selectLookupHistoryScope,
    selectLookupTab,
  } as CommandDeps)
})

let panel: VueWrapper | null = null

async function lookupIn(chapter: number | null, query: string): Promise<void> {
  setLookupChapter(chapter)
  await runLookup(query)
  await flushPromises()
}

const queries = () => visibleLookupHistory.value.map((row) => row.query)

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  panel?.unmount()
  panel = null
  resetDictSources()
  resetLookupPanel()
  resetLookupHistory()
})

describe('Chương này chip', () => {
  it('A in Chapter 1, B in Chapter 2, Chapter 2 open: only B; looking A up again in Chapter 2 puts A in both', async () => {
    await lookupIn(1, 'A')
    await lookupIn(2, 'B')
    selectLookupHistoryScope('chapter')
    expect(queries()).toEqual(['B'])

    await lookupIn(2, 'A')
    expect(queries()).toEqual(['A', 'B'])
    expect(lookupHistory.value.find((row) => row.query === 'A')?.chapters).toEqual([1, 2])

    setLookupChapter(1)
    expect(queries()).toEqual(['A'])
  })

  it('session scope shows every row regardless of Chapter', async () => {
    await lookupIn(1, 'A')
    await lookupIn(2, 'B')
    expect(lookupHistoryScope.value).toBe('session')
    expect(queries()).toEqual(['B', 'A'])
  })

  it('no Chapter open: the effective scope falls back to session and the row records no Chapter', async () => {
    await lookupIn(null, 'A')
    selectLookupHistoryScope('chapter')
    expect(lookupHistoryScope.value).toBe('session')
    expect(queries()).toEqual(['A'])
    expect(lookupHistory.value[0]!.chapters).toEqual([])
  })

  it('a lookup made with no Chapter open never matches a Chapter filter', async () => {
    await lookupIn(null, 'A')
    setLookupChapter(3)
    selectLookupHistoryScope('chapter')
    expect(queries()).toEqual([])
  })
})

describe('chips in the history tab', () => {
  async function historyTab(): Promise<VueWrapper> {
    await loadDictSources('')
    selectLookupTab('history')
    panel = mount(LookupPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    return panel
  }
  const chip = (w: VueWrapper, label: string) => w.findAll('.lookup-scope').find((c) => c.text() === label)!
  const rowWords = (w: VueWrapper) => w.findAll('.lookup-row:not(.is-pin) .lookup-row-word').map((r) => r.text())

  it('Chương này is disabled with no Chapter open, and Phiên này is the pressed chip', async () => {
    const w = await historyTab()
    expect(chip(w, 'Chương này').attributes('disabled')).toBeDefined()
    expect(chip(w, 'Phiên này').attributes('aria-pressed')).toBe('true')
  })

  it('clicking Chương này filters the rows to the open Chapter; Phiên này restores them', async () => {
    await lookupIn(1, 'A')
    await lookupIn(2, 'B')
    const w = await historyTab()
    expect(rowWords(w)).toEqual(['B', 'A'])

    await chip(w, 'Chương này').trigger('click')
    expect(chip(w, 'Chương này').attributes('aria-pressed')).toBe('true')
    expect(rowWords(w)).toEqual(['B'])

    await chip(w, 'Phiên này').trigger('click')
    expect(rowWords(w)).toEqual(['B', 'A'])
  })

  it('a Chapter with no lookups says so instead of showing an empty list', async () => {
    await lookupIn(1, 'A')
    const w = await historyTab()
    setLookupChapter(9)
    await chip(w, 'Chương này').trigger('click')
    expect(rowWords(w)).toEqual([])
    expect(w.text()).toContain('Chưa tra gì trong Chương này')
  })
})
