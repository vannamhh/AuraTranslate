import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'

const anchor = ref<number | null>(null)
const SEGMENT_COUNT = 1050
const BASE_ID = 100

const SEGMENTS: ChapterSegment[] = Array.from({ length: SEGMENT_COUNT }, (_, i) => ({
  id: BASE_ID + i,
  ord: i + 1,
  source_text: `Cau ${i}.`,
  target_text: i % 3 === 0 ? '' : `Dich ${i}.`,
  is_paragraph_end: false,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  unlinked: false,
  is_target_paragraph_end: false,
  role: null,
  translation_origin: '',
}))
let openSegments: readonly ChapterSegment[] = SEGMENTS

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    confirmSegment: async (segmentId: number) => ({
      outcome: { segment_id: segmentId, status: 'confirmed', version_created: true, chapter_started: false, tm_filled: [], tm_prefill: { kind: 'not_asked' } },
      error: null,
    }),
    readOpenChapterSegments: () =>
      Promise.resolve({
        loaded: {
          chapter_id: 7,
          segments: openSegments.map((s) => ({ ...s })),
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
  TmFuzzyStrip: { template: '<i />' },
  GlossaryConfirmStrip: { template: '<i />' },
  ProofreaderStrip: { template: '<i />' },
}

const updatedBlocks: number[] = []

beforeEach(() => {
  document.body.innerHTML = ''
  anchor.value = null
  updatedBlocks.length = 0
  openSegments = SEGMENTS
})

async function mountGrid() {
  vi.resetModules()
  vi.doMock('../../src/inlineStripEligibility', () => ({ inlineStripAnchorSegmentId: anchor, stripTrackFloats: ref(false) }))
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const mixin = {
    updated(this: { $props?: { block?: { index: number } } }) {
      const index = this.$props?.block?.index
      if (index !== undefined) updatedBlocks.push(index)
    },
  }
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS, mixins: [mixin] },
    attachTo: document.body,
  })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { wrapper, state, root: wrapper.element as unknown as Element }
}

const idAt = (index: number): number => BASE_ID + index

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<number[]> {
  await wrapper.vm.$nextTick()
  const blocks = [...new Set(updatedBlocks)].sort((a, b) => a - b)
  updatedBlocks.length = 0
  return blocks
}

