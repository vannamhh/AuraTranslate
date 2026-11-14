import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { WorkRow } from '../../src/config/library'

const listWorks = vi.fn()
const openWorkById = vi.fn()

vi.mock('../../src/config/library', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/library')>()),
  listLibraryWorks: (...args: unknown[]) => listWorks(...args),
}))
vi.mock('../../src/modes/libraryChapters', () => ({
  openWorkById: (...args: unknown[]) => openWorkById(...args),
}))

function work(id: string, name: string): WorkRow {
  return {
    work_id: id,
    atproj_path: `/w/${id}.atproj`,
    name,
    source_lang: 'zh',
    genre: 'x',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    chapter_count: 10,
    status: 'in_progress',
    status_is_override: false,
    chapter_done_count: 3,
  }
}

function report(works: WorkRow[], total = works.length) {
  return { report: { works, total, matched: works.length, genres: [], source_langs: [] }, error: null }
}

const WORKS = [work('a', 'Đấu Phá Thương Khung'), work('b', 'Toàn Chức Pháp Sư')]

async function setup(extra: Record<string, unknown> = {}) {
  vi.resetModules()
  const commands = await import('../../src/commands')
  const modes = await import('../../src/modes/modeState')
  const palette = await import('../../src/paletteState')
  const setMode = vi.fn()
  const keymap = commands.installCommands({
    setMode,
    isMac: false,
    ...palette.paletteCommandDeps(),
    ...extra,
  } as import('../../src/commands').CommandDeps)
  const Overlay = (await import('../../src/PaletteOverlay.vue')).default
  const library = await import('../../src/modes/libraryWorks')
  const i18n = await import('../../src/i18n')
  return { commands, modes, palette, setMode, keymap, Overlay, library, i18n }
}

async function typeInto(wrapper: ReturnType<typeof mount>, value: string) {
  const input = wrapper.get('input')
  await input.setValue(value)
}

