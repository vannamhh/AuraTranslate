/**
 * "Tra riêng" popup of the Lookup panel. Only the IPC boundary is faked; panel, popup, command
 * registry and state modules are real. Placement geometry is WebKit's business.
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import LookupPanel from '../../src/panels/LookupPanel.vue'
import { installCommands } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import { loadDictSources, resetDictSources, toggleDictSource } from '../../src/panels/dictSourcesState'
import { activateAimedLookupChar, resetLookupPanel, runLookup } from '../../src/panels/lookupPanelState'
import { lookupCharPopupChar, toggleLookupCharPopup } from '../../src/panels/lookupCharPopupState'
import { lookupHistory, selectLookupTab, resetLookupHistory, toggleLookupPin } from '../../src/panels/lookupHistoryState'
import { flushPromises } from './support/flushMicrotasks'
import { DA, NOT_FOUND, SOURCES, ZOULANG } from './support/lookupFixtures'

const h = vi.hoisted(() => ({
  lookups: [] as string[],
  pins: [] as { sourceCode: string; entryId: number; headword: string; gloss: string | null }[],
  answers: new Map<string, unknown>(),
  gates: new Map<string, Promise<void>>(),
  failing: new Set<string>(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args: Record<string, unknown>) => {
    if (cmd === 'list_dict_sources') return { sources: SOURCES, skipped: [] }
    if (cmd === 'put_config') return undefined
    if (cmd === 'lookup_dictionary') {
      const query = String(args.query ?? '')
      h.lookups.push(query)
      const gate = h.gates.get(query)
      if (gate !== undefined) await gate
      if (h.failing.has(query)) throw { code: 'dict.read', message_key: 'err.unknown', params: {}, retryable: false }
      return h.answers.get(query)
    }
    if (cmd === 'read_han_viet') return { lookup: null, error: null }
    if (cmd === 'list_pinned_entries') return []
    if (cmd === 'tm_concordance') return { query: '', tm_empty: false, total: 0, hits: [] }
    if (cmd === 'pin_entry') {
      h.pins.push(args as (typeof h.pins)[number])
      return h.pins.map((p, i) => ({
        id: i + 1,
        source_code: p.sourceCode,
        entry_id: p.entryId,
        headword: p.headword,
        gloss: p.gloss,
        pinned_at: '2026-10-26 00:00:00',
        source_version: null,
      }))
    }
    throw new Error(`lookupCharPopup.test.ts: unknown command "${cmd}"`)
  }),
}))

beforeAll(() => {
  installCommands({
    isMac: true,
    setMode: () => {},
    toggleLookupPin,
    lookupAimedChar: activateAimedLookupChar,
  } as CommandDeps)
})

let panel: VueWrapper | null = null

async function mountNotFound(query = '鐵鏽'): Promise<VueWrapper> {
  h.answers.set(query, NOT_FOUND)
  await loadDictSources('')
  panel = mount(LookupPanel, { props: { params: { params: {} } }, attachTo: document.body })
  await runLookup(query)
  await flushPromises()
  return panel
}

const charButtons = (w: VueWrapper) => w.findAll('.lookup-char')
const popupOf = (w: VueWrapper) => w.find('[data-lookup-char-popup]')

async function press(button: ReturnType<typeof charButtons>[number]): Promise<void> {
  await button.trigger('mousedown')
  await button.trigger('click')
  await flushPromises()
}

beforeEach(() => {
  document.body.innerHTML = ''
  h.lookups = []
  h.pins = []
  h.answers.clear()
  h.gates.clear()
  h.failing.clear()
})

afterEach(() => {
  panel?.unmount()
  panel = null
  resetDictSources()
  resetLookupPanel()
  resetLookupHistory()
  vi.useRealTimers()
})

describe('opening', () => {
  it('shows the record of the pressed character while the panel keeps its query and strip', async () => {
    h.answers.set('鐵', DA)
    const w = await mountNotFound()
    const buttons = charButtons(w)
    await press(buttons[0]!)
    expect(popupOf(w).exists()).toBe(true)
    expect(popupOf(w).get('[data-lookup-char-popup-headword]').text()).toBe('鐵')
    expect(popupOf(w).findAll('.lookup-source').length).toBeGreaterThan(0)
    expect(w.get('.lookup-empty-title').text()).toBe('Không nguồn nào có 鐵鏽')
    expect(charButtons(w).map((b) => b.text())).toEqual(['鐵', '鏽'])
    expect(buttons[0]!.attributes('aria-expanded')).toBe('true')
    expect(buttons[1]!.attributes('aria-expanded')).toBe('false')
  })

  it('three different characters in turn each show their own record, the panel title never changes', async () => {
    h.answers.set('甲', DA)
    h.answers.set('乙', ZOULANG)
    h.answers.set('丙', NOT_FOUND)
    const w = await mountNotFound('甲乙丙')
    for (const [i, ch] of ['甲', '乙', '丙'].entries()) {
      await press(charButtons(w)[i]!)
      expect(popupOf(w).get('[data-lookup-char-popup-headword]').text()).toBe(ch)
      expect(w.get('.lookup-empty-title').text()).toBe('Không nguồn nào có 甲乙丙')
      expect(charButtons(w)).toHaveLength(3)
    }
    expect(popupOf(w).text()).toContain('Không nguồn nào có 丙')
  })

  it('a character the dictionaries lack shows the not-found sentence and the checked count', async () => {
    h.answers.set('鐵', NOT_FOUND)
    const w = await mountNotFound()
    await press(charButtons(w)[0]!)
    expect(popupOf(w).get('[data-lookup-char-popup-not-found]').text()).toBe('Không nguồn nào có 鐵')
    expect(popupOf(w).text()).toContain('Đã tra 3 nguồn đang bật.')
  })

  it('an IPC failure shows lookup_failed in the popup', async () => {
    h.failing.add('鐵')
    const w = await mountNotFound()
    await press(charButtons(w)[0]!)
    expect(popupOf(w).get('[data-lookup-char-popup-error]').text()).toContain('Không tra cứu được')
  })

  it('every source off shows the all_sources_off sentence instead of not-found', async () => {
    h.answers.set('鐵', NOT_FOUND)
    const w = await mountNotFound()
    expect(popupOf(w).exists()).toBe(false)
    for (const s of SOURCES) toggleDictSource(s.code)
    await flushPromises()
    toggleLookupCharPopup(document.createElement('button'), '鐵')
    await flushPromises()
    expect(popupOf(w).text()).toContain('Mọi nguồn từ điển đang tắt')
    expect(popupOf(w).find('[data-lookup-char-popup-not-found]').exists()).toBe(false)
  })

  it('never writes a Lookup history row and never re-runs the panel lookup', async () => {
    h.answers.set('鐵', DA)
    h.answers.set('鏽', NOT_FOUND)
    const w = await mountNotFound()
    const rowsBefore = lookupHistory.value.map((r) => r.query)
    h.lookups = []
    await press(charButtons(w)[0]!)
    await press(charButtons(w)[1]!)
    expect(h.lookups).toEqual(['鐵', '鏽'])
    expect(lookupHistory.value.map((r) => r.query)).toEqual(rowsBefore)
    expect(rowsBefore).toEqual(['鐵鏽'])
  })

  it('pressing A then B before A answers anchors under B and shows only B', async () => {
    let releaseA: () => void = () => {}
    h.gates.set('鐵', new Promise<void>((r) => (releaseA = r)))
    h.answers.set('鐵', DA)
    h.answers.set('鏽', ZOULANG)
    const w = await mountNotFound()
    await press(charButtons(w)[0]!)
    await press(charButtons(w)[1]!)
    releaseA()
    await flushPromises()
    expect(popupOf(w).get('[data-lookup-char-popup-headword]').text()).toBe('鏽')
    expect(popupOf(w).text()).toContain('hành lang')
    expect(popupOf(w).text()).not.toContain('đánh nhau')
    expect(charButtons(w)[1]!.attributes('aria-expanded')).toBe('true')
  })
})

describe('closing', () => {
  async function opened(): Promise<VueWrapper> {
    h.answers.set('鐵', DA)
    h.answers.set('鏽', DA)
    const w = await mountNotFound()
    await press(charButtons(w)[0]!)
    expect(popupOf(w).exists()).toBe(true)
    return w
  }

  it('pressing the open character again closes it', async () => {
    const w = await opened()
    await press(charButtons(w)[0]!)
    expect(popupOf(w).exists()).toBe(false)
    expect(charButtons(w)[0]!.attributes('aria-expanded')).toBe('false')
  })

  it('a mousedown outside the popup closes it; one inside does not', async () => {
    const w = await opened()
    popupOf(w).element.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await flushPromises()
    expect(popupOf(w).exists()).toBe(true)
    w.get('.lookup-empty-title').element.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)
  })

  it('Esc closes it and returns focus to the character button', async () => {
    const w = await opened()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)
    expect(document.activeElement).toBe(charButtons(w)[0]!.element)
  })

  it('a mousedown on empty panel space closes it and focus lands on the button once the browser has moved it', async () => {
    const w = await opened()
    vi.useFakeTimers()
    const blank = w.get('.lookup-scroll').element as HTMLElement
    blank.tabIndex = -1
    blank.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    blank.focus()
    await flushPromises()
    vi.runAllTimers()
    expect(popupOf(w).exists()).toBe(false)
    expect(document.activeElement).toBe(charButtons(w)[0]!.element)
  })

  it('Esc with focus on an element outside the panel (Mod+K input) leaves it open', async () => {
    const w = await opened()
    const input = document.createElement('input')
    document.body.appendChild(input)
    input.focus()
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flushPromises()
    expect(popupOf(w).exists()).toBe(true)
  })

  it('Esc while an IME is composing leaves it open', async () => {
    const w = await opened()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, isComposing: true }))
    await flushPromises()
    expect(popupOf(w).exists()).toBe(true)
  })

  it('a new panel lookup closes it, including the same query rerun after a source toggle', async () => {
    const w = await opened()
    await runLookup('鐵鏽')
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)

    await press(charButtons(w)[0]!)
    expect(popupOf(w).exists()).toBe(true)
    toggleDictSource('vietphrase')
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)
  })

  it('a different panel query closes it', async () => {
    const w = await opened()
    h.answers.set('銅鏽', NOT_FOUND)
    await runLookup('銅鏽')
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)
  })

  it('a Work change (resetLookupPanel) closes it', async () => {
    const w = await opened()
    resetLookupPanel()
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)
  })

  it('unmounting the panel closes the popup state', async () => {
    const w = await opened()
    w.unmount()
    panel = null
    expect(lookupCharPopupChar.value).toBeNull()
  })

  it('leaving the record tab closes it', async () => {
    const w = await opened()
    selectLookupTab('concordance')
    await flushPromises()
    expect(popupOf(w).exists()).toBe(false)
    expect(lookupCharPopupChar.value).toBeNull()
    selectLookupTab('record')
  })

  it('a mousedown outside the panel or on a panel button does not pull focus to the character button', async () => {
    const w = await opened()
    vi.useFakeTimers()
    const outside = document.createElement('div')
    document.body.appendChild(outside)
    outside.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await flushPromises()
    vi.runAllTimers()
    expect(popupOf(w).exists()).toBe(false)
    expect(document.activeElement).not.toBe(charButtons(w)[0]!.element)

    await press(charButtons(w)[0]!)
    const other = w.find('.lookup-concordance-pointer, [role="tab"]')
    const target = (other.exists() ? other.element : w.get('.lookup-char:last-child').element) as HTMLElement
    target.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await flushPromises()
    vi.runAllTimers()
    expect(document.activeElement).not.toBe(charButtons(w)[0]!.element)
  })
})

describe('pinning inside the popup', () => {
  it('pins exactly the entry pressed even though the panel itself has no targets, with no pin_no_target', async () => {
    h.answers.set('鐵', DA)
    const w = await mountNotFound()
    await press(charButtons(w)[0]!)
    const pins = popupOf(w).findAll('.lookup-pin')
    expect(pins.length).toBeGreaterThan(1)
    await pins[1]!.trigger('mousedown')
    await pins[1]!.trigger('click')
    await flushPromises()
    expect(h.pins).toHaveLength(1)
    expect(h.pins[0]).toMatchObject({ sourceCode: 'cvdict', entryId: 2 })
    expect(popupOf(w).findAll('.lookup-pin')[1]!.attributes('aria-pressed')).toBe('true')
    expect(popupOf(w).findAll('.lookup-pin')[0]!.attributes('aria-pressed')).toBe('false')
    expect(w.text()).not.toContain('Chạm vào một mục')
  })
})

describe('keyboard', () => {
  it('a focused character button opens the popup through the command (no mouse aim)', async () => {
    h.answers.set('鐵', DA)
    const w = await mountNotFound()
    const button = charButtons(w)[0]!.element as HTMLElement
    button.focus()
    button.click()
    await flushPromises()
    expect(popupOf(w).exists()).toBe(true)
  })
})
