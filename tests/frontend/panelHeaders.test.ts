/** Panel tab bars: hidden for single-panel dock groups unless arrange mode is on. */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

import WorkspaceDock from '../../src/layout/WorkspaceDock.vue'
import { applyLayoutJson } from '../../src/layout/dockController'
import { resetPanelHeadersShown, setPanelHeadersShown } from '../../src/layout/panelHeadersState'

const FULL = { width: 1200, height: 1000 }
const SHORT = { width: 1200, height: 800 }

const TABBED_LAYOUT =
  '{"grid":{"root":{"type":"branch","data":[{"type":"leaf","data":{"views":["panel.grid"],"activeView":"panel.grid","id":"1"},"size":578},{"type":"leaf","data":{"views":["panel.lookup","panel.ai_translation"],"activeView":"panel.lookup","id":"2"},"size":907}],"size":833},"width":1485,"height":833,"orientation":"HORIZONTAL"},"panels":{"panel.grid":{"id":"panel.grid","contentComponent":"grid","tabComponent":"aura","params":{"titleKey":"panel.grid.title"},"title":"panel.grid"},"panel.lookup":{"id":"panel.lookup","contentComponent":"lookup","tabComponent":"aura","params":{"titleKey":"panel.lookup.title"},"title":"panel.lookup"},"panel.ai_translation":{"id":"panel.ai_translation","contentComponent":"aiTranslation","tabComponent":"aura","params":{"titleKey":"panel.ai_translation.title"},"title":"panel.ai_translation"}},"activeGroup":"1"}'

function setWindowSize(width: number, height: number): void {
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: width })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: height })
}

function setChromeTokens(): void {
  document.documentElement.style.setProperty('--space-titlebar-height', '40px')
  document.documentElement.style.setProperty('--space-status-height', '34px')
}

async function settle(): Promise<void> {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

/** One entry per dock group: whether its tab bar is hidden. */
function headerHiddenFlags(): boolean[] {
  return Array.from(document.querySelectorAll<HTMLElement>('.dv-tabs-and-actions-container')).map(
    (el) => el.style.display === 'none',
  )
}

beforeEach(() => {
  mockInvoke.mockReset()
  mockInvoke.mockRejectedValue({ code: 'x', message_key: 'err.unknown', params: {}, retryable: true })
  resetPanelHeadersShown()
  setChromeTokens()
  setWindowSize(FULL.width, FULL.height)
})

afterEach(() => {
  document.body.innerHTML = ''
})

async function mountDock(savedLayout = '') {
  const wrapper = mount(WorkspaceDock, { props: { savedLayout }, attachTo: document.body })
  await settle()
  return wrapper
}

describe('panel tab bars', () => {
  it('first open: three single-panel groups, no tab bar shown', async () => {
    const wrapper = await mountDock()
    try {
      expect(headerHiddenFlags()).toEqual([true, true, true])
    } finally {
      wrapper.unmount()
    }
  })

  it('arrange mode on shows every tab bar, off hides single-panel groups again', async () => {
    const wrapper = await mountDock()
    try {
      await setPanelHeadersShown(true)
      await settle()
      expect(headerHiddenFlags()).toEqual([false, false, false])
      await setPanelHeadersShown(false)
      await settle()
      expect(headerHiddenFlags()).toEqual([true, true, true])
    } finally {
      wrapper.unmount()
    }
  })

  it('tabs drag with pointer events, not HTML5 drag-and-drop the native webview swallows', async () => {
    const wrapper = await mountDock()
    try {
      const tabs = Array.from(document.querySelectorAll<HTMLElement>('.dv-tab'))
      expect(tabs.length).toBeGreaterThanOrEqual(3)
      expect(tabs.filter((tab) => tab.draggable)).toEqual([])
    } finally {
      wrapper.unmount()
    }
  })

  it('a group with two panels keeps its tab bar when arrange mode is off', async () => {
    const wrapper = await mountDock(TABBED_LAYOUT)
    try {
      expect(headerHiddenFlags().filter((hidden) => hidden)).toHaveLength(1)
      expect(headerHiddenFlags().filter((hidden) => !hidden)).toHaveLength(1)
    } finally {
      wrapper.unmount()
    }
  })

  it('narrow-tier merge shows the merged group tab bar in the same tick, and undoing hides it again', async () => {
    const wrapper = await mountDock()
    try {
      setWindowSize(SHORT.width, SHORT.height)
      window.dispatchEvent(new Event('resize'))
      expect(headerHiddenFlags().filter((hidden) => !hidden)).toHaveLength(1)
      await settle()
      expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBe('true')

      setWindowSize(FULL.width, FULL.height)
      window.dispatchEvent(new Event('resize'))
      expect(headerHiddenFlags().every((hidden) => hidden)).toBe(true)
      await settle()
      expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBeUndefined()
    } finally {
      wrapper.unmount()
    }
  })

  it('the saved layout never carries hideHeader, and toggling the mode schedules no layout write', async () => {
    const wrapper = await mountDock()
    try {
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      const writes = wrapper.emitted('persist') ?? []
      expect(writes.length).toBeGreaterThan(0)
      expect(String(writes.at(-1)?.[0])).not.toContain('hideHeader')

      await setPanelHeadersShown(true)
      await settle()
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      expect((wrapper.emitted('persist') ?? []).length).toBe(writes.length)
    } finally {
      wrapper.unmount()
    }
  })

  it('a layout saved with hidden tab bars restores with them shown while arrange mode is on', async () => {
    const wrapper = await mountDock()
    try {
      const hiddenJson = TABBED_LAYOUT.replace('"id":"1"', '"id":"1","hideHeader":true')
      await setPanelHeadersShown(true)
      expect(applyLayoutJson(hiddenJson)).toBe(true)
      await settle()
      expect(headerHiddenFlags().every((hidden) => !hidden)).toBe(true)
    } finally {
      wrapper.unmount()
    }
  })

  it('a failed config write keeps the chosen state and reports the error', async () => {
    const state = await import('../../src/layout/panelHeadersState')
    await state.setPanelHeadersShown(true)
    expect(state.panelHeadersShown.value).toBe(true)
    expect(state.panelHeadersSaveError.value?.code).toBe('x')
  })
})

describe('panelHeadersShown at startup', () => {
  const wire = (shown: unknown) => ({
    theme: 'light',
    mode: 'library',
    shortcuts: {},
    layout_presets: {},
    workspace_layout: '',
    dict_sources_disabled: '',
    ...(shown === undefined ? {} : { panel_headers_shown: shown }),
  })

  it.each([
    [true, true],
    [false, false],
    [undefined, false],
    ['on', false],
  ])('wire %j gives %j', async (raw, expected) => {
    vi.resetModules()
    mockInvoke.mockReset()
    mockInvoke.mockResolvedValueOnce(wire(raw))
    const bootstrap = await import('../../src/config/bootstrap')
    await bootstrap.loadBootstrapConfig()
    const state = await import('../../src/layout/panelHeadersState')
    expect(state.panelHeadersShown.value).toBe(expected)
  })
})
