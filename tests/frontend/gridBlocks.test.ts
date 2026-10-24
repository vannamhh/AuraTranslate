import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { GRID_BLOCK_ROWS, gridBlocksOf } from '../../src/panels/gridBlocks'

const BASE_ID = 1000
const anchor = ref<number | null>(null)
let segmentCount = 450
let insertAt: number | null = null
let loadedCaretIndex: number | null = null

function makeSegment(id: number, text: string): ChapterSegment {
  return {
    id,
    ord: 0,
    source_text: text,
    target_text: '',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    unlinked: false,
    is_target_paragraph_end: false,
    role: null,
    translation_origin: '',
  }
}

function buildSegments(): ChapterSegment[] {
  const list = Array.from({ length: segmentCount }, (_, i) => makeSegment(BASE_ID + i, `Cau ${i}.`))
  if (insertAt !== null) list.splice(insertAt, 0, makeSegment(1, 'Cau tach ra.'))
  return list.map((s, i) => ({ ...s, ord: i + 1 }))
}

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: () =>
      Promise.resolve({
        loaded: {
          chapter_id: 7,
          segments: buildSegments(),
          caret_segment_id: loadedCaretIndex === null ? null : BASE_ID + loadedCaretIndex,
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

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
  anchor.value = null
  segmentCount = 450
  insertAt = null
  loadedCaretIndex = null
})

async function mountGrid() {
  vi.resetModules()
  vi.doMock('../../src/inlineStripEligibility', () => ({ inlineStripAnchorSegmentId: anchor }))
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const contract = await import('../../src/panels/selectionContract')
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { wrapper, state, contract, root: wrapper.element as unknown as Element }
}

const blocksOf = (root: Element): Element[] => [...root.querySelectorAll('.grid-block')]
const rowsIn = (block: Element): number => block.querySelectorAll('.col-tgt > .cell-tgt').length
const COLUMNS = ['.col-rule', '.col-num', '.col-src', '.col-tgt']

describe('gridBlocksOf', () => {
  it('a chapter shorter than one block is exactly one block of that many rows', () => {
    expect(gridBlocksOf(20)).toEqual([{ start: 0, rows: 20 }])
  })

  it('450 sentences split into 200, 200 and 50', () => {
    expect(gridBlocksOf(450)).toEqual([
      { start: 0, rows: 200 },
      { start: 200, rows: 200 },
      { start: 400, rows: 50 },
    ])
  })

  it('exactly one full block does not open an empty second block', () => {
    expect(gridBlocksOf(GRID_BLOCK_ROWS)).toEqual([{ start: 0, rows: GRID_BLOCK_ROWS }])
    expect(gridBlocksOf(0)).toEqual([])
  })

  it('a 201st row opens a second block of one row', () => {
    expect(gridBlocksOf(GRID_BLOCK_ROWS + 1)).toEqual([
      { start: 0, rows: GRID_BLOCK_ROWS },
      { start: GRID_BLOCK_ROWS, rows: 1 },
    ])
  })
})

describe('GridPanel grid blocks', { timeout: 30_000 }, () => {
  it('20 sentences mount as one block of 20 rows', async () => {
    segmentCount = 20
    const { wrapper, root } = await mountGrid()
    const blocks = blocksOf(root)
    expect(blocks).toHaveLength(1)
    expect(rowsIn(blocks[0]!)).toBe(20)
    wrapper.unmount()
  })

  it('450 sentences mount as 3 blocks, none above the constant, and no column spans the chapter', async () => {
    const { wrapper, root } = await mountGrid()
    const blocks = blocksOf(root)
    expect(blocks.map(rowsIn)).toEqual([200, 200, 50])
    for (const block of blocks) {
      expect(rowsIn(block)).toBeLessThanOrEqual(GRID_BLOCK_ROWS)
      for (const selector of COLUMNS) expect(block.querySelectorAll(`:scope > ${selector}`)).toHaveLength(1)
    }
    for (const selector of COLUMNS) {
      expect(root.querySelectorAll(selector)).toHaveLength(3)
      for (const column of root.querySelectorAll(selector)) {
        expect(column.parentElement?.classList.contains('grid-block')).toBe(true)
      }
    }
    expect(root.querySelectorAll('.cell-tgt')).toHaveLength(450)
    wrapper.unmount()
  })

  it('row numbers continue across blocks', async () => {
    const { wrapper, root } = await mountGrid()
    const numbers = [...root.querySelectorAll('.col-num > .cell-num')].map((el) => el.textContent.trim())
    expect(numbers[0]).toBe('1')
    expect(numbers[199]).toBe('200')
    expect(numbers[200]).toBe('201')
    expect(numbers[449]).toBe('450')
    wrapper.unmount()
  })

  it('the strip pad of an anchor on the last row of block 0 stays inside block 0', async () => {
    const { wrapper, root } = await mountGrid()
    anchor.value = BASE_ID + 199
    await wrapper.vm.$nextTick()
    const pads = [...root.querySelectorAll('.strip-pad')]
    expect(pads.length).toBeGreaterThan(0)
    const first = blocksOf(root)[0]!
    for (const pad of pads) expect(first.contains(pad)).toBe(true)
    expect(first.querySelectorAll('.strip-pad')).toHaveLength(pads.length)
    wrapper.unmount()
  })

  it('a split at row 199 leaves block 0 at 200 rows and pushes one row into block 1, keeping the caret cell', async () => {
    loadedCaretIndex = 10
    const { wrapper, state, root } = await mountGrid()
    const before = root.querySelector('.cell-tgt.editing')
    expect(before?.getAttribute('data-segment-id')).toBe(String(BASE_ID + 10))
    expect(rowsIn(blocksOf(root)[0]!)).toBe(200)

    insertAt = 199
    segmentCount = 450
    state.resetEditorPanel()
    await state.ensureSegmentsLoaded()
    await wrapper.vm.$nextTick()

    const blocks = blocksOf(root)
    expect(blocks.map(rowsIn)).toEqual([200, 200, 51])
    expect(blocks[0]!.querySelectorAll('.col-tgt > .cell-tgt')[199]!.getAttribute('data-segment-id')).toBe('1')
    expect(blocks[1]!.querySelector('.col-tgt > .cell-tgt')!.getAttribute('data-segment-id')).toBe(String(BASE_ID + 199))
    expect(root.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id')).toBe(String(BASE_ID + 10))
    wrapper.unmount()
  })

  it('a merge that removes one row keeps block 0 at 200 rows and shrinks only the last block', async () => {
    const { wrapper, state, root } = await mountGrid()
    segmentCount = 449
    state.resetEditorPanel()
    await state.ensureSegmentsLoaded()
    await wrapper.vm.$nextTick()
    const blocks = blocksOf(root)
    expect(blocks.map(rowsIn)).toEqual([200, 200, 49])
    wrapper.unmount()
  })

  it('moving the caret to a sentence in block 30 puts the editing cell there', async () => {
    segmentCount = 6100
    const { wrapper, state, root } = await mountGrid()
    expect(blocksOf(root)).toHaveLength(31)
    const target = BASE_ID + 30 * GRID_BLOCK_ROWS + 7
    expect(state.moveEditorCaretTo(target)).toBe(true)
    await wrapper.vm.$nextTick()
    const editing = root.querySelector('.cell-tgt.editing')
    expect(editing?.getAttribute('data-segment-id')).toBe(String(target))
    expect(blocksOf(root)[30]!.contains(editing)).toBe(true)
    wrapper.unmount()
  })
})

function selectIn(node: Node, start: number, end: number): void {
  const range = document.createRange()
  range.setStart(node, start)
  range.setEnd(node, end)
  window.getSelection()?.removeAllRanges()
  window.getSelection()?.addRange(range)
}

function firstText(host: Element): Text {
  const walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT)
  let node = walker.nextNode()
  while (node !== null && (node.textContent ?? '').trim() === '') node = walker.nextNode()
  if (node === null) throw new Error('no text node')
  return node as Text
}

describe('GridPanel source surface is per cell', { timeout: 30_000 }, () => {
  it('a selection inside a source cell of block 5 triggers the lookup', async () => {
    segmentCount = 1250
    const { wrapper, contract, root } = await mountGrid()
    const dispatch = vi.fn()
    const detach = contract.attachSelectionWatcher(document, dispatch)
    const cell = blocksOf(root)[5]!.querySelector('.cell-src[data-col="src"]') as Element
    const text = firstText(cell)
    selectIn(text, 0, text.length)
    expect(contract.currentSelectionText()).not.toBe('')
    document.dispatchEvent(new MouseEvent('mouseup'))
    expect(dispatch).toHaveBeenCalledTimes(1)
    detach()
    wrapper.unmount()
  })

  it('a selection that starts in the source column but outside any cell triggers nothing', async () => {
    segmentCount = 1250
    const { wrapper, contract, root } = await mountGrid()
    const dispatch = vi.fn()
    const detach = contract.attachSelectionWatcher(document, dispatch)
    const block = blocksOf(root)[5]!
    const column = block.querySelector('.col-src') as Element
    const cell = block.querySelector('.cell-src') as Element
    const range = document.createRange()
    range.setStart(column, 0)
    range.setEnd(firstText(cell), firstText(cell).length)
    window.getSelection()?.removeAllRanges()
    window.getSelection()?.addRange(range)
    expect(contract.currentSelectionText()).toBe('')
    document.dispatchEvent(new MouseEvent('mouseup'))
    expect(dispatch).not.toHaveBeenCalled()
    detach()
    wrapper.unmount()
  })
})

describe('source selection across a block boundary through GridPanel', () => {
  function sourceTextNode(root: Element, index: number): Text {
    const cell = root.querySelectorAll('[data-col="src"]')[index]
    const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
    let node = walker.nextNode()
    while (node !== null && (node.textContent ?? '').trim() === '') node = walker.nextNode()
    return node as Text
  }

  function copyEvent(): { event: Event; data: Record<string, string> } {
    const data: Record<string, string> = {}
    const event = new Event('copy', { bubbles: true, cancelable: true })
    Object.defineProperty(event, 'clipboardData', { value: { setData: (k: string, v: string) => (data[k] = v) } })
    return { event, data }
  }

  const FROM = GRID_BLOCK_ROWS - 1
  const TO = GRID_BLOCK_ROWS + 1
  const EXPECTED_TEXT = `au ${FROM}.\nCau ${GRID_BLOCK_ROWS}.\nCau`

  for (const backward of [false, true]) {
    it(`a ${backward ? 'backward' : 'forward'} selection from block 0 into block 1 yields only source text and copy writes only source text`, { timeout: 60_000 }, async () => {
      segmentCount = GRID_BLOCK_ROWS + 5
      const { wrapper, contract, root } = await mountGrid()
      const first = sourceTextNode(root, FROM)
      const last = sourceTextNode(root, TO)
      const selection = window.getSelection()!
      if (backward) selection.setBaseAndExtent(last, 3, first, 1)
      else selection.setBaseAndExtent(first, 1, last, 3)

      const text = contract.currentSelectionText()
      expect(selection.toString()).not.toBe(EXPECTED_TEXT)
      expect(text).toBe(EXPECTED_TEXT)

      const { event, data } = copyEvent()
      root.querySelector('.grid')!.dispatchEvent(event)
      expect(event.defaultPrevented).toBe(true)
      expect(data['text/plain']).toBe(EXPECTED_TEXT)
      wrapper.unmount()
    })
  }
})
