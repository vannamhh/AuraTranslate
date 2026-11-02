import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { KeepAlive, defineComponent, h } from 'vue'
import type { VueWrapper } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const NO_WORK = { code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }

function row(over: Record<string, unknown>) {
  return {
    work_id: 'id',
    atproj_path: '/tmp/A.atproj',
    name: 'A',
    source_lang: 'zh',
    genre: 'Tiên hiệp',
    created_at: '2026-08-01T00:00:00.000Z',
    updated_at: '2026-08-01T00:00:00.000Z',
    chapter_count: 1,
    status: 'not_started',
    status_is_override: false,
    chapter_done_count: 0,
    ...over,
  }
}

const LIBRARY_STYLE = readFileSync(join(__dirname, '../../src/modes/LibraryMode.vue'), 'utf8').split('<style scoped>')[1] ?? ''

let page: VueWrapper | null = null

async function mountLibrary(works: unknown[], deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  mockInvoke.mockReset()
  mockInvoke.mockImplementation((cmd: string) =>
    cmd === 'library_list_works'
      ? Promise.resolve({ total: works.length, matched: works.length, works, genres: [], source_langs: [] })
      : Promise.reject(NO_WORK),
  )
  const commands = await import('../../src/commands')
  commands.installCommands({ isMac: false, ...deps } as CommandDeps)
  const Library = (await import('../../src/modes/LibraryMode.vue')).default
  const state = await import('../../src/modes/libraryWorks')
  page = mount(Library, { attachTo: document.body })
  await state.loadWorks()
  await page.vm.$nextTick()
  return page
}

afterEach(() => {
  page?.unmount()
  page = null
  document.body.innerHTML = ''
})

