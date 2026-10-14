/**
 * The `Nguồn N/M ▾` popup of the Lookup panel. Only the IPC boundary is faked; the panel, the
 * popup, the command registry and the real `lookup.toggle_source` handler run.
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import LookupPanel from '../../src/panels/LookupPanel.vue'
import { installCommands } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import {
  loadDictSources,
  resetDictSources,
  sourcesMenuIsOpen,
  toggleFocusedDictSource,
  toggleSourcesMenu,
} from '../../src/panels/dictSourcesState'
import { resetLookupPanel, runLookup, sourceEntryCount } from '../../src/panels/lookupPanelState'
import { resetLookupHistory } from '../../src/panels/lookupHistoryState'
import { flushPromises } from './support/flushMicrotasks'
import { DA, NOT_FOUND, SOURCES, group, hit, response } from './support/lookupFixtures'

const h = vi.hoisted(() => ({
  lookups: [] as string[],
  puts: [] as unknown[],
  putFails: false,
  answer: null as unknown,
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args: { query?: string }) => {
    if (cmd === 'list_dict_sources') return { sources: SOURCES, skipped: [] }
    if (cmd === 'put_config') {
      h.puts.push(args)
      if (h.putFails) throw { code: 'config.write', message_key: 'err.unknown', params: {}, retryable: false }
      return undefined
    }
    if (cmd === 'lookup_dictionary') {
      h.lookups.push(args.query ?? '')
      return h.answer
    }
    if (cmd === 'read_han_viet') return { lookup: null, error: null }
    if (cmd === 'list_pinned_entries') return []
    throw new Error(`lookupSourcesPopup.test.ts: unknown command "${cmd}"`)
  }),
}))

beforeAll(() => {
  installCommands({
    isMac: true,
    setMode: () => {},
    toggleDictSource: toggleFocusedDictSource,
    toggleLookupSourcesMenu: toggleSourcesMenu,
  } as CommandDeps)
})

let panel: VueWrapper | null = null

async function openPanel(answer: unknown = DA): Promise<VueWrapper> {
  h.answer = answer
  await loadDictSources('')
  panel = mount(LookupPanel, { props: { params: { params: {} } }, attachTo: document.body })
  await flushPromises()
  return panel
}

const trigger = (w: VueWrapper) => w.get('[data-sources-trigger]')
const rows = (w: VueWrapper) => w.findAll('[role="menuitemcheckbox"]')

beforeEach(() => {
  document.body.innerHTML = ''
  h.lookups = []
  h.puts = []
  h.putFails = false
})

afterEach(() => {
  panel?.unmount()
  panel = null
  resetDictSources()
  resetLookupPanel()
  resetLookupHistory()
})

describe('Nguồn N/M popup', () => {
  it('the trigger reads enabled/total, the list is closed until the command runs and rows carry the source code', async () => {
    const w = await openPanel()
    expect(trigger(w).text()).toBe('Nguồn 3/3 ▾')
    expect(trigger(w).attributes('aria-haspopup')).toBe('menu')
    expect(trigger(w).attributes('aria-expanded')).toBe('false')
    expect(rows(w)).toHaveLength(0)

    await trigger(w).trigger('click')
    expect(sourcesMenuIsOpen.value).toBe(true)
    expect(trigger(w).attributes('aria-expanded')).toBe('true')
    expect(rows(w).map((r) => r.attributes('data-source-code'))).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
    expect(rows(w).map((r) => r.attributes('aria-checked'))).toEqual(['true', 'true', 'true'])
  })

  it('ArrowDown twice then Space turns the second source off, re-runs the lookup and lowers N', async () => {
    const w = await openPanel()
    await runLookup('打')
    await flushPromises()
    h.lookups = []

    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    const second = rows(w)[1]!
    expect(document.activeElement).toBe(second.element)

    // Space on a focused <button> is a native click; happy-dom does not synthesise it.
    await second.trigger('click')
    await flushPromises()

    expect(h.puts).toHaveLength(1)
    expect(h.lookups).toEqual(['打'])
    expect(trigger(w).text()).toBe('Nguồn 2/3 ▾')
    expect(rows(w).map((r) => r.attributes('aria-checked'))).toEqual(['true', 'false', 'true'])
    expect(sourcesMenuIsOpen.value).toBe(true)
  })

  it('ArrowUp from no focus lands on the last row and wraps', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowUp' })
    expect(document.activeElement).toBe(rows(w)[2]!.element)
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(rows(w)[0]!.element)
  })

  it('Escape closes the list and puts focus on the Nguồn N/M button', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(rows(w)[0]!.element)

    await w.get('.sources-menu').trigger('keydown', { key: 'Escape' })
    expect(sourcesMenuIsOpen.value).toBe(false)
    expect(rows(w)).toHaveLength(0)
    expect(document.activeElement).toBe(trigger(w).element)
  })

  it('a failed config write rolls the toggle back and the label returns to 3/3', async () => {
    const w = await openPanel()
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    h.putFails = true
    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await rows(w)[0]!.trigger('click')
    await flushPromises()

    expect(rows(w).map((r) => r.attributes('aria-checked'))).toEqual(['true', 'true', 'true'])
    expect(trigger(w).text()).toBe('Nguồn 3/3 ▾')
    warn.mockRestore()
  })

  it('an off source shows the word "tắt" and no count; a source with no group shows 0; nothing looked up shows no count', async () => {
    const w = await openPanel(response([group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 1, '打', 'da2')])], {}))
    await trigger(w).trigger('click')
    expect(w.findAll('.sources-count')).toHaveLength(0)

    await runLookup('打')
    await flushPromises()
    expect(rows(w).map((r) => r.find('.sources-count').text())).toEqual(['1', '0', '0'])

    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await rows(w)[1]!.trigger('click')
    await flushPromises()
    expect(rows(w)[1]!.classes()).toContain('off')
    expect(rows(w)[1]!.find('.sources-count').text()).toBe('tắt')
  })
})

describe('sourceEntryCount', () => {
  const cv = SOURCES[0]!
  const cc = SOURCES[1]!
  const looked = (patch: Partial<typeof DA.grouped>) => ({ ...DA.grouped, ...patch })

  it('counts entries for an enabled source with a group, and 0 only when enabled, uncut and groupless', () => {
    expect(sourceEntryCount(DA.grouped, cv, false)).toBe(2)
    expect(sourceEntryCount(DA.grouped, cc, false)).toBe(0)
  })

  it('is unknown for nothing looked up, a disabled source, a skipped layer, a LIMIT-hidden source and a partly cut group', () => {
    expect(sourceEntryCount(null, cv, false)).toBeNull()
    expect(sourceEntryCount(DA.grouped, cv, true)).toBeNull()
    expect(sourceEntryCount(looked({ skipped: ['layer-cc'] }), cc, false)).toBeNull()
    expect(sourceEntryCount(looked({ hidden_sources: [['CC-CEDICT', 3]] }), cc, false)).toBeNull()
    const cut = looked({ groups: [group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 1, '打', null)], 40)] })
    expect(sourceEntryCount(cut, cv, false)).toBeNull()
    expect(sourceEntryCount(NOT_FOUND.grouped, cv, false)).toBe(0)
  })
})
