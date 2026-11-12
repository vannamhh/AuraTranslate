import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS, FIXTURE_CHAPTER_ID, readFixture, recordSave, resetRecorder } from './support/segmentFixture'

const override = vi.hoisted(() => ({
  segments: null as ChapterSegment[] | null,
  merge: null as (() => Promise<unknown>) | null,
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: (...args: Parameters<typeof readFixture>) =>
      override.segments === null
        ? readFixture(...args)
        : Promise.resolve({
            loaded: {
              chapter_id: FIXTURE_CHAPTER_ID,
              segments: override.segments,
              caret_segment_id: null,
              assets: [],
              assets_dir: '',
              tm_filled_segment_ids: [],
              tm_prefill: { kind: 'ran', hidden_count: 0 },
            },
            error: null,
          }),
    saveSegmentTargets: recordSave,
    mergeSegments: () => override.merge?.() ?? Promise.resolve({ outcome: null, error: null }),
  }
})

const BASE_ID = 1000
const manySegments = (count: number): ChapterSegment[] =>
  Array.from({ length: count }, (_, i) => ({
    ...FIXTURE_SEGMENTS[0],
    id: BASE_ID + i,
    ord: i + 1,
    source_text: `Cau ${String(i)}.`,
    target_text: 'Dich chung.',
    status: 'draft',
    is_paragraph_end: false,
    is_target_paragraph_end: false,
  }))

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

async function mountEditor() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, wrapper }
}

function targetCell(id: number): HTMLElement {
  const el = document.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${id}"]`)
  if (el === null) throw new Error(`ô bản dịch của câu ${id} không có trong DOM`)
  return el
}

function typeInto(id: number, text: string): void {
  const cell = targetCell(id)
  ;(cell.firstChild as Text).data = text
  cell.dispatchEvent(new Event('input', { bubbles: true }))
}

function clearByTyping(id: number): void {
  const cell = targetCell(id)
  cell.replaceChildren()
  cell.dispatchEvent(new Event('input', { bubbles: true }))
}

beforeEach(() => {
  override.segments = null
  override.merge = null
  resetRecorder()
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('restoreEditedText — đặt lại chữ đang gõ dở sau một lượt lưới dựng lại', () => {
  it('ô có chữ chờ ghi lấy lại chữ đó khi ảnh chụp đổi bên dưới nó; ô khác giữ chữ trên đĩa', async () => {
    const { state, wrapper } = await mountEditor()
    typeInto(12, 'Gió thổi tới từ đầu hành lang.')
    await wrapper.vm.$nextTick()

    state.replaceEditorSegment(12, { target_text: 'bản trên đĩa' })
    await wrapper.vm.$nextTick()

    expect(targetCell(12).textContent).toBe('Gió thổi tới từ đầu hành lang.')
    expect(targetCell(11).textContent).toBe('Hắn đẩy cánh cửa ấy ra.')
    wrapper.unmount()
  })

  it('không quét mọi ô của cột bản dịch, chỉ tra các id trong tập chờ', async () => {
    const { state, wrapper } = await mountEditor()
    typeInto(12, 'Gió thổi tới từ đầu hành lang.')
    await wrapper.vm.$nextTick()

    const scan = vi.spyOn(Element.prototype, 'querySelectorAll')
    state.replaceEditorSegment(12, { target_text: 'bản trên đĩa' })
    await wrapper.vm.$nextTick()

    const wholeColumnScans = scan.mock.calls.filter(([selector]) => selector === '[data-segment-id]')
    expect(wholeColumnScans).toHaveLength(0)
    expect(targetCell(12).textContent).toBe('Gió thổi tới từ đầu hành lang.')
    wrapper.unmount()
  })

  it('ô đã xoá trắng rồi ghi xuống nhường cho một lượt ghi không qua gõ phím: ô hiện văn bản mới', async () => {
    const { state, wrapper } = await mountEditor()
    clearByTyping(12)
    await state.flushEditorNow()

    state.replaceEditorSegment(12, { target_text: 'Kết quả AI vừa ghi.' })
    await wrapper.vm.$nextTick()

    expect(targetCell(12).textContent).toBe('Kết quả AI vừa ghi.')
    expect(state.editorEditedText.value.get(12)).toBe('Kết quả AI vừa ghi.')
    wrapper.unmount()
  })

  it('gỡ rồi gắn lại GridPanel: cả 20 ô sửa chưa lưu hiện chữ đã gõ', async () => {
    override.segments = manySegments(30)
    const { state, wrapper } = await mountEditor()
    for (let i = 0; i < 20; i += 1) typeInto(BASE_ID + i, `Chữ đã gõ ${String(i)}.`)
    await wrapper.vm.$nextTick()
    wrapper.unmount()
    document.body.innerHTML = ''

    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const again = mount(GridPanel, {
      props: { params: {} } as never,
      global: { stubs: STUBS },
      attachTo: document.body,
    })
    await again.vm.$nextTick()

    for (let i = 0; i < 20; i += 1) {
      expect(targetCell(BASE_ID + i).textContent).toBe(`Chữ đã gõ ${String(i)}.`)
    }
    expect(targetCell(BASE_ID + 25).textContent).toBe('Dich chung.')
    expect(state.editorEditedText.value.size).toBe(20)
    again.unmount()
  })

  it('gộp câu đẩy một hàng sửa chưa lưu sang khối khác: ô mới gắn vẫn hiện chữ đã gõ', async () => {
    const rows = manySegments(205)
    override.segments = rows
    const { state, wrapper } = await mountEditor()
    const moving = BASE_ID + 200
    typeInto(moving, 'Chữ đang gõ dở ở khối sau.')
    await wrapper.vm.$nextTick()

    const merged: ChapterSegment = { ...rows[0], id: 9000, target_text: 'Dich gop.' }
    override.merge = () =>
      Promise.resolve({ outcome: { retired: [rows[0], rows[1]], new_segments: [merged] }, error: null })
    state.setEditorCaret(rows[0].id)
    expect(await state.mergeCurrentSegment()).toBe('done')
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(state.editorSegments.value).toHaveLength(204)
    expect(targetCell(moving).textContent).toBe('Chữ đang gõ dở ở khối sau.')
    wrapper.unmount()
  })

  it('ô đang giữ tiêu điểm không bị ghi khi lượt chép lại chạy', async () => {
    const { state, wrapper } = await mountEditor()
    typeInto(12, 'Chữ đã gõ.')
    await wrapper.vm.$nextTick()
    const cell = targetCell(12)
    cell.focus()
    expect(document.activeElement).toBe(cell)

    state.replaceEditorSegment(12, { target_text: 'bản trên đĩa' })
    await wrapper.vm.$nextTick()

    expect(cell.textContent).toBe('bản trên đĩa')
    wrapper.unmount()
  })
})
