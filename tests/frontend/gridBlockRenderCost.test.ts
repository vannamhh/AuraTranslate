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

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    confirmSegment: async (segmentId: number) => ({
      outcome: { segment_id: segmentId, status: 'confirmed', version_created: true },
      error: null,
    }),
    readOpenChapterSegments: () =>
      Promise.resolve({
        loaded: {
          chapter_id: 7,
          segments: SEGMENTS.map((s) => ({ ...s })),
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
})

async function mountGrid() {
  vi.resetModules()
  vi.doMock('../../src/inlineStripEligibility', () => ({ inlineStripAnchorSegmentId: anchor }))
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
})