describe('GridPanel block render cost', { timeout: 30_000 }, () => {
  it('moving the caret inside one block renders that block only', async () => {
    const { wrapper, state, root } = await mountGrid()
    expect(root.querySelectorAll('.grid-block').length).toBeGreaterThanOrEqual(5)
    state.setEditorCaret(idAt(250))
    await settle(wrapper)
    state.setEditorCaret(idAt(251))
    const blocks = await settle(wrapper)
    expect(blocks).toEqual([1])
    expect(root.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id')).toBe(String(idAt(251)))
    wrapper.unmount()
  })

  it('moving the caret across a block boundary renders the block it leaves and the block it enters', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(idAt(399))
    await settle(wrapper)
    state.setEditorCaret(idAt(400))
    const blocks = await settle(wrapper)
    expect(blocks).toEqual([1, 2])
    expect(root.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id')).toBe(String(idAt(400)))
    wrapper.unmount()
  })

  it('moving the strip anchor and the caret to another block renders the two blocks involved', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(idAt(250))
    anchor.value = idAt(250)
    await settle(wrapper)
    state.setEditorCaret(idAt(650))
    anchor.value = idAt(650)
    const blocks = await settle(wrapper)
    expect(blocks).toEqual([1, 3])
    expect(root.querySelectorAll('.strip-pad')).toHaveLength(4)
    wrapper.unmount()
  })

  it('moving only the strip anchor to another block renders the two blocks involved', async () => {
    const { wrapper } = await mountGrid()
    anchor.value = idAt(250)
    await settle(wrapper)
    anchor.value = idAt(850)
    const blocks = await settle(wrapper)
    expect(blocks).toEqual([1, 4])
    wrapper.unmount()
  })

  it('confirming a segment renders the block that holds it and the block the caret enters', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(idAt(399))
    await settle(wrapper)
    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    const blocks = await settle(wrapper)
    expect(blocks).toEqual([1, 2])
    expect(root.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id')).toBe(String(idAt(400)))
    expect(root.querySelectorAll('.rule-confirmed')).toHaveLength(1)
    wrapper.unmount()
  })

  it('confirming a segment inside one block renders that block only', async () => {
    const { wrapper, state } = await mountGrid()
    state.setEditorCaret(idAt(250))
    await settle(wrapper)
    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    expect(await settle(wrapper)).toEqual([1])
    wrapper.unmount()
  })

  describe('segment selection', () => {
    async function select(anchorIndex: number, focusIndex: number) {
      const grid = await mountGrid()
      const selection = await import('../../src/panels/segmentSelectionState')
      selection.pressSegmentSelection(idAt(anchorIndex), false)
      selection.dragSegmentSelectionFocus(idAt(focusIndex))
      await settle(grid.wrapper)
      return { ...grid, selection }
    }
    const selectedIds = (root: Element) =>
      [...root.querySelectorAll('.cell-num.row-selected')].map((e) => Number(e.getAttribute('data-num-segment-id')))

    it('dragging inside one block renders that block only', async () => {
      const { wrapper, selection } = await select(10, 10)
      selection.dragSegmentSelectionFocus(idAt(11))
      expect(await settle(wrapper)).toEqual([0])
      wrapper.unmount()
    })

    it('dragging across a block boundary renders the block it enters and paints contiguously', async () => {
      const { wrapper, selection, root } = await select(10, 199)
      selection.dragSegmentSelectionFocus(idAt(200))
      expect(await settle(wrapper)).toEqual([1])
      expect(selectedIds(root)).toHaveLength(191)
      wrapper.unmount()
    })

    it('dragging the focus of a multi-block selection renders the block holding the focus only', async () => {
      const { wrapper, selection } = await select(10, 450)
      selection.dragSegmentSelectionFocus(idAt(451))
      expect(await settle(wrapper)).toEqual([2])
      wrapper.unmount()
    })

    it('dragging backwards past the anchor repaints the blocks the range grows into', async () => {
      const { wrapper, selection, root } = await select(450, 250)
      selection.dragSegmentSelectionFocus(idAt(150))
      expect(await settle(wrapper)).toEqual([0, 1])
      expect(selectedIds(root)).toEqual(Array.from({ length: 301 }, (_, i) => idAt(150 + i)))
      wrapper.unmount()
    })

    it('dragging back into an earlier block drops the paint of the block that leaves the range', async () => {
      const { wrapper, selection, root } = await select(10, 450)
      selection.dragSegmentSelectionFocus(idAt(250))
      expect(await settle(wrapper)).toEqual([1, 2])
      expect(selectedIds(root)).toHaveLength(241)
      wrapper.unmount()
    })

    it('selecting the whole chapter paints every row and counts them all', async () => {
      const { wrapper, selection, root } = await select(0, SEGMENT_COUNT - 1)
      expect(selectedIds(root)).toHaveLength(SEGMENT_COUNT)
      expect(selection.segmentSelectionCount.value).toBe(SEGMENT_COUNT)
      wrapper.unmount()
    })

    it('paints nothing when a Chapter change leaves the anchor dead', async () => {
      const { wrapper, state, selection, root } = await select(10, 20)
      expect(selectedIds(root)).toHaveLength(11)
      openSegments = SEGMENTS.map((s) => ({ ...s, id: s.id + SEGMENT_COUNT }))
      state.resetEditorPanel()
      await state.ensureSegmentsLoaded()
      await settle(wrapper)
      expect(selection.segmentSelectionAnchorId.value).toBe(idAt(10))
      expect(selection.segmentSelectionRange.value).toBeNull()
      expect(selectedIds(root)).toEqual([])
      expect(root.querySelectorAll('.cell-num')).toHaveLength(SEGMENT_COUNT)
      wrapper.unmount()
    })
  })
})
