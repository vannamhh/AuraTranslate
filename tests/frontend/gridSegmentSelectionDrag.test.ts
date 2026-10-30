import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

vi.mock('../../src/config/segment', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/segment')>()),
  readOpenChapterSegments: readFixture,
  saveSegmentTargets: recordSave,
}))

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }
const mounted: { unmount: () => void }[] = []

afterEach(() => {
  for (const w of mounted.splice(0)) w.unmount()
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

beforeEach(() => {
  resetRecorder()
})

async function mountGrid() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const selection = await import('../../src/panels/segmentSelectionState')
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  mounted.push(wrapper)
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, selection, wrapper }
}

const numCell = (id: number): HTMLElement => {
  const el = document.querySelector<HTMLElement>(`[data-num-segment-id="${String(id)}"]`)
  if (el === null) throw new Error(`no number cell for ${String(id)}`)
  return el
}
const cell = (col: 'src' | 'tgt', id: number): HTMLElement => {
  const el = document.querySelector<HTMLElement>(`[data-col="${col}"][data-segment-id="${String(id)}"]`)
  if (el === null) throw new Error(`no ${col} cell for ${String(id)}`)
  return el
}

function fire(el: Element, type: string, init: MouseEventInit = {}): MouseEvent {
  const event = new MouseEvent(type, { button: 0, buttons: 1, bubbles: true, cancelable: true, ...init })
  el.dispatchEvent(event)
  return event
}

const down = (id: number, init: MouseEventInit = {}) => fire(numCell(id), 'mousedown', init)
const over = (el: Element, init: MouseEventInit = {}) => fire(el, 'mouseover', init)
const up = (el: Element) => fire(el, 'mouseup', { buttons: 0 })

