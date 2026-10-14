/**
 * Workspace grid, 12.7: the row status text is not drawn; it is the translation cell's
 * accessible name. happy-dom does not apply the `<style scoped>` of a `.vue` file to
 * `getComputedStyle`, so the stroke difference between margin rules is guarded by the source
 * scan in `scripts/check-commands.mjs` (Kiểm I) and by `e2e/specs/grid-row-error-label`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import vi_json from '../../src/i18n/vi.json'

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke is not expected here')),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }
const LABELS = vi_json as Record<string, string>

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
  return wrapper
}

describe('GridPanel.vue — trạng thái hàng nằm ở tên truy cập, không ở chữ nhìn thấy', () => {
  it('đã xác nhận · đã dịch · chưa dịch: không chữ trạng thái trong DOM, tên truy cập ô bản dịch mang số câu và trạng thái', async () => {
    const wrapper = await mountGrid()
    const root = wrapper.element as unknown as Element

    const expectedStates = [
      LABELS['panel.grid.state_confirmed'],
      LABELS['panel.grid.state_draft'],
      LABELS['panel.grid.state_untranslated'],
    ]
    const cells = Array.from(root.querySelectorAll<HTMLElement>('.col-tgt .cell-tgt'))
    expect(cells).toHaveLength(3)
    cells.forEach((cell, index) => {
      expect(cell.getAttribute('role')).toBe('textbox')
      expect(cell.getAttribute('aria-multiline')).toBe('true')
      expect(cell.getAttribute('aria-label')).toBe(`Câu ${index + 1}, ${expectedStates[index]}`)
      expect(cell.hasAttribute('data-row-error')).toBe(false)
    })

    const visible = root.textContent
    for (const label of expectedStates) expect(visible).not.toContain(label)
    expect(root.querySelector('.col-state')).toBeNull()
    expect(root.querySelector('.cell-state')).toBeNull()

    wrapper.unmount()
  })

  it('vạch lề: đã xác nhận · đã dịch chưa ký mang hai lớp khác nhau, chưa dịch không dựng phần tử vạch', async () => {
    const wrapper = await mountGrid()
    const rows = Array.from((wrapper.element as unknown as Element).querySelectorAll('.col-rule .cell-rule'))

    const ruleClasses = rows.map((row) => {
      const rule = row.querySelector('.rule')
      return rule === null ? null : Array.from(rule.classList).find((c) => c.startsWith('rule-')) ?? null
    })
    expect(ruleClasses).toEqual(['rule-confirmed', 'rule-draft', null])
    expect((wrapper.element as unknown as Element).querySelector('.col-rule')?.getAttribute('aria-hidden')).toBe('true')

    wrapper.unmount()
  })
})
