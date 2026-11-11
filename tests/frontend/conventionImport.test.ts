import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'

const exportMock = vi.fn()
const openPreviewMock = vi.fn()
const confirmMock = vi.fn()
const cancelMock = vi.fn()
const listMock = vi.fn()
const scanNowMock = vi.fn()
const dispatchMock = vi.fn()
const chapterId = ref<number | null>(7)

vi.mock('../../src/config/convention', () => ({
  scanConventionSegments: () => Promise.resolve({ scan: null, error: null }),
  CONVENTION_WORK_TIER_UNAVAILABLE: 'convention.work_tier_unavailable',
  addConventionRule: vi.fn(),
  editConventionRule: vi.fn(),
  deleteConventionRule: vi.fn(),
  setConventionRuleEnabled: vi.fn(),
  listConventionRules: (tier: string) => listMock(tier),
  addConventionIgnore: vi.fn(),
  listConventionIgnores: () => Promise.resolve({ rows: [], error: null }),
  removeConventionIgnore: vi.fn(),
  clearConventionIgnores: vi.fn(),
  exportConventionTier: (tier: string) => exportMock(tier),
  openConventionImportPreview: (tier: string) => openPreviewMock(tier),
  confirmConventionImport: (decisions: Record<string, string>) => confirmMock(decisions),
  cancelConventionImport: () => cancelMock(),
}))
vi.mock('../../src/conventionState', () => ({ scanConventionNow: () => scanNowMock() }))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorChapterId: chapterId,
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))
vi.mock('../../src/commands', () => ({ dispatch: (...args: unknown[]) => dispatchMock(...args) }))

const manage = await import('../../src/conventionManageState')
const imp = await import('../../src/conventionImportState')
const ManageOverlay = (await import('../../src/ConventionManageOverlay.vue')).default
const ImportOverlay = (await import('../../src/ConventionImportOverlay.vue')).default

const UNAVAILABLE = {
  code: 'convention.work_tier_unavailable',
  message_key: 'err.convention.work_tier_unavailable',
  params: {},
  retryable: false,
}

function previewWire(over: Record<string, unknown> = {}) {
  return {
    file_name: 'in.csv',
    tier: 'global',
    row_count: 4,
    new_count: 1,
    identical_count: 1,
    conflicts: [
      { rule_key: 'a', existing_avoid_phrase: 'a', existing_replacement_phrase: 'x', file_avoid_phrase: 'a', file_replacement_phrase: 'y' },
      { rule_key: 'b', existing_avoid_phrase: 'b', existing_replacement_phrase: null, file_avoid_phrase: 'b', file_replacement_phrase: 'z' },
    ],
    ...over,
  }
}

let workAvailable: boolean

beforeEach(() => {
  document.body.innerHTML = ''
  imp.resetConventionImport()
  manage.resetConventionManage()
  workAvailable = true
  exportMock.mockReset().mockResolvedValue({ outcome: 'done', path: '/tmp/convention_work.csv' })
  openPreviewMock.mockReset().mockResolvedValue({ outcome: 'loaded', preview: previewWire() })
  confirmMock.mockReset().mockResolvedValue({ applied: { added: 1, replaced: 0, kept: 1 }, error: null })
  cancelMock.mockReset().mockResolvedValue({ error: null })
  scanNowMock.mockReset()
  dispatchMock.mockReset()
  listMock.mockReset().mockImplementation((tier: string) =>
    Promise.resolve(
      tier === 'work' && !workAvailable ? { rules: null, error: UNAVAILABLE } : { rules: [], error: null },
    ),
  )
  chapterId.value = 7
})

