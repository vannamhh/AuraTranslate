import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'

const anchor = ref<number | null>(null)

const SEGMENTS: ChapterSegment[] = [1, 2, 3].map((id) => ({
  id,
  ord: id,
  source_text: `Cau ${id}.`,
  target_text: `Dich ${id}.`,
  is_paragraph_end: false,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  unlinked: false,
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
          tm_prefill: { kind: 'ran', hidden_count: 0 },
        },
        error: null,
      }),
  }
})

const STUBS = {
  PanelFrame: { template: '<div class="panel-frame"><slot /></div>' },
  TmFuzzyStrip: { template: '<section class="stub-strip" tabindex="-1"><button class="stub-btn">x</button></section>' },
  GlossaryConfirmStrip: { template: '<i />' },
  ProofreaderStrip: { template: '<i />' },
}

beforeEach(() => {
  document.body.innerHTML = ''
  anchor.value = null
})

async function mountGrid() {
  vi.resetModules()
  vi.doMock('../../src/inlineStripEligibility', () => ({ inlineStripAnchorSegmentId: anchor }))
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { wrapper, state, root: wrapper.element as unknown as Element }
}

describe('GridPanel strip track', () => {
  it('has N tracks and no padding cell while no strip anchors', async () => {
    const { root } = await mountGrid()
    const grid = root.querySelector('.grid-block') as HTMLElement
    expect(grid.style.gridTemplateRows).toBe('repeat(3, auto)')
    expect(root.querySelectorAll('.strip-pad')).toHaveLength(0)
    expect(root.querySelector('.strip-track')?.classList.contains('strip-track-off')).toBe(true)
  })

  it('adds one track, one padding cell per column right after the anchor row, and places the strip under it', async () => {
    const { wrapper, root } = await mountGrid()
    anchor.value = 2
    await wrapper.vm.$nextTick()
    const grid = root.querySelector('.grid-block') as HTMLElement
    expect(grid.style.gridTemplateRows).toBe('repeat(4, auto)')
    for (const col of ['rule', 'num', 'src', 'tgt']) {
      const kids = Array.from(root.querySelector(`.col-${col}`)!.children)
      expect(kids).toHaveLength(4)
      expect(kids[2].classList.contains('strip-pad')).toBe(true)
      expect(kids[1].classList.contains('strip-pad')).toBe(false)
    }
    const track = root.querySelector('.strip-track') as HTMLElement
    expect(track.classList.contains('strip-track-off')).toBe(false)
    expect(track.style.gridRow).toBe('3')
  })

  it('drops the track and every padding cell when the strip collapses', async () => {
    const { wrapper, root } = await mountGrid()
    anchor.value = 2
    await wrapper.vm.$nextTick()
    anchor.value = null
    await wrapper.vm.$nextTick()
    expect((root.querySelector('.grid-block') as HTMLElement).style.gridTemplateRows).toBe('repeat(3, auto)')
    expect(root.querySelectorAll('.strip-pad')).toHaveLength(0)
    expect(root.querySelector('.strip-track')?.classList.contains('strip-track-off')).toBe(true)
  })

  it('takes no track when the anchor is not a row of the open Chapter', async () => {
    const { wrapper, root } = await mountGrid()
    anchor.value = 99
    await wrapper.vm.$nextTick()
    expect((root.querySelector('.grid-block') as HTMLElement).style.gridTemplateRows).toBe('repeat(3, auto)')
    expect(root.querySelectorAll('.strip-pad')).toHaveLength(0)
    expect(root.querySelector('.strip-track')?.classList.contains('strip-track-off')).toBe(true)
  })

  it('points aria-details at the strip track from the anchor translation cell only', async () => {
    const { wrapper, root } = await mountGrid()
    anchor.value = 2
    await wrapper.vm.$nextTick()
    const cells = Array.from(root.querySelectorAll('.cell-tgt'))
    const withDetails = cells.filter((c) => c.hasAttribute('aria-details'))
    expect(withDetails).toHaveLength(1)
    expect((withDetails[0] as HTMLElement).dataset.segmentId).toBe('2')
    expect(withDetails[0].getAttribute('aria-details')).toBe(root.querySelector('.strip-track')!.id)
  })

  it('keeps the caret when focus moves from the cell into the strip, and clears it when focus leaves both', async () => {
    const { wrapper, state, root } = await mountGrid()
    anchor.value = 1
    await wrapper.vm.$nextTick()
    state.setEditorCaret(1)
    const cell = root.querySelector('.cell-tgt') as HTMLElement
    const btn = root.querySelector('.stub-btn') as HTMLElement
    cell.dispatchEvent(new FocusEvent('focusout', { bubbles: true, relatedTarget: btn }))
    expect(state.editorCaretSegmentId.value).toBe(1)
    btn.dispatchEvent(new FocusEvent('focusout', { bubbles: true, relatedTarget: document.body }))
    expect(state.editorCaretSegmentId.value).toBeNull()
  })
})
