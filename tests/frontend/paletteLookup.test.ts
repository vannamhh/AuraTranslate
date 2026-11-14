import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { LookupResponse, QuickGroup, QuickLookupResponse, QuickRow } from '../../src/config/dict'

const quickLookup = vi.fn()
const lookupDictionary = vi.fn()
const listWorks = vi.fn()

vi.mock('../../src/config/dict', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/dict')>()),
  quickLookup: (...args: unknown[]) => quickLookup(...args),
  lookupDictionary: (...args: unknown[]) => lookupDictionary(...args),
}))
vi.mock('../../src/config/library', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/library')>()),
  listLibraryWorks: (...args: unknown[]) => listWorks(...args),
}))

function row(headword: string, reading: string | null, gloss: string | null): QuickRow {
  return {
    headword,
    headword_simp: null,
    reading,
    gloss,
    source_code: 'cvdict',
    source_name: 'CVDict',
    source_count: 1,
  }
}

function group(id: QuickGroup['id'], rows: QuickRow[], extra: Partial<QuickGroup> = {}): QuickGroup {
  return {
    id,
    status: rows.length > 0 ? 'rows' : 'empty',
    rows,
    has_more: false,
    incomplete: false,
    error: null,
    ...extra,
  }
}

function quick(groups: QuickGroup[], extra: Partial<QuickLookupResponse> = {}) {
  const response: QuickLookupResponse = {
    route: 'en',
    query_truncated: false,
    layers_loaded: true,
    skipped: [],
    groups,
    ...extra,
  }
  return { response, error: null }
}

function preview(headword: string) {
  const response: LookupResponse = {
    grouped: {
      route: 'zh',
      branch: 'exact_btree',
      groups: [
        {
          layer: 'core',
          source: { code: 'cvdict', display_name: 'CVDict' },
          entries: [
            { entry_id: 1, source_code: 'cvdict', lang: 'zh', headword, headword_simp: null, reading: 'ni3 hao3' },
          ],
          total_entries: null,
        },
      ],
      skipped: [],
      truncated_layers: [],
      hidden_sources: [],
      layers_loaded: true,
    },
    senses_by_layer: {
      core: [
        {
          sense_id: 1,
          entry_id: 1,
          pos: null,
          pos_lang: null,
          pos_is_foreign: false,
          gloss: `nghia-${headword}`,
          examples: [],
          citations: [],
          note: null,
        },
      ],
    },
    query_truncated: false,
    senses_failed: [],
  } as unknown as LookupResponse
  return { response, error: null }
}

const NIHAO = quick([
  group('pinyin', [row('你好', 'ni3 hao3', 'xin chào'), row('你们', 'ni3 men5', 'các bạn')]),
  group('vi_zh', []),
  group('en_vi', []),
])

async function setup(runLookup?: (query: string) => void | Promise<void>) {
  vi.resetModules()
  const commands = await import('../../src/commands')
  const palette = await import('../../src/paletteState')
  const lookupState = await import('../../src/paletteLookupState')
  const history = await import('../../src/panels/lookupHistoryState')
  const port = runLookup ?? vi.fn()
  commands.installCommands({
    setMode: vi.fn(),
    isMac: false,
    ...palette.paletteCommandDeps({ runLookup: port }),
  } as import('../../src/commands').CommandDeps)
  const Overlay = (await import('../../src/PaletteOverlay.vue')).default
  const i18n = await import('../../src/i18n')
  const w = mount(Overlay, { attachTo: document.body })
  palette.openPalette()
  await flushPromises()
  return { commands, palette, lookupState, history, w, i18n, runLookup: port }
}

async function type(w: ReturnType<typeof mount>, value: string) {
  await w.get('input').setValue(value)
  await flushPromises()
}

const scrollIntoView = vi.fn()

