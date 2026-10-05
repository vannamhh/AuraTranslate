import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterSegment } from '../../src/config/segment'
import vi_json from '../../src/i18n/vi.json'

const SEGMENTS: ChapterSegment[] = [1, 2].map((id) => ({
  id,
  ord: id,
  source_text: `Cau ${id}.`,
  target_text: `Dich ${id}.`,
  is_paragraph_end: false,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: false,
  role: null,
  translation_origin: id === 2 ? 'self' : '',
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
          tm_filled_segment_ids: [2],
          tm_prefill: { kind: 'ran' },
        },
        error: null,
      }),
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('GridPanel.vue — segment TM điền sẵn', () => {
  it('hàng được Rust báo điền sẵn mang `rule-tm-rule` và nhãn trạng thái TM; hàng kia là `rule-draft`', async () => {
    vi.resetModules()
    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const state = await import('../../src/panels/editorPanelState')
    const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
    await state.ensureSegmentsLoaded()
    await wrapper.vm.$nextTick()

    const root = wrapper.element as unknown as Element
    const rules = root.querySelectorAll('.col-rule .rule')
    expect(rules).toHaveLength(2)
    expect(rules[0].classList.contains('rule-draft')).toBe(true)
    expect(rules[1].classList.contains('rule-tm-rule')).toBe(true)

    const states = root.querySelectorAll('.col-state > *')
    expect(states[1].textContent).toContain((vi_json as Record<string, string>)['panel.grid.state_tm'])
  })
})
