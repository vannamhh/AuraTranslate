import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import type { CommandDeps } from '../../src/commands'

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
  it('lists exactly eight items in IA order, showing a chord only where the command has one', async () => {
    const { state, Menu, i18n } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    expect(wrapper.findAll('[role="menuitem"]')).toHaveLength(0)
    state.toggleToolsMenu()
    await flush()

    const rows = wrapper.findAll('[role="menuitem"]').map((item) => item.findAll('span').map((s) => s.text()))
    expect(rows.map((r) => r[0])).toEqual(
      [
        'command.glossary.manage.open',
        'command.tm.manage.open',
        'command.settings.open',
        'command.ai.proofread.scope.open',
        'command.ai.proofread.memory.open',
        'command.export.reviewer_import.open',
        'command.export.alignment.open',
        'command.glossary.harvest.open',
      ].map((key) => i18n.t(key)),
    )
    expect(rows.map((r) => r[1])).toEqual(['Ctrl+Alt+M', 'Ctrl+Alt+B', 'Ctrl+,', '', '', '', '', ''])
    wrapper.unmount()
  })

  it('the trigger carries all eight opener hooks and the items carry none', async () => {
    const { state, Menu } = await setup()
    const wrapper = mount(Menu, { attachTo: document.body })
    state.toggleToolsMenu()
    await flush()
    const hooks = [
      'glossary-manage',
      'tm-manage',
      'settings',
      'proofread-scope',
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
    const items = wrapper.findAll('[role="menuitem"]')
    expect(document.activeElement).toBe(items[0]?.element)
    await items[0]?.trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(items[1]?.element)
    await items[1]?.trigger('keydown', { key: 'ArrowUp' })
    await items[0]?.trigger('keydown', { key: 'ArrowUp' })
    expect(document.activeElement).toBe(items[7]?.element)

    await items[7]?.trigger('keydown', { key: 'Escape' })
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

    await wrapper.findAll('[role="menuitem"]')[1]?.trigger('click')
    expect(openTmManage).toHaveBeenCalledTimes(1)
    expect(state.toolsMenuIsOpen.value).toBe(false)

    state.toggleToolsMenu()
    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    expect(state.toolsMenuIsOpen.value).toBe(false)
    wrapper.unmount()
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
  const header = /<header class="titlebar">[\s\S]*?<\/header>/.exec(app)?.[0] ?? ''

  it('holds the three tabs, the layout switch and the Công cụ ▾ menu, and no loose entry buttons', () => {
    expect(header).not.toBe('')
    expect(header).not.toContain('titlebar-act')
    expect((header.match(/class="mode-tab"/g) ?? []).length).toBe(3)
    expect(header).toContain('<TitlebarToolsMenu />')
    expect(header).not.toMatch(/glossary\.queue\.open|review\.open/)
  })

  it('shows the layout switch only in Workspace, marks the active preset and disables it in Review Mode', () => {
    expect(header).toMatch(/v-if="currentMode === 'workspace'"\s+class="layout-switch"/)
    expect(header).toContain(":disabled=\"reviewModeIsOpen\"")
    expect(header).toContain("dispatch('layout.preset_grid')")
    expect(header).toContain("dispatch('layout.preset_columns')")
    expect(header).toContain("activeLayoutPreset === 'layout.preset_grid'")
    expect(header).toContain("activeLayoutPreset === 'layout.preset_columns'")
  })
})
