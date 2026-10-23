import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS, failNextSave, recordSave, resetRecorder } from './support/segmentFixture'

const probe = vi.hoisted(() => ({ ruleInputCalls: 0, segments: [] as unknown[], failConfirm: false, marks: [] as unknown[] }))

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/panels/editorSegments', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorSegments')>()
  return {
    ...actual,
    segmentRuleInputOf: (...args: Parameters<typeof actual.segmentRuleInputOf>) => {
      probe.ruleInputCalls += 1
      return actual.segmentRuleInputOf(...args)
    },
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: async () => ({
      loaded: {
        chapter_id: 7,
        segments: (probe.segments as ChapterSegment[]).map((s) => ({ ...s })),
        caret_segment_id: null,
        assets: [],
        assets_dir: '',
        tm_filled_segment_ids: [],
        tm_prefill: { kind: 'ran', hidden_count: 0 },
      },
      error: null,
    }),
    saveSegmentTargets: recordSave,
    confirmSegment: async (segmentId: number) =>
      probe.failConfirm
        ? {
            outcome: null,
            error: {
              code: 'segment.nothing_to_confirm',
              message_key: 'err.segment.nothing_to_confirm',
              params: { segment_id: String(segmentId) },
              retryable: false,
            },
          }
        : { outcome: { segment_id: segmentId, status: 'confirmed', version_created: true }, error: null },
  }
})

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return {
    ...actual,
    readOpenChapter: async () => ({ chapter: { chapter_id: 7, source_text: '', source_lang: 'zh' }, error: null }),
  }
})

vi.mock('../../src/config/glossary', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/glossary')>()
  return { ...actual, glossaryMarksForChapter: async () => ({ marks: probe.marks, error: null }) }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
  probe.failConfirm = false
  probe.marks = []
  probe.segments = FIXTURE_SEGMENTS.map((s) => ({ ...s }))
})

function many(n: number): ChapterSegment[] {
  return Array.from({ length: n }, (_, i) => ({
    ...FIXTURE_SEGMENTS[1],
    id: 100 + i,
    ord: i + 1,
    source_text: `Cau ${i}.`,
    target_text: i % 3 === 0 ? '' : `Dich ${i}.`,
    is_paragraph_end: false,
    is_target_paragraph_end: false,
  }))
}

async function mountGrid() {
  vi.resetModules()
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await (await import('../../src/panels/sourcePanelState')).ensureChapterLoaded()
  await wrapper.vm.$nextTick()
  return { wrapper, state, root: wrapper.element as unknown as Element }
}

function tgt(root: Element, id: number): HTMLElement {
  return root.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${id}"]`)!
}

function slotClass(root: Element, index: number): string {
  return root.querySelectorAll('.col-rule .margin-slot')[index].className
}