describe('Lưới — kéo và Shift+bấm trên cột số câu', () => {
  it('kéo xuống: nhấn 11, qua 12, thả ở 13 ⇒ tô 11–13, caret và tiêu điểm giữ nguyên', async () => {
    const { state, selection, wrapper } = await mountGrid()
    state.setEditorCaret(12)
    const before = document.activeElement
    const event = down(11)
    expect(event.defaultPrevented).toBe(true)
    over(numCell(12))
    over(numCell(13))
    up(numCell(13))
    await wrapper.vm.$nextTick()
    expect(selection.segmentSelectionIds.value).toEqual([11, 12, 13])
    expect(state.editorCaretSegmentId.value).toBe(12)
    expect(document.activeElement).toBe(before)
  })

  it('kéo ngược: nhấn 13, lên 11 ⇒ tô 11–13, anchor là 13', async () => {
    const { selection } = await mountGrid()
    down(13)
    over(numCell(12))
    over(numCell(11))
    up(numCell(11))
    expect(selection.segmentSelectionIds.value).toEqual([11, 12, 13])
    expect(selection.segmentSelectionAnchorId.value).toBe(13)
  })

  it('bấm trơn trên 12 khi đang có vùng khác ⇒ vùng chọn chỉ còn câu 12', async () => {
    const { selection } = await mountGrid()
    down(11)
    over(numCell(13))
    up(numCell(13))
    expect(selection.segmentSelectionIds.value).toEqual([11, 12, 13])
    down(12)
    up(numCell(12))
    expect(selection.segmentSelectionIds.value).toEqual([12])
  })

  it('Shift+bấm có vùng: anchor 11, Shift+bấm 13 ⇒ 11–13', async () => {
    const { selection } = await mountGrid()
    down(11)
    up(numCell(11))
    down(13, { shiftKey: true })
    up(numCell(13))
    expect(selection.segmentSelectionIds.value).toEqual([11, 12, 13])
  })

  it('Shift+bấm chưa có vùng: caret câu 12 ⇒ 12–13', async () => {
    const { state, selection } = await mountGrid()
    state.setEditorCaret(12)
    down(13, { shiftKey: true })
    up(numCell(13))
    expect(selection.segmentSelectionIds.value).toEqual([12, 13])
  })

  it('Shift+bấm không có gốc: caret null ⇒ vùng chọn chỉ là câu bấm', async () => {
    const { state, selection } = await mountGrid()
    expect(state.editorCaretSegmentId.value).toBeNull()
    down(13, { shiftKey: true })
    up(numCell(13))
    expect(selection.segmentSelectionIds.value).toEqual([13])
  })

  it('thả trên ô bản dịch hoặc nguyên văn ⇒ caret không đổi, ô không nhận tiêu điểm', async () => {
    const { state, selection } = await mountGrid()
    state.setEditorCaret(11)
    const before = document.activeElement
    for (const col of ['tgt', 'src'] as const) {
      down(12)
      over(cell(col, 13))
      up(cell(col, 13))
      expect(state.editorCaretSegmentId.value).toBe(11)
      expect(document.activeElement).toBe(before)
    }
    expect(selection.segmentSelectionIds.value).toEqual([12, 13])
  })

  it('thả trên nguyên văn với Mod giữ ⇒ không ghi điểm cắt', async () => {
    const { state } = await mountGrid()
    const src = cell('src', 13)
    const text = src.querySelector('.src-piece')?.firstChild
    if (text === null || text === undefined) throw new Error('no source text')
    Object.defineProperty(document, 'caretRangeFromPoint', {
      configurable: true,
      value: () => ({ startContainer: text, startOffset: 1 }),
    })
    try {
      down(12)
      fire(src, 'mouseup', { buttons: 0, metaKey: true, ctrlKey: true })
      expect(state.editorSourceCut.value).toBeNull()
    } finally {
      Reflect.deleteProperty(document, 'caretRangeFromPoint')
    }
  })

  it('chuột trôi sang ô khác của câu vẫn dời focus', async () => {
    const { selection } = await mountGrid()
    down(11)
    over(cell('tgt', 13))
    expect(selection.segmentSelectionIds.value).toEqual([11, 12, 13])
    up(cell('tgt', 13))
  })

  it('thả ngoài cửa sổ: mouseover kế tiếp có buttons 0 ⇒ kéo kết thúc, không dời focus', async () => {
    const { selection } = await mountGrid()
    down(11)
    over(numCell(12))
    over(numCell(13), { buttons: 0 })
    expect(selection.segmentSelectionIds.value).toEqual([11, 12])
    over(numCell(13))
    expect(selection.segmentSelectionIds.value).toEqual([11, 12])
  })

  it('nhấn phụ ⇒ vùng chọn không đổi', async () => {
    const { selection } = await mountGrid()
    down(11)
    up(numCell(11))
    down(13, { button: 2, buttons: 2 })
    expect(selection.segmentSelectionIds.value).toEqual([11])
  })

  it('Ctrl+nút trái trên macOS là nhấn phụ ⇒ vùng chọn không đổi', async () => {
    vi.spyOn(navigator, 'platform', 'get').mockReturnValue('MacIntel')
    const { selection } = await mountGrid()
    down(11)
    up(numCell(11))
    down(13, { ctrlKey: true })
    expect(selection.segmentSelectionIds.value).toEqual([11])
  })

  it('nhấn trên đệm cột số ⇒ không làm gì', async () => {
    const { selection } = await mountGrid()
    const pad = document.createElement('div')
    pad.className = 'strip-pad'
    numCell(11).parentElement?.appendChild(pad)
    const event = fire(pad, 'mousedown')
    expect(event.defaultPrevented).toBe(false)
    expect(selection.segmentSelectionIds.value).toEqual([])
  })

  it('nhấn ngoài ô số ⇒ không làm gì và không gắn listener kéo', async () => {
    const { selection } = await mountGrid()
    const add = vi.spyOn(document, 'addEventListener')
    const event = fire(cell('tgt', 12), 'mousedown')
    expect(event.defaultPrevented).toBe(false)
    expect(selection.segmentSelectionIds.value).toEqual([])
    expect(add.mock.calls.filter(([type]) => type === 'mouseover' || type === 'mouseup')).toEqual([])
  })

  it('gỡ Lưới giữa lượt kéo ⇒ gỡ cả hai listener', async () => {
    const { wrapper } = await mountGrid()
    const remove = vi.spyOn(document, 'removeEventListener')
    down(11)
    wrapper.unmount()
    mounted.splice(0)
    const types = remove.mock.calls.map(([type]) => type)
    expect(types).toContain('mouseover')
    expect(types).toContain('mouseup')
  })
})
