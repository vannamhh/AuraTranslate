import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, recordSave } from './support/segmentFixture'

const openProofreadFindingAtSpy = vi.fn()
const openConventionAtSpy = vi.fn()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})
vi.mock('../../src/conventionState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/conventionState')>()
  return {
    ...actual,
    openProofreadFindingAt: (...args: [number, number]) => openProofreadFindingAtSpy(...args),
    openConventionAt: (...args: [number, number]) => openConventionAtSpy(...args),
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }
const mounted: { unmount: () => void }[] = []

afterEach(() => {
  for (const w of mounted.splice(0)) w.unmount()
  document.body.innerHTML = ''
  openProofreadFindingAtSpy.mockReset()
  openConventionAtSpy.mockReset()
  window.getSelection()?.removeAllRanges()
})

async function mountGridWithMark(className = 'proofread-error', dataKey: 'proofreadStart' | 'conventionStart' = 'proofreadStart') {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const commands = await import('../../src/commands')
  commands.installCommands({ isMac: true, setMode: () => {} })
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  mounted.push(wrapper)
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  const cell = document.querySelector<HTMLElement>('[data-col="tgt"][data-segment-id="11"]')
  expect(cell).not.toBeNull()
  const mark = document.createElement('span')
  mark.className = className
  mark.dataset[dataKey] = '4'
  mark.textContent = 'lỗi'
  cell?.appendChild(mark)
  return mark
}

describe('bấm chữ gạch chân của Proofreader trong lưới', () => {
  it.each(['proofread-error', 'proofread-meaning'])('mở lại dải ở đúng câu và vị trí của lỗi %s được bấm', async (className) => {
    const mark = await mountGridWithMark(className)

    mark.dispatchEvent(new MouseEvent('click', { bubbles: true, button: 0 }))

    expect(openProofreadFindingAtSpy).toHaveBeenCalledExactlyOnceWith(11, 4)
    expect(openConventionAtSpy).not.toHaveBeenCalled()
  })

  it('bấm gạch chân quy ước vẫn mở dải quy ước, không mở Proofreader', async () => {
    const mark = await mountGridWithMark('convention-mark', 'conventionStart')

    mark.dispatchEvent(new MouseEvent('click', { bubbles: true, button: 0 }))

    expect(openConventionAtSpy).toHaveBeenCalledExactlyOnceWith(11, 4)
    expect(openProofreadFindingAtSpy).not.toHaveBeenCalled()
  })

  it('không mở gì khi người dùng vừa bôi đen chữ', async () => {
    const mark = await mountGridWithMark()
    const text = mark.firstChild as Text
    window.getSelection()?.setBaseAndExtent(text, 0, text, 2)

    mark.dispatchEvent(new MouseEvent('click', { bubbles: true, button: 0 }))

    expect(openProofreadFindingAtSpy).not.toHaveBeenCalled()
  })
})
