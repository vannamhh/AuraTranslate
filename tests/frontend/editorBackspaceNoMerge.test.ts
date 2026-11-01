import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'

const mergeSegmentsMock = vi.fn(() => Promise.resolve({ outcome: null, error: null }))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    mergeSegments: mergeSegmentsMock,
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }
const mounted: { unmount: () => void }[] = []

async function mountEditor() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const commands = await import('../../src/commands')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    mergeSegments: () => {
      void state.mergeCurrentSegment()
    },
  })
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  mounted.push(wrapper)
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, commands, wrapper }
}

function targetCell(segmentId: number): HTMLElement {
  const cell = document.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
  if (cell === null) throw new Error(`no target cell for segment ${segmentId}`)
  return cell
}

function placeCaretAtStart(cell: HTMLElement): void {
  cell.focus()
  const range = document.createRange()
  range.setStart(cell.firstChild ?? cell, 0)
  range.collapse(true)
  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
}

function pressDelete(cell: HTMLElement): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key: 'Backspace', code: 'Backspace', bubbles: true, cancelable: true })
  cell.dispatchEvent(event)
  return event
}

const secondSegment = FIXTURE_SEGMENTS[1]?.id ?? -1

beforeAll(async () => {
  await import('../../src/panels/GridPanel.vue')
})

beforeEach(() => {
  resetRecorder()
  mergeSegmentsMock.mockClear()
})

afterEach(() => {
  for (const wrapper of mounted.splice(0)) wrapper.unmount()
  document.body.innerHTML = ''
})

describe('Backspace ở đầu ô bản dịch không gộp câu', () => {
  it('ô có chữ, caret đầu ô, bấm năm cú: không gọi lệnh gộp và để phím cho engine', async () => {
    const { state, wrapper } = await mountEditor()
    state.setEditorCaret(secondSegment)
    const cell = targetCell(secondSegment)
    placeCaretAtStart(cell)

    for (let press = 0; press < 5; press += 1) {
      expect(pressDelete(cell).defaultPrevented).toBe(false)
      await wrapper.vm.$nextTick()
    }

    expect(mergeSegmentsMock).not.toHaveBeenCalled()
    expect(document.querySelectorAll('[data-col="tgt"]').length).toBe(FIXTURE_SEGMENTS.length)
  })

  it('ô đã xoá hết chữ, bấm thêm: không gọi lệnh gộp', async () => {
    const { state, wrapper } = await mountEditor()
    state.setEditorCaret(secondSegment)
    const cell = targetCell(secondSegment)
    cell.textContent = ''
    placeCaretAtStart(cell)

    expect(pressDelete(cell).defaultPrevented).toBe(false)
    await wrapper.vm.$nextTick()

    expect(mergeSegmentsMock).not.toHaveBeenCalled()
  })

  it('lệnh gộp vẫn tới IPC khi đi qua `editor.merge_segments`', async () => {
    const { state, commands } = await mountEditor()
    state.setEditorCaret(secondSegment)

    commands.dispatch('editor.merge_segments')
    await vi.waitFor(() => {
      expect(mergeSegmentsMock).toHaveBeenCalledWith(secondSegment)
    })
  })
})
