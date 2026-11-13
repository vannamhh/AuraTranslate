import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import type { ChapterSegment } from '../../src/config/segment'
import type { GlossaryMark } from '../../src/config/glossary'

const picked = vi.hoisted(() => ({ span: null as { start: number; end: number } | null }))

const base = {
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  unlinked: false,
  is_paragraph_end: false,
  is_target_paragraph_end: false,
  role: null,
  translation_origin: '',
  target_text: '',
} as const
const SEGMENTS: ChapterSegment[] = [
  { ...base, id: 1, ord: 1, source_text: '群體和追蹤' },
  { ...base, id: 2, ord: 2, source_text: '追蹤群體' },
]
const MARK: GlossaryMark = {
  start: 0,
  end: 2,
  tier: 'global',
  is_confirmed: true,
  translation: 'Nhom',
  id: 1,
  source_term: '群體',
  han_viet_suggestion: null,
  han_viet_status: 'not_requested',
  occurrence_count: null,
  term_origin: 'manual',
  category: 'other',
}

vi.mock('../../src/config/segment', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/segment')>()),
  readOpenChapterSegments: async () => ({
    loaded: { chapter_id: 7, segments: SEGMENTS.map((s) => ({ ...s })), caret_segment_id: null, assets: [], assets_dir: '', tm_filled_segment_ids: [], tm_prefill: { kind: 'ran', hidden_count: 0 } },
    error: null,
  }),
}))
vi.mock('../../src/config/chapter', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/chapter')>()),
  readOpenChapter: async () => ({ chapter: { chapter_id: 7, source_text: '', source_lang: 'zh' }, error: null }),
}))
vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  glossaryMarksForChapter: async () => ({ marks: [MARK], error: null }),
}))
vi.mock('../../src/config/dict', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/dict')>()),
  lookupDictionary: vi.fn(async () => ({ response: null, error: null })),
}))
vi.mock('../../src/panels/editorSegments', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorSegments')>()),
  lookupSpanAtPoint: () => picked.span,
}))

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }
const mounted: { unmount: () => void }[] = []

afterEach(() => {
  for (const w of mounted.splice(0)) w.unmount()
  document.body.innerHTML = ''
  Reflect.deleteProperty(document, 'caretRangeFromPoint')
})

async function mountGrid() {
  vi.resetModules()
  const editorState = await import('../../src/panels/editorPanelState')
  const sourceState = await import('../../src/panels/sourcePanelState')
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  mounted.push(wrapper)
  await editorState.ensureSegmentsLoaded()
  await sourceState.ensureChapterLoaded()
  await wrapper.vm.$nextTick()
  await flushPromises()
  await wrapper.vm.$nextTick()
  return { editorState, wrapper }
}

function cellOf(id: number): HTMLElement {
  const cell = document.querySelector<HTMLElement>(`[data-col="src"][data-segment-id="${id}"]`)
  if (cell === null) throw new Error(`no source cell ${id}`)
  return cell
}

async function clickSpan(wrapper: { vm: { $nextTick: () => Promise<void> } }, id: number, start: number, end: number): Promise<void> {
  const cell = cellOf(id)
  const range = document.createRange()
  range.setStart(cell, 0)
  Object.defineProperty(document, 'caretRangeFromPoint', { value: () => range, configurable: true })
  picked.span = { start, end }
  cell.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, button: 0, detail: 1 }))
  await wrapper.vm.$nextTick()
}

function marked(): string[] {
  return [...document.querySelectorAll('[data-lookup-active]')].map((el) => el.textContent)
}

describe('Original tab lookup paint', () => {
  it('clicking A then B in one sentence marks only the pieces of B', async () => {
    const { wrapper } = await mountGrid()
    await clickSpan(wrapper, 1, 0, 2)
    expect(marked().join('')).toBe('群體')
    await clickSpan(wrapper, 1, 3, 5)
    expect(marked().join('')).toBe('追蹤')
    expect(cellOf(1).querySelector('[data-lookup-active]')?.getAttribute('data-src-start')).toBe('3')
  })

  it('clicking A then B in another sentence leaves no mark in the first cell', async () => {
    const { wrapper } = await mountGrid()
    await clickSpan(wrapper, 1, 3, 5)
    await clickSpan(wrapper, 2, 0, 2)
    expect(cellOf(1).querySelectorAll('[data-lookup-active]')).toHaveLength(0)
    expect(marked().join('')).toBe('追蹤')
    expect(cellOf(2).querySelectorAll('[data-lookup-active]')).toHaveLength(1)
  })

  it('a confirmed Glossary term keeps its class and carries the lookup mark', async () => {
    const { wrapper } = await mountGrid()
    await clickSpan(wrapper, 1, 0, 2)
    const piece = cellOf(1).querySelector('[data-lookup-active]')
    expect(piece?.classList.contains('glossary-confirmed')).toBe(true)
    expect(piece?.textContent).toBe('群體')
  })

  it('lookup boundaries add no cut-mark next to a pending cut', async () => {
    const { editorState, wrapper } = await mountGrid()
    editorState.setEditorSourceCut(1, 3)
    await wrapper.vm.$nextTick()
    expect(cellOf(1).querySelectorAll('.cut-mark')).toHaveLength(1)
    await clickSpan(wrapper, 1, 3, 5)
    expect(cellOf(1).querySelectorAll('.cut-mark')).toHaveLength(1)
    await clickSpan(wrapper, 1, 0, 2)
    expect(cellOf(1).querySelectorAll('.cut-mark')).toHaveLength(1)
    expect(marked().join('')).toBe('群體')
  })

  it('a stale mark disappears when the Han Viet tab opens', async () => {
    const { wrapper } = await mountGrid()
    await clickSpan(wrapper, 1, 0, 2)
    const sourceState = await import('../../src/panels/sourcePanelState')
    sourceState.selectSourceTab('han_viet')
    await flushPromises()
    await wrapper.vm.$nextTick()
    expect(document.querySelectorAll('[data-lookup-active]')).toHaveLength(0)
  })

  it('a lookup of another phrase from elsewhere removes the mark', async () => {
    const { wrapper } = await mountGrid()
    await clickSpan(wrapper, 1, 0, 2)
    const panel = await import('../../src/panels/lookupPanelState')
    await panel.runLookup('其他')
    await flushPromises()
    await wrapper.vm.$nextTick()
    expect(document.querySelectorAll('[data-lookup-active]')).toHaveLength(0)
  })
})
