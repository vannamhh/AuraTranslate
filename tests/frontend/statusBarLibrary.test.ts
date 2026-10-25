import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const flushResult: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => flushResult.value }
})

const WORK_ROW = {
  work_id: 'id-a',
  atproj_path: '/tmp/A.atproj',
  name: 'A',
  source_lang: 'zh',
  genre: '',
  created_at: '2026-08-01T00:00:00.000Z',
  updated_at: '2026-08-01T00:00:00.000Z',
  chapter_count: 1,
  status: 'not_started',
  status_is_override: false,
  chapter_done_count: 0,
}

const OPEN_ERROR = {
  code: 'work.none_open',
  message_key: 'err.work.none_open',
  params: {},
  retryable: false,
}

async function setup(mode: 'library' | 'workspace') {
  vi.resetModules()
  mockInvoke.mockReset()
  flushResult.value = 'clean'
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: 1, matched: 1, works: [WORK_ROW], genres: [], source_langs: [] })
    }
    return Promise.reject(OPEN_ERROR)
  })
  const modes = await import('../../src/modes/modeState')
  modes.setMode(mode)
  const works = await import('../../src/modes/libraryWorks')
  const chapters = await import('../../src/modes/libraryChapters')
  const { t } = await import('../../src/i18n')
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  const wrapper = mount(StatusBar, { attachTo: document.body })
  return { works, chapters, t, wrapper }
}

describe('StatusBar — Library branch', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('shows the Works count and no Workspace items once Works are loaded', async () => {
    const { works, t, wrapper } = await setup('library')
    expect(wrapper.find('[data-status-library-works]').exists()).toBe(false)
    await works.loadWorks()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-works]').text()).toBe(t('status.library_works', { count: '1' }))
    expect(wrapper.find('[data-status-position]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the Workspace items and not the Library group in Workspace mode', async () => {
    const { works, wrapper } = await setup('workspace')
    await works.loadWorks()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-works]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the text of an open-work IPC error', async () => {
    const { works, chapters, wrapper } = await setup('library')
    await works.loadWorks()
    expect(await chapters.openWorkById('id-a')).toBe(false)
    await wrapper.vm.$nextTick()
    const error = wrapper.find('[data-status-library-error]')
    expect(error.exists()).toBe(true)
    expect(error.text()).not.toBe('')
    wrapper.unmount()
  })

  it('shows the still-dirty notice when the flush blocks the open', async () => {
    const { works, chapters, t, wrapper } = await setup('library')
    await works.loadWorks()
    flushResult.value = 'still-dirty'
    expect(await chapters.openWorkById('id-a')).toBe(false)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-error]').text()).toBe(t('mode.library.open_work_still_dirty'))
    wrapper.unmount()
  })
})
