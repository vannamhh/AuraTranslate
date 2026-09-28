/**
 * `GridPanel.vue` — dải tab Hán Việt: mũi tên phải dời tiêu điểm DOM tới tab vừa thành
 * `activeTab`, không chỉ đổi `tabindex` roving. Khuôn mount THẬT `gridPanelImages.test.ts`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterSegment } from '../../src/config/segment'
import type { OpenChapter } from '../../src/config/chapter'

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke khong duoc goi truc tiep trong ca nay')),
}))

vi.mock('../../src/commands', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/commands')>()
  return { ...actual, dispatch: vi.fn() }
})

const SEGMENTS: ChapterSegment[] = [
  {
    id: 1,
    ord: 1,
    source_text: 'Cau mot.',
    target_text: 'Dich mot.',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: null,
    translation_origin: '',
  },
]

const CHAPTER: OpenChapter = { chapter_id: 7, source_text: '', source_lang: 'zh' }

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: () =>
      Promise.resolve({
        loaded: { chapter_id: 7, segments: SEGMENTS, caret_segment_id: null, assets: [], assets_dir: '' },
        error: null,
      }),
  }
})

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, readOpenChapter: () => Promise.resolve({ chapter: CHAPTER, error: null }) }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

async function mountGrid() {
  vi.resetModules()
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const sourcePanel = await import('../../src/panels/sourcePanelState')
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await sourcePanel.ensureChapterLoaded()
  await wrapper.vm.$nextTick()
  return wrapper
}

function rootOf(wrapper: ReturnType<typeof mount>): Element {
  return wrapper.element as unknown as Element
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('GridPanel.vue — dải tab Hán Việt, mũi tên dời tiêu điểm DOM', () => {
  it('mũi tên phải trên tab "nguyên văn" dời tiêu điểm DOM sang nút "Hán Việt"', async () => {
    const wrapper = await mountGrid()

    const original = rootOf(wrapper).querySelector<HTMLButtonElement>('#grid-tab-original')
    const hanViet = rootOf(wrapper).querySelector<HTMLButtonElement>('#grid-tab-han-viet')
    expect(original, 'dải tab phải hiện khi nguồn là zh và có segment').not.toBeNull()
    expect(hanViet).not.toBeNull()

    original?.focus()
    expect(document.activeElement).toBe(original)

    await original?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, cancelable: true }))
    await wrapper.vm.$nextTick()

    expect(
      document.activeElement,
      'tiêu điểm DOM phải theo tab vừa được chọn, không ở lại nút cũ mang tabindex="-1"',
    ).toBe(hanViet)
  })
})
