/** The Lookup tab title shows the real chord of `lookup.lookup_selection`, and follows a rebind. */
import { beforeAll, beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { installCommands } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import PanelTab from '../../src/panels/PanelTab.vue'
import * as shortcuts from '../../src/config/shortcutsState'

const api = {
  isActive: true,
  isGroupActive: true,
  onDidActiveChange: () => ({ dispose: () => {} }),
  onDidActiveGroupChange: () => ({ dispose: () => {} }),
}

function tab(titleKey: string) {
  return mount(PanelTab, { props: { params: { params: { titleKey }, api } } })
}

function assignChord(id: string, code: string): void {
  const row = document.createElement('div')
  row.setAttribute('data-command-id', id)
  const cell = document.createElement('span')
  cell.setAttribute('data-key-cell', '')
  cell.tabIndex = 0
  row.appendChild(cell)
  document.body.appendChild(row)
  row.addEventListener('focusin', shortcuts.aimRowFrom)
  cell.focus()
  shortcuts.captureShortcut()
  if (!shortcuts.handleCaptureKey({ code, metaKey: true, altKey: true, shiftKey: true })) throw new Error('capture refused')
  if (shortcuts.shortcutNotice.value !== null) throw new Error(`capture notice: ${shortcuts.shortcutNotice.value.key}`)
}

const chordOf = (id: string) => shortcuts.shortcutRows.value.find((r) => r.id === id)!.display.join(' / ')

beforeAll(() => {
  installCommands({ isMac: true, setMode: () => {} } as CommandDeps)
})

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('PanelTab chord', () => {
  it('the Tra cứu tab carries the chord of lookup.lookup_selection next to its title', () => {
    const w = tab('panel.lookup.title')
    expect(w.text()).toContain('Tra cứu')
    expect(chordOf('lookup.lookup_selection')).not.toBe('')
    expect(w.get('.tab-chord').text()).toBe(chordOf('lookup.lookup_selection'))
  })

  it('rebinding lookup.lookup_selection changes the chord on the tab', async () => {
    const w = tab('panel.lookup.title')
    const before = w.get('.tab-chord').text()
    assignChord('lookup.lookup_selection', 'KeyY')
    await flushPromises()
    const after = w.get('.tab-chord').text()
    expect(after).toBe(chordOf('lookup.lookup_selection'))
    expect(after).not.toBe(before)
  })

  it('no other panel tab shows a chord', () => {
    expect(tab('panel.ai_translation.title').find('.tab-chord').exists()).toBe(false)
  })
})
