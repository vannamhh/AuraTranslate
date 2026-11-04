/** The reading-marks list as a dock panel: shown from state, never persisted, first to give way. */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

const mockInvoke = vi.fn()
const mockOpenChapterById = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: () => {},
}))

import WorkspaceDock from '../../src/layout/WorkspaceDock.vue'
import { enterFocus, installCommands } from '../../src/commands'
import { currentLayoutJson, panelRing, togglePanel } from '../../src/layout/dockController'
import { visibleDockPanels } from '../../src/layout/panelVisibilityState'
import * as state from '../../src/modes/readingState'
import { setMode } from '../../src/modes/modeState'

const MARK = (id: number, chapter: number) => ({
  segment_id: id,
  navigation_segment_id: id,
  chapter_id: chapter,
  chapter_ord: chapter,
  chapter_title: null,
  source_text: '原文',
  target_text: `Câu ${String(id)}`,
  is_retired: false,
  marked_at: '2026-11-04T00:00:00.000Z',
})

async function settle(): Promise<void> {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

function setWindow(width: number, height: number): void {
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: width })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: height })
  window.dispatchEvent(new Event('resize'))
}

let commandsInstalled = false
beforeEach(() => {
  if (!commandsInstalled) {
    installCommands({
      isMac: true,
      setMode: () => {},
      nextReadingMark: state.nextReadingMark,
      prevReadingMark: state.prevReadingMark,
      unmarkCurrentReadingMark: () => void state.unmarkCurrentReadingMark(),
      closeReadingMarksPanel: state.closeReadingMarksPanel,
    } as never)
    commandsInstalled = true
  }
  mockInvoke.mockReset()
  mockInvoke.mockResolvedValue(null)
  mockOpenChapterById.mockReset()
  mockOpenChapterById.mockResolvedValue(true)
  document.documentElement.style.setProperty('--space-titlebar-height', '40px')
  document.documentElement.style.setProperty('--space-status-height', '34px')
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: 1200 })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: 1000 })
  state.resetReading()
  setMode('reading')
})

afterEach(() => {
  state.resetReading()
})

async function openFromReading(): Promise<void> {
  mockInvoke.mockImplementation((cmd: string) =>
    Promise.resolve(cmd === 'list_reading_marks' ? [MARK(12, 4), MARK(13, 5)] : null),
  )
  await state.openReadingMarks()
  await state.openCurrentReadingMark()
}

describe('panel.reading_marks in the Workspace dock', () => {
  it('appears after Sửa →, lists every row, and stays out of the saved layout and the panel switches', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      await openFromReading()
      await settle()

      expect(panelRing()).toContain('panel.reading_marks')
      expect(wrapper.findAll('.marks-row')).toHaveLength(2)
      expect(wrapper.findAll('.marks-row')[0]?.classes()).toContain('current')
      expect(visibleDockPanels.value.has('panel.reading_marks' as never)).toBe(false)
      expect(currentLayoutJson()).not.toContain('reading_marks')
      expect(currentLayoutJson()).not.toContain('readingMarks')
    } finally {
      wrapper.unmount()
    }
  })

  it('a retired row keeps its "Câu này đã đổi" note in the Workspace panel', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      mockInvoke.mockImplementation((cmd: string) =>
        Promise.resolve(cmd === 'list_reading_marks' ? [MARK(12, 4), { ...MARK(13, 5), is_retired: true }] : null),
      )
      await state.openReadingMarks()
      await state.openCurrentReadingMark()
      await settle()

      const rows = wrapper.findAll('.marks-row')
      expect(rows).toHaveLength(2)
      expect(rows[0]?.find('.retired-note').exists()).toBe(false)
      expect(rows[1]?.find('.retired-note').text()).toBe('Câu này đã đổi')
    } finally {
      wrapper.unmount()
    }
  })

  it('Bỏ dấu on the last row leaves the empty message, and Đóng removes the panel and lands focus in the grid', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      await openFromReading()
      await settle()

      mockInvoke.mockResolvedValue(true)
      for (let left = 2; left > 0; left -= 1) {
        await wrapper.get('.marks-row button:nth-of-type(2)').trigger('click')
        await vi.waitFor(() => expect(wrapper.findAll('.marks-row')).toHaveLength(left - 1))
      }
      expect(wrapper.text()).toContain('chưa có chỗ nào được đánh dấu')

      const close = wrapper.findAll('.marks-actions .btn').at(-1)
      await close?.trigger('click')
      await settle()
      expect(panelRing()).not.toContain('panel.reading_marks')
      await vi.waitFor(() => {
        const active = document.activeElement
        expect(active).not.toBe(document.body)
        expect(active?.closest('section.panel')?.getAttribute('data-panel-owner')).toBe('panel.grid')
      })
    } finally {
      wrapper.unmount()
    }
  })

  it('entering the panel puts focus on the list so the arrow keys move the cursor', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      await openFromReading()
      await settle()

      expect(enterFocus('panel.reading_marks')).toBe(true)
      expect(document.activeElement?.classList.contains('marks-body')).toBe(true)
      document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }))
      expect(state.readingMarksCursor.value).toBe(1)
      document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }))
      expect(state.readingMarksCursor.value).toBe(0)
      expect(wrapper.text()).not.toContain('Mở Workspace tại đúng câu')
    } finally {
      wrapper.unmount()
    }
  })

  it('goes when the window narrows, comes back with the list at the default short window and stays at full', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      await openFromReading()
      await settle()

      setWindow(1000, 1000)
      await settle()
      expect(panelRing()).toEqual(['panel.grid'])
      expect(state.readingMarks.value).toHaveLength(2)

      setWindow(1280, 860)
      await settle()
      expect(panelRing()).toContain('panel.reading_marks')
      expect(wrapper.findAll('.marks-row')).toHaveLength(2)

      setWindow(1200, 1000)
      await settle()
      expect(panelRing()).toContain('panel.reading_marks')
      expect(wrapper.findAll('.marks-row')).toHaveLength(2)
    } finally {
      wrapper.unmount()
    }
  })

  it('a manual panel toggle leaves the marks panel alone', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      await openFromReading()
      await settle()
      expect(togglePanel('panel.lookup')).toBe(true)
      await settle()
      expect(panelRing()).toContain('panel.reading_marks')
      expect(panelRing()).not.toContain('panel.lookup')
    } finally {
      wrapper.unmount()
    }
  })

  it('hiding the grid is refused when only the transient panel would be left', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()
      await openFromReading()
      await settle()
      expect(togglePanel('panel.lookup')).toBe(true)
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      expect(togglePanel('panel.grid')).toBe(false)
      expect(panelRing()).toContain('panel.grid')
    } finally {
      wrapper.unmount()
    }
  })

  it('a layout restored after a restart has no marks panel', async () => {
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    let saved = ''
    try {
      await settle()
      await openFromReading()
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      saved = currentLayoutJson() ?? ''
    } finally {
      wrapper.unmount()
    }
    state.resetReading()
    const restored = mount(WorkspaceDock, { props: { savedLayout: saved }, attachTo: document.body })
    try {
      await settle()
      expect(saved).not.toBe('')
      expect(panelRing()).not.toContain('panel.reading_marks')
    } finally {
      restored.unmount()
    }
  })
})
