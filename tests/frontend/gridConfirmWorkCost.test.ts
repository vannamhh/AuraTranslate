import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'

const anchor = ref<number | null>(null)
const BASE_ID = 100
const sizeHolder = vi.hoisted(() => ({ count: 1050 }))
const rulePasses = vi.hoisted(() => ({ calls: 0 }))

const marksOf = (count: number) => {
  let offset = 0
  const marks = []
  for (let i = 0; i < count; i++) {
    const text = `Cau ${String(i)}.`
    if (i % 5 === 0) {
      marks.push({
        start: offset,
        end: offset + 3,
        tier: 'global',
        is_confirmed: true,
        translation: 'Tu',
        id: i + 1,
        source_term: 'Cau',
        han_viet_suggestion: null,
        han_viet_status: 'not_requested',
        occurrence_count: null,
        term_origin: 'manual',
        category: 'other',
      })
    }
    offset += text.length + 1
  }
  return marks
}

const segmentsOf = (count: number): ChapterSegment[] =>
  Array.from({ length: count }, (_, i) => ({
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

vi.mock('../../src/panels/editorSegments', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorSegments')>()
  return {
    ...actual,
    resolveSegmentRule: (input: Parameters<typeof actual.resolveSegmentRule>[0]) => {
      rulePasses.calls += 1
      return actual.resolveSegmentRule(input)
    },
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
  return { ...actual, glossaryMarksForChapter: async () => ({ marks: marksOf(sizeHolder.count), error: null }) }
})

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
          segments: segmentsOf(sizeHolder.count),
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
  ProofreaderStrip: { template: '<i />' },
}

const restore: (() => void)[] = []
const work = { sourceScans: 0, gridQueries: 0 }

function countWork(gridRoot: Element): void {
  const iterator = String.prototype[Symbol.iterator]
  String.prototype[Symbol.iterator] = function (this: string) {
    work.sourceScans += 1
    return iterator.call(this)
  }
  restore.push(() => {
    String.prototype[Symbol.iterator] = iterator
  })
  for (const name of ['querySelector', 'querySelectorAll'] as const) {
    const original = Element.prototype[name] as (this: Element, selector: string) => unknown
    Object.defineProperty(Element.prototype, name, {
      configurable: true,
      writable: true,
      value: function (this: Element, selector: string) {
        if (this === gridRoot) work.gridQueries += 1
        return original.call(this, selector)
      },
    })
    restore.push(() => {
      Object.defineProperty(Element.prototype, name, { configurable: true, writable: true, value: original })
    })
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
  anchor.value = null
})

afterEach(() => {
  while (restore.length > 0) restore.pop()?.()
})

async function confirmWork(count: number): Promise<{ rules: number; sourceScans: number; gridQueries: number }> {
  sizeHolder.count = count
  vi.resetModules()
  vi.doMock('../../src/inlineStripEligibility', () => ({ inlineStripAnchorSegmentId: anchor }))
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  state.setEditorCaret(BASE_ID + 250)
  await wrapper.vm.$nextTick()

  const gridRoot = wrapper.element.querySelector('.grid')
  expect(gridRoot).not.toBeNull()
  rulePasses.calls = 0
  work.sourceScans = 0
  work.gridQueries = 0
  countWork(gridRoot as Element)
  expect(await state.confirmCurrentSegment()).toBe('confirmed')
  await wrapper.vm.$nextTick()
  while (restore.length > 0) restore.pop()?.()
  const result = { rules: rulePasses.calls, sourceScans: work.sourceScans, gridQueries: work.gridQueries }
  wrapper.unmount()
  return result
}

describe('GridPanel confirm work cost', { timeout: 60_000 }, () => {
  it('one confirm does work that does not grow with the number of segments', async () => {
    const small = await confirmWork(1050)
    const large = await confirmWork(2100)
    console.log('confirm-work', JSON.stringify({ small, large }))
    expect(large.rules).toBe(small.rules)
    expect(large.sourceScans).toBe(small.sourceScans)
    expect(large.gridQueries).toBe(small.gridQueries)
    expect(small.gridQueries).toBe(0)
  })
})