describe('tầng xuất/nhập', () => {
  it('follows the filter, and Both means the Work when one is open and Global when none is', async () => {
    await manage.openConventionManage()
    manage.setConventionManageTier('global')
    expect(manage.conventionManageExchangeTier.value).toBe('global')
    manage.setConventionManageTier('work')
    expect(manage.conventionManageExchangeTier.value).toBe('work')
    manage.setConventionManageTier('both')
    expect(manage.conventionManageExchangeTier.value).toBe('work')

    manage.resetConventionManage()
    workAvailable = false
    await manage.openConventionManage()
    manage.setConventionManageTier('both')
    expect(manage.conventionManageExchangeTier.value).toBe('global')
  })

  it('exports the derived tier and keeps the path for the status line', async () => {
    await manage.openConventionManage()
    manage.setConventionManageTier('global')
    await manage.exportConventionManageTier()
    expect(exportMock).toHaveBeenCalledWith('global')
    expect(manage.conventionManageExportedPath.value).toBe('/tmp/convention_work.csv')
  })

  it('stays silent when the save dialog is cancelled', async () => {
    exportMock.mockResolvedValue({ outcome: 'cancelled' })
    await manage.openConventionManage()
    await manage.exportConventionManageTier()
    expect(manage.conventionManageExportedPath.value).toBeNull()
    expect(manage.conventionManageError.value).toBeNull()
    expect(manage.conventionManageExchangeBusy.value).toBe(false)
  })
})

describe('xem trước nhập', () => {
  it('opens on the derived tier and defaults every disagreement to keep mine', async () => {
    await manage.openConventionManage()
    manage.setConventionManageTier('both')
    await imp.openConventionImportOverlay()
    expect(openPreviewMock).toHaveBeenCalledWith('work')
    expect(imp.conventionImportIsOpen.value).toBe(true)
    expect(imp.conventionImportDecisionFor('a')).toBe('keep_mine')
    expect(imp.conventionImportSummary.value).toEqual({ new_count: '1', kept_count: '3' })
  })

  it('does not open, and says nothing, when the pick dialog is cancelled', async () => {
    openPreviewMock.mockResolvedValue({ outcome: 'cancelled' })
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    expect(imp.conventionImportIsOpen.value).toBe(false)
    expect(imp.conventionImportLoadError.value).toBeNull()
    expect(manage.conventionManageExchangeBusy.value).toBe(false)
  })

  it('refuses a second open while the first dialog is still showing', async () => {
    let release: (value: unknown) => void = () => {}
    openPreviewMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    await manage.openConventionManage()
    const first = imp.openConventionImportOverlay()
    await imp.openConventionImportOverlay()
    expect(openPreviewMock).toHaveBeenCalledTimes(1)
    release({ outcome: 'cancelled' })
    await first
  })

  it('sends only the take-file decision and rescans exactly once after confirming', async () => {
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    imp.setConventionImportCursor(1)
    imp.decideConventionImportAtCursor('take_file')
    expect(imp.conventionImportSummary.value).toEqual({ new_count: '2', kept_count: '2' })
    await imp.confirmConventionImportPreview()
    expect(confirmMock).toHaveBeenCalledWith({ b: 'take_file' })
    expect(imp.conventionImportIsOpen.value).toBe(false)
    expect(scanNowMock).toHaveBeenCalledTimes(1)
  })

  it('confirming without a change sends no decision at all', async () => {
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    await imp.confirmConventionImportPreview()
    expect(confirmMock).toHaveBeenCalledWith({})
  })

  it('keeps the overlay, rescans nothing and blocks Apply after a stale plan', async () => {
    confirmMock.mockResolvedValue({
      applied: null,
      error: { code: 'convention.import_stale', message_key: 'err.convention.import_stale', params: {}, retryable: false },
    })
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    await imp.confirmConventionImportPreview()
    expect(imp.conventionImportIsOpen.value).toBe(true)
    expect(imp.conventionImportBlocked.value).toBe(true)
    expect(scanNowMock).not.toHaveBeenCalled()
    await imp.confirmConventionImportPreview()
    expect(confirmMock).toHaveBeenCalledTimes(1)
  })

  it('cancel closes the preview, tells Rust to drop the plan and rescans nothing', async () => {
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    await imp.cancelConventionImportPreview()
    expect(imp.conventionImportIsOpen.value).toBe(false)
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(confirmMock).not.toHaveBeenCalled()
    expect(scanNowMock).not.toHaveBeenCalled()
  })
})

