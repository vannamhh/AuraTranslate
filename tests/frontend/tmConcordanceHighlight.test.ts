/** The source column highlights exactly the mapped span of the active segment. */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { shallowRef } from 'vue'
import type { GlossaryMark } from '../../src/config/glossary'
import type { ChapterSegment } from '../../src/config/segment'

const SEGMENTS: ChapterSegment[] = [1, 2].map((id) => ({
  id,
  ord: id,
  source_text: '他叫师父来了。',
  target_text: '',
  is_paragraph_end: false,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: false,
  role: null,
  translation_origin: '',
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: () =>
      Promise.resolve({
        loaded: {
          chapter_id: 7,
          segments: SEGMENTS,
          caret_segment_id: null,
          assets: [],
          assets_dir: '',
          tm_filled_segment_ids: [],
        },
        error: null,
      }),
  }
})

const marks = shallowRef<GlossaryMark[]>([
  {
    start: 10,
    end: 12,
    tier: 'work',
    is_confirmed: true,
    translation: 'Sư phụ',
    id: 1,
    source_term: '师父',
    han_viet_suggestion: null,
    han_viet_status: 'not_requested',
    occurrence_count: null,
  },
])

vi.mock('../../src/panels/glossaryMarksState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/glossaryMarksState')>()
  return { ...actual, glossaryMarks: marks }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('GridPanel.vue — vùng nguồn của Concordance', () => {
  it('marks only the pieces inside the span, in the segment it names, and clears with the state', async () => {
    vi.resetModules()
    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const editor = await import('../../src/panels/editorPanelState')
    const concordance = await import('../../src/panels/concordanceState')
    const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
    await editor.ensureSegmentsLoaded()
    await wrapper.vm.$nextTick()

    const root = wrapper.element as unknown as Element
    expect(root.querySelectorAll('.src-piece.concordance-source')).toHaveLength(0)

    concordance.setConcordanceSourceHighlight({ segmentId: 2, start: 2, end: 4 })
    await wrapper.vm.$nextTick()
    const lit = [...root.querySelectorAll('.src-piece.concordance-source')]
    expect(lit.map((el) => el.textContent)).toEqual(['师父'])
    expect(lit[0]?.closest('.cell')?.textContent).toContain('他叫师父来了。')

    concordance.resetConcordance()
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.src-piece.concordance-source')).toHaveLength(0)
    wrapper.unmount()
  })
})
