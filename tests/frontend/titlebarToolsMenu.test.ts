import { afterEach, beforeEach, describe, expect, it, onTestFinished, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import type { CommandDeps } from '../../src/commands'
import type { PanelId } from '../../src/layout/workspaceLayout'
import { installFakeDragJs, pressDragHeader } from './support/dragRegionPress'

async function setup(extra: Partial<CommandDeps> = {}) {
  vi.resetModules()
  const commands = await import('../../src/commands')
  const state = await import('../../src/titlebarToolsState')
  const keymap = commands.installCommands({
    setMode: vi.fn(),
    isMac: false,
    toggleToolsMenu: state.toggleToolsMenu,
    ...extra,
  } as CommandDeps)
  const Menu = (await import('../../src/TitlebarToolsMenu.vue')).default
  const i18n = await import('../../src/i18n')
  return { commands, keymap, state, Menu, i18n }
}

async function flush(): Promise<void> {
  for (let i = 0; i < 4; i += 1) await Promise.resolve()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('Công cụ ▾ list', () => {
  it('lists exactly nine items in IA order, showing a chord only where the command has one', async () => {
    const { state, Menu, i18n } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    expect(wrapper.findAll('[role="menuitem"]')).toHaveLength(0)
    state.toggleToolsMenu()
    await flush()

    const rows = wrapper.findAll('[role="menuitem"]').map((item) => item.findAll('.ui-row > span').map((s) => s.text()))
    expect(rows.map((r) => r[0])).toEqual(
      [
        'command.glossary.manage.open',
        'command.glossary.settings.open',
        'command.tm.manage.open',
        'command.settings.open',
        'command.ai.proofread.memory.open',
        'command.export.reviewer_import.open',
        'command.export.alignment.open',
        'command.glossary.harvest.open',
      ].map((key) => i18n.t(key)),
    )
    expect(rows.map((r) => r[1])).toEqual(['Ctrl+Alt+M', 'Ctrl+Alt+T', 'Ctrl+Alt+B', 'Ctrl+,', '', '', '', ''])
    wrapper.unmount()
  })

  it('the trigger carries all seven opener hooks and the items carry none', async () => {
    const { state, Menu } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    state.toggleToolsMenu()
    await flush()
    const hooks = [
      'glossary-manage',
      'tm-manage',
      'settings',
      'proofread-memory',
      'reviewer-import',
      'alignment',
      'glossary-harvest',
    ]
    for (const hook of hooks) expect(wrapper.get('.tools-trigger').attributes()).toHaveProperty(`data-${hook}-open`)
    for (const item of wrapper.findAll('[role="menuitem"]')) {
      expect(Object.keys(item.attributes()).filter((a) => a.endsWith('-open'))).toEqual([])
    }
    wrapper.unmount()
  })

  it('arrow keys walk the items, Escape closes and returns focus to the trigger', async () => {
    const { state, Menu } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    const trigger = wrapper.get('.tools-trigger')
    ;(trigger.element as HTMLElement).focus()
    state.toggleToolsMenu()
    await flush()

    await trigger.trigger('keydown', { key: 'ArrowDown' })
    const items = wrapper.findAll('[role="menuitem"], [role="menuitemcheckbox"]')
    expect(document.activeElement).toBe(items[0]?.element)
    await items[0]?.trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(items[1]?.element)
    await items[1]?.trigger('keydown', { key: 'ArrowUp' })
    await items[0]?.trigger('keydown', { key: 'ArrowUp' })
    expect(document.activeElement).toBe(items[8]?.element)

    await items[8]?.trigger('keydown', { key: 'Escape' })
    expect(state.toolsMenuIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(trigger.element)
    wrapper.unmount()
  })

  it('choosing an item dispatches its command and closes the list; an outside press closes it too', async () => {
    const openTmManage = vi.fn()
    const { state, Menu } = await setup({ openTmManage })
    const wrapper = mount(Menu, { attachTo: document.body })
    state.toggleToolsMenu()
    await flush()

    await wrapper.findAll('[role="menuitem"]')[2]?.trigger('click')
    expect(openTmManage).toHaveBeenCalledTimes(1)
    expect(state.toolsMenuIsOpen.value).toBe(false)

    state.toggleToolsMenu()
    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    expect(state.toolsMenuIsOpen.value).toBe(false)
    wrapper.unmount()
  })

  it('a press on a drag-region header closes the open list although drag.js swallows the bubble phase', async () => {
    installFakeDragJs()
    const { state, Menu } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    onTestFinished(() => wrapper.unmount())
    state.toggleToolsMenu()
    await flush()
    expect(state.toolsMenuIsOpen.value).toBe(true)
    pressDragHeader()
    expect(state.toolsMenuIsOpen.value).toBe(false)
  })

  it('the trigger opens and closes the list through titlebar.tools.toggle', async () => {
    const { state, Menu } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    await wrapper.get('.tools-trigger').trigger('click')
    expect(state.toolsMenuIsOpen.value).toBe(true)
    expect(wrapper.findAll('[role="menuitem"]')).toHaveLength(8)
    await wrapper.get('.tools-trigger').trigger('click')
    expect(state.toolsMenuIsOpen.value).toBe(false)
    wrapper.unmount()
  })
})

describe('Công cụ ▾ — Ngưỡng quét Glossary', () => {
  it('the item sits under Quản lý Glossary and dispatches glossary.settings.open, then closes the list', async () => {
    const openGlossarySettings = vi.fn()
    const { state, Menu } = await setup({ openGlossarySettings })
    const wrapper = mount(Menu, { attachTo: document.body })
    state.toggleToolsMenu()
    await flush()

    const ids = wrapper.findAll('[role="menuitem"]').map((item) => item.attributes('data-tools-item'))
    expect(ids.slice(0, 2)).toEqual(['glossary.manage.open', 'glossary.settings.open'])

    await wrapper.get('[data-tools-item="glossary.settings.open"]').trigger('click')
    expect(openGlossarySettings).toHaveBeenCalledTimes(1)
    expect(state.toolsMenuIsOpen.value).toBe(false)
    wrapper.unmount()
  })
})

describe('Công cụ ▾ trigger while Attribution is open', () => {
  it('is disabled while the Attribution screen is open and enabled after it closes', async () => {
    const { Menu } = await setup()
    const dict = await import('../../src/panels/dictSourcesState')
    const wrapper = mount(Menu, { attachTo: document.body })
    const trigger = wrapper.get('[data-tools-trigger]')
    expect(trigger.attributes('disabled')).toBeUndefined()
    dict.openAttribution()
    await flush()
    expect(trigger.attributes('disabled')).toBeDefined()
    dict.closeAttribution()
    await flush()
    expect(trigger.attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })
})

describe('chords', () => {
  it('Mod+Alt+B opens TM management, Mod+N opens the import form, Mod+M is untouched', async () => {
    const { keymap } = await setup()
    const bound = new Map(keymap.bindings().map((b) => [b.id, b.chord]))
    expect(bound.get('tm.manage.open')).toBe('Mod+Alt+B')
    expect(bound.get('library.import.open')).toBe('Mod+N')
    expect(keymap.bindings().filter((b) => b.chord === 'Mod+M').map((b) => b.id)).not.toContain('tm.manage.open')
    expect(bound.has('ai.proofread.scope.open')).toBe(false)
    expect(bound.has('library.import_text')).toBe(false)
  })

  it('Mod+N dispatches openLibraryImportForm', async () => {
    const openLibraryImportForm = vi.fn()
    const { commands } = await setup({ openLibraryImportForm })
    commands.dispatch('library.import.open')
    expect(openLibraryImportForm).toHaveBeenCalledTimes(1)
  })
})

describe('openLibraryImportForm', () => {
  it('moves to Library and opens the import overlay', async () => {
    vi.resetModules()
    const { currentMode, setMode } = await import('../../src/modes/modeState')
    const overlay = await import('../../src/modes/libraryImportOverlayState')
    setMode('workspace')
    const { openLibraryImportForm } = await import('../../src/modes/libraryImportFocus')
    openLibraryImportForm()
    expect(currentMode.value).toBe('library')
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })
})

describe('title bar in App.vue', () => {
  const app = readFileSync(resolve(__dirname, '../../src/App.vue'), 'utf8')
  const header = /<header class="titlebar"[^>]*>[\s\S]*?<\/header>/.exec(app)?.[0] ?? ''

  it('holds the three tabs, the layout switch and the Công cụ ▾ menu, and no loose entry buttons', () => {
    expect(header).not.toBe('')
    expect(header).not.toContain('titlebar-act')
    expect((header.match(/class="mode-tab"/g) ?? []).length).toBe(3)
    expect(header).toContain('<TitlebarToolsMenu />')
    expect(header).not.toMatch(/glossary\.queue\.open/)
    const layoutSwitch = /<div[^>]*class="layout-switch[^"]*"[\s\S]*?<\/div>/.exec(header)?.[0] ?? ''
    expect(layoutSwitch).toContain("dispatch('review.open')")
    expect(layoutSwitch).toContain("dispatch('review.close')")
    expect(header.match(/dispatch\('review\.(open|close)'\)/g) ?? []).toHaveLength(2)
  })

  it('shows the layout switch only in Workspace, marks the active preset and disables it in Review Mode', () => {
    expect(header).toMatch(/v-if="currentMode === 'workspace'"\s+class="layout-switch ui-seg ui-seg--icons ui-seg--bare"/)
    expect(header.match(/:disabled="reviewModeIsOpen \|\| attributionIsOpen"/g) ?? []).toHaveLength(2)
    expect(header).toContain("dispatch('layout.preset_grid')")
    expect(header).toContain("dispatch('layout.preset_columns')")
    expect(header).toContain("activeLayoutPreset === 'layout.preset_grid'")
    expect(header).toContain("activeLayoutPreset === 'layout.preset_columns'")
  })

  it('the Sắp xếp panel item is a checkbox that follows arrange mode and toggles it', async () => {
    const togglePanelHeaders = vi.fn()
    const { state, Menu } = await setup({ togglePanelHeaders })
    const panelHeaders = await import('../../src/layout/panelHeadersState')
    const wrapper = mount(Menu, { attachTo: document.body })
    state.toggleToolsMenu()
    await flush()

    const item = wrapper.get('[role="menuitemcheckbox"]')
    expect(item.attributes('aria-checked')).toBe('false')
    expect(item.get('.tools-switch').classes()).not.toContain('on')
    await item.trigger('click')
    expect(togglePanelHeaders).toHaveBeenCalledTimes(1)
    expect(state.toolsMenuIsOpen.value).toBe(false)

    await panelHeaders.setPanelHeadersShown(true)
    state.toggleToolsMenu()
    await flush()
    expect(wrapper.get('[role="menuitemcheckbox"]').attributes('aria-checked')).toBe('true')
    expect(wrapper.get('[role="menuitemcheckbox"] .tools-switch').classes()).toContain('on')
    wrapper.unmount()
  })

  const PANEL_SWITCHES = [
    ['layout.toggle_grid', 'panel.grid'],
    ['layout.toggle_lookup', 'panel.lookup'],
    ['layout.toggle_ai_translation', 'panel.ai_translation'],
  ] as const

  async function workspaceMenu(visible: readonly PanelId[]) {
    const togglePanel = vi.fn()
    const made = await setup({ togglePanel })
    ;(await import('../../src/modes/modeState')).setMode('workspace')
    const visibility = await import('../../src/layout/panelVisibilityState')
    visibility.setVisibleDockPanels(visible)
    const wrapper = mount(made.Menu, { attachTo: document.body })
    made.state.toggleToolsMenu()
    await flush()
    return { ...made, togglePanel, wrapper }
  }

  for (const [commandId, panelId] of PANEL_SWITCHES) {
    it(`the ${commandId} switch reports the dock and toggles ${panelId}`, async () => {
      const { togglePanel, wrapper, state } = await workspaceMenu(['panel.grid', 'panel.ai_translation'])
      const item = wrapper.get(`[data-tools-item="${commandId}"]`)
      expect(item.attributes('role')).toBe('menuitemcheckbox')
      expect(item.attributes('aria-checked')).toBe(String(panelId !== 'panel.lookup'))
      await item.trigger('click')
      expect(togglePanel).toHaveBeenCalledWith(panelId)
      expect(state.toolsMenuIsOpen.value).toBe(false)
      wrapper.unmount()
    })
  }

  it('lists the panel switches after Sắp xếp panel in the order Đối chiếu, Tra cứu, Đề xuất AI', async () => {
    const { wrapper } = await workspaceMenu(['panel.grid', 'panel.lookup', 'panel.ai_translation'])
    const ids = wrapper.findAll('[role="menuitemcheckbox"]').map((el) => el.attributes('data-tools-item'))
    expect(ids).toEqual([
      'layout.toggle_panel_headers',
      'layout.toggle_grid',
      'layout.toggle_lookup',
      'layout.toggle_ai_translation',
    ])
    wrapper.unmount()
  })

  it('disables the switch of the only visible panel', async () => {
    const { wrapper, state } = await workspaceMenu(['panel.lookup'])
    const only = wrapper.get('[data-tools-item="layout.toggle_lookup"]')
    expect(only.attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-tools-item="layout.toggle_grid"]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-tools-item="layout.toggle_ai_translation"]').attributes('disabled')).toBeUndefined()
    await only.get('.ui-row > span').element.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flush()
    expect(state.toolsMenuIsOpen.value).toBe(true)
    wrapper.unmount()
  })

  it('shows the switches disabled in Review Mode while aria-checked keeps the Workspace state', async () => {
    const { computed } = await import('vue')
    vi.doMock('../../src/reviewModeState', async (original) => ({
      ...(await original<Record<string, unknown>>()),
      reviewModeIsOpen: computed(() => true),
    }))
    try {
      const { wrapper } = await workspaceMenu(['panel.grid', 'panel.lookup'])
      for (const [commandId, panelId] of PANEL_SWITCHES) {
        const item = wrapper.get(`[data-tools-item="${commandId}"]`)
        expect(item.attributes('disabled'), commandId).toBeDefined()
        expect(item.attributes('aria-checked'), commandId).toBe(String(panelId !== 'panel.ai_translation'))
      }
      wrapper.unmount()
    } finally {
      vi.doUnmock('../../src/reviewModeState')
    }
  })

  it('hides the switches outside Workspace', async () => {
    const { wrapper } = await workspaceMenu(['panel.grid', 'panel.lookup'])
    ;(await import('../../src/modes/modeState')).setMode('library')
    await flush()
    expect(wrapper.find('[data-tools-item="layout.toggle_grid"]').exists()).toBe(false)
    expect(wrapper.findAll('[role="menuitemcheckbox"]')).toHaveLength(1)
    wrapper.unmount()
  })

  it('ArrowDown and ArrowUp skip a disabled switch', async () => {
    const { wrapper } = await workspaceMenu(['panel.lookup'])
    const lookup = wrapper.get('[data-tools-item="layout.toggle_lookup"]').element as HTMLElement
    const grid = wrapper.get('[data-tools-item="layout.toggle_grid"]').element as HTMLElement
    grid.focus()
    await wrapper.get('.tools').trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).not.toBe(lookup)
    expect(document.activeElement).toBe(wrapper.get('[data-tools-item="layout.toggle_ai_translation"]').element)
    await wrapper.get('.tools').trigger('keydown', { key: 'ArrowUp' })
    expect(document.activeElement).toBe(grid)
    wrapper.unmount()
  })

  it('each panel switch carries the full command name in its title', async () => {
    const { wrapper, i18n } = await workspaceMenu(['panel.grid', 'panel.lookup', 'panel.ai_translation'])
    for (const [commandId] of PANEL_SWITCHES) {
      expect(wrapper.get(`[data-tools-item="${commandId}"]`).attributes('title')).toContain(i18n.t(`command.${commandId}`))
    }
    wrapper.unmount()
  })
})
