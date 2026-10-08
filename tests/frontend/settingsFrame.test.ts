/**
 * The Settings frame: one nav of sections that have a body, `⌘,` opens it, and the Glossary
 * threshold and Shortcuts screens live inside it instead of in overlays of their own.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { watch } from 'vue'

const putConfigMock = vi.fn()
const deleteConfigMock = vi.fn()

vi.mock('../../src/config/bootstrap', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
  putConfig: (...args: unknown[]) => putConfigMock(...args),
  deleteConfig: (...args: unknown[]) => deleteConfigMock(...args),
}))

vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: async () => ({ fields: [], workTierAvailable: false, keyConfigured: false, error: null }),
  aiConfigSaveField: vi.fn(),
  aiConfigClearOverride: vi.fn(),
  aiConfigSaveKey: vi.fn(),
  aiConfigDeleteKey: vi.fn(),
}))

const EXPECTED_SECTIONS = ['ai_and_model', 'prompt', 'glossary', 'tm', 'export', 'shortcuts', 'privacy']

const cleanups: Array<() => void> = []

beforeEach(() => {
  document.body.innerHTML = ''
  vi.resetModules()
  putConfigMock.mockReset()
  putConfigMock.mockResolvedValue(null)
  deleteConfigMock.mockReset()
  deleteConfigMock.mockResolvedValue(null)
})

afterEach(() => {
  while (cleanups.length > 0) cleanups.pop()?.()
})

async function freshFrame() {
  const settings = await import('../../src/settingsState')
  const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
  const i18n = await import('../../src/i18n')
  return { settings, SettingsOverlay, t: i18n.t }
}

async function freshCommands(deps: Record<string, unknown> = {}) {
  const commands = await import('../../src/commands')
  commands.installCommands({ isMac: true, setMode: () => {}, ...deps })
  return commands
}

describe('Settings nav', () => {
  it('lists exactly the sections that have a body, in mockup order with privacy last', async () => {
    const { settings, SettingsOverlay, t } = await freshFrame()
    expect([...settings.SETTINGS_SECTIONS]).toEqual(EXPECTED_SECTIONS)

    settings.openSettings()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await flushPromises()

    const labels = wrapper.findAll('.set-nav-item').map((b) => b.text())
    expect(labels).toEqual(EXPECTED_SECTIONS.map((s) => t(`settings.nav.${s}`)))
    wrapper.unmount()
  })

  it('every nav entry opens a body, never an empty frame', async () => {
    const { settings, SettingsOverlay, t } = await freshFrame()
    settings.openSettings()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await flushPromises()

    const bodyMarkers: Record<string, (root: typeof wrapper) => boolean> = {
      ai_and_model: (root) => root.find('.ai-key-field').exists(),
      prompt: (root) => root.find('[data-prompt-library-open]').exists(),
      glossary: (root) => root.find('input.gs-input').exists(),
      export: (root) => root.find('[data-export-translator-input]').exists(),
      shortcuts: (root) => root.find('table.sc-table').exists(),
      privacy: (root) => root.find('.set-main').text().includes(t('settings.privacy.empty')),
    }
    const forms = wrapper.findAll('.set-nav-form')
    expect(forms).toHaveLength(EXPECTED_SECTIONS.length)
    for (const [index, section] of EXPECTED_SECTIONS.entries()) {
      await forms[index].trigger('submit')
      await flushPromises()
      expect(settings.settingsActiveSection.value).toBe(section)
      for (const [name, present] of Object.entries(bodyMarkers)) {
        expect(present(wrapper), `${section}: marker of ${name}`).toBe(name === section)
      }
    }
    wrapper.unmount()
  })

  it('openSettingsToSection opens the one frame on the glossary and shortcuts sections', async () => {
    const { settings, SettingsOverlay } = await freshFrame()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })

    settings.openSettingsToSection('glossary')
    await flushPromises()
    expect(settings.settingsOverlayIsOpen.value).toBe(true)
    expect(wrapper.findAll('.set-panel')).toHaveLength(1)
    expect(wrapper.find('input.gs-input').exists()).toBe(true)

    settings.closeSettings()
    settings.openSettingsToSection('shortcuts')
    await flushPromises()
    expect(wrapper.findAll('.set-panel')).toHaveLength(1)
    expect(wrapper.find('table.sc-table').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('chords', () => {
  it('Mod+Comma resolves to settings.open and to nothing else; shortcuts.open has no default chord', async () => {
    const commands = await freshCommands()

    const holders = commands.effectiveBindings().filter((b) => b.chord === 'Mod+Comma')
    expect(holders.map((b) => b.id)).toEqual(['settings.open'])
    expect(commands.defaultChordsFor('shortcuts.open')).toEqual([])
    expect(commands.effectiveBindings().some((b) => b.id === 'shortcuts.open')).toBe(false)
  })

  it('pressing ⌘, dispatches settings.open', async () => {
    const openSettings = vi.fn()
    const commands = await freshCommands({ openSettings })
    cleanups.push(commands.attachKeyboard(document))

    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'Comma', metaKey: true, bubbles: true }))

    expect(openSettings).toHaveBeenCalledTimes(1)
  })

  it('shortcuts.open and glossary.settings.open reach Settings through their injected ports', async () => {
    const openShortcuts = vi.fn()
    const openGlossarySettings = vi.fn()
    const commands = await freshCommands({ openShortcuts, openGlossarySettings })

    commands.dispatch('shortcuts.open')
    commands.dispatch('glossary.settings.open')

    expect(openShortcuts).toHaveBeenCalledTimes(1)
    expect(openGlossarySettings).toHaveBeenCalledTimes(1)
  })

  it('editor.clear_source_cuts still defaults to Escape', async () => {
    const commands = await freshCommands()
    const holders = commands.effectiveBindings().filter((b) => b.chord === 'Escape')
    expect(holders.map((b) => b.id)).toEqual(['editor.clear_source_cuts'])
  })
})

describe('Settings › Shortcuts', () => {
  async function openShortcutsSection(extraDeps: Record<string, unknown> = {}) {
    const shortcutsState = await import('../../src/config/shortcutsState')
    const commands = await freshCommands({
      captureShortcut: shortcutsState.captureShortcut,
      unassignShortcut: shortcutsState.unassignShortcut,
      resetShortcut: shortcutsState.resetShortcut,
      ...extraDeps,
    })
    const frame = await freshFrame()
    frame.settings.openSettingsToSection('shortcuts')
    const wrapper = mount(frame.SettingsOverlay, { attachTo: document.body })
    await flushPromises()
    cleanups.push(() => wrapper.unmount())
    return { ...frame, commands, shortcutsState, wrapper }
  }

  async function armCapture(wrapper: Awaited<ReturnType<typeof openShortcutsSection>>['wrapper'], id: string) {
    const row = wrapper.get(`tr[data-command-id="${id}"]`)
    await row.trigger('mousedown')
    const cell = row.get('[data-key-cell]')
    await cell.trigger('click')
    await flushPromises()
    return cell
  }

  it('reassigning a chord bumps the epoch once and the new chord dispatches', async () => {
    const setMode = vi.fn()
    const { commands, shortcutsState, wrapper } = await openShortcutsSection({ setMode })
    cleanups.push(commands.attachKeyboard(document))
    let recomputes = 0
    const stop = watch(
      () => shortcutsState.shortcutRows.value,
      () => {
        recomputes += 1
      },
      { flush: 'sync' },
    )
    cleanups.push(stop)

    const cell = await armCapture(wrapper, 'mode.reading')
    expect(shortcutsState.captureIsArmed.value).toBe(true)
    await cell.trigger('keydown', { code: 'KeyZ', metaKey: true, altKey: true })
    await flushPromises()

    expect(recomputes).toBe(1)
    expect(commands.effectiveBindings().filter((b) => b.id === 'mode.reading').map((b) => b.chord)).toEqual(['Meta+Alt+Z'])
    expect(putConfigMock).toHaveBeenCalledTimes(1)

    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyZ', metaKey: true, altKey: true, bubbles: true }))
    expect(setMode).toHaveBeenCalledWith('reading')
  })

  it('Escape while capturing cancels the capture: nothing is assigned and Settings stays open', async () => {
    const { commands, shortcutsState, settings, wrapper } = await openShortcutsSection()
    const before = commands.effectiveBindings().map((b) => `${b.id}=${b.chord}`)

    const cell = await armCapture(wrapper, 'mode.reading')
    expect(shortcutsState.captureIsArmed.value).toBe(true)
    await cell.trigger('keydown', { code: 'Escape' })
    await flushPromises()

    expect(shortcutsState.captureIsArmed.value).toBe(false)
    expect(shortcutsState.shortcutNotice.value).toBeNull()
    expect(commands.effectiveBindings().map((b) => `${b.id}=${b.chord}`)).toEqual(before)
    expect(putConfigMock).not.toHaveBeenCalled()
    expect(settings.settingsOverlayIsOpen.value).toBe(true)
  })

  it('leaving the section drops a pending capture', async () => {
    const { settings, shortcutsState, wrapper } = await openShortcutsSection()
    await armCapture(wrapper, 'mode.reading')
    expect(shortcutsState.captureIsArmed.value).toBe(true)

    settings.selectSettingsSection('privacy')
    await flushPromises()

    expect(shortcutsState.captureIsArmed.value).toBe(false)
  })
})
