import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { SEGMENT_RULE_VALUES } from '../../src/panels/editorSegments'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
})

async function mountGrid() {
  vi.resetModules()
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { wrapper, state, root: wrapper.element as unknown as Element }
}

function ruleStates(root: Element): (string | null)[] {
  return Array.from(root.querySelectorAll('.col-rule .cell-rule')).map((row) => {
    const rule = row.querySelector('.rule')
    return rule === null ? null : (Array.from(rule.classList).find((c) => c.startsWith('rule-')) ?? null)
  })
}

describe('GridPanel margin rule column', () => {
  it('moving the caret adds and removes no element in the rule column, and the primary rule follows the caret', async () => {
    const { wrapper, state, root } = await mountGrid()
    const col = root.querySelector('.col-rule')!
    const slots = Array.from(col.querySelectorAll('.margin-slot'))
    expect(slots).toHaveLength(3)
    expect(ruleStates(root)).toEqual(['rule-confirmed', 'rule-draft', null])

    const records: MutationRecord[] = []
    const observer = new MutationObserver((batch) => records.push(...batch))
    observer.observe(col, { childList: true, subtree: true })

    for (const id of [13, 12, 11, null]) {
      state.setEditorCaret(id)
      await wrapper.vm.$nextTick()
      const rest = ['rule-confirmed', 'rule-draft', null]
      const expected = [11, 12, 13].map((rowId, i) => (rowId === id ? 'rule-primary' : rest[i]))
      expect(ruleStates(root)).toEqual(expected)
    }
    await new Promise((resolve) => setTimeout(resolve, 20))
    observer.disconnect()

    const structural = records.filter((r) => r.addedNodes.length > 0 || r.removedNodes.length > 0)
    expect(structural).toHaveLength(0)
    expect(Array.from(col.querySelectorAll('.margin-slot'))).toEqual(slots)
    expect(ruleStates(root)).toEqual(['rule-confirmed', 'rule-draft', null])
    wrapper.unmount()
  })

  it('a row with no rule shows nothing: the slot carries no rule class', async () => {
    const { wrapper, root } = await mountGrid()
    const slot = root.querySelectorAll('.col-rule .margin-slot')[2]
    expect(slot.className).toBe('margin-slot')
    expect(root.querySelectorAll('.col-rule .rule')).toHaveLength(2)
    wrapper.unmount()
  })

  it('every rule block keeps the empty slot border width, so a caret move changes no box size', () => {
    const css = readFileSync(resolve(process.cwd(), 'src/panels/GridPanel.vue'), 'utf8')
    const widthOf = (selector: string) => {
      const block = new RegExp(`${selector.replace(/\./g, '\\.')}\\s*\\{([^}]*)\\}`).exec(css)
      return block === null ? null : (/border-left:\s*(\d+px)/.exec(block[1])?.[1] ?? null)
    }
    const slotWidth = widthOf('.margin-slot')
    expect(slotWidth).not.toBeNull()
    for (const value of SEGMENT_RULE_VALUES.filter((v) => v !== 'none')) {
      expect([value, widthOf(`.rule.rule-${value}`)]).toEqual([value, slotWidth])
    }
  })

  it('the rule class follows the written text: typing into an empty row gives the draft rule', async () => {
    const { wrapper, root } = await mountGrid()
    const cell = root.querySelector<HTMLElement>('[data-col="tgt"][data-segment-id="13"]')!
    cell.textContent = 'moi'
    cell.dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()
    expect(ruleStates(root)[2]).toBe('rule-draft')
    wrapper.unmount()
  })
})
