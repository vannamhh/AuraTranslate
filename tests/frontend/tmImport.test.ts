/**
 * TMX export and import (FR64): state, the preview overlay and the exchange block of the TM
 * management overlay. `config/tm.ts` is the IPC boundary and is mocked; the command registry
 * and both overlays are real.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { TmManageListing, TmxImportPreview } from '../../src/config/tm'

const listMock = vi.fn()
const exportMock = vi.fn()
const openPreviewMock = vi.fn()
const confirmMock = vi.fn()
const cancelMock = vi.fn()

vi.mock('../../src/config/tm', () => ({
  tmListPairs: (...args: unknown[]) => listMock(...args),
  tmUpdatePairTarget: vi.fn(),
  tmDeletePair: vi.fn(),
  tmDeleteOthers: vi.fn(),
  tmPushPairToGlobal: vi.fn(),
  tmExportTier: (...args: unknown[]) => exportMock(...args),
  tmOpenImportPreview: (...args: unknown[]) => openPreviewMock(...args),
  tmConfirmImport: (...args: unknown[]) => confirmMock(...args),
  tmCancelImport: (...args: unknown[]) => cancelMock(...args),
}))

function listing(over: Partial<TmManageListing> = {}): TmManageListing {
  return {
    work_open: true,
    tm_empty: false,
    health: [
      { translation_origin: 'self', count: 1 },
      { translation_origin: 'other', count: 0 },
      { translation_origin: 'bilingual_import', count: 0 },
    ],
    total_pairs: 1,
    total_groups: 1,
    groups: [
      {
        source_text: '你好',
        distinct_targets: 1,
        rows: [
          {
            tier: 'work',
            unit_id: 1,
            target_text: 'Xin chào',
            translation_origin: 'self',
            side: 'mine',
            created_at: '2026-10-01T08:00:00.000Z',
            copies: [{ tier: 'work', unit_id: 1 }],
            hidden_copies: 0,
          },
        ],
      },
    ],
    ...over,
  }
}

function preview(over: Partial<TmxImportPreview> = {}): TmxImportPreview {
  return { file_name: 'x.tmx', tier: 'global', unit_count: 5, new_count: 3, already_count: 1, skipped_count: 1, ...over }
}

function ipcError(code: string, params: Record<string, string> = {}) {
  return { code, message_key: 'err.unknown', params, retryable: false }
}

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, exportMock, openPreviewMock, confirmMock, cancelMock]) m.mockReset()
  listMock.mockResolvedValue({ listing: listing(), error: null })
  cancelMock.mockResolvedValue({ ok: true, error: null })
  const manage = await import('../../src/tmManageState')
  const imp = await import('../../src/tmImportState')
  const i18n = await import('../../src/i18n')
  const commands = await import('../../src/commands')
  const { tmManageCommandDeps } = await import('../../src/tmManageCommandDeps')
  const deps: Partial<CommandDeps> = tmManageCommandDeps()
  commands.installCommands(deps as CommandDeps)
  const ManageOverlay = (await import('../../src/TmManageOverlay.vue')).default
  const ImportOverlay = (await import('../../src/TmImportOverlay.vue')).default
  return { manage, imp, i18n, commands, ManageOverlay, ImportOverlay }
}

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  for (let i = 0; i < 4; i += 1) await wrapper.vm.$nextTick()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('export', () => {
  it('exports the chosen tier; a cancelled dialog is silent; the Work tier defaults when a Work is open', async () => {
    const { manage } = await fresh()
    await manage.openTmManage()
    expect(manage.tmManageExchangeTier.value).toBe('work')

    exportMock.mockResolvedValue({ outcome: 'cancelled' })
    await manage.exportTmManageTier()
    expect(exportMock).toHaveBeenLastCalledWith('work')
    expect(manage.tmManageExportedPath.value).toBeNull()
    expect(manage.tmManageExportError.value).toBeNull()
    expect(manage.tmManageExportIpcUnavailable.value).toBe(false)

    manage.setTmManageExchangeTier('global')
    exportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/tm_global.tmx', leftOutCount: 0 })
    await manage.exportTmManageTier()
    expect(exportMock).toHaveBeenLastCalledWith('global')
    expect(manage.tmManageExportedPath.value).toBe('/tmp/tm_global.tmx')
  })

  it('tells an error and an absent bridge apart, and drops the old path before the dialog opens', async () => {
    const { manage } = await fresh()
    await manage.openTmManage()
    exportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/a.tmx', leftOutCount: 0 })
    await manage.exportTmManageTier()

    let release: (v: unknown) => void = () => undefined
    exportMock.mockImplementationOnce(() => new Promise((r) => (release = r)))
    const pending = manage.exportTmManageTier()
    expect(manage.tmManageExportedPath.value).toBeNull()
    expect(manage.tmManageExportBusy.value).toBe(true)
    release({ outcome: 'error', error: ipcError('tm.tmx_write_failed') })
    await pending
    expect(manage.tmManageExportError.value?.code).toBe('tm.tmx_write_failed')

    exportMock.mockResolvedValue({ outcome: 'ipc_unavailable' })
    await manage.exportTmManageTier()
    expect(manage.tmManageExportError.value).toBeNull()
    expect(manage.tmManageExportIpcUnavailable.value).toBe(true)
  })

  it('a second export while the dialog is open is ignored', async () => {
    const { manage } = await fresh()
    await manage.openTmManage()
    let release: (v: unknown) => void = () => undefined
    exportMock.mockImplementation(() => new Promise((r) => (release = r)))
    const first = manage.exportTmManageTier()
    await manage.exportTmManageTier()
    expect(exportMock).toHaveBeenCalledTimes(1)
    release({ outcome: 'cancelled' })
    await first
  })

  it('the Work tier cannot be chosen without a Work, and a refresh that finds none falls back to Global', async () => {
    const { manage } = await fresh()
    listMock.mockResolvedValue({ listing: listing({ work_open: false }), error: null })
    await manage.openTmManage()
    expect(manage.tmManageExchangeTier.value).toBe('global')
    manage.setTmManageExchangeTier('work')
    expect(manage.tmManageExchangeTier.value).toBe('global')
  })

  it('a refresh that finds the Work closed moves a chosen Work tier to Global', async () => {
    const { manage } = await fresh()
    await manage.openTmManage()
    manage.setTmManageExchangeTier('work')
    expect(manage.tmManageExchangeTier.value).toBe('work')
    listMock.mockResolvedValue({ listing: listing({ work_open: false }), error: null })
    await manage.refreshTmManage()
    expect(manage.tmManageExchangeTier.value).toBe('global')
  })
})

describe('import preview and confirm', () => {
  it('opens with the chosen tier; a cancelled dialog opens nothing and leaves no plan to cancel', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'cancelled' })
    await imp.openTmImportPreviewOverlay('work')
    expect(openPreviewMock).toHaveBeenCalledWith('work')
    expect(imp.tmImportOverlayIsOpen.value).toBe(false)
    expect(imp.tmImportOpening.value).toBe(false)
    expect(cancelMock).not.toHaveBeenCalled()
  })

  it('shows the counts of the preview; an error and an absent bridge are different statuses', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await imp.openTmImportPreviewOverlay('global')
    expect(imp.tmImportOverlayIsOpen.value).toBe(true)
    expect(imp.tmImportStatus.value).toBe('loaded')
    expect(imp.tmImportPreview.value?.new_count).toBe(3)
    await imp.cancelTmImportPreview()

    openPreviewMock.mockResolvedValue({ outcome: 'error', error: ipcError('tm.tmx_malformed', { line: '7' }) })
    await imp.openTmImportPreviewOverlay('global')
    expect(imp.tmImportStatus.value).toBe('error')
    expect(imp.tmImportLoadError.value?.code).toBe('tm.tmx_malformed')
    await imp.cancelTmImportPreview()

    openPreviewMock.mockResolvedValue({ outcome: 'ipc_unavailable' })
    await imp.openTmImportPreviewOverlay('global')
    expect(imp.tmImportStatus.value).toBe('ipc_unavailable')
  })

  it('confirm closes the overlay, reloads the list and reports the counts in the manage overlay', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await imp.openTmImportPreviewOverlay('global')
    listMock.mockClear()
    confirmMock.mockResolvedValue({ summary: { inserted: 3, already_count: 1, future_dated_count: 0 }, error: null })

    await imp.confirmTmImportPreview()
    expect(confirmMock).toHaveBeenCalledTimes(1)
    expect(imp.tmImportOverlayIsOpen.value).toBe(false)
    expect(listMock).toHaveBeenCalledTimes(1)
    expect(manage.tmManageImportDone.value).toEqual({ inserted: 3, already_count: 1, future_dated_count: 0 })
  })

  it('a failed confirm keeps the preview open with the error; a second press can retry', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await imp.openTmImportPreviewOverlay('work')
    confirmMock.mockResolvedValue({ summary: null, error: ipcError('tm.no_pending_import') })
    await imp.confirmTmImportPreview()
    expect(imp.tmImportOverlayIsOpen.value).toBe(true)
    expect(imp.tmImportConfirmError.value?.code).toBe('tm.no_pending_import')
    expect(imp.tmImportConfirming.value).toBe(false)
    expect(manage.tmManageImportDone.value).toBeNull()

    confirmMock.mockResolvedValue({ summary: { inserted: 0, already_count: 4, future_dated_count: 0 }, error: null })
    await imp.confirmTmImportPreview()
    expect(imp.tmImportConfirmError.value).toBeNull()
    expect(imp.tmImportOverlayIsOpen.value).toBe(false)
  })

  it('an absent bridge on confirm is its own state, not a silent success', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await imp.openTmImportPreviewOverlay('global')
    confirmMock.mockResolvedValue({ summary: null, error: null })
    await imp.confirmTmImportPreview()
    expect(imp.tmImportConfirmUnavailable.value).toBe(true)
    expect(imp.tmImportOverlayIsOpen.value).toBe(true)
  })

  it('cancel closes at once and tells Rust to drop the plan', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await imp.openTmImportPreviewOverlay('global')
    const pending = imp.cancelTmImportPreview()
    expect(imp.tmImportOverlayIsOpen.value).toBe(false)
    await pending
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(confirmMock).not.toHaveBeenCalled()
  })

  it('a second open while the file dialog is up is ignored, and export is locked out too', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    let release: (v: unknown) => void = () => undefined
    openPreviewMock.mockImplementation(() => new Promise((r) => (release = r)))
    const first = imp.openTmImportPreviewOverlay('global')
    await imp.openTmImportPreviewOverlay('global')
    await manage.exportTmManageTier()
    expect(openPreviewMock).toHaveBeenCalledTimes(1)
    expect(exportMock).not.toHaveBeenCalled()
    release({ outcome: 'cancelled' })
    await first
  })
})

describe('exchange gate', () => {
  it('an import cannot open while the export dialog is up', async () => {
    const { manage, imp } = await fresh()
    await manage.openTmManage()
    let release: (v: unknown) => void = () => undefined
    exportMock.mockImplementation(() => new Promise((r) => (release = r)))
    const pending = manage.exportTmManageTier()
    await imp.openTmImportPreviewOverlay('global')
    expect(openPreviewMock).not.toHaveBeenCalled()
    release({ outcome: 'cancelled' })
    await pending
  })
})

describe('overlays', () => {
  it('the exchange block offers both tiers, disables Work without a Work and dispatches export and import', async () => {
    const { manage, i18n, ManageOverlay } = await fresh()
    listMock.mockResolvedValue({ listing: listing({ work_open: false }), error: null })
    await manage.openTmManage()
    const wrapper = mount(ManageOverlay, { attachTo: document.body })
    await settle(wrapper)

    const radios = wrapper.findAll('.tm-exchange-tier input[type="radio"]')
    expect(radios).toHaveLength(2)
    expect((radios[0].element as HTMLInputElement).disabled).toBe(true)
    expect((radios[1].element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.text()).toContain(i18n.t('tm.exchange.work_unavailable'))

    exportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/tm_global.tmx', leftOutCount: 0 })
    const [exportButton] = wrapper.findAll('.tm-exchange-actions button')
    await exportButton.trigger('click')
    await settle(wrapper)
    expect(exportMock).toHaveBeenCalledWith('global')
    expect(wrapper.text()).toContain(i18n.t('tm.exchange.export_done', { path: '/tmp/tm_global.tmx' }))
    expect(wrapper.text()).not.toContain(i18n.t('tm.exchange.export_left_out', { count: '2' }))

    exportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/tm_global.tmx', leftOutCount: 2 })
    await exportButton.trigger('click')
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.exchange.export_left_out', { count: '2' }))
    wrapper.unmount()
  })

  it('choosing Work in the radio group changes what export sends', async () => {
    const { manage, ManageOverlay } = await fresh()
    await manage.openTmManage()
    manage.setTmManageExchangeTier('global')
    const wrapper = mount(ManageOverlay, { attachTo: document.body })
    await settle(wrapper)
    const radios = wrapper.findAll('.tm-exchange-tier input[type="radio"]')
    await radios[0].setValue(true)
    expect(manage.tmManageExchangeTier.value).toBe('work')
    exportMock.mockResolvedValue({ outcome: 'cancelled' })
    await wrapper.findAll('.tm-exchange-actions button')[0].trigger('click')
    expect(exportMock).toHaveBeenCalledWith('work')
    wrapper.unmount()
  })

  it('the import button opens the preview for the chosen tier; confirm reports the counts in the manage overlay', async () => {
    const { manage, i18n, ManageOverlay, ImportOverlay } = await fresh()
    await manage.openTmManage()
    manage.setTmManageExchangeTier('global')
    const manageWrapper = mount(ManageOverlay, { attachTo: document.body })
    const importWrapper = mount(ImportOverlay, { attachTo: document.body })
    await settle(manageWrapper)

    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await manageWrapper.get('[data-tm-import-open]').trigger('click')
    await vi.waitFor(() => expect(importWrapper.find('.ti-panel').exists()).toBe(true))
    expect(openPreviewMock).toHaveBeenCalledWith('global')
    expect(importWrapper.text()).toContain(i18n.t('tm.exchange.global_note'))
    expect(importWrapper.text()).toContain(i18n.t('tm.import.new_count', { count: '3' }))
    expect(importWrapper.text()).toContain(i18n.t('tm.import.already_count', { count: '1' }))
    expect(importWrapper.text()).toContain(i18n.t('tm.import.skipped_count', { count: '1' }))

    confirmMock.mockResolvedValue({ summary: { inserted: 3, already_count: 1, future_dated_count: 0 }, error: null })
    await importWrapper.get('.ti-act-primary').trigger('click')
    await vi.waitFor(() => expect(importWrapper.find('.ti-panel').exists()).toBe(false))
    await settle(manageWrapper)
    expect(manageWrapper.text()).toContain(i18n.t('tm.exchange.import_done', { inserted: '3', already: '1' }))
    manageWrapper.unmount()
    importWrapper.unmount()
  })

  it('the ownership checkbox value reaches the confirm call and resets on every new preview', async () => {
    const { manage, imp, ImportOverlay } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    await imp.openTmImportPreviewOverlay('work')
    await vi.waitFor(() => expect(wrapper.find('.ti-panel').exists()).toBe(true))
    const box = wrapper.get('.ti-own input[type="checkbox"]')
    expect((box.element as HTMLInputElement).checked).toBe(false)

    await box.setValue(true)
    await imp.cancelTmImportPreview()
    await imp.openTmImportPreviewOverlay('work')
    await vi.waitFor(() => expect(wrapper.find('.ti-panel').exists()).toBe(true))
    expect((wrapper.get('.ti-own input[type="checkbox"]').element as HTMLInputElement).checked).toBe(false)

    confirmMock.mockResolvedValue({ summary: { inserted: 1, already_count: 0, future_dated_count: 0 }, error: null })
    await wrapper.get('.ti-own input[type="checkbox"]').setValue(true)
    await imp.confirmTmImportPreview()
    expect(confirmMock).toHaveBeenLastCalledWith(true)

    await imp.openTmImportPreviewOverlay('work')
    await imp.confirmTmImportPreview()
    expect(confirmMock).toHaveBeenLastCalledWith(false)
    wrapper.unmount()
  })

  it('the future-date sentence shows after import only when the count is above zero', async () => {
    const { manage, i18n, imp, ManageOverlay } = await fresh()
    await manage.openTmManage()
    const wrapper = mount(ManageOverlay, { attachTo: document.body })
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await imp.openTmImportPreviewOverlay('work')
    confirmMock.mockResolvedValue({ summary: { inserted: 1, already_count: 0, future_dated_count: 2 }, error: null })
    await imp.confirmTmImportPreview()
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.exchange.import_future_dated', { count: '2' }))

    await imp.openTmImportPreviewOverlay('work')
    confirmMock.mockResolvedValue({ summary: { inserted: 1, already_count: 0, future_dated_count: 0 }, error: null })
    await imp.confirmTmImportPreview()
    await settle(wrapper)
    expect(wrapper.text()).not.toContain('ở tương lai')
    wrapper.unmount()
  })

  it('a re-import preview (nothing new) says so and still offers confirm and cancel', async () => {
    const { manage, i18n, imp, ImportOverlay } = await fresh()
    await manage.openTmManage()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({ tier: 'work', new_count: 0, already_count: 5, skipped_count: 0 }),
    })
    await imp.openTmImportPreviewOverlay('work')
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.import.nothing_new'))
    expect(wrapper.text()).not.toContain(i18n.t('tm.exchange.global_note'))
    expect(wrapper.findAll('.ti-actions button')).toHaveLength(2)
    wrapper.unmount()
  })

  it('error codes map to their own copy, with the line of a malformed file', async () => {
    const { manage, i18n, imp, ImportOverlay } = await fresh()
    await manage.openTmManage()
    const wrapper = mount(ImportOverlay, { attachTo: document.body })
    const cases: Array<[ReturnType<typeof ipcError>, string]> = [
      [ipcError('tm.tmx_malformed', { line: '12' }), i18n.t('tm.exchange.err_malformed', { line: '12' })],
      [ipcError('tm.tmx_no_body'), i18n.t('tm.exchange.err_no_body')],
      [ipcError('tm.tmx_too_large'), i18n.t('tm.exchange.err_too_large')],
      [ipcError('tm.tmx_no_usable_pair'), i18n.t('tm.exchange.err_no_usable_pair')],
      [
        ipcError('tm.tmx_no_usable_pair', { source_lang: 'zh' }),
        i18n.t('tm.exchange.err_no_usable_pair_lang', { source_lang: 'zh' }),
      ],
      [ipcError('work.none_open'), i18n.t('tm.exchange.err_work_none_open')],
    ]
    for (const [error, text] of cases) {
      openPreviewMock.mockResolvedValue({ outcome: 'error', error })
      await imp.openTmImportPreviewOverlay('work')
      await settle(wrapper)
      expect(wrapper.get('.ti-error').text()).toBe(text)
      await imp.cancelTmImportPreview()
      await settle(wrapper)
    }
    wrapper.unmount()
  })

  it('keyboard: Esc cancels, Tab stays inside the dialog, focus returns to the import button', async () => {
    const { manage, imp, ManageOverlay, ImportOverlay } = await fresh()
    await manage.openTmManage()
    const manageWrapper = mount(ManageOverlay, { attachTo: document.body })
    const importWrapper = mount(ImportOverlay, { attachTo: document.body })
    await settle(manageWrapper)

    const opener = manageWrapper.get<HTMLButtonElement>('[data-tm-import-open]').element
    opener.focus()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await manageWrapper.get('[data-tm-import-open]').trigger('click')
    await vi.waitFor(() => expect(importWrapper.find('.ti-panel').exists()).toBe(true))
    await settle(importWrapper)
    expect(document.activeElement).toBe(importWrapper.get('.ti-panel').element)

    const buttons = importWrapper.findAll('button').map((b) => b.element)
    buttons[buttons.length - 1].focus()
    await importWrapper.get('.ti-scrim').trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(buttons[0])
    await importWrapper.get('.ti-scrim').trigger('keydown', { key: 'Tab', shiftKey: true })
    expect(document.activeElement).toBe(buttons[buttons.length - 1])

    await importWrapper.get('.ti-scrim').trigger('keydown', { key: 'Escape' })
    await vi.waitFor(() => expect(imp.tmImportOverlayIsOpen.value).toBe(false))
    await settle(importWrapper)
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(document.activeElement).toBe(opener)
    manageWrapper.unmount()
    importWrapper.unmount()
  })
})
