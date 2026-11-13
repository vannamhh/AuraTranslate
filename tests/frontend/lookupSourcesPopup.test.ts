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
  moveFocusedDictSourceDown,
  moveFocusedDictSourceUp,
  resetDictSources,
  sourcesMenuIsOpen,
  toggleFocusedDictSource,
  toggleSourcesMenu,
} from '../../src/panels/dictSourcesState'
import { resetLookupPanel, runLookup, sourceEntryCount } from '../../src/panels/lookupPanelState'
import { resetLookupHistory } from '../../src/panels/lookupHistoryState'
import { flushPromises } from './support/flushMicrotasks'
import { installFakeDragJs, pressDragHeader } from './support/dragRegionPress'
import { DA, NOT_FOUND, SOURCES, group, hit, response } from './support/lookupFixtures'

const h = vi.hoisted(() => ({
  lookups: [] as string[],
  puts: [] as unknown[],
  putFails: false,
  answer: null as unknown,
  order: [] as string[],
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args: { query?: string }) => {
    if (cmd === 'list_dict_sources') {
      const rank = (code: string) => (h.order.includes(code) ? h.order.indexOf(code) : h.order.length)
      return { sources: [...SOURCES].sort((a, b) => rank(a.code) - rank(b.code)), skipped: [] }
    }
    if (cmd === 'put_config') {
      h.puts.push(args)
      if (h.putFails) throw { code: 'config.write', message_key: 'err.unknown', params: {}, retryable: false }
      const put = args as unknown as { key: string; value: string }
      if (put.key === 'dict_sources_order') h.order = put.value.split(',')
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
    moveDictSourceUp: moveFocusedDictSourceUp,
    moveDictSourceDown: moveFocusedDictSourceDown,
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
  h.order = []
})

afterEach(() => {
  vi.restoreAllMocks()
  panel?.unmount()
  panel = null
  resetDictSources()
  resetLookupPanel()
  resetLookupHistory()
})

describe('Nguồn N/M popup', () => {
  it('the trigger reads enabled/total, the list is closed until the command runs and rows carry the source code', async () => {
    const w = await openPanel()
    expect(trigger(w).text()).toBe('Nguồn 3/3')
    expect(trigger(w).attributes('aria-haspopup')).toBe('menu')
    expect(trigger(w).attributes('aria-expanded')).toBe('false')
    expect(rows(w)).toHaveLength(0)

    await trigger(w).trigger('click')
    expect(sourcesMenuIsOpen.value).toBe(true)
    expect(trigger(w).attributes('aria-expanded')).toBe('true')
    expect(rows(w).map((r) => r.attributes('data-source-code'))).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
    expect(rows(w).map((r) => r.attributes('aria-checked'))).toEqual(['true', 'true', 'true'])
  })

  it('a press on a drag-region header closes the open list although drag.js swallows the bubble phase', async () => {
    installFakeDragJs()
    const w = await openPanel()
    await trigger(w).trigger('click')
    expect(sourcesMenuIsOpen.value).toBe(true)
    pressDragHeader()
    expect(sourcesMenuIsOpen.value).toBe(false)
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
    expect(trigger(w).text()).toBe('Nguồn 2/3')
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
    expect(trigger(w).text()).toBe('Nguồn 3/3')
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

describe('source order dragging', () => {
  const codes = (w: VueWrapper) => rows(w).map((r) => r.attributes('data-source-code'))
  const rowEls = (w: VueWrapper) => w.findAll('.sources-row')
  const grip = (w: VueWrapper, i: number) => rowEls(w)[i]!.get('.sources-grip')
  const ROW_H = 28
  const list = (w: VueWrapper) => w.get('.sources-list').element

  // happy-dom lays nothing out: every row is a 28px slab stacked in its current DOM order.
  function stubLayout(w: VueWrapper) {
    const rect = (top: number, height: number) =>
      ({ top, bottom: top + height, height, left: 0, right: 300, width: 300 }) as DOMRect
    vi.spyOn(list(w), 'getBoundingClientRect').mockImplementation(() => rect(0, ROW_H * rowEls(w).length))
    for (const li of rowEls(w)) {
      vi.spyOn(li.element, 'getBoundingClientRect').mockImplementation(() =>
        rect(rowEls(w).findIndex((r) => r.element === li.element) * ROW_H, ROW_H),
      )
    }
  }

  function patchCapture(w: VueWrapper) {
    const el = list(w) as HTMLElement
    const held = new Set<number>()
    el.setPointerCapture = (id: number) => void held.add(id)
    el.releasePointerCapture = (id: number) => void held.delete(id)
    el.hasPointerCapture = (id: number) => held.has(id)
    return held
  }

  const ptr = (type: string, y: number) => new PointerEvent(type, { pointerId: 1, button: 0, clientY: y, bubbles: true })

  async function startDrag(w: VueWrapper, from: number) {
    await trigger(w).trigger('click')
    await flushPromises()
    stubLayout(w)
    const held = patchCapture(w)
    grip(w, from).element.dispatchEvent(ptr('pointerdown', from * ROW_H + 14))
    await flushPromises()
    return held
  }

  it('has no up/down buttons and the grip is hidden from assistive tech', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    expect(rowEls(w)[0]!.findAll('button')).toHaveLength(1)
    expect(grip(w, 0).attributes('aria-hidden')).toBe('true')
  })

  it('dragging the last row to the top previews the order, then writes the full order once and re-runs the lookup', async () => {
    const w = await openPanel()
    await runLookup('打')
    await flushPromises()
    h.lookups = []
    await startDrag(w, 2)

    list(w).dispatchEvent(ptr('pointermove', 2))
    await flushPromises()
    expect(codes(w)).toEqual(['vietphrase', 'cvdict', 'cc-cedict'])
    expect(h.puts).toHaveLength(0)

    list(w).dispatchEvent(ptr('pointerup', 2))
    await flushPromises()
    expect(codes(w)).toEqual(['vietphrase', 'cvdict', 'cc-cedict'])
    expect(h.puts).toEqual([{ kind: 'app_config', key: 'dict_sources_order', value: 'vietphrase,cvdict,cc-cedict' }])
    expect(h.lookups).toEqual(['打'])
    expect(sourcesMenuIsOpen.value).toBe(true)
  })

  it('dropping back at the original position writes nothing', async () => {
    const w = await openPanel()
    await startDrag(w, 0)
    list(w).dispatchEvent(ptr('pointermove', 3 * ROW_H))
    await flushPromises()
    expect(codes(w)[2]).toBe('cvdict')
    list(w).dispatchEvent(ptr('pointermove', 5))
    await flushPromises()
    list(w).dispatchEvent(ptr('pointerup', 5))
    await flushPromises()
    expect(h.puts).toHaveLength(0)
    expect(codes(w)).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
  })

  it('Escape while dragging closes the menu, restores the order and writes nothing', async () => {
    const w = await openPanel()
    const held = await startDrag(w, 2)
    list(w).dispatchEvent(ptr('pointermove', 2))
    await flushPromises()
    expect(codes(w)[0]).toBe('vietphrase')

    await w.get('.sources-menu').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(sourcesMenuIsOpen.value).toBe(false)
    expect(held.size).toBe(0)
    expect(h.puts).toHaveLength(0)
    await trigger(w).trigger('click')
    expect(codes(w)).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
  })

  it('losing pointer capture mid-drag cancels without a write', async () => {
    const w = await openPanel()
    await startDrag(w, 2)
    list(w).dispatchEvent(ptr('pointermove', 2))
    await flushPromises()
    list(w).dispatchEvent(ptr('lostpointercapture', 2))
    await flushPromises()
    list(w).dispatchEvent(ptr('pointerup', 2))
    await flushPromises()
    expect(h.puts).toHaveLength(0)
    expect(codes(w)).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
  })

  it('holding the pointer in the bottom edge zone keeps scrolling the list and marks the dragged row', async () => {
    const w = await openPanel()
    const frames: FrameRequestCallback[] = []
    vi.spyOn(window, 'requestAnimationFrame').mockImplementation((cb) => frames.push(cb))
    await startDrag(w, 0)
    expect(rowEls(w)[0]!.classes()).toContain('dragged')
    expect(list(w).classList.contains('is-dragging')).toBe(true)

    const edgeY = ROW_H * 3 - 2
    list(w).dispatchEvent(ptr('pointermove', edgeY))
    expect(frames).toHaveLength(1)
    const before = list(w).scrollTop
    frames.shift()!(0)
    expect(list(w).scrollTop).toBeGreaterThan(before)
    expect(frames).toHaveLength(1)
    await flushPromises()
    expect(codes(w)[2]).toBe('cvdict')

    list(w).dispatchEvent(ptr('pointerup', edgeY))
    await flushPromises()
    expect(rowEls(w).some((r) => r.classes().includes('dragged'))).toBe(false)
  })

  it('pressing and releasing the grip toggles nothing and writes nothing', async () => {
    const w = await openPanel()
    await startDrag(w, 1)
    list(w).dispatchEvent(ptr('pointerup', 1 * ROW_H + 14))
    await flushPromises()
    expect(h.puts).toHaveLength(0)
    expect(rows(w).every((r) => !r.classes().includes('off'))).toBe(true)
  })

  it('a click on the switch or name still toggles the source without writing the order', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await rows(w)[1]!.trigger('click')
    await flushPromises()
    expect(rows(w)[1]!.classes()).toContain('off')
    expect(h.puts.every((p) => (p as { key: string }).key !== 'dict_sources_order')).toBe(true)
  })

  it('a failed write after a drop puts the rows back in their previous order', async () => {
    const w = await openPanel()
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    h.putFails = true
    await startDrag(w, 2)
    list(w).dispatchEvent(ptr('pointermove', 2))
    list(w).dispatchEvent(ptr('pointerup', 2))
    await flushPromises()
    expect(h.puts).toEqual([{ kind: 'app_config', key: 'dict_sources_order', value: 'vietphrase,cvdict,cc-cedict' }])
    expect(codes(w)).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
    warn.mockRestore()
  })

  it('bounds the list width by the distance from the panel body left edge to the list right edge, and titles each name', async () => {
    const w = await openPanel()
    const body = w.element.querySelector('.lookup-body')!
    vi.spyOn(body, 'getBoundingClientRect').mockReturnValue({ top: 0, bottom: 500, left: 20, right: 900 } as DOMRect)
    const proto = HTMLElement.prototype
    const real = proto.getBoundingClientRect
    vi.spyOn(proto, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
      if (this.classList.contains('sources-list')) return { top: 0, bottom: 10, left: 0, right: 320 } as DOMRect
      return real.call(this)
    })
    await trigger(w).trigger('click')
    await flushPromises()
    expect((list(w) as HTMLElement).style.maxWidth).toBe('300px')
    expect(w.get('.sources-name').attributes('title')).toBe(SOURCES[0]!.display_name)
  })

  it('the Alt+Shift+Arrow handler moves the focused row and focus stays on that row', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(rows(w)[1]!.element)

    moveFocusedDictSourceUp()
    await flushPromises()
    expect(codes(w)).toEqual(['cc-cedict', 'cvdict', 'vietphrase'])
    expect(document.activeElement).toBe(rows(w)[0]!.element)
    expect(rows(w)[0]!.attributes('data-source-code')).toBe('cc-cedict')

    moveFocusedDictSourceDown()
    await flushPromises()
    expect(codes(w)).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
    expect(document.activeElement).toBe(rows(w)[1]!.element)
  })

  it('arrow keys with a modifier do not move focus between rows', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(rows(w)[0]!.element)
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown', altKey: true, shiftKey: true })
    expect(document.activeElement).toBe(rows(w)[0]!.element)
  })

  it('a move at the edge or with no focused row does nothing', async () => {
    const w = await openPanel()
    await trigger(w).trigger('click')
    moveFocusedDictSourceUp()
    await w.get('.sources-menu').trigger('keydown', { key: 'ArrowDown' })
    moveFocusedDictSourceUp()
    await flushPromises()
    expect(h.puts).toHaveLength(0)
    expect(codes(w)).toEqual(['cvdict', 'cc-cedict', 'vietphrase'])
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
