import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { h } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS, recordSave, resetRecorder, saveCalls } from './support/segmentFixture'

const probe = vi.hoisted(() => ({ ruleInputCalls: 0, frameRenders: 0, segments: [] as unknown[], marks: [] as unknown[] }))

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

const STUBS = {
  PanelFrame: {
    setup(_: unknown, { slots }: { slots: { default?: () => unknown } }) {
      return () => {
        probe.frameRenders += 1
        return h('div', { class: 'panel-frame' }, slots.default?.() as never)
      }
    },
  },
}

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
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
    status: 'draft',
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

async function type(wrapper: { vm: { $nextTick: () => Promise<void> } }, cell: HTMLElement, text: string) {
  cell.textContent = text
  cell.dispatchEvent(new Event('input', { bubbles: true }))
  await wrapper.vm.$nextTick()
}

describe('GridPanel typing cost', () => {
  it.each([20, 200])('a keystroke inside a written row renders no slot and resolves no rule at %i rows', async (n) => {
    probe.segments = many(n)
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(101)
    await wrapper.vm.$nextTick()
    probe.ruleInputCalls = 0
    probe.frameRenders = 0
    const cell = tgt(root, 101)
    for (const text of ['Dich 1.a', 'Dich 1.ab', 'Dich 1.abc']) await type(wrapper, cell, text)
    expect(probe.frameRenders).toBe(0)
    expect(probe.ruleInputCalls).toBe(0)
    expect(state.editorEditedText.value.get(101)).toBe('Dich 1.abc')
    wrapper.unmount()
  })

  it('the first character in an empty row turns the empty class off at once, and the rule is draft after the caret leaves', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(13)
    await wrapper.vm.$nextTick()
    const cell = tgt(root, 13)
    expect(cell.classList.contains('empty')).toBe(true)
    await type(wrapper, cell, 'm')
    expect(cell.classList.contains('empty')).toBe(false)
    state.setEditorCaret(11)
    await wrapper.vm.$nextTick()
    expect(slotClass(root, 2)).toContain('rule-draft')
    wrapper.unmount()
  })

  it('deleting the last character turns the empty class on, and the rule is none after the caret leaves', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(12)
    await wrapper.vm.$nextTick()
    const cell = tgt(root, 12)
    await type(wrapper, cell, 'x')
    await type(wrapper, cell, '')
    expect(cell.classList.contains('empty')).toBe(true)
    state.setEditorCaret(11)
    await wrapper.vm.$nextTick()
    expect(slotClass(root, 1)).not.toContain('rule-draft')
    wrapper.unmount()
  })

  it('navigation right after typing counts the typed row as translated', async () => {
    const { wrapper, state, root } = await mountGrid()
    state.setEditorCaret(11)
    await wrapper.vm.$nextTick()
    await type(wrapper, tgt(root, 13), 'vua go')
    expect(state.goToNextUntranslated()).toBe(false)
    expect(state.editorCaretSegmentId.value).toBe(11)
    wrapper.unmount()
  })

  it('a flush after typing carries the last text', async () => {
    const { wrapper, state, root } = await mountGrid()
    const cell = tgt(root, 12)
    state.setEditorCaret(12)
    await type(wrapper, cell, 'a')
    await type(wrapper, cell, 'ab')
    await state.flushEditorNow()
    expect(saveCalls.at(-1)?.edits).toEqual([{ id: 12, target_text: 'ab' }])
    wrapper.unmount()
  })

  it('resetting the panel (chapter switch) empties the typed map and restores the stored empty state', async () => {
    const { wrapper, state, root } = await mountGrid()
    await type(wrapper, tgt(root, 13), 'dang go')
    expect(tgt(root, 13).classList.contains('empty')).toBe(false)
    state.resetEditorPanel()
    expect(state.editorEditedText.value.size).toBe(0)
    wrapper.unmount()
  })
})
