import { beforeEach, describe, expect, it, vi } from 'vitest'
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
  return wrapper
}

function targetCell(id: number): HTMLElement {
  const el = document.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${id}"]`)
  if (el === null) throw new Error(`ô bản dịch của câu ${id} không có trong DOM`)
  return el
}

const afterReassert = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 50))

beforeEach(() => {
  resetRecorder()
  document.body.innerHTML = ''
})

describe('onCellMouseUp — vùng bôi đen bằng chuột', () => {
  it('nhả chuột sau khi kéo giữ nguyên đoạn đã bôi đen, qua cả lượt vá ở frame sau', async () => {
    const wrapper = await mountEditor()
    const cell = targetCell(11)
    const text = cell.firstChild as Text
    const range = document.createRange()
    range.setStart(text, 0)
    range.setEnd(text, 3)
    window.getSelection()?.removeAllRanges()
    window.getSelection()?.addRange(range)

    cell.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: 1, clientY: 1 }))
    await afterReassert()

    expect(document.activeElement).toBe(cell)
    expect(window.getSelection()?.toString()).toBe('Hắn')
    wrapper.unmount()
  })

  it('cú bấm đơn vẫn đặt caret', async () => {
    const wrapper = await mountEditor()
    const cell = targetCell(11)
    window.getSelection()?.removeAllRanges()

    cell.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: 1, clientY: 1 }))
    await afterReassert()

    expect(document.activeElement).toBe(cell)
    expect(window.getSelection()?.isCollapsed).toBe(true)
    expect(window.getSelection()?.rangeCount).toBe(1)
    wrapper.unmount()
  })
})