beforeEach(() => {
  document.body.innerHTML = ''
  listWorks.mockReset()
  openWorkById.mockReset()
  listWorks.mockResolvedValue(report(WORKS))
  openWorkById.mockResolvedValue(true)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('Mod+K palette', () => {
  it('opens on the default chord Mod+K', async () => {
    const { keymap } = await setup()
    const bound = new Map(keymap.bindings().map((b) => [b.id, b.chord]))
    expect(bound.get('palette.open')).toBe('Mod+K')
  })

  it('lists only marked commands: settings.close is absent, settings.open present', async () => {
    const { palette, i18n } = await setup()
    palette.openPalette()
    await flushPromises()
    const labels = palette.paletteCommandItems.value.map((i) => i.label)
    expect(labels).toContain(i18n.t('command.settings.open'))
    expect(labels).not.toContain(i18n.t('command.settings.close'))
    palette.setPaletteQuery(i18n.t('command.settings.close'))
    expect(palette.paletteCommandItems.value.map((i) => i.label)).not.toContain(i18n.t('command.settings.close'))
  })

  it('finds a Work without diacritics', async () => {
    const { palette } = await setup()
    palette.openPalette()
    await flushPromises()
    palette.setPaletteQuery('dau pha')
    expect(palette.paletteWorkItems.value.map((i) => i.label)).toEqual(['Đấu Phá Thương Khung'])
    expect(palette.foldVietnamese('Đ Ế')).toBe('d e')
  })

  it('shows a loading state and then the Works when Library was never loaded', async () => {
    let resolve: (v: unknown) => void = () => undefined
    listWorks.mockReturnValue(new Promise((r) => (resolve = r)))
    const { palette, Overlay, i18n } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    expect(w.text()).toContain(i18n.t('palette.works_loading'))
    expect(w.text()).not.toContain(i18n.t('palette.no_results'))
    resolve(report(WORKS))
    await flushPromises()
    expect(w.text()).toContain('Toàn Chức Pháp Sư')
    w.unmount()
  })

  it('shows the load error, not an empty list', async () => {
    listWorks.mockResolvedValue({
      report: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    const { palette, Overlay, i18n } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    expect(w.find('[role="alert"]').text()).toContain(i18n.t('palette.works_error'))
    expect(w.text()).not.toContain(i18n.t('palette.no_results'))
    w.unmount()
  })

  it('Enter on a Work closes first, then opens it and moves to Workspace', async () => {
    const { palette, Overlay, setMode, modes } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    await typeInto(w, 'dau pha')
    await w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(palette.paletteIsOpen.value).toBe(false)
    expect(openWorkById).toHaveBeenCalledWith('a')
    expect(modes.currentMode.value).toBe('workspace')
    expect(setMode).not.toHaveBeenCalled()
    w.unmount()
  })

  it('a blocked open (flush still dirty) leaves the mode alone and closes the palette', async () => {
    openWorkById.mockResolvedValue(false)
    const { palette, Overlay, modes } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    await typeInto(w, 'toan chuc')
    await w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(openWorkById).toHaveBeenCalledWith('b')
    expect(palette.paletteIsOpen.value).toBe(false)
    expect(modes.currentMode.value).toBe('library')
    w.unmount()
  })

  it('runs a command after the palette closed and focus went back', async () => {
    const openSettings = vi.fn()
    const { palette, Overlay, i18n } = await setup({ openSettings })
    const origin = document.createElement('button')
    document.body.appendChild(origin)
    origin.focus()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    expect(document.activeElement).toBe(w.get('input').element)
    await typeInto(w, i18n.t('command.settings.open'))
    let openAtRun: boolean | null = null
    let focusAtRun: Element | null = null
    openSettings.mockImplementation(() => {
      openAtRun = palette.paletteIsOpen.value
      focusAtRun = document.activeElement
    })
    await w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(openSettings).toHaveBeenCalledTimes(1)
    expect(openAtRun).toBe(false)
    expect(focusAtRun).toBe(origin)
    w.unmount()
  })

  it('the dialog resolves its accessible name to the visible title', async () => {
    const { palette, Overlay, i18n } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    const id = w.get('[role="dialog"]').attributes('aria-labelledby') ?? ''
    expect(document.getElementById(id)?.textContent).toBe(i18n.t('palette.title'))
    w.unmount()
  })

  it('Escape closes and returns focus to where it was', async () => {
    const { palette, Overlay } = await setup()
    const origin = document.createElement('button')
    document.body.appendChild(origin)
    origin.focus()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    await w.get('.pal-scrim').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(palette.paletteIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(origin)
    w.unmount()
  })

  it('no match shows one line and Enter does nothing', async () => {
    const openSettings = vi.fn()
    const { palette, Overlay, i18n } = await setup({ openSettings })
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    await typeInto(w, 'zzzzqqqq')
    expect(w.findAll('[role="option"]')).toHaveLength(0)
    expect(w.text()).toContain(i18n.t('palette.no_results'))
    await w.get('input').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(openWorkById).not.toHaveBeenCalled()
    expect(openSettings).not.toHaveBeenCalled()
    w.unmount()
  })

  it('a click on a row runs that row, and Enter during IME composition does nothing', async () => {
    const { palette, Overlay } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    await w.get('input').trigger('keydown', { key: 'Enter', isComposing: true })
    await flushPromises()
    expect(openWorkById).not.toHaveBeenCalled()
    const rows = w.findAll('[role="option"]')
    await rows[1].trigger('mousedown')
    await rows[1].trigger('click')
    await flushPromises()
    expect(openWorkById).toHaveBeenCalledWith('b')
    w.unmount()
  })

  it('the arrow keys move the cursor through both groups', async () => {
    const { palette, Overlay } = await setup()
    const w = mount(Overlay, { attachTo: document.body })
    palette.openPalette()
    await flushPromises()
    await w.get('input').trigger('keydown', { key: 'ArrowDown' })
    expect(palette.paletteCursor.value).toBe(1)
    await w.get('input').trigger('keydown', { key: 'ArrowUp' })
    await w.get('input').trigger('keydown', { key: 'ArrowUp' })
    expect(palette.paletteCursor.value).toBe(palette.paletteItems.value.length - 1)
    w.unmount()
  })

  it('Mod+K from inside a typing zone opens the palette, and Escape puts focus back in that field', async () => {
    const { commands, palette, Overlay } = await setup()
    const field = document.createElement('textarea')
    document.body.appendChild(field)
    field.focus()
    const w = mount(Overlay, { attachTo: document.body })
    const off = commands.attachKeyboard(document, { isBlocked: () => palette.paletteIsOpen.value })
    field.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyK', ctrlKey: true, bubbles: true, cancelable: true }))
    await flushPromises()
    expect(palette.paletteIsOpen.value).toBe(true)
    expect(document.activeElement).toBe(w.get('input').element)
    await w.get('.pal-scrim').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(document.activeElement).toBe(field)
    off()
    w.unmount()
  })

  it('a user override of palette.open opens on the new chord and Mod+K no longer does', async () => {
    const { commands, palette } = await setup()
    expect(commands.applyBindings({ 'palette.open': ['Mod+J'] }).ok).toBe(true)
    const off = commands.attachKeyboard(document, { isBlocked: () => false })
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyK', ctrlKey: true, bubbles: true }))
    expect(palette.paletteIsOpen.value).toBe(false)
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyJ', ctrlKey: true, bubbles: true }))
    expect(palette.paletteIsOpen.value).toBe(true)
    off()
  })

  it('does not open while another overlay holds the keyboard (attachKeyboard gate)', async () => {
    const { commands, palette } = await setup()
    let blocked = true
    const off = commands.attachKeyboard(document, { isBlocked: () => blocked })
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyK', ctrlKey: true, bubbles: true }))
    expect(palette.paletteIsOpen.value).toBe(false)
    blocked = false
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyK', ctrlKey: true, bubbles: true }))
    expect(palette.paletteIsOpen.value).toBe(true)
    off()
  })
})