describe('Library top bar and cards', { timeout: 30_000 }, () => {
  it('has no Tìm button and no search label; Enter in the field runs library.search', async () => {
    const runLibrarySearch = vi.fn()
    const p = await mountLibrary([row({})], { runLibrarySearch })
    expect(p.find('[data-library-search-button]').exists()).toBe(false)
    expect(p.find('label[for], .search-block label').exists()).toBe(false)
    await p.find('[data-library-search-input]').trigger('keyup', { key: 'Enter' })
    expect(runLibrarySearch).toHaveBeenCalledTimes(1)
  })

  it('the search field carries the shared svg icon, not a text glyph sitting on the baseline', async () => {
    const p = await mountLibrary([row({})])
    const icon = p.find('.search-field svg.search-icon')
    expect(icon.exists()).toBe(true)
    expect(icon.attributes('aria-hidden')).toBe('true')
    expect(p.find('.search-field').text()).toBe('')
  })

  it('Mod+Alt+F belongs to library.focus_search, which dispatches its port; library.search has no default chord', async () => {
    const focusLibrarySearch = vi.fn()
    await mountLibrary([row({})], { focusLibrarySearch })
    const commands = await import('../../src/commands')
    expect(commands.defaultChordsFor('library.focus_search')).toEqual(['Mod+Alt+F'])
    expect(commands.defaultChordsFor('library.search')).toEqual([])
    commands.dispatch('library.focus_search')
    expect(focusLibrarySearch).toHaveBeenCalledTimes(1)
  })

  it('in Library, a search-focus request moves the caret from the card into the field with the old text selected', async () => {
    const p = await mountLibrary([row({})])
    const search = await import('../../src/modes/librarySearch')
    search.librarySearchQuery.value = 'tu tiên'
    await p.vm.$nextTick()
    p.find<HTMLElement>('[data-library-work-cell]').element.focus()
    search.requestLibrarySearchFocus()
    await p.vm.$nextTick()
    await p.vm.$nextTick()
    const input = p.find<HTMLInputElement>('[data-library-search-input]').element
    expect(document.activeElement).toBe(input)
    expect([input.selectionStart, input.selectionEnd]).toEqual([0, 'tu tiên'.length])
  })

  it('with a Work page open, the request closes it and the caret lands in the field, not on the card the close focuses', async () => {
    const p = await mountLibrary([row({})])
    const listed = mockInvoke.getMockImplementation()
    mockInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === 'open_work') return Promise.resolve({ meta: { name: 'A' }, folder: '/tmp/A', chapter_id: 1 })
      if (cmd === 'list_chapters') return Promise.resolve([])
      return listed?.(cmd, args)
    })
    const chapters = await import('../../src/modes/libraryChapters')
    await chapters.openCurrentLibraryWork()
    await flushPromises()
    expect(chapters.libraryWorkPageIsOpen.value).toBe(true)
    const search = await import('../../src/modes/librarySearch')
    search.requestLibrarySearchFocus()
    await flushPromises()
    expect(chapters.libraryWorkPageIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(p.find('[data-library-search-input]').element)
  })

  it('from another mode, the request switches to Library and the caret lands in the field, not on the card the activation focuses', async () => {
    await mountLibrary([row({})])
    page?.unmount()
    const { currentMode, setMode } = await import('../../src/modes/modeState')
    const Library = (await import('../../src/modes/LibraryMode.vue')).default
    const search = await import('../../src/modes/librarySearch')
    const Shell = defineComponent(() => () => h(KeepAlive, null, [currentMode.value === 'library' ? h(Library) : null]))
    setMode('library')
    page = mount(Shell, { attachTo: document.body })
    await flushPromises()
    setMode('workspace')
    await flushPromises()
    search.requestLibrarySearchFocus()
    await flushPromises()
    expect(currentMode.value).toBe('library')
    expect(document.activeElement).toBe(page.find('[data-library-search-input]').element)
  })

  it('the top bar has no diacritic-mode segments', async () => {
    const p = await mountLibrary([row({})])
    expect(p.find('[data-library-search-mode]').exists()).toBe(false)
    expect(p.text()).not.toContain('Chính xác dấu')
  })

  it('the Library body has no import form and no Nhập tài liệu button (the button lives in the title bar)', async () => {
    const p = await mountLibrary([row({})])
    expect(p.find('.import-form').exists()).toBe(false)
    expect(p.find('[data-library-import-open]').exists()).toBe(false)
  })

  it('a card shows no source-language badge, only the genre · edit line', async () => {
    const p = await mountLibrary([row({ updated_at: new Date(Date.now() - 3 * 3600_000).toISOString() })])
    expect(p.find('.work-cell').text()).not.toContain('Tiếng Trung')
    expect(p.find('[data-library-work-meta]').text()).toBe('Tiên hiệp · sửa 3 giờ trước')
  })

  it('a card with no genre and an unreadable date has no meta line and no stray text', async () => {
    const p = await mountLibrary([row({ genre: '', updated_at: 'hong', source_lang: '' })])
    const cell = p.find('.work-cell')
    expect(cell.find('[data-library-work-meta]').text()).toBe('')
    expect(cell.text()).not.toMatch(/Invalid|undefined|NaN/)
  })

  it('two selected status filter items are pressed ui-menu-items under a "Trạng thái (2)" field, tinted surface-accent', async () => {
    const p = await mountLibrary([row({})])
    const works = await import('../../src/modes/libraryWorks')
    works.toggleStatusFilter('done')
    works.toggleStatusFilter('not_started')
    await p.vm.$nextTick()
    for (const status of ['done', 'not_started']) {
      const item = p.find(`[data-lifecycle-filter="${status}"]`)
      expect(item.classes()).toContain('ui-menu-item')
      expect(item.attributes('aria-pressed')).toBe('true')
    }
    expect(p.find('[data-lifecycle-filter="paused"]').attributes('aria-pressed')).toBe('false')
    const trigger = p.find('[data-library-chip="status"] summary')
    expect(trigger.classes()).toContain('ui-field')
    expect(trigger.text()).toContain('Trạng thái (2)')
    const pressedRule = LIBRARY_STYLE.match(/\.menu-item\[aria-pressed='true'\]\s*\{([^}]*)\}/)?.[1] ?? ''
    expect(pressedRule).toContain('var(--color-surface-accent)')
    expect(pressedRule).not.toContain('primary')
  })

  it('the chosen search scope radio stays checked inside a ui-seg__item label', async () => {
    const p = await mountLibrary([row({})])
    const search = await import('../../src/modes/librarySearch')
    search.librarySearchQuery.value = 'khoang'
    search.setLibrarySearchScope('target')
    await p.vm.$nextTick()
    const radio = p.find<HTMLInputElement>('[data-library-search-scope="target"]')
    expect(radio.element.checked).toBe(true)
    expect(radio.element.closest('label')?.classList.contains('ui-seg__item')).toBe(true)
    expect(p.find<HTMLInputElement>('[data-library-search-scope="both"]').element.checked).toBe(false)
    const names = p.findAll<HTMLInputElement>('[data-library-search-scope]').map((r) => r.element.name)
    expect(new Set(names)).toEqual(new Set(['library-search-scope']))
    expect(radio.classes()).toContain('lib-sr-only')
    const hiddenRule = LIBRARY_STYLE.match(/\.lib-sr-only\s*\{([^}]*)\}/)?.[1] ?? ''
    expect(hiddenRule).toContain('clip: rect(0, 0, 0, 0)')
    expect(hiddenRule).not.toMatch(/display:\s*none|visibility:\s*hidden/)
  })
})
