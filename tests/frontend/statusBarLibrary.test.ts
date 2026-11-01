import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { computed, ref } from 'vue'
import type { NavNotice } from '../../src/panels/editorPanelState'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const flushResult: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }
const failSave = { value: false }
const forcedNavNotice = ref<NavNotice | null>(null)
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return {
    ...actual,
    flushEditorBeforeDiscreteWrite: async () => flushResult.value,
    editorNavNotice: computed(() => forcedNavNotice.value ?? actual.editorNavNotice.value),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/segment')>()),
  readOpenChapterSegments: async () => ({
    loaded: {
      chapter_id: 1,
      segments: FIXTURE_SEGMENTS.map((row) => ({ ...row })),
      caret_segment_id: FIXTURE_SEGMENTS[0].id,
      assets: [],
      assets_dir: '',
      tm_filled_segment_ids: [],
      tm_prefill: { kind: 'ran', hidden_count: 0 },
    },
    error: null,
  }),
  saveSegmentTargets: async (chapterId: number) => {
    if (failSave.value) {
      return { outcome: null, error: { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: true } }
    }
    return { outcome: { chapter_id: chapterId, saved: 1 }, error: null }
  },
}))

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
  failSave.value = false
  forcedNavNotice.value = null
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: 1, matched: 1, works: [WORK_ROW], genres: [], source_langs: [] })
    }
    return Promise.reject(OPEN_ERROR)
  })
  // `vi.resetModules` keeps a `vi.mock` factory's result, so this module outlives each case.
  ;(await import('../../src/panels/editorPanelState')).resetEditorPanel()
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

  it('shows the short "created" line with the folder, and an error takes its place', async () => {
    const { works, chapters, wrapper } = await setup('library')
    const imp = await import('../../src/modes/libraryImport')
    imp.createdWork.value = {
      meta: {
        meta_schema_version: 1,
        work_id: 'id-b',
        name: 'test 1',
        source_lang: 'zh',
        genre: '',
        created_at: '2026-10-25T00:00:00.000Z',
        updated_at: '2026-10-25T00:00:00.000Z',
        chapter_count: 1,
      },
      folder: '/Users/x/Documents/AuraTranslate/test 1.atproj',
      images_saved: 0,
      images_failed: 0,
      source_lang_mismatch: false,
    }
    await wrapper.vm.$nextTick()
    const created = wrapper.find('[data-status-library-created]')
    expect(created.text()).toBe('Đã tạo Tác phẩm "test 1" — /Users/x/Documents/AuraTranslate/test 1.atproj')
    expect(created.attributes('title')).toBe(created.text())

    await works.loadWorks()
    expect(await chapters.openWorkById('id-a')).toBe(false)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-error]').exists()).toBe(true)
    expect(wrapper.find('[data-status-library-created]').exists()).toBe(false)
    imp.createdWork.value = null
    wrapper.unmount()
  })

  it('shows the flush-failed chapter-open notice in Library', async () => {
    const { works, t, wrapper } = await setup('library')
    await works.loadWorks()
    const editor = await import('../../src/panels/editorPanelState')
    await editor.ensureSegmentsLoaded()
    editor.setEditorCaret(11)
    editor.noteEditorEdit(11, 'dang go')
    failSave.value = true
    expect(await editor.openChapterById(2)).toBe(false)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-nav-notice]').text()).toBe(t('panel.grid.nav_chapter_flush_failed'))
    failSave.value = false
    await editor.flushEditorNow()
    wrapper.unmount()
  })

  it('shows the switch-failed notice in Library when open_chapter errors', async () => {
    const { works, t, wrapper } = await setup('library')
    await works.loadWorks()
    const editor = await import('../../src/panels/editorPanelState')
    expect(await editor.openChapterById(2)).toBe(false)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-nav-notice]').text()).toBe(t('panel.grid.nav_chapter_switch_failed'))
    wrapper.unmount()
  })

  it('shows the still-dirty notice in Library', async () => {
    const { t, wrapper } = await setup('library')
    forcedNavNotice.value = 'chapter-still-dirty'
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-nav-notice]').text()).toBe(t('panel.grid.nav_chapter_still_dirty'))
    wrapper.unmount()
  })

  it('keeps a Workspace navigation notice out of Library', async () => {
    const { wrapper } = await setup('library')
    forcedNavNotice.value = 'at-last'
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-status-library-nav-notice]').exists()).toBe(false)
    wrapper.unmount()
  })
})