describe('GridPanel caret move cost', () => {
  it.each([20, 200])('moving the caret resolves no row rule again at %i rows', async (n) => {
    probe.segments = many(n)
    const { wrapper, state } = await mountGrid()
    state.setEditorCaret(100)
    await wrapper.vm.$nextTick()
    probe.ruleInputCalls = 0
    state.setEditorCaret(101)
    await wrapper.vm.$nextTick()
    state.setEditorCaret(102)
    await wrapper.vm.$nextTick()
    expect(probe.ruleInputCalls).toBe(0)
    wrapper.unmount()
  })

  it('the caret value: editing class, aria-label and primary rule move to the new row, and leave the old one', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(12)
    await wrapper.vm.$nextTick()
    expect(tgt(root, 12).classList.contains('editing')).toBe(true)
    expect(tgt(root, 12).getAttribute('aria-label')).toContain('Câu 2')
    expect(slotClass(root, 1)).toContain('rule-primary')
    state.setEditorCaret(13)
    await wrapper.vm.$nextTick()
    expect(tgt(root, 12).classList.contains('editing')).toBe(false)
    expect(slotClass(root, 1)).toContain('rule-draft')
    expect(slotClass(root, 2)).toContain('rule-primary')
    wrapper.unmount()
  })

  it('the written-text value: typing changes the empty class and the rule of a row that is not under the caret', async () => {
    const { wrapper, root } = await mountGrid()
    const cell = tgt(root, 13)
    expect(cell.classList.contains('empty')).toBe(true)
    cell.textContent = 'moi'
    cell.dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()
    expect(cell.classList.contains('empty')).toBe(false)
    expect(slotClass(root, 2)).toContain('rule-draft')
    wrapper.unmount()
  })

  it('the written-text value reaches the empty class when the rule does not change: a draft row edited to blanks', async () => {
    const { wrapper, root } = await mountGrid()
    const cell = tgt(root, 12)
    expect(cell.classList.contains('empty')).toBe(false)
    cell.textContent = '  '
    cell.dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()
    expect(slotClass(root, 1)).toContain('rule-draft')
    expect(cell.classList.contains('empty')).toBe(true)
    wrapper.unmount()
  })

  it('the glossary value: a mark on one row tints its source piece and leaves the others', async () => {
    const { wrapper, root } = await mountGrid()
    const marksState = await import('../../src/panels/glossaryMarksState')
    probe.marks = [
      { start: 0, end: 1, tier: 'work', is_confirmed: true, translation: 'nhat', id: 1, source_term: '二', han_viet_suggestion: null },
    ]
    await marksState.refreshGlossaryMarks(7, FIXTURE_SEGMENTS.map((s) => ({ ...s })), 'zh')
    await wrapper.vm.$nextTick()
    const cells = root.querySelectorAll('.col-src .cell-src')
    expect(root.querySelectorAll('.glossary-confirmed').length).toBeGreaterThan(0)
    expect(cells[0].querySelector('.glossary-confirmed')).not.toBeNull()
    expect(cells[1].querySelector('.glossary-confirmed')).toBeNull()
    wrapper.unmount()
  })

  it('the Han Viet value: switching to the Han Viet tab replaces the source pieces of every row', async () => {
    const { wrapper, root } = await mountGrid()
    expect(root.querySelectorAll('.col-src .src-piece').length).toBe(3)
    ;(await import('../../src/panels/sourcePanelState')).selectSourceTab('han_viet')
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.col-src .src-piece')).toHaveLength(0)
    wrapper.unmount()
  })

  it('the row-error value: a refused confirm marks the row and carries the error text', async () => {
    probe.failConfirm = true
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(12)
    await wrapper.vm.$nextTick()
    await state.confirmCurrentSegment()
    await wrapper.vm.$nextTick()
    expect(tgt(root, 12).classList.contains('refused')).toBe(true)
    expect(tgt(root, 12).hasAttribute('data-row-error')).toBe(true)
    expect(tgt(root, 11).classList.contains('refused')).toBe(false)
    wrapper.unmount()
  })

  it('the row-error value from a failed flush reaches the row', async () => {
    const { wrapper, state, root } = await mountGrid()
    const cell = tgt(root, 12)
    state.setEditorCaret(12)
    cell.textContent = 'sua'
    cell.dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()
    failNextSave.value = true
    await state.flushEditorNow()
    await wrapper.vm.$nextTick()
    expect(tgt(root, 12).classList.contains('refused')).toBe(true)
    wrapper.unmount()
  })

  it('the unlinked value: a successful confirm clears the unlinked mark, class and label', async () => {
    probe.segments = FIXTURE_SEGMENTS.map((s) => (s.id === 12 ? { ...s, unlinked: true } : { ...s }))
    const { wrapper, state, root } = await mountGrid()
    expect(tgt(root, 12).getAttribute('data-unlinked')).toBe('true')
    expect(root.querySelectorAll('.cell-unlinked')).toHaveLength(1)
    state.setEditorCaret(12)
    await wrapper.vm.$nextTick()
    await state.confirmCurrentSegment()
    await wrapper.vm.$nextTick()
    expect(tgt(root, 12).getAttribute('data-unlinked')).toBeNull()
    expect(root.querySelectorAll('.cell-unlinked')).toHaveLength(0)
    expect(tgt(root, 12).getAttribute('aria-label')).not.toContain('chưa nối')
    wrapper.unmount()
  })

  it('the selection value: extending the selection tints the source, number and target cells of the row', async () => {
    const { wrapper, state, root } = await mountGrid()
    const sel = await import('../../src/panels/segmentSelectionState')
    state.setEditorCaret(11)
    await wrapper.vm.$nextTick()
    sel.extendSegmentSelectionDown()
    await wrapper.vm.$nextTick()
    const before = root.querySelectorAll('.col-src .cell-src')[1].className
    expect(before).not.toBe(root.querySelectorAll('.col-src .cell-src')[2].className)
    sel.clearSegmentSelection()
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.col-src .cell-src')[1].className).not.toBe(before)
    wrapper.unmount()
  })

  it('the source-cut value: a pending cut splits the source cell of its row only', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorSourceCut(11, 1)
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.col-src .cell-src')[0].getAttribute('data-cut-count')).toBe('1')
    expect(root.querySelectorAll('.col-src .cell-src')[1].getAttribute('data-cut-count')).toBe('0')
    state.clearEditorSourceCut()
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.col-src .cell-src')[0].getAttribute('data-cut-count')).toBe('0')
    wrapper.unmount()
  })

  it('the concordance value: the highlight marks the piece of its row and clears again', async () => {
    const { wrapper, root } = await mountGrid()
    const conc = await import('../../src/panels/concordanceState')
    conc.setConcordanceSourceHighlight({ segmentId: 12, start: 0, end: 2 })
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.col-src .cell-src')[1].querySelector('.concordance-source')).not.toBeNull()
    expect(root.querySelectorAll('.col-src .cell-src')[0].querySelector('.concordance-source')).toBeNull()
    conc.setConcordanceSourceHighlight(null)
    await wrapper.vm.$nextTick()
    expect(root.querySelector('.concordance-source')).toBeNull()
    wrapper.unmount()
  })

  it('the strip-anchor value: the padding cell follows the anchor in every column', async () => {
    vi.resetModules()
    const anchor = (await import('vue')).ref<number | null>(null)
    vi.doMock('../../src/inlineStripEligibility', () => ({ inlineStripAnchorSegmentId: anchor }))
    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const state = await import('../../src/panels/editorPanelState')
    const wrapper = mount(GridPanel, {
      props: { params: {} } as never,
      global: {
        stubs: {
          ...STUBS,
          TmFuzzyStrip: { template: '<i />' },
          GlossaryConfirmStrip: { template: '<i />' },
          ProofreaderStrip: { template: '<i />' },
        },
      },
      attachTo: document.body,
    })
    await state.ensureSegmentsLoaded()
    await wrapper.vm.$nextTick()
    const root = wrapper.element as unknown as Element
    anchor.value = 11
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.strip-pad')).toHaveLength(4)
    anchor.value = 12
    await wrapper.vm.$nextTick()
    for (const col of ['rule', 'num', 'src', 'tgt']) {
      const kids = Array.from(root.querySelector(`.col-${col}`)!.children)
      expect(kids.map((k) => k.classList.contains('strip-pad'))).toEqual([false, false, true, false])
    }
    anchor.value = null
    await wrapper.vm.$nextTick()
    expect(root.querySelectorAll('.strip-pad')).toHaveLength(0)
    wrapper.unmount()
    vi.doUnmock('../../src/inlineStripEligibility')
  })
})