beforeEach(() => {
  document.body.innerHTML = ''
  scrollIntoView.mockReset()
  Element.prototype.scrollIntoView = scrollIntoView
  quickLookup.mockReset()
  lookupDictionary.mockReset()
  listWorks.mockReset()
  listWorks.mockResolvedValue({ report: { works: [], total: 0, matched: 0, genres: [], source_langs: [] }, error: null })
  lookupDictionary.mockImplementation((q: string) => Promise.resolve(preview(q)))
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('palette quick-lookup mode', () => {
  it('a leading slash enters the mode, is not kept as text, and hides the command list', async () => {
    const { w, i18n } = await setup()
    expect(w.find('[data-palette-kind="command"]').exists()).toBe(true)
    await type(w, '/')
    expect(w.find('[data-palette-mode="lookup"]').text()).toBe(i18n.t('palette.lookup.mode_label'))
    expect((w.get('input').element as HTMLInputElement).value).toBe('')
    expect(w.find('[data-palette-kind="command"]').exists()).toBe(false)
    expect(quickLookup).not.toHaveBeenCalled()
    w.unmount()
  })

  it('Backspace on an empty box leaves the mode; on a non-empty box it does not', async () => {
    const { w, lookupState } = await setup()
    quickLookup.mockResolvedValue(NIHAO)
    await type(w, '/nihao')
    await w.get('input').trigger('keydown', { key: 'Backspace' })
    expect(lookupState.paletteLookupActive.value).toBe(true)
    await type(w, '')
    await w.get('input').trigger('keydown', { key: 'Backspace' })
    expect(lookupState.paletteLookupActive.value).toBe(false)
    expect(w.find('[data-palette-kind="command"]').exists()).toBe(true)
    w.unmount()
  })

  it('/nihao queries once, lists the first pinyin row with tone marks and previews it', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w } = await setup()
    await type(w, '/nihao')
    expect(quickLookup).toHaveBeenCalledTimes(1)
    expect(quickLookup).toHaveBeenCalledWith('nihao')
    const options = w.findAll('[data-palette-kind="lookup"]')
    expect(options).toHaveLength(2)
    expect(options[0]!.text()).toContain('你好')
    expect(options[0]!.text()).toContain('nǐ hǎo')
    expect(options[0]!.text()).toContain('xin chào')
    expect(w.find('[data-palette-lookup-group="vi_zh"]').exists()).toBe(false)
    expect(lookupDictionary).toHaveBeenCalledWith('你好')
    expect(w.get('[data-palette-preview]').text()).toContain('nghia-你好')
    w.unmount()
  })

  it('moving the cursor previews the new row and never records Lookup history', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w, history } = await setup()
    await type(w, '/nihao')
    await w.get('input').trigger('keydown', { key: 'ArrowDown' })
    await flushPromises()
    expect(lookupDictionary).toHaveBeenLastCalledWith('你们')
    expect(w.get('[data-palette-preview]').text()).toContain('nghia-你们')
    expect(history.lookupHistory.value).toHaveLength(0)
    w.unmount()
  })

  it('shows only the newest preview when an older one arrives late', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w } = await setup()
    await type(w, '/nihao')
    const resolvers: ((v: unknown) => void)[] = []
    lookupDictionary.mockImplementation(() => new Promise((r) => resolvers.push(r)))
    await w.get('input').trigger('keydown', { key: 'ArrowDown' })
    await w.get('input').trigger('keydown', { key: 'ArrowUp' })
    expect(resolvers).toHaveLength(2)
    resolvers[1]!(preview('你好'))
    await flushPromises()
    resolvers[0]!(preview('你们'))
    await flushPromises()
    const text = w.get('[data-palette-preview]').text()
    expect(text).toContain('nghia-你好')
    expect(text).not.toContain('nghia-你们')
    w.unmount()
  })

  it('the preview has no pin button', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w } = await setup()
    await type(w, '/nihao')
    expect(w.find('[data-palette-preview] .lookup-pin').exists()).toBe(false)
    w.unmount()
  })

  it('Enter closes the palette and runs the Lookup panel lookup for the row', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const runLookup = vi.fn()
    const { w, palette } = await setup(runLookup)
    await type(w, '/nihao')
    await w.get('input').trigger('keydown', { key: 'ArrowDown' })
    await w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(palette.paletteIsOpen.value).toBe(false)
    expect(runLookup).toHaveBeenCalledTimes(1)
    expect(runLookup).toHaveBeenCalledWith('你们')
    w.unmount()
  })

  it('Enter does nothing while a newer query is still in flight', async () => {
    let resolve: (v: unknown) => void = () => undefined
    quickLookup.mockReturnValueOnce(Promise.resolve(NIHAO)).mockReturnValueOnce(new Promise((r) => (resolve = r)))
    const runLookup = vi.fn()
    const { w, palette } = await setup(runLookup)
    await type(w, '/nihao')
    await type(w, 'nihaoa')
    await w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(runLookup).not.toHaveBeenCalled()
    expect(palette.paletteIsOpen.value).toBe(true)
    resolve(NIHAO)
    await flushPromises()
    w.unmount()
  })

  it('sends no query while an IME composition is open, and one when it ends', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w } = await setup()
    await type(w, '/')
    const input = w.get('input')
    const el = input.element as HTMLInputElement
    el.value = 'ni'
    el.dispatchEvent(new InputEvent('input', { isComposing: true, bubbles: true }))
    await flushPromises()
    expect(quickLookup).not.toHaveBeenCalled()
    el.value = '你'
    el.dispatchEvent(new InputEvent('input', { isComposing: true, bubbles: true }))
    await input.trigger('compositionend')
    await flushPromises()
    expect(quickLookup).toHaveBeenCalledTimes(1)
    expect(quickLookup).toHaveBeenCalledWith('你')
    w.unmount()
  })

  it('applies only the newest reply when an older one arrives late', async () => {
    const resolvers: ((v: unknown) => void)[] = []
    quickLookup.mockImplementation(() => new Promise((r) => resolvers.push(r)))
    const { w } = await setup()
    await type(w, '/ni')
    await type(w, 'nih')
    resolvers[1]!(quick([group('pinyin', [row('你好', 'ni3 hao3', 'xin chào')])]))
    await flushPromises()
    resolvers[0]!(quick([group('pinyin', [row('泥', 'ni2', 'bùn')])]))
    await flushPromises()
    const text = w.text()
    expect(text).toContain('你好')
    expect(text).not.toContain('泥')
    w.unmount()
  })

  it('hides empty groups but names an outdated and a failed group', async () => {
    quickLookup.mockResolvedValue(
      quick([
        group('pinyin', [], { status: 'outdated_dictionary' }),
        group('vi_zh', [], {
          status: 'error',
          error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
        }),
        group('en_vi', [row('an', null, 'an')]),
      ]),
    )
    const { w, i18n } = await setup()
    await type(w, '/an')
    expect(w.get('[data-palette-lookup-group="pinyin"]').text()).toContain(i18n.t('palette.lookup.group_outdated'))
    expect(w.get('[data-palette-lookup-group="vi_zh"]').text()).toContain(i18n.t('palette.lookup.group_error'))
    expect(w.find('[data-palette-lookup-group="en_vi"]').exists()).toBe(true)
    expect(w.find('[data-palette-lookup-empty]').exists()).toBe(false)
    w.unmount()
  })

  it('names an all-empty answer, a missing layer set, a failed call and a missing reply', async () => {
    const { w, i18n } = await setup()
    await type(w, '/')
    quickLookup.mockResolvedValueOnce(quick([group('pinyin', []), group('vi_zh', []), group('en_vi', [])]))
    await type(w, 'zzz')
    expect(w.text()).toContain(i18n.t('palette.lookup.no_results'))
    quickLookup.mockResolvedValueOnce(quick([], { layers_loaded: false }))
    await type(w, 'zzy')
    expect(w.text()).toContain(i18n.t('palette.lookup.no_layers'))
    expect(w.text()).not.toContain(i18n.t('palette.lookup.no_results'))
    quickLookup.mockResolvedValueOnce({
      response: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await type(w, 'zzx')
    expect(w.get('[data-palette-lookup-error]').text()).toContain(i18n.t('palette.lookup.error'))
    quickLookup.mockResolvedValueOnce({ response: null, error: null })
    await type(w, 'zzw')
    expect(w.text()).toContain(i18n.t('palette.lookup.unavailable'))
    w.unmount()
  })

  it('the Tra từ button enters the mode with focus in the box; the command row keeps the palette open', async () => {
    const { w, lookupState, palette } = await setup()
    await w.get('[data-palette-lookup-button]').trigger('click')
    await flushPromises()
    expect(lookupState.paletteLookupActive.value).toBe(true)
    expect(document.activeElement).toBe(w.get('input').element)
    await w.get('input').trigger('keydown', { key: 'Backspace' })
    expect(lookupState.paletteLookupActive.value).toBe(false)
    const rows = w.findAll('[data-palette-kind="command"]')
    const target = rows.find((r) => r.text().includes('Tra từ'))
    expect(target).toBeDefined()
    await target!.trigger('mousedown')
    await target!.trigger('click')
    await flushPromises()
    expect(palette.paletteIsOpen.value).toBe(true)
    expect(lookupState.paletteLookupActive.value).toBe(true)
    w.unmount()
  })

  it('reopening the palette starts in the Command list with no stale rows', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w, palette, lookupState } = await setup()
    await type(w, '/nihao')
    palette.closePalette()
    palette.openPalette()
    await flushPromises()
    expect(lookupState.paletteLookupActive.value).toBe(false)
    expect(lookupState.paletteLookupRows.value).toHaveLength(0)
    w.unmount()
  })

  it('reports loading while a newer query is in flight and keeps the old rows visible', async () => {
    let resolve: (v: unknown) => void = () => undefined
    quickLookup.mockResolvedValueOnce(NIHAO).mockReturnValueOnce(new Promise((r) => (resolve = r)))
    const { w, lookupState, i18n } = await setup()
    await type(w, '/nihao')
    expect(lookupState.paletteLookupStatus.value).toBe('ready')
    await type(w, 'nihaoa')
    expect(lookupState.paletteLookupStatus.value).toBe('loading')
    expect(w.text()).toContain(i18n.t('palette.lookup.loading'))
    expect(w.findAll('[data-palette-kind="lookup"]')).toHaveLength(2)
    resolve(NIHAO)
    await flushPromises()
    expect(lookupState.paletteLookupStatus.value).toBe('ready')
    w.unmount()
  })

  it('a slash in front of existing text stays text; pasting /nihao into an empty box enters the mode', async () => {
    const { w, lookupState } = await setup()
    await type(w, 'abc')
    await type(w, '/abc')
    expect(lookupState.paletteLookupActive.value).toBe(false)
    expect((w.get('input').element as HTMLInputElement).value).toBe('/abc')
    await type(w, '')
    quickLookup.mockResolvedValue(NIHAO)
    await type(w, '/nihao')
    expect(lookupState.paletteLookupActive.value).toBe(true)
    expect(quickLookup).toHaveBeenCalledWith('nihao')
    w.unmount()
  })

  it('exposes the list as expanded and scrolls the current row into view in lookup mode', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const { w } = await setup()
    await type(w, '/nihao')
    expect(w.get('input').attributes('aria-expanded')).toBe('true')
    scrollIntoView.mockClear()
    await w.get('input').trigger('keydown', { key: 'ArrowDown' })
    await flushPromises()
    expect(scrollIntoView).toHaveBeenCalledWith({ block: 'nearest' })
    w.unmount()
  })

  it('with the real runLookup, previewing adds no history and Enter adds exactly one entry', async () => {
    quickLookup.mockResolvedValue(NIHAO)
    const rest = await setup(async (query) => {
      const lp = await import('../../src/panels/lookupPanelState')
      await lp.runLookup(query)
    })
    await type(rest.w, '/nihao')
    await rest.w.get('input').trigger('keydown', { key: 'ArrowDown' })
    await flushPromises()
    expect(rest.history.lookupHistory.value).toHaveLength(0)
    await rest.w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(rest.history.lookupHistory.value).toHaveLength(1)
    rest.w.unmount()
  })
})