describe('lớp phủ', () => {
  it('Export and Import are the first two Tab stops, ahead of Close', async () => {
    await manage.openConventionManage()
    const wrapper = mount(ManageOverlay, { attachTo: document.body })
    const root = wrapper.find('.cvm-scrim')
    const order: string[] = []
    for (let i = 0; i < 3; i += 1) {
      await root.trigger('keydown', { key: 'Tab' })
      const active = document.activeElement as HTMLElement
      order.push(
        active.hasAttribute('data-convention-export')
          ? 'export'
          : active.hasAttribute('data-convention-import-open')
            ? 'import'
            : active.hasAttribute('data-convention-manage-close')
              ? 'close'
              : 'other',
      )
    }
    expect(order).toEqual(['export', 'import', 'close'])
    wrapper.unmount()
  })

  it('the buttons dispatch their commands and the footer names the tier', async () => {
    await manage.openConventionManage()
    manage.setConventionManageTier('global')
    const wrapper = mount(ManageOverlay, { attachTo: document.body })
    await wrapper.find('[data-convention-export]').trigger('click')
    await wrapper.find('[data-convention-import-open]').trigger('click')
    expect(dispatchMock.mock.calls.map((call) => call[0])).toEqual([
      'convention.manage.export',
      'convention.manage.import',
    ])
    expect(wrapper.find('[data-convention-exchange-tier]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('pressing a row button moves the cursor to that row and dispatches the command that acts on it', async () => {
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    const take = wrapper.findAll('[data-convention-import-take]')
    expect(take).toHaveLength(2)
    await take[1].trigger('mousedown')
    await take[1].trigger('click')
    expect(imp.conventionImportCursor.value).toBe(1)
    expect(dispatchMock).toHaveBeenCalledWith('convention.import.take_file')
    expect(wrapper.find('[data-convention-import-apply]').exists()).toBe(true)
    expect(wrapper.find('[data-convention-import-keep]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('Esc in the preview dispatches cancel, and closing it returns focus to the Import button', async () => {
    await manage.openConventionManage()
    const manageWrapper = mount(ManageOverlay, { attachTo: document.body })
    const importButton = manageWrapper.find('[data-convention-import-open]').element as HTMLElement
    importButton.focus()
    await imp.openConventionImportOverlay()
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    await nextTick()
    await wrapper.find('.cvi-scrim').trigger('keydown', { key: 'Escape' })
    expect(dispatchMock).toHaveBeenCalledWith('convention.import.cancel')
    await imp.cancelConventionImportPreview()
    await nextTick()
    expect(document.activeElement).toBe(importButton)
    wrapper.unmount()
    manageWrapper.unmount()
  })

  it('ArrowDown and ArrowUp move the cursor row and focus its decision button', async () => {
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    const keep = () => wrapper.findAll('[data-convention-import-keep]')
    await keep()[0].trigger('keydown', { key: 'ArrowDown' })
    await nextTick()
    await nextTick()
    expect(imp.conventionImportCursor.value).toBe(1)
    expect(document.activeElement).toBe(keep()[1].element)
    await keep()[1].trigger('keydown', { key: 'ArrowUp' })
    await nextTick()
    await nextTick()
    expect(imp.conventionImportCursor.value).toBe(0)
    expect(document.activeElement).toBe(keep()[0].element)
    wrapper.unmount()
  })

  it('Tab stays inside the preview overlay and wraps from the last stop to the first', async () => {
    await manage.openConventionManage()
    await imp.openConventionImportOverlay()
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    const root = wrapper.find('.cvi-scrim')
    const panelEl = wrapper.find('.cvi-panel').element
    const seen: HTMLElement[] = []
    for (let i = 0; i < 12; i += 1) {
      await root.trigger('keydown', { key: 'Tab' })
      const active = document.activeElement as HTMLElement
      expect(panelEl.contains(active)).toBe(true)
      seen.push(active)
    }
    expect(new Set(seen).size).toBeLessThan(seen.length)
    wrapper.unmount()
  })

  it('clears the exported path when the tier filter changes, not when it is set to itself', async () => {
    await manage.openConventionManage()
    manage.setConventionManageTier('global')
    await manage.exportConventionManageTier()
    expect(manage.conventionManageExportedPath.value).not.toBeNull()
    manage.setConventionManageTier('global')
    expect(manage.conventionManageExportedPath.value).not.toBeNull()
    manage.setConventionManageTier('work')
    expect(manage.conventionManageExportedPath.value).toBeNull()

    manage.setConventionManageTier('work')
    await manage.exportConventionManageTier()
    expect(manage.conventionManageExportedPath.value).not.toBeNull()
    manage.nextConventionManageTier()
    expect(manage.conventionManageExportedPath.value).toBeNull()
  })
})
