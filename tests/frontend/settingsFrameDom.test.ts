/**
 * Settings frame, driven through its templates with `dispatch` mocked: the Glossary form
 * and the frame's own `Escape` handler.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const dispatchMock = vi.fn()
const putConfigMock = vi.fn()

vi.mock('../../src/commands', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/commands')>()),
  dispatch: (...args: unknown[]) => dispatchMock(...args),
}))

vi.mock('../../src/config/bootstrap', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
  putConfig: (...args: unknown[]) => putConfigMock(...args),
  deleteConfig: vi.fn(),
}))

vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

const mounted: Array<{ unmount: () => void }> = []

beforeEach(() => {
  document.body.innerHTML = ''
  vi.resetModules()
  dispatchMock.mockReset()
  putConfigMock.mockReset()
  putConfigMock.mockResolvedValue(null)
})

afterEach(() => {
  while (mounted.length > 0) mounted.pop()?.unmount()
})

async function openAt(section: 'glossary' | 'shortcuts') {
  const settings = await import('../../src/settingsState')
  const commands = await import('../../src/commands')
  if (commands.commandRegistry.list().length === 0) commands.installCommands({ isMac: true, setMode: () => {} })
  const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
  settings.openSettingsToSection(section)
  const wrapper = mount(SettingsOverlay, { attachTo: document.body })
  mounted.push(wrapper)
  await flushPromises()
  return { settings, wrapper }
}

describe('Settings › Glossary form', () => {
  it('typing feeds the state, submit dispatches the save command, and the saved line follows the input', async () => {
    const { wrapper } = await openAt('glossary')
    const state = await import('../../src/glossarySettingsState')
    const input = wrapper.get('input.gs-input')

    await input.setValue('9')
    expect(state.glossarySettingsThresholdInput.value).toBe('9')

    await wrapper.get('form.gs-form').trigger('submit')
    expect(dispatchMock).toHaveBeenCalledWith('glossary.settings.save')

    await state.saveGlossarySettings()
    await flushPromises()
    expect(wrapper.find('.gs-saved').exists()).toBe(true)

    await input.setValue('10')
    expect(wrapper.find('.gs-saved').exists()).toBe(false)
  })
})

describe('Settings frame Escape', () => {
  it('with no capture armed, Escape on the scrim dispatches settings.close', async () => {
    const { wrapper } = await openAt('shortcuts')

    await wrapper.get('.set-scrim').trigger('keydown', { key: 'Escape' })

    expect(dispatchMock).toHaveBeenCalledWith('settings.close')
  })

  it('with a capture armed, Escape cancels the capture and does not close Settings', async () => {
    const { wrapper } = await openAt('shortcuts')
    const shortcuts = await import('../../src/config/shortcutsState')
    await wrapper.get('tr[data-command-id="mode.reading"]').trigger('mousedown')
    shortcuts.captureShortcut()
    expect(shortcuts.captureIsArmed.value).toBe(true)

    await wrapper.get('.set-scrim').trigger('keydown', { key: 'Escape' })

    expect(shortcuts.captureIsArmed.value).toBe(false)
    expect(dispatchMock).not.toHaveBeenCalledWith('settings.close')
  })
})
