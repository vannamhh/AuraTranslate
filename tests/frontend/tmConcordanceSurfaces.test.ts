/** The real GridPanel target column and AiTranslationPanel register as Vietnamese surfaces. */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterSegment } from '../../src/config/segment'

const SEGMENTS: ChapterSegment[] = [
  {
    id: 1,
    ord: 1,
    source_text: '他叫师父。',
    target_text: 'Hắn gọi sư phụ.',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: null,
    translation_origin: '',
  },
]

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
          tm_prefill: { kind: 'ran' },
        },
        error: null,
      }),
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
})

function selectFirstText(host: Element): void {
  const walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT)
  let node = walker.nextNode()
  while (node !== null && (node.textContent ?? '').trim() === '') node = walker.nextNode()
  if (node === null) throw new Error('no text node in surface')
  const range = document.createRange()
  range.setStart(node, 0)
  range.setEnd(node, (node.textContent ?? '').length)
  window.getSelection()?.removeAllRanges()
  window.getSelection()?.addRange(range)
}

describe('Vietnamese surface declaration through the real panels', () => {
  it('GridPanel target column is Vietnamese, its source column is not', async () => {
    vi.resetModules()
    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const editor = await import('../../src/panels/editorPanelState')
    const contract = await import('../../src/panels/selectionContract')
    const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
    await editor.ensureSegmentsLoaded()
    await wrapper.vm.$nextTick()
    const root = wrapper.element as unknown as Element

    selectFirstText(root.querySelector('.col-tgt') as Element)
    expect(contract.currentSelectionTextForConcordance()).toContain('Hắn')
    expect(contract.currentSelectionIsVietnameseForConcordance()).toBe(true)

    selectFirstText(root.querySelector('.col-src') as Element)
    expect(contract.currentSelectionTextForConcordance()).not.toBe('')
    expect(contract.currentSelectionIsVietnameseForConcordance()).toBe(false)
    wrapper.unmount()
  })

  it('AiTranslationPanel surface is Vietnamese', async () => {
    vi.resetModules()
    const Panel = (await import('../../src/panels/AiTranslationPanel.vue')).default
    const contract = await import('../../src/panels/selectionContract')
    const wrapper = mount(Panel, { props: { params: { params: {} } }, attachTo: document.body })
    await wrapper.vm.$nextTick()
    selectFirstText(wrapper.get('.ai-surface').element)
    expect(contract.currentSelectionTextForConcordance()).not.toBe('')
    expect(contract.currentSelectionIsVietnameseForConcordance()).toBe(true)
    wrapper.unmount()
  })
})
