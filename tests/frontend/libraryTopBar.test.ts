import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
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

  it('the two segments read Chính xác dấu | Không dấu and each dispatches its own mode command', async () => {
    const setLibrarySearchModeExact = vi.fn()
    const setLibrarySearchModeLenient = vi.fn()
    const p = await mountLibrary([row({})], { setLibrarySearchModeExact, setLibrarySearchModeLenient })
    const exact = p.find('[data-library-search-mode="exact"]')
    const lenient = p.find('[data-library-search-mode="lenient"]')
    expect(exact.text()).toBe('Chính xác dấu')
    expect(lenient.text()).toBe('Không dấu')
    await lenient.trigger('click')
    expect(setLibrarySearchModeLenient).toHaveBeenCalledTimes(1)
    expect(setLibrarySearchModeExact).not.toHaveBeenCalled()
    await exact.trigger('click')
    expect(setLibrarySearchModeExact).toHaveBeenCalledTimes(1)
  })

  it('Nhập tài liệu dispatches library.import.open exactly once and the page has no import form', async () => {
    const openLibraryImportForm = vi.fn()
    const p = await mountLibrary([row({})], { openLibraryImportForm })
    expect(p.find('.import-form').exists()).toBe(false)
    const open = p.find('[data-library-import-open]')
    expect(open.text()).toBe('Nhập tài liệu')
    await open.trigger('click')
    expect(openLibraryImportForm).toHaveBeenCalledTimes(1)
  })

  it('a card shows the source language on the cover and the genre · edit line', async () => {
    const p = await mountLibrary([row({ updated_at: new Date(Date.now() - 3 * 3600_000).toISOString() })])
    expect(p.find('.work-cover [data-library-work-lang]').text()).toBe('Tiếng Trung')
    expect(p.find('[data-library-work-meta]').text()).toBe('Tiên hiệp · sửa 3 giờ trước')
  })

  it('a card with no genre and an unreadable date has no meta line and no stray text', async () => {
    const p = await mountLibrary([row({ genre: '', updated_at: 'hong', source_lang: '' })])
    const cell = p.find('.work-cell')
    expect(cell.find('[data-library-work-meta]').exists()).toBe(false)
    expect(cell.find('[data-library-work-lang]').exists()).toBe(false)
    expect(cell.text()).not.toMatch(/Invalid|undefined|NaN/)
  })
})
