import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, resetRecorder, recordSave } from './support/segmentFixture'

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

type Frame = () => void
let frames: Frame[] = []

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
  frames = []
  vi.stubGlobal('requestAnimationFrame', (cb: Frame) => {
    frames.push(cb)
    return frames.length
  })
  vi.stubGlobal('cancelAnimationFrame', () => {})
})

afterEach(() => {
  vi.unstubAllGlobals()
})

function runFrames(): void {
  const pending = frames
  frames = []
  for (const cb of pending) cb()
}

async function loadModules() {
  vi.resetModules()
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  return { GridPanel, state }
}

async function mountGrid(GridPanel: Awaited<ReturnType<typeof loadModules>>['GridPanel'], state: Awaited<ReturnType<typeof loadModules>>['state']) {
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  return wrapper
}

function scrollEl(root: Element): HTMLElement {
  return root.querySelector('.grid-scroll') as HTMLElement
}

function scrollTo(el: HTMLElement, top: number): void {
  el.scrollTop = top
  el.dispatchEvent(new Event('scroll'))
}

describe('GridPanel scroll memory', () => {
  it('scrollTop returns after a remount without resetEditorPanel (dock preset change)', async () => {
    const { GridPanel, state } = await loadModules()
    const first = await mountGrid(GridPanel, state)
    scrollTo(scrollEl(first.element), 600)
    first.unmount()

    const second = await mountGrid(GridPanel, state)
    const el = scrollEl(second.element)
    expect(el.scrollTop).toBe(0)
    runFrames()
    expect(el.scrollTop).toBe(600)
    second.unmount()
  })

  it('a grid rebuilt after resetEditorPanel starts at the top for the same chapterId', async () => {
    const { GridPanel, state } = await loadModules()
    const first = await mountGrid(GridPanel, state)
    const chapterBefore = state.editorChapterId.value
    scrollTo(scrollEl(first.element), 600)
    first.unmount()

    state.resetEditorPanel()
    const second = await mountGrid(GridPanel, state)
    expect(state.editorChapterId.value).toBe(chapterBefore)
    runFrames()
    expect(scrollEl(second.element).scrollTop).toBe(0)
    second.unmount()
  })

  it('the position survives a second remount', async () => {
    const { GridPanel, state } = await loadModules()
    const first = await mountGrid(GridPanel, state)
    scrollTo(scrollEl(first.element), 600)
    first.unmount()

    const second = await mountGrid(GridPanel, state)
    const el = scrollEl(second.element)
    runFrames()
    el.dispatchEvent(new Event('scroll'))
    second.unmount()

    const third = await mountGrid(GridPanel, state)
    runFrames()
    expect(scrollEl(third.element).scrollTop).toBe(600)
    third.unmount()
  })
})
