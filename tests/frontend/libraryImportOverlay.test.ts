import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const CREATED = {
  meta: {
    meta_schema_version: 1,
    work_id: 'w1',
    name: 'Ten',
    source_lang: 'zh',
    genre: '',
    created_at: '',
    updated_at: '',
    chapter_count: 1,
  },
  folder: '/tmp/Ten.atproj',
  images_saved: 0,
  images_failed: 0,
  source_lang_mismatch: false,
}

const NO_WORK = { code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }

let pages: VueWrapper[] = []

async function setup() {
  vi.resetModules()
  mockInvoke.mockReset()
  mockInvoke.mockImplementation((cmd: string) =>
    cmd === 'library_list_works'
      ? Promise.resolve({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
      : Promise.reject(NO_WORK),
  )
  const commands = await import('../../src/commands')
  const overlay = await import('../../src/modes/libraryImportOverlayState')
  const modeState = await import('../../src/modes/modeState')
  const focusMod = await import('../../src/modes/libraryImportFocus')
  const imp = await import('../../src/modes/libraryImport')
  const keymap = commands.installCommands({
    setMode: modeState.setMode,
    isMac: false,
    openLibraryImportForm: focusMod.openLibraryImportForm,
    closeLibraryImportForm: overlay.closeLibraryImportOverlay,
  } as CommandDeps)
  const Overlay = (await import('../../src/LibraryImportOverlay.vue')).default
  const Library = (await import('../../src/modes/LibraryMode.vue')).default
  return { commands, overlay, modeState, imp, keymap, Overlay, Library }
}

async function flush(): Promise<void> {
  for (let i = 0; i < 6; i += 1) await Promise.resolve()
  await new Promise((resolve) => setTimeout(resolve, 0))
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  for (const page of pages) page.unmount()
  pages = []
  document.body.innerHTML = ''
})

describe('Library import overlay', { timeout: 30_000 }, () => {
  it('the Mod+N chord is bound to library.import.open', async () => {
    const { keymap } = await setup()
    const binding = keymap.bindings().find((b) => b.chord === 'Mod+N')
    expect(binding?.id).toBe('library.import.open')
  })

  it('library.import.open from Workspace switches to Library and opens the overlay', async () => {
    const { commands, overlay, modeState, Overlay } = await setup()
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)
    modeState.setMode('workspace')
    expect(wrapper.find('.import-form').exists()).toBe(false)

    commands.dispatch('library.import.open')
    await flush()

    expect(modeState.currentMode.value).toBe('library')
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
    expect(wrapper.find('.li-panel .import-form').exists()).toBe(true)
  })

  it('each empty-state invitation opens the overlay', async () => {
    const { overlay, Library } = await setup()
    const page = mount(Library, { attachTo: document.body })
    pages.push(page)
    const state = await import('../../src/modes/libraryWorks')
    await state.loadWorks()
    await flush()
    const invites = page.findAll('[data-library-empty-invite]')
    expect(invites).toHaveLength(2)
    for (const invite of invites) {
      overlay.resetLibraryImportOverlay()
      await invite.trigger('click')
      expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
    }
  })

  it('opening focuses the first field; Escape closes and focus returns to the opener button', async () => {
    const { commands, overlay, Overlay } = await setup()
    document.body.innerHTML = '<button data-library-import-open id="opener"></button>'
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)
    await flush()
    const opener = { element: document.getElementById('opener') as HTMLButtonElement }
    opener.element.focus()

    commands.dispatch('library.import.open')
    await flush()
    const first = wrapper.find('.import-form input[type="text"], .import-form textarea')
    expect(first.exists()).toBe(true)
    expect(wrapper.find('.li-panel').element.contains(document.activeElement)).toBe(true)

    await wrapper.find('.li-scrim').trigger('keydown', { key: 'Escape' })
    await flush()
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(false)
    expect(wrapper.find('.li-panel').exists()).toBe(false)
    expect(document.activeElement).toBe(opener.element)
  })

  it('the genre field is a select menu listing "Chưa chọn" plus 29 genres, not a text box', async () => {
    const { commands, Overlay } = await setup()
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)
    commands.dispatch('library.import.open')
    await flush()
    const field = wrapper
      .findAll('.import-form .field')
      .find((f) => f.find('span').text() === 'Thể loại')
    expect(field).toBeDefined()
    expect(field!.find('input[type="text"]').exists()).toBe(false)
    const labels = field!.findAll('.select-item').map((n) => n.text())
    expect(labels).toHaveLength(30)
    expect(labels[0]).toBe('Chưa chọn')
    expect(labels[1]).toBe('Thời sự')
    expect(field!.find('.select-options').classes()).toContain('select-options--cols-3')
  })

  it('source language is a row of chips; clicking one sets sourceLang and marks only that chip', async () => {
    const { commands, imp, Overlay } = await setup()
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)
    commands.dispatch('library.import.open')
    await flush()
    const group = wrapper.find('[data-import-source-lang]')
    expect(group.attributes('role')).toBe('radiogroup')
    expect(group.attributes('aria-label')).toBe('Ngôn ngữ nguồn')
    const chips = group.findAll('.lang-chip')
    expect(chips.map((c) => c.text())).toEqual(['Tiếng Trung', 'Tiếng Anh'])

    await chips[1]!.find('input').setValue(true)
    expect(imp.sourceLang.value).toBe('en')
    expect(group.findAll('.lang-chip--on').map((c) => c.text())).toEqual(['Tiếng Anh'])
  })

  it('Tab from the checked language chip skips the other chip and the radios of the closed genre menu', async () => {
    const { commands, Overlay } = await setup()
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)
    commands.dispatch('library.import.open')
    await flush()
    const checked = wrapper.find<HTMLInputElement>('[data-import-source-lang] input:checked')
    expect(checked.element.value).toBe('zh')
    checked.element.focus()

    await wrapper.find('.li-panel').trigger('keydown', { key: 'Tab' })
    const genreField = wrapper.findAll('.import-form .field').find((f) => f.find('span').text() === 'Thể loại')!
    expect(document.activeElement).toBe(genreField.find('summary').element)

    await wrapper.find('.li-panel').trigger('keydown', { key: 'Tab' })
    expect(document.activeElement?.tagName).toBe('TEXTAREA')
  })

  it('focus falls back to [data-library-import-open] when the opener node was replaced', async () => {
    const { commands, overlay, Overlay } = await setup()
    document.body.innerHTML = '<button data-library-import-open id="a"></button>'
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)
    document.getElementById('a')?.focus()
    commands.dispatch('library.import.open')
    await flush()
    document.body.insertAdjacentHTML('beforeend', '<button data-library-import-open id="b"></button>')
    document.getElementById('a')?.remove()
    overlay.closeLibraryImportOverlay()
    await flush()
    expect(document.activeElement?.id).toBe('b')
  })

  it('finishImportSubmission closes the overlay on success and leaves it open, showing the error, on failure', async () => {
    const { overlay, imp, Overlay } = await setup()
    const wrapper = mount(Overlay, { attachTo: document.body })
    pages.push(wrapper)

    overlay.openLibraryImportOverlay()
    await flush()
    imp.finishImportSubmission(null, {
      code: 'work.none_open',
      message_key: 'err.work.none_open',
      params: {},
      retryable: false,
    })
    await flush()
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
    expect(wrapper.find('.li-panel .error').text()).not.toBe('')

    imp.finishImportSubmission(CREATED, null)
    await flush()
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(false)
    expect(wrapper.find('.li-panel').exists()).toBe(false)
  })
})
