import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { SEED_GENRES } from './support/genreSeed'

const genreListMock = vi.fn()
const genreCreateMock = vi.fn()
const genreRenameMock = vi.fn()
const genreDeleteMock = vi.fn()

vi.mock('../../src/config/genre', () => ({
  genreList: (...args: unknown[]) => genreListMock(...args),
  genreCreate: (...args: unknown[]) => genreCreateMock(...args),
  genreRename: (...args: unknown[]) => genreRenameMock(...args),
  genreDelete: (...args: unknown[]) => genreDeleteMock(...args),
}))

type Wire = { id: string; name: string }

async function openSection(rows: Wire[] = [...SEED_GENRES]) {
  vi.resetModules()
  document.body.innerHTML = ''
  for (const m of [genreListMock, genreCreateMock, genreRenameMock, genreDeleteMock]) m.mockReset()
  genreListMock.mockResolvedValue({ genres: rows, error: null })
  genreRenameMock.mockResolvedValue(null)
  genreDeleteMock.mockResolvedValue(null)
  const commands = await import('../../src/commands')
  const state = await import('../../src/genreState')
  const Section = (await import('../../src/SettingsGenreSection.vue')).default
  const i18n = await import('../../src/i18n')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    addGenre: () => {
      void state.addGenre()
    },
    renameGenre: () => {
      void state.renameSelectedGenre()
    },
    deleteGenre: () => {
      void state.deleteSelectedGenre()
    },
  })
  const wrapper = mount(Section, { attachTo: document.body })
  await flushPromises()
  return { wrapper, state, i18n }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('SettingsGenreSection.vue', () => {
  it('lists the 29 seeded genres in menu order', async () => {
    const { wrapper } = await openSection()
    const rows = wrapper.findAll('[data-genre-row]')
    expect(rows).toHaveLength(29)
    expect(rows[0]!.attributes('data-genre-row')).toBe('news')
    expect((rows[0]!.get('[data-genre-name]').element as HTMLInputElement).value).toBe('Thời sự')
    expect(rows[28]!.attributes('data-genre-row')).toBe('other')
  })

  it('adds a genre by typing a name and submitting, then the list is reloaded', async () => {
    const { wrapper } = await openSection()
    genreCreateMock.mockResolvedValue({ genre: { id: 'g30', name: 'Truyện ngắn' }, error: null })
    genreListMock.mockResolvedValue({ genres: [...SEED_GENRES, { id: 'g30', name: 'Truyện ngắn' }], error: null })
    await wrapper.get('[data-genre-new-name]').setValue('Truyện ngắn')
    await wrapper.get('form.sg-add').trigger('submit')
    await flushPromises()
    expect(genreCreateMock).toHaveBeenCalledWith('Truyện ngắn')
    const rows = wrapper.findAll('[data-genre-row]')
    expect(rows).toHaveLength(30)
    expect(rows[29]!.attributes('data-genre-row')).toBe('g30')
    expect((wrapper.get('[data-genre-new-name]').element as HTMLInputElement).value).toBe('')
  })

  it('shows the Rust error for a duplicate name and keeps the draft', async () => {
    const { wrapper, i18n } = await openSection()
    const err = { code: 'genre.name_taken', message_key: 'err.genre.name_taken', params: { name: 'tiên hiệp' }, retryable: false }
    genreCreateMock.mockResolvedValue({ genre: null, error: err })
    await wrapper.get('[data-genre-new-name]').setValue('tiên hiệp')
    await wrapper.get('form.sg-add').trigger('submit')
    await flushPromises()
    expect(wrapper.get('[data-genre-error]').text()).toBe(i18n.tError(err))
    expect((wrapper.get('[data-genre-new-name]').element as HTMLInputElement).value).toBe('tiên hiệp')
  })

  it('renames the row being edited with the keyboard form submit', async () => {
    const { wrapper } = await openSection()
    const row = wrapper.get('[data-genre-row="news"]')
    await row.get('[data-genre-name]').setValue('Tin tức')
    genreListMock.mockResolvedValue({
      genres: SEED_GENRES.map((g) => (g.id === 'news' ? { id: 'news', name: 'Tin tức' } : g)),
      error: null,
    })
    await row.get('form').trigger('submit')
    await flushPromises()
    expect(genreRenameMock).toHaveBeenCalledWith('news', 'Tin tức')
    expect(wrapper.get('[data-genre-row="news"] [data-genre-name]').element).toHaveProperty('value', 'Tin tức')
  })

  it('deletes the clicked row and a blocked delete shows the Work count', async () => {
    const { wrapper, i18n } = await openSection()
    const err = { code: 'genre.in_use', message_key: 'err.genre.in_use', params: { count: '2' }, retryable: false }
    genreDeleteMock.mockResolvedValueOnce(err)
    const news = wrapper.get('[data-genre-row="news"]')
    await news.trigger('mousedown')
    await news.get('[data-genre-delete]').trigger('click')
    await flushPromises()
    expect(genreDeleteMock).toHaveBeenCalledWith('news')
    expect(wrapper.get('[data-genre-error]').text()).toBe(i18n.tError(err))
    expect(wrapper.get('[data-genre-error]').text()).toContain('2')
    expect(wrapper.findAll('[data-genre-row]')).toHaveLength(29)

    genreListMock.mockResolvedValue({ genres: SEED_GENRES.filter((g) => g.id !== 'wuxia'), error: null })
    const wuxia = wrapper.get('[data-genre-row="wuxia"]')
    await wuxia.trigger('mousedown')
    await wuxia.get('[data-genre-delete]').trigger('click')
    await flushPromises()
    expect(genreDeleteMock).toHaveBeenLastCalledWith('wuxia')
    expect(wrapper.find('[data-genre-row="wuxia"]').exists()).toBe(false)
  })

  it('focusing a row delete button selects that row, so the click deletes it without a mousedown', async () => {
    const { wrapper } = await openSection()
    genreListMock.mockResolvedValue({ genres: SEED_GENRES.filter((g) => g.id !== 'tcm'), error: null })
    const button = wrapper.get('[data-genre-row="tcm"] [data-genre-delete]')
    await wrapper.get('[data-genre-row="tcm"]').trigger('focusin')
    await button.trigger('click')
    await flushPromises()
    expect(genreDeleteMock).toHaveBeenCalledWith('tcm')
  })

  it('a rename rejected by Rust shows the error and the row keeps the typed draft', async () => {
    const { wrapper, i18n } = await openSection()
    const err = { code: 'genre.name_taken', message_key: 'err.genre.name_taken', params: { name: 'Đô thị' }, retryable: false }
    genreRenameMock.mockResolvedValue(err)
    const row = wrapper.get('[data-genre-row="news"]')
    await row.get('[data-genre-name]').setValue('đô thị')
    await row.get('form').trigger('submit')
    await flushPromises()
    expect(wrapper.get('[data-genre-error]').text()).toBe(i18n.tError(err))
    expect((wrapper.get('[data-genre-row="news"] [data-genre-name]').element as HTMLInputElement).value).toBe('đô thị')
  })
})
