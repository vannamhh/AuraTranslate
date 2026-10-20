import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

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

beforeEach(() => {
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
})
