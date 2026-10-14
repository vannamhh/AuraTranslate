/**
 * Lookup panel body: source blocks, readings, panel head, Pin, collapse, the two empty states and
 * "Tra riêng". Only the IPC boundary is faked; panel, commands and state modules are real.
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import LookupPanel from '../../src/panels/LookupPanel.vue'
import { installCommands } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import { loadDictSources, resetDictSources, toggleDictSource, toggleFocusedDictSource } from '../../src/panels/dictSourcesState'
import {
  computeSpine,
  lookupAimedChar,
  resetLookupPanel,
  runLookup,
  singleCharsOf,
  toggleAimedSourceCollapse,
} from '../../src/panels/lookupPanelState'
import { resetLookupHistory, toggleLookupPin } from '../../src/panels/lookupHistoryState'
import { shortcutRows } from '../../src/config/shortcutsState'
import { flushPromises } from './support/flushMicrotasks'
import { DA, NOT_FOUND, SOURCES, ZOULANG, group, hit, response, sense } from './support/lookupFixtures'

const h = vi.hoisted(() => ({
  lookups: [] as string[],
  pins: [] as unknown[],
  concordanceTotal: 0,
  answers: new Map<string, unknown>(),
}))

vi.mock('../../src/config/segment', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/segment')>()),
  tmConcordance: async (query: string) => ({
    outcome: { query, tm_empty: false, total: h.concordanceTotal, hits: [] },
    error: null,
  }),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args: { query?: string }) => {
    if (cmd === 'list_dict_sources') return { sources: SOURCES, skipped: [] }
    if (cmd === 'put_config') return undefined
    if (cmd === 'lookup_dictionary') {
      const query = args.query ?? ''
      h.lookups.push(query)
      return h.answers.get(query) ?? h.answers.get('*')
    }
    if (cmd === 'read_han_viet') return { lookup: null, error: null }
    if (cmd === 'list_pinned_entries') return []
    if (cmd === 'pin_entry') {
      h.pins.push(args)
      return []
    }
    throw new Error(`lookupPanelBlocks.test.ts: unknown command "${cmd}"`)
  }),
}))

beforeAll(() => {
  installCommands({
    isMac: true,
    setMode: () => {},
    toggleDictSource: toggleFocusedDictSource,
    toggleLookupPin,
    toggleLookupSourceCollapse: toggleAimedSourceCollapse,
    lookupAimedChar,
  } as CommandDeps)
})

let panel: VueWrapper | null = null

async function lookedUp(query: string, answer: unknown, keepDisabled = false): Promise<VueWrapper> {
  h.answers.set(query, answer)
  if (!keepDisabled) await loadDictSources('')
  panel = mount(LookupPanel, { props: { params: { params: {} } }, attachTo: document.body })
  await runLookup(query)
  await flushPromises()
  return panel
}

const blocks = (w: VueWrapper) => w.findAll('.lookup-source')

beforeEach(() => {
  document.body.innerHTML = ''
  h.lookups = []
  h.pins = []
  h.concordanceTotal = 0
  h.answers.clear()
})

afterEach(() => {
  panel?.unmount()
  panel = null
  resetDictSources()
  resetLookupPanel()
  resetLookupHistory()
})

describe('source blocks (Tra 打)', () => {
  it('stacks one block per source with no lead-in line and no per-source chips', async () => {
    const w = await lookedUp('打', DA)
    expect(blocks(w)).toHaveLength(2)
    expect(w.find('.lookup-disagree').exists()).toBe(false)
    expect(w.find('.lookup-sources').exists()).toBe(false)
    expect(w.find('.source-chip').exists()).toBe(false)
    expect(w.text()).not.toContain('Có nhiều hơn một nguồn')
  })

  it('every block opens with name, sense count and the collapse glyph', async () => {
    const w = await lookedUp('打', DA)
    const heads = blocks(w).map((b) => {
      const head = b.get('.lookup-source-head')
      return [head.get('.lookup-source-name').text(), head.get('.lookup-source-count').text(), head.get('.lookup-collapse').text()]
    })
    expect(heads).toEqual([
      ['CVDICT', '3 nghĩa', '⌄'],
      ['Vietphrase', '2 nghĩa', '⌄'],
    ])
  })

  it('a source with readings groups its senses by reading: dá · 1 nghĩa and dǎ · 2 nghĩa', async () => {
    const w = await lookedUp('打', DA)
    const clusters = blocks(w)[0]!.findAll('.lookup-entry')
    expect(clusters.map((c) => [c.get('.lookup-entry-reading').text(), c.get('.lookup-entry-count').text()])).toEqual([
      ['dá', '1 nghĩa'],
      ['dǎ', '2 nghĩa'],
    ])
  })

  it('a source without readings shows no reading line and no per-group count', async () => {
    const w = await lookedUp('打', DA)
    const vp = blocks(w)[1]!
    expect(vp.find('.lookup-entry-reading').exists()).toBe(false)
    expect(vp.find('.lookup-entry-count').exists()).toBe(false)
    expect(vp.findAll('.lookup-gloss').map((g) => g.text())).toEqual(['đánh (vp)', 'làm (vp)'])
  })

  it('the spine is only "N nguồn · M nghĩa"', async () => {
    const w = await lookedUp('打', DA)
    expect(w.get('.lookup-spine-count').text().replace(/\s+/g, ' ')).toBe('2 nguồn · 5 nghĩa')
  })

  it('a LIMIT-cut source reads "ít nhất N nghĩa" in its block and in the spine', async () => {
    const cut = response([group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 1, '打', null)], 40)], {
      'layer-cv': [sense(1, 11, 'đánh')],
    })
    const w = await lookedUp('打', cut)
    expect(blocks(w)[0]!.get('.lookup-source-count').text()).toBe('ít nhất 1 nghĩa')
    expect(w.get('.lookup-spine-count').text()).toContain('ít nhất')
  })
})

describe('computeSpine', () => {
  it('counts hidden sources in N and turns the sense total into a lower bound', () => {
    const spine = computeSpine(DA.grouped.groups, DA.senses_by_layer, [['CC-CEDICT', 3]])
    expect(spine).toEqual({ sourceCount: 3, senseCount: 5, senseCountAtLeast: true })
  })

  it('is exact when nothing was cut or hidden', () => {
    expect(computeSpine(DA.grouped.groups, DA.senses_by_layer)).toEqual({
      sourceCount: 2,
      senseCount: 5,
      senseCountAtLeast: false,
    })
  })
})

describe('reading beside the headword', () => {
  it('shows the shared key when every source with a reading agrees (走廊: zou3 lang2 and zǒuláng)', async () => {
    const w = await lookedUp('走廊', ZOULANG)
    expect(w.get('.lookup-head-reading').text()).toBe('zǒuláng')
  })

  it('shows nothing when two readings differ (打: da2, da3)', async () => {
    const w = await lookedUp('打', DA)
    expect(w.find('.lookup-head-reading').exists()).toBe(false)
  })
})

describe('Pin', () => {
  const single = response([group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 5, '走廊', 'zou3 lang2')])], {
    'layer-cv': [sense(5, 51, 'hành lang')],
  })

  it('the panel-head button pins the only entry of a one-entry result', async () => {
    const w = await lookedUp('走廊', single)
    await w.get('.lookup-head-pin').trigger('mousedown')
    await w.get('.lookup-head-pin').trigger('click')
    await flushPromises()
    expect(h.pins).toEqual([{ sourceCode: 'cvdict', entryId: 5, headword: '走廊', gloss: 'hành lang' }])
  })

  it('with several entries the head button answers pin_no_target and pins nothing', async () => {
    const w = await lookedUp('打', DA)
    await w.get('.lookup-head-pin').trigger('mousedown')
    await w.get('.lookup-head-pin').trigger('click')
    await flushPromises()
    expect(h.pins).toEqual([])
    expect(w.text()).toContain('Chưa nhắm được mục từ nào để ghim')
  })

  it('the small button on a group still pins exactly that entry', async () => {
    const w = await lookedUp('打', DA)
    const second = blocks(w)[0]!.findAll('.lookup-entry')[1]!
    await second.get('.lookup-pin').trigger('mousedown')
    await second.get('.lookup-pin').trigger('click')
    await flushPromises()
    expect(h.pins).toEqual([{ sourceCode: 'cvdict', entryId: 2, headword: '打', gloss: 'đánh' }])
  })

  it('the head button is absent when there is no result to pin', async () => {
    const w = await lookedUp('鐵鏽', NOT_FOUND)
    expect(w.find('.lookup-head-pin').exists()).toBe(false)
  })
})

describe('collapse', () => {
  it('⌄ folds the block to its head line, ⌃ opens it again, and the other block is untouched', async () => {
    const w = await lookedUp('打', DA)
    const collapse = () => blocks(w)[0]!.get('.lookup-collapse')

    await collapse().trigger('mousedown')
    await collapse().trigger('click')
    expect(blocks(w)[0]!.findAll('.lookup-entry')).toHaveLength(0)
    expect(blocks(w)[0]!.get('.lookup-source-count').text()).toBe('3 nghĩa')
    expect(collapse().text()).toBe('⌃')
    expect(collapse().attributes('aria-expanded')).toBe('false')
    expect(blocks(w)[1]!.findAll('.lookup-entry')).toHaveLength(1)

    await collapse().trigger('mousedown')
    await collapse().trigger('click')
    expect(blocks(w)[0]!.findAll('.lookup-entry')).toHaveLength(2)
    expect(collapse().text()).toBe('⌄')
  })

  it('the folded state is remembered per source_code across a new lookup', async () => {
    const w = await lookedUp('打', DA)
    const collapse = blocks(w)[0]!.get('.lookup-collapse')
    await collapse.trigger('mousedown')
    await collapse.trigger('click')

    h.answers.set('打2', DA)
    await runLookup('打2')
    await flushPromises()
    expect(blocks(w)[0]!.findAll('.lookup-entry')).toHaveLength(0)
    expect(blocks(w)[1]!.findAll('.lookup-entry')).toHaveLength(1)
  })

  it('resetLookupPanel forgets the folded state', async () => {
    const w = await lookedUp('打', DA)
    const collapse = blocks(w)[0]!.get('.lookup-collapse')
    await collapse.trigger('mousedown')
    await collapse.trigger('click')
    resetLookupPanel()
    await runLookup('打')
    await flushPromises()
    expect(blocks(w)[0]!.findAll('.lookup-entry')).toHaveLength(2)
  })
})

describe('empty states', () => {
  it('never looked up: the instruction carries the real chord of lookup.lookup_selection', async () => {
    await loadDictSources('')
    panel = mount(LookupPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    const chord = shortcutRows.value.find((row) => row.id === 'lookup.lookup_selection')!.display.join(' / ')
    expect(chord).not.toBe('')
    expect(panel.get('.lookup-empty').text()).toBe(`Chọn một từ trong Nguyên văn rồi bấm ${chord} để tra cứu.`)
  })

  it('not found: title, checked count and one "Tra riêng" button per character; clicking looks that character up', async () => {
    h.answers.set('鐵', DA)
    const w = await lookedUp('鐵鏽', NOT_FOUND)
    expect(w.get('.lookup-empty-title').text()).toBe('Không nguồn nào có 鐵鏽')
    expect(w.get('.lookup-empty').text()).toBe('Đã tra 3 nguồn đang bật.')
    const chars = w.findAll('.lookup-char')
    expect(chars.map((c) => c.text())).toEqual(['鐵', '鏽'])

    h.lookups = []
    await chars[0]!.trigger('mousedown')
    await chars[0]!.trigger('click')
    await flushPromises()
    expect(h.lookups).toEqual(['鐵'])
  })

  it('a one-character query has no "Tra riêng"', async () => {
    const w = await lookedUp('鐵', NOT_FOUND)
    expect(w.find('.lookup-chars').exists()).toBe(false)
    expect(w.get('.lookup-empty-title').text()).toBe('Không nguồn nào có 鐵')
  })

  it('the Concordance pointer shows when the background count is above zero', async () => {
    h.concordanceTotal = 3
    const w = await lookedUp('鐵鏽', NOT_FOUND)
    expect(w.get('.lookup-concordance-pointer').text()).toBe('Concordance có 3 kết quả')
  })

  it('every source off keeps the all_sources_off sentence and none of the not-found block', async () => {
    await loadDictSources('')
    for (const s of SOURCES) toggleDictSource(s.code)
    await flushPromises()
    const w = await lookedUp('鐵鏽', NOT_FOUND, true)
    expect(w.text()).toContain('Mọi nguồn từ điển đang tắt')
    expect(w.find('.lookup-empty-title').exists()).toBe(false)
    expect(w.find('.lookup-chars').exists()).toBe(false)
  })
})

describe('singleCharsOf', () => {
  it('splits a multi-character query into distinct characters and ignores spaces', () => {
    expect(singleCharsOf('鐵鏽')).toEqual(['鐵', '鏽'])
    expect(singleCharsOf('鐵 鏽 鐵')).toEqual(['鐵', '鏽'])
  })

  it('is empty for a one-character or blank query', () => {
    expect(singleCharsOf('鐵')).toEqual([])
    expect(singleCharsOf(' 鐵 ')).toEqual([])
    expect(singleCharsOf('')).toEqual([])
  })
})
