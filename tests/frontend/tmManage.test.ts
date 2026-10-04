/**
 * TM management overlay (FR62, FR63): state, wire handling and keyboard paths.
 * `config/tm.ts` is the IPC boundary and is mocked; the command registry and the overlay are real.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { TmManageListing, TmManageRow, TmPairOrigin } from '../../src/config/tm'

const listMock = vi.fn()
const updateMock = vi.fn()
const deleteMock = vi.fn()
const deleteOthersMock = vi.fn()
const pushMock = vi.fn()

vi.mock('../../src/config/tm', () => ({
  tmListPairs: (...args: unknown[]) => listMock(...args),
  tmUpdatePairTarget: (...args: unknown[]) => updateMock(...args),
  tmDeletePair: (...args: unknown[]) => deleteMock(...args),
  tmDeleteOthers: (...args: unknown[]) => deleteOthersMock(...args),
  tmPushPairToGlobal: (...args: unknown[]) => pushMock(...args),
}))

function row(over: Partial<TmManageRow> = {}): TmManageRow {
  const merged = {
    tier: 'work' as const,
    unit_id: 1,
    target_text: 'Xin chào',
    translation_origin: 'self' as const,
    side: 'mine' as const,
    created_at: '2026-10-01T08:00:00.000Z',
    ...over,
  }
  return {
    ...merged,
    copies: over.copies ?? [{ tier: merged.tier, unit_id: merged.unit_id }],
  }
}

function origin(o: TmPairOrigin): Pick<TmManageRow, 'translation_origin' | 'side'> {
  return { translation_origin: o, side: o === 'self' ? 'mine' : 'others' }
}

function listing(over: Partial<TmManageListing> = {}): TmManageListing {
  const rows = [row()]
  return {
    work_open: true,
    tm_empty: false,
    health: [
      { translation_origin: 'self', count: 1 },
      { translation_origin: 'other', count: 0 },
      { translation_origin: 'bilingual_import', count: 0 },
    ],
    total_pairs: rows.length,
    total_groups: 1,
    groups: [{ source_text: '你好', distinct_targets: 1, rows }],
    ...over,
  }
}

function loaded(value: TmManageListing) {
  return { listing: value, error: null }
}

function ipcError(code: string) {
  return { code, message_key: 'err.unknown', params: {}, retryable: false }
}

const THREE_ROWS = listing({
  health: [
    { translation_origin: 'self', count: 1 },
    { translation_origin: 'other', count: 1 },
    { translation_origin: 'bilingual_import', count: 1 },
  ],
  total_pairs: 3,
  total_groups: 2,
  groups: [
    {
      source_text: '你好',
      distinct_targets: 2,
      rows: [
        row({ unit_id: 1, target_text: 'Xin chào' }),
        row({ unit_id: 2, target_text: 'Chào', ...origin('other') }),
      ],
    },
    {
      source_text: '再见',
      distinct_targets: 1,
      rows: [
        row({
          tier: 'global',
          unit_id: 9,
          target_text: 'Tạm biệt',
          ...origin('bilingual_import'),
        }),
      ],
    },
  ],
})

async function fresh() {
  vi.resetModules()
  listMock.mockReset()
  updateMock.mockReset()
  deleteMock.mockReset()
  deleteOthersMock.mockReset()
  pushMock.mockReset()
  return import('../../src/tmManageState')
}

async function freshOverlay() {
  const state = await fresh()
  const i18n = await import('../../src/i18n')
  const commands = await import('../../src/commands')
  const { tmManageCommandDeps } = await import('../../src/tmManageCommandDeps')
  const deps: Partial<CommandDeps> = tmManageCommandDeps()
  const keymap = commands.installCommands(deps as CommandDeps)
  const Overlay = (await import('../../src/TmManageOverlay.vue')).default
  return { state, i18n, Overlay, keymap, commands }
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

describe('open and list', () => {
  it('loads with no filters and flattens groups; a header only for 2+ distinct targets', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()

    expect(listMock).toHaveBeenCalledWith('all', 'both', '')
    expect(s.tmManageStatus.value).toBe('loaded')
    expect(s.tmManageFlatRows.value.map((f) => [f.row.unit_id, f.groupHeader, f.showSource])).toEqual([
      [1, true, false],
      [2, false, false],
      [9, false, true],
    ])
  })

  it('two copies of one target are one row that carries both copies, and it is not a header group', async () => {
    const s = await fresh()
    const copies = [
      { tier: 'work' as const, unit_id: 1 },
      { tier: 'global' as const, unit_id: 5 },
    ]
    listMock.mockResolvedValue(
      loaded(
        listing({
          groups: [
            {
              source_text: '你好',
              distinct_targets: 1,
              rows: [row({ copies })],
            },
          ],
        }),
      ),
    )
    await s.openTmManage()
    expect(s.tmManageFlatRows.value).toHaveLength(1)
    expect(s.tmManageFlatRows.value.some((f) => f.groupHeader)).toBe(false)
    expect(s.tmManageFlatRows.value[0].showSource).toBe(true)
    expect(s.tmManageCurrentCopyCount.value).toBe(2)
  })

  it('tells an empty TM, a filter that hides everything, an absent bridge and a failed load apart', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(
      loaded(
        listing({
          tm_empty: true,
          groups: [],
          total_groups: 0,
          total_pairs: 0,
        }),
      ),
    )
    await s.openTmManage()
    expect(s.tmManageEmptyReasonFor(s.tmManageStatus.value, s.tmManageTmEmpty.value, 0)).toBe('tm_empty')

    listMock.mockResolvedValue(
      loaded(
        listing({
          tm_empty: false,
          groups: [],
          total_groups: 0,
          total_pairs: 0,
        }),
      ),
    )
    s.setTmManageSearch('zzz')
    await vi.waitFor(() => expect(s.tmManageTmEmpty.value).toBe(false))
    expect(s.tmManageEmptyReasonFor(s.tmManageStatus.value, s.tmManageTmEmpty.value, 0)).toBe('filter_no_match')

    expect(s.tmManageEmptyReasonFor('unknown', false, 0)).toBe('not_loaded')
    expect(s.tmManageEmptyReasonFor('ipc_unavailable', false, 0)).toBe('ipc_unavailable')
    expect(s.tmManageEmptyReasonFor('error', false, 0)).toBeNull()
  })

  it('an absent bridge and an IPC error map to different statuses', async () => {
    const s = await fresh()
    listMock.mockResolvedValue({ listing: null, error: null })
    await s.openTmManage()
    expect(s.tmManageStatus.value).toBe('ipc_unavailable')
    s.resetTmManage()

    listMock.mockResolvedValue({
      listing: null,
      error: ipcError('tm.lookup_failed'),
    })
    await s.openTmManage()
    expect(s.tmManageStatus.value).toBe('error')
    expect(s.tmManageLoadError.value?.code).toBe('tm.lookup_failed')
  })

  it('each filter change re-lists with the three arguments and puts the cursor back on the first row', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    s.nextTmManageRow()
    expect(s.tmManageCursor.value).toBe(1)

    s.setTmManageOriginFilter('others')
    expect(s.tmManageCursor.value).toBe(0)
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('others', 'both', ''))
    s.setTmManageTierFilter('global')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('others', 'global', ''))
    s.setTmManageSearch('你好')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('others', 'global', '你好'))
  })

  it('a slow answer to an older query never overwrites the newer one', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()

    let releaseOld: (v: unknown) => void = () => undefined
    listMock.mockImplementationOnce(() => new Promise((resolve) => (releaseOld = resolve)))
    s.setTmManageOriginFilter('others')
    listMock.mockResolvedValueOnce(loaded(listing()))
    s.setTmManageTierFilter('global')
    await vi.waitFor(() => expect(s.tmManageFlatRows.value).toHaveLength(1))
    releaseOld(loaded(THREE_ROWS))
    await Promise.resolve()
    await Promise.resolve()
    expect(s.tmManageFlatRows.value).toHaveLength(1)
  })

  it('shows how many groups Rust cut off', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(listing({ total_groups: 250 })))
    await s.openTmManage()
    expect(s.tmManageShownGroups.value).toBe(1)
    expect(s.tmManageTotalGroups.value).toBe(250)
  })
})

describe('edit', () => {
  it('prefills the target, writes what was typed, re-lists and keeps the cursor on the same pair', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    s.nextTmManageRow()
    s.beginTmManageEdit()
    expect(s.tmManageEditTarget.value).toBe('Chào')

    s.tmManageEditTarget.value = 'Chào bạn hiền'
    updateMock.mockResolvedValue({
      pair: { ...row({ unit_id: 2 }), source_text: '你好' },
      error: null,
    })
    const reordered = listing({
      groups: [
        {
          source_text: '你好',
          distinct_targets: 2,
          rows: [row({ unit_id: 2, target_text: 'Chào bạn hiền' }), row({ unit_id: 1, target_text: 'Xin chào' })],
        },
      ],
    })
    listMock.mockResolvedValue(loaded(reordered))
    await s.saveTmManageEdit()

    expect(updateMock).toHaveBeenCalledWith([{ tier: 'work', unit_id: 2 }], '你好', 'Chào', 'Chào bạn hiền')
    expect(s.tmManageEditing.value).toBe(false)
    expect(s.tmManageCurrentRow.value?.unit_id).toBe(2)
    expect(s.tmManageCursor.value).toBe(0)
  })

  it('a refused blank target keeps the form open and surfaces the code', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(listing()))
    await s.openTmManage()
    s.beginTmManageEdit()
    s.tmManageEditTarget.value = '  '
    updateMock.mockResolvedValue({
      pair: null,
      error: ipcError('tm.target_empty'),
    })
    await s.saveTmManageEdit()

    expect(s.tmManageEditing.value).toBe(true)
    expect(s.tmManageActionError.value?.code).toBe('tm.target_empty')
  })

  it('filters and search are locked while the form is open', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(listing()))
    await s.openTmManage()
    s.beginTmManageEdit()
    listMock.mockClear()
    s.setTmManageSearch('x')
    s.setTmManageOriginFilter('others')
    s.setTmManageTierFilter('work')
    expect(listMock).not.toHaveBeenCalled()
    expect(s.tmManageSearchQuery.value).toBe('')
  })

  it('a stale pair closes the form, re-lists and reports tm.pair_not_found', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(listing()))
    await s.openTmManage()
    s.beginTmManageEdit()
    updateMock.mockResolvedValue({
      pair: null,
      error: ipcError('tm.pair_not_found'),
    })
    listMock.mockClear()
    listMock.mockResolvedValue(
      loaded(
        listing({
          groups: [],
          total_groups: 0,
          total_pairs: 0,
          tm_empty: true,
        }),
      ),
    )
    await s.saveTmManageEdit()

    expect(listMock).toHaveBeenCalledTimes(1)
    expect(s.tmManageEditing.value).toBe(false)
    expect(s.tmManageActionError.value?.code).toBe('tm.pair_not_found')
  })
})

describe('delete needs two presses', () => {
  it('first press writes nothing, second deletes that pair and re-lists', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    await s.deleteTmManagePair()
    expect(deleteMock).not.toHaveBeenCalled()
    expect(s.tmManageDeletePending.value).toBe(true)

    deleteMock.mockResolvedValue({ ok: true, error: null })
    listMock.mockClear()
    await s.deleteTmManagePair()
    expect(deleteMock).toHaveBeenCalledExactlyOnceWith([{ tier: 'work', unit_id: 1 }], '你好', 'Xin chào')
    expect(listMock).toHaveBeenCalledTimes(1)
    expect(s.tmManageDeletePending.value).toBe(false)
  })

  it('moving the cursor drops the pending press, so the next press arms again instead of deleting', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    await s.deleteTmManagePair()
    s.nextTmManageRow()
    expect(s.tmManageDeletePending.value).toBe(false)
    await s.deleteTmManagePair()
    expect(deleteMock).not.toHaveBeenCalled()
    expect(s.tmManageDeletePending.value).toBe(true)
  })

  it('a stale pair on the second press re-lists and reports the code', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    await s.deleteTmManagePair()
    deleteMock.mockResolvedValue({
      ok: false,
      error: ipcError('tm.pair_not_found'),
    })
    listMock.mockClear()
    await s.deleteTmManagePair()
    expect(listMock).toHaveBeenCalledTimes(1)
    expect(s.tmManageActionError.value?.code).toBe('tm.pair_not_found')
  })
})

describe('bulk delete of the others side', () => {
  it('counts other plus bilingual_import over the tier filter, two presses, passes the tier filter', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    expect(s.tmManageOthersCount.value).toBe(2)

    s.setTmManageTierFilter('work')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'work', ''))
    await s.deleteTmManageOthers()
    expect(deleteOthersMock).not.toHaveBeenCalled()
    expect(s.tmManageBulkPending.value).toBe(true)

    deleteOthersMock.mockResolvedValue({
      outcome: { deleted_work: 2, deleted_global: 0 },
      error: null,
    })
    await s.deleteTmManageOthers()
    expect(deleteOthersMock).toHaveBeenCalledExactlyOnceWith('work')
    expect(s.tmManageBulkDeleted.value).toEqual({ work: 2, global: 0 })
    expect(s.tmManageBulkPending.value).toBe(false)
  })

  it('does nothing when the others side is empty', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(listing()))
    await s.openTmManage()
    await s.deleteTmManageOthers()
    await s.deleteTmManageOthers()
    expect(deleteOthersMock).not.toHaveBeenCalled()
  })

  it('changing a filter between the presses disarms it', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    await s.deleteTmManageOthers()
    s.setTmManageTierFilter('global')
    expect(s.tmManageBulkPending.value).toBe(false)
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'global', ''))
    await s.deleteTmManageOthers()
    expect(deleteOthersMock).not.toHaveBeenCalled()
  })
})

describe('push to Global', () => {
  it('sends the row copies, source and target, re-lists, and the cursor follows the moved pair', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    const { copies: _copies, ...moved } = row({ tier: 'global', unit_id: 30 })
    pushMock.mockResolvedValue({
      pair: { ...moved, source_text: '你好' },
      error: null,
    })
    listMock.mockResolvedValue(
      loaded(
        listing({
          groups: [
            {
              source_text: '再见',
              distinct_targets: 1,
              rows: [row({ tier: 'global', unit_id: 9, target_text: 'Tạm biệt' })],
            },
            {
              source_text: '你好',
              distinct_targets: 1,
              rows: [row({ tier: 'global', unit_id: 30 })],
            },
          ],
        }),
      ),
    )
    listMock.mockClear()
    await s.pushTmManagePair()
    expect(pushMock).toHaveBeenCalledExactlyOnceWith([{ tier: 'work', unit_id: 1 }], '你好', 'Xin chào')
    expect(listMock).toHaveBeenCalledTimes(1)
    expect(s.tmManageCurrentRow.value?.unit_id).toBe(30)
    expect(s.tmManageActionNotice.value).toBe('pushed')
  })

  it('a Global pair is not pushed and says why', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    s.nextTmManageRow()
    s.nextTmManageRow()
    await s.pushTmManagePair()
    expect(pushMock).not.toHaveBeenCalled()
    expect(s.tmManageActionNotice.value).toBe('push_not_applicable')
  })

  it('with no Work open nothing is sent', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(listing({ work_open: false })))
    await s.openTmManage()
    await s.pushTmManagePair()
    expect(pushMock).not.toHaveBeenCalled()
    expect(s.tmManageActionNotice.value).toBe('work_not_open')
  })

  it('an identical pair already in Global surfaces tm.global_pair_exists and re-lists', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    pushMock.mockResolvedValue({
      pair: null,
      error: ipcError('tm.global_pair_exists'),
    })
    listMock.mockClear()
    await s.pushTmManagePair()
    expect(listMock).toHaveBeenCalledTimes(1)
    expect(s.tmManageActionError.value?.code).toBe('tm.global_pair_exists')
  })

  it('a stale pair re-lists', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    pushMock.mockResolvedValue({
      pair: null,
      error: ipcError('tm.pair_not_found'),
    })
    listMock.mockClear()
    await s.pushTmManagePair()
    expect(listMock).toHaveBeenCalledTimes(1)
  })
})

describe('overlay, keyboard only', () => {
  it('shows source, target, the stored origin, tier and date per row, the header, and the health strip with percentages', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    const text = wrapper.text()
    expect(text).toContain('你好')
    expect(text).toContain('Xin chào')
    expect(text).toContain(i18n.t('tm.manage.origin_self'))
    expect(text).toContain(i18n.t('tm.manage.origin_other'))
    expect(text).toContain(i18n.t('tm.manage.origin_bilingual_import'))
    expect(text).toContain(i18n.t('tm.fuzzy.tier_global'))
    expect(text).toContain(i18n.t('tm.manage.group_header', { count: '2' }))
    expect(wrapper.findAll('.tm-health li')).toHaveLength(3)
    expect(wrapper.get('.tm-health').text()).toContain(
      i18n.t('tm.manage.health_item', {
        label: i18n.t('tm.manage.origin_self'),
        count: '1',
        percent: '33',
      }),
    )
    expect(wrapper.findAll('[role="option"]')).toHaveLength(3)
    wrapper.unmount()
  })

  it('arrows move, Enter edits, typing and Enter saves, with no pointer', async () => {
    const { state, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    const scrim = wrapper.get('.tm-scrim')

    await scrim.trigger('keydown', { key: 'ArrowDown' })
    expect(state.tmManageCursor.value).toBe(1)
    await scrim.trigger('keydown', { key: 'ArrowUp' })
    expect(state.tmManageCursor.value).toBe(0)

    await scrim.trigger('keydown', { key: 'Enter' })
    await settle(wrapper)
    expect(state.tmManageEditing.value).toBe(true)
    const input = wrapper.get('.tm-edit-form input')
    await input.setValue('Chào thế giới')
    updateMock.mockResolvedValue({
      pair: { ...row(), source_text: '你好' },
      error: null,
    })
    await wrapper.get('.tm-edit-form').trigger('submit')
    await settle(wrapper)
    expect(updateMock).toHaveBeenCalledWith([{ tier: 'work', unit_id: 1 }], '你好', 'Xin chào', 'Chào thế giới')
    wrapper.unmount()
  })

  it('Backspace twice deletes, and Esc between the presses cancels the confirm without closing', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    const scrim = wrapper.get('.tm-scrim')

    await scrim.trigger('keydown', { key: 'Backspace' })
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.delete_confirm_hint_one'))
    expect(deleteMock).not.toHaveBeenCalled()

    await scrim.trigger('keydown', { key: 'Escape' })
    await settle(wrapper)
    expect(state.tmManageOverlayIsOpen.value).toBe(true)
    expect(state.tmManageDeletePending.value).toBe(false)

    await scrim.trigger('keydown', { key: 'Backspace' })
    deleteMock.mockResolvedValue({ ok: true, error: null })
    await scrim.trigger('keydown', { key: 'Backspace' })
    await settle(wrapper)
    expect(deleteMock).toHaveBeenCalledExactlyOnceWith([{ tier: 'work', unit_id: 1 }], '你好', 'Xin chào')

    await scrim.trigger('keydown', { key: 'Escape' })
    await settle(wrapper)
    expect(state.tmManageOverlayIsOpen.value).toBe(false)
    wrapper.unmount()
  })

  it('a key typed in the search box does not reach the row shortcuts', async () => {
    const { state, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    await wrapper.get('.tm-toolbar input').trigger('keydown', { key: 'Backspace' })
    await wrapper.get('.tm-toolbar input').trigger('keydown', { key: 'Enter' })
    expect(state.tmManageDeletePending.value).toBe(false)
    expect(state.tmManageEditing.value).toBe(false)
    wrapper.unmount()
  })

  it('bulk delete: the button names the count on the first press and the second press deletes', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    const bulk = wrapper.get('.tm-actions button:nth-of-type(4)')
    expect(bulk.text()).toBe(i18n.t('tm.manage.delete_others_both'))
    await bulk.trigger('click')
    await settle(wrapper)
    expect(bulk.text()).toBe(i18n.t('tm.manage.bulk_confirm_button_both', { count: '2' }))
    expect(wrapper.text()).toContain(i18n.t('tm.manage.bulk_confirm_hint_both', { count: '2' }))
    expect(i18n.t('tm.manage.bulk_confirm_hint_both', { count: '2' })).toContain('Toàn cục')
    expect(deleteOthersMock).not.toHaveBeenCalled()

    deleteOthersMock.mockResolvedValue({
      outcome: { deleted_work: 1, deleted_global: 1 },
      error: null,
    })
    await bulk.trigger('click')
    await settle(wrapper)
    expect(deleteOthersMock).toHaveBeenCalledExactlyOnceWith('both')
    expect(wrapper.text()).toContain(i18n.t('tm.manage.bulk_done', { work: '1', global: '1' }))
    wrapper.unmount()
  })

  it('the push button is disabled on a Global row and on a closed Work; enabled on a Work row', async () => {
    const { state, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    const push = wrapper.get('.tm-actions button:nth-of-type(3)')
    expect(push.attributes('disabled')).toBeUndefined()

    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'ArrowDown' })
    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'ArrowDown' })
    await settle(wrapper)
    expect(push.attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('a push button click dispatches tm.manage.push and moves the pair', async () => {
    const { state, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    pushMock.mockResolvedValue({
      pair: { ...row({ tier: 'global', unit_id: 30 }), source_text: '你好' },
      error: null,
    })
    await wrapper.get('.tm-actions button:nth-of-type(3)').trigger('click')
    await settle(wrapper)
    expect(pushMock).toHaveBeenCalledExactlyOnceWith([{ tier: 'work', unit_id: 1 }], '你好', 'Xin chào')
    wrapper.unmount()
  })

  it('maps wire codes to their own sentences, not the generic one', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    pushMock.mockResolvedValue({
      pair: null,
      error: ipcError('tm.global_pair_exists'),
    })
    await wrapper.get('.tm-actions button:nth-of-type(3)').trigger('click')
    await settle(wrapper)
    expect(wrapper.get('[role="alert"]').text()).toBe(i18n.t('tm.manage.push_exists'))
    wrapper.unmount()
  })

  it('shows 200 / N when Rust cut the list, and an empty-TM sentence apart from a no-match sentence', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(listing({ total_groups: 250 })))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.capped', { shown: '1', total: '250' }))

    listMock.mockResolvedValue(
      loaded(
        listing({
          tm_empty: true,
          groups: [],
          total_groups: 0,
          total_pairs: 0,
        }),
      ),
    )
    state.setTmManageSearch('x')
    await vi.waitFor(() => expect(wrapper.text()).toContain(i18n.t('tm.manage.empty_tm')))

    listMock.mockResolvedValue(
      loaded(
        listing({
          tm_empty: false,
          groups: [],
          total_groups: 0,
          total_pairs: 0,
        }),
      ),
    )
    state.setTmManageSearch('xy')
    await vi.waitFor(() => expect(wrapper.text()).toContain(i18n.t('tm.manage.empty_filter_no_match')))
    expect(wrapper.text()).not.toContain(i18n.t('tm.manage.empty_tm'))
    wrapper.unmount()
  })

  it('the open command has no default chord and is reachable by id', async () => {
    const { state, keymap, commands } = await freshOverlay()
    listMock.mockResolvedValue(loaded(listing()))
    const spec = commands.commandRegistry.list().find((c) => c.id === 'tm.manage.open')
    expect(spec).toBeDefined()
    expect(keymap.bindings().filter((b) => b.id.startsWith('tm.manage.'))).toEqual([])
    commands.dispatch('tm.manage.open')
    await vi.waitFor(() => expect(state.tmManageOverlayIsOpen.value).toBe(true))
  })
})

describe('search debounce', () => {
  it('re-lists once, 150 ms after the last keystroke', async () => {
    vi.useFakeTimers()
    try {
      const s = await fresh()
      listMock.mockResolvedValue(loaded(THREE_ROWS))
      await s.openTmManage()
      listMock.mockClear()

      s.setTmManageSearch('你')
      await vi.advanceTimersByTimeAsync(100)
      s.setTmManageSearch('你好')
      await vi.advanceTimersByTimeAsync(100)
      expect(listMock).not.toHaveBeenCalled()
      await vi.advanceTimersByTimeAsync(60)
      expect(listMock).toHaveBeenCalledExactlyOnceWith('all', 'both', '你好')
    } finally {
      vi.useRealTimers()
    }
  })

  it('closing the overlay drops a pending search', async () => {
    vi.useFakeTimers()
    try {
      const s = await fresh()
      listMock.mockResolvedValue(loaded(THREE_ROWS))
      await s.openTmManage()
      listMock.mockClear()
      s.setTmManageSearch('x')
      s.closeTmManage()
      await vi.advanceTimersByTimeAsync(400)
      expect(listMock).not.toHaveBeenCalled()
    } finally {
      vi.useRealTimers()
    }
  })
})

describe('failed re-list and locks', () => {
  it('a failed re-list clears the listing, so no action can aim at a stale row', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    expect(s.tmManageCurrentRow.value).not.toBeNull()

    listMock.mockResolvedValue({
      listing: null,
      error: ipcError('tm.lookup_failed'),
    })
    s.setTmManageOriginFilter('others')
    await vi.waitFor(() => expect(s.tmManageStatus.value).toBe('error'))
    expect(s.tmManageFlatRows.value).toHaveLength(0)
    expect(s.tmManageCurrentRow.value).toBeNull()
    await s.deleteTmManagePair()
    expect(s.tmManageDeletePending.value).toBe(false)
  })

  it('tier work with no Work open falls back to both tiers instead of staying on Work', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    listMock.mockImplementation((_o: string, tier: string) =>
      Promise.resolve(tier === 'work' ? { listing: null, error: ipcError('work.none_open') } : loaded(THREE_ROWS)),
    )
    s.setTmManageTierFilter('work')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'both', ''))
    expect(s.tmManageTierFilter.value).toBe('both')
    expect(s.tmManageStatus.value).toBe('loaded')
  })

  it('search and both selects are disabled while a write is in flight', async () => {
    const { state, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    let release: (v: unknown) => void = () => undefined
    deleteMock.mockImplementation(() => new Promise((resolve) => (release = resolve)))
    await state.deleteTmManagePair()
    void state.deleteTmManagePair()
    await settle(wrapper)
    expect(state.tmManageSaving.value).toBe(true)
    expect(wrapper.get('.tm-toolbar input').attributes('disabled')).toBeDefined()
    expect(wrapper.findAll('.tm-toolbar select').every((el) => el.attributes('disabled') !== undefined)).toBe(true)

    release({ ok: true, error: null })
    await settle(wrapper)
    wrapper.unmount()
  })

  it('the origin filter has no "mine" option, only the three stored origins and the others side', async () => {
    const { state, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    const values = wrapper
      .findAll('.tm-toolbar select')[1]
      .findAll('option')
      .map((o) => o.attributes('value'))
    expect(values).toEqual(['all', 'others', 'self', 'other', 'bilingual_import'])
    wrapper.unmount()
  })
})

describe('collapsed row', () => {
  const COPIES = [
    { tier: 'work' as const, unit_id: 1 },
    { tier: 'global' as const, unit_id: 5 },
  ]
  const collapsed = () =>
    listing({
      groups: [
        {
          source_text: '你好',
          distinct_targets: 1,
          rows: [row({ copies: COPIES })],
        },
      ],
    })

  it('shows one row with both tiers and the copy count; the confirm names the count and delete sends both copies', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(collapsed()))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    expect(wrapper.findAll('[role="option"]')).toHaveLength(1)
    const text = wrapper.get('[role="option"]').text()
    expect(text).toContain(i18n.t('tm.fuzzy.tier_work'))
    expect(text).toContain(i18n.t('tm.fuzzy.tier_global'))
    expect(text).toContain(i18n.t('tm.manage.copies', { count: '2' }))

    const scrim = wrapper.get('.tm-scrim')
    await scrim.trigger('keydown', { key: 'Backspace' })
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.delete_confirm_hint', { count: '2' }))

    deleteMock.mockResolvedValue({ ok: true, error: null })
    await scrim.trigger('keydown', { key: 'Backspace' })
    await settle(wrapper)
    expect(deleteMock).toHaveBeenCalledExactlyOnceWith(COPIES, '你好', 'Xin chào')
    wrapper.unmount()
  })

  it('a row with a Global copy is not pushed', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(collapsed()))
    await s.openTmManage()
    await s.pushTmManagePair()
    expect(pushMock).not.toHaveBeenCalled()
    expect(s.tmManageActionNotice.value).toBe('push_not_applicable')
  })
})

describe('bulk delete names its tier', () => {
  it('Work and Global have their own wording; Global says it is shared by every Work; done keeps the counts apart', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    const bulk = () => wrapper.get('.tm-actions button:nth-of-type(4)')

    state.setTmManageTierFilter('global')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'global', ''))
    await settle(wrapper)
    expect(bulk().text()).toBe(i18n.t('tm.manage.delete_others_global'))
    await bulk().trigger('click')
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.bulk_confirm_hint_global', { count: '2' }))
    expect(i18n.t('tm.manage.bulk_confirm_hint_global', { count: '2' })).toContain('mọi Tác phẩm')

    state.setTmManageTierFilter('work')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'work', ''))
    await settle(wrapper)
    expect(bulk().text()).toBe(i18n.t('tm.manage.delete_others_work'))

    deleteOthersMock.mockResolvedValue({
      outcome: { deleted_work: 3, deleted_global: 0 },
      error: null,
    })
    await bulk().trigger('click')
    await bulk().trigger('click')
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.bulk_done_work', { work: '3', global: '0' }))
    wrapper.unmount()
  })
})

describe('row date', () => {
  it('shows the date label from historyTimeLabel, not the raw ISO string', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    const { historyTimeLabel } = await import('../../src/panels/segmentHistoryTime')
    const createdAt = '2020-03-05T08:00:00.000Z'
    listMock.mockResolvedValue(
      loaded(
        listing({
          groups: [
            {
              source_text: '你好',
              distinct_targets: 1,
              rows: [row({ created_at: createdAt })],
            },
          ],
        }),
      ),
    )
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)
    const label = historyTimeLabel(createdAt, Date.now())
    expect(wrapper.get('.tm-date').text()).toBe(i18n.t(label.key, label.params))
    expect(wrapper.get('.tm-date').text()).not.toContain(createdAt)
    wrapper.unmount()
  })
})

describe('focus and the edit form', () => {
  async function mounted(value: TmManageListing = THREE_ROWS) {
    const ctx = await freshOverlay()
    listMock.mockResolvedValue(loaded(value))
    const wrapper = mount(ctx.Overlay, { attachTo: document.body })
    await ctx.state.openTmManage()
    await settle(wrapper)
    return { ...ctx, wrapper }
  }

  it('the title-bar opener in App.vue dispatches tm.manage.open; opening from it puts focus on the list and closing returns it there', async () => {
    const appSource = (await import('../../src/App.vue?raw')).default
    const button = /<button[^>]*data-tm-manage-open[^>]*>/.exec(appSource)?.[0] ?? ''
    expect(button).toContain('@click="dispatch(\'tm.manage.open\')"')
    expect(button).toContain('focusOnPointerDown')

    const ctx = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    const opener = document.createElement('button')
    opener.setAttribute('data-tm-manage-open', '')
    opener.addEventListener('click', () => ctx.commands.dispatch('tm.manage.open'))
    document.body.appendChild(opener)
    const wrapper = mount(ctx.Overlay, { attachTo: document.body })
    opener.focus()
    opener.click()
    await vi.waitFor(() => expect(ctx.state.tmManageStatus.value).toBe('loaded'))
    await settle(wrapper)
    expect(document.activeElement).toBe(wrapper.get('.tm-list').element)

    ctx.state.closeTmManage()
    await settle(wrapper)
    expect(document.activeElement).toBe(opener)
    wrapper.unmount()
  })

  it('Enter moves focus into the edit field; Esc there cancels the edit and keeps the overlay, focus back on the list', async () => {
    const { state, wrapper } = await mounted()
    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'Enter' })
    await settle(wrapper)
    const input = wrapper.get('.tm-edit-form input').element
    expect(document.activeElement).toBe(input)

    await wrapper.get('.tm-edit-form input').trigger('keydown', { key: 'Escape' })
    await settle(wrapper)
    expect(state.tmManageEditing.value).toBe(false)
    expect(state.tmManageOverlayIsOpen.value).toBe(true)
    expect(document.activeElement).toBe(wrapper.get('.tm-list').element)
    wrapper.unmount()
  })

  it('Esc with focus outside the form still cancels the edit instead of closing the overlay', async () => {
    const { state, wrapper } = await mounted()
    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'Enter' })
    await settle(wrapper)
    await wrapper.get('.tm-panel').trigger('keydown', { key: 'Escape' })
    await settle(wrapper)
    expect(state.tmManageEditing.value).toBe(false)
    expect(state.tmManageOverlayIsOpen.value).toBe(true)
    wrapper.unmount()
  })

  it('while editing, list keys and delete, next, prev, edit, push and bulk do nothing', async () => {
    const { state, commands, wrapper } = await mounted()
    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'Enter' })
    await settle(wrapper)
    state.tmManageEditTarget.value = 'đang gõ dở'

    for (const key of ['ArrowDown', 'ArrowUp', 'Backspace', 'Delete', 'Enter']) {
      await wrapper.get('.tm-scrim').trigger('keydown', { key })
    }
    for (const id of [
      'tm.manage.next',
      'tm.manage.prev',
      'tm.manage.delete',
      'tm.manage.edit',
      'tm.manage.push',
      'tm.manage.delete_others',
    ]) {
      commands.dispatch(id)
    }
    await settle(wrapper)
    expect(state.tmManageCursor.value).toBe(0)
    expect(state.tmManageEditTarget.value).toBe('đang gõ dở')
    expect(state.tmManageDeletePending.value).toBe(false)
    expect(state.tmManageBulkPending.value).toBe(false)
    expect(deleteMock).not.toHaveBeenCalled()
    expect(pushMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('keyboard only, through real focus: Enter on the focused list, type, submit, focus lands on the list', async () => {
    const { wrapper } = await mounted()
    const list = wrapper.get('.tm-list').element as HTMLElement
    expect(document.activeElement).toBe(list)

    list.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await settle(wrapper)
    const input = wrapper.get('.tm-edit-form input')
    expect(document.activeElement).toBe(input.element)
    await input.setValue('Chào bạn')

    updateMock.mockResolvedValue({
      pair: { ...row(), source_text: '你好' },
      error: null,
    })
    await wrapper.get('.tm-edit-form').trigger('submit')
    await settle(wrapper)
    expect(updateMock).toHaveBeenCalledTimes(1)
    expect(document.activeElement).toBe(wrapper.get('.tm-list').element)
    wrapper.unmount()
  })

  it('deleting the last row moves focus to the panel, never to body', async () => {
    const { state, wrapper } = await mounted(listing())
    expect(document.activeElement).toBe(wrapper.get('.tm-list').element)

    deleteMock.mockResolvedValue({ ok: true, error: null })
    listMock.mockResolvedValue(
      loaded(
        listing({
          groups: [],
          total_groups: 0,
          total_pairs: 0,
          tm_empty: true,
        }),
      ),
    )
    await state.deleteTmManagePair()
    await state.deleteTmManagePair()
    await settle(wrapper)
    expect(wrapper.find('.tm-list').exists()).toBe(false)
    expect(document.activeElement).toBe(wrapper.get('.tm-panel').element)
    wrapper.unmount()
  })

  it('an auto-repeating Backspace or Delete never arms and never confirms a delete', async () => {
    const { state, wrapper } = await mounted()
    const scrim = wrapper.get('.tm-scrim')

    await scrim.trigger('keydown', { key: 'Backspace', repeat: true })
    expect(state.tmManageDeletePending.value).toBe(false)

    await scrim.trigger('keydown', { key: 'Backspace' })
    expect(state.tmManageDeletePending.value).toBe(true)
    await scrim.trigger('keydown', { key: 'Delete', repeat: true })
    await settle(wrapper)
    expect(deleteMock).not.toHaveBeenCalled()
    expect(state.tmManageDeletePending.value).toBe(true)
    wrapper.unmount()
  })
})

describe('commands while the overlay is closed', () => {
  it('every row command is a no-op: delete_others pressed twice deletes nothing', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    s.closeTmManage()

    await s.deleteTmManageOthers()
    expect(s.tmManageBulkPending.value).toBe(false)
    await s.deleteTmManageOthers()
    await s.deleteTmManagePair()
    await s.deleteTmManagePair()
    await s.pushTmManagePair()
    s.beginTmManageEdit()
    await s.saveTmManageEdit()
    s.nextTmManageRow()

    expect(deleteOthersMock).not.toHaveBeenCalled()
    expect(deleteMock).not.toHaveBeenCalled()
    expect(pushMock).not.toHaveBeenCalled()
    expect(updateMock).not.toHaveBeenCalled()
    expect(s.tmManageEditing.value).toBe(false)
    expect(s.tmManageDeletePending.value).toBe(false)
    expect(s.tmManageCursor.value).toBe(0)
  })
})

describe('an edit pins its row', () => {
  it('a re-list that lands after the edit began keeps the cursor on the pinned row, and Save writes that row', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()

    let release: (v: unknown) => void = () => undefined
    listMock.mockImplementationOnce(() => new Promise((resolve) => (release = resolve)))
    s.setTmManageOriginFilter('all')
    s.beginTmManageEdit()
    expect(s.tmManageCurrentRow.value?.unit_id).toBe(1)

    release(
      loaded(
        listing({
          groups: [
            {
              source_text: '再见',
              distinct_targets: 1,
              rows: [row({ tier: 'global', unit_id: 9, target_text: 'Tạm biệt' })],
            },
            THREE_ROWS.groups[0],
          ],
        }),
      ),
    )
    await vi.waitFor(() => expect(s.tmManageFlatRows.value[0].row.unit_id).toBe(9))
    expect(s.tmManageCurrentRow.value?.unit_id).toBe(1)
    expect(s.tmManageEditing.value).toBe(true)

    s.tmManageEditTarget.value = 'đã sửa'
    updateMock.mockResolvedValue({ pair: { ...row(), source_text: '你好' }, error: null })
    await s.saveTmManageEdit()
    expect(updateMock).toHaveBeenCalledWith([{ tier: 'work', unit_id: 1 }], '你好', 'Xin chào', 'đã sửa')
  })

  it('a re-list that no longer holds the pinned row ends the edit', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    let release: (v: unknown) => void = () => undefined
    listMock.mockImplementationOnce(() => new Promise((resolve) => (release = resolve)))
    s.setTmManageOriginFilter('others')
    s.beginTmManageEdit()
    release(loaded(listing({ groups: [THREE_ROWS.groups[1]] })))
    await vi.waitFor(() => expect(s.tmManageEditing.value).toBe(false))
    await s.saveTmManageEdit()
    expect(updateMock).not.toHaveBeenCalled()
  })

  it('beginning an edit cancels a pending debounced search', async () => {
    vi.useFakeTimers()
    try {
      const s = await fresh()
      listMock.mockResolvedValue(loaded(THREE_ROWS))
      await s.openTmManage()
      listMock.mockClear()
      s.setTmManageSearch('你')
      s.beginTmManageEdit()
      await vi.advanceTimersByTimeAsync(400)
      expect(listMock).not.toHaveBeenCalled()
      expect(s.tmManageCursor.value).toBe(0)
    } finally {
      vi.useRealTimers()
    }
  })
})

describe('push under the Work-only filter', () => {
  it('says the pair left the list instead of claiming the cursor is on it', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    s.setTmManageTierFilter('work')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'work', ''))
    const { copies: _c, ...moved } = row({ tier: 'global', unit_id: 30 })
    pushMock.mockResolvedValue({ pair: { ...moved, source_text: '你好' }, error: null })
    listMock.mockResolvedValue(loaded(listing({ groups: [THREE_ROWS.groups[0]] })))
    await s.pushTmManagePair()
    expect(s.tmManageActionNotice.value).toBe('pushed_unlisted')
  })
})

describe('write failures re-list', () => {
  it('a failure with any code re-lists, since the write may be partial', async () => {
    const s = await fresh()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await s.openTmManage()
    await s.deleteTmManagePair()
    deleteMock.mockResolvedValue({ ok: false, error: ipcError('tm.lookup_failed') })
    listMock.mockClear()
    await s.deleteTmManagePair()
    expect(listMock).toHaveBeenCalledTimes(1)
    expect(s.tmManageActionError.value?.code).toBe('tm.lookup_failed')
  })
})

describe('copy and scrolling in the overlay', () => {
  it('the delete hint names Backspace and Delete; a one-copy row has the single-copy wording; bulk-done names only the Global tier under Global', async () => {
    const { state, i18n, Overlay } = await freshOverlay()
    listMock.mockResolvedValue(loaded(THREE_ROWS))
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await settle(wrapper)

    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'Delete' })
    await settle(wrapper)
    const hint = i18n.t('tm.manage.delete_confirm_hint_one')
    expect(wrapper.text()).toContain(hint)
    expect(hint).toContain('Backspace/Delete')
    expect(hint).not.toMatch(/1 bản/)

    state.setTmManageTierFilter('global')
    await vi.waitFor(() => expect(listMock).toHaveBeenLastCalledWith('all', 'global', ''))
    deleteOthersMock.mockResolvedValue({ outcome: { deleted_work: 0, deleted_global: 2 }, error: null })
    await state.deleteTmManageOthers()
    await state.deleteTmManageOthers()
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.bulk_done_global', { work: '0', global: '2' }))
    expect(wrapper.text()).not.toContain('Tác phẩm này và')
    wrapper.unmount()
  })

  it('moving the cursor scrolls the current row into view, nearest edge', async () => {
    const scroll = vi.fn()
    Object.defineProperty(Element.prototype, 'scrollIntoView', { value: scroll, configurable: true, writable: true })
    try {
      const { state, Overlay } = await freshOverlay()
      listMock.mockResolvedValue(loaded(THREE_ROWS))
      await state.openTmManage()
      const wrapper = mount(Overlay, { attachTo: document.body })
      await settle(wrapper)
      scroll.mockClear()

      state.nextTmManageRow()
      await settle(wrapper)
      expect(scroll).toHaveBeenCalledWith({ block: 'nearest' })
      expect((scroll.mock.contexts.at(-1) as HTMLElement).id).toBe('tm-option-work-2')
      wrapper.unmount()
    } finally {
      Reflect.deleteProperty(Element.prototype, 'scrollIntoView')
    }
  })
})

describe('accessible names, button keys, Work-closed view and Tab wrap', () => {
  async function mountedWith(value: TmManageListing) {
    const ctx = await freshOverlay()
    listMock.mockResolvedValue(loaded(value))
    await ctx.state.openTmManage()
    const wrapper = mount(ctx.Overlay, { attachTo: document.body })
    await settle(wrapper)
    return { ...ctx, wrapper }
  }

  it('an option inside a multi-target group is named by the group source and its own content', async () => {
    const { wrapper } = await mountedWith(THREE_ROWS)
    const options = wrapper.findAll('[role="option"]')
    for (const opt of options.slice(0, 2)) {
      const ids = (opt.attributes('aria-labelledby') ?? '').split(' ')
      expect(ids).toHaveLength(2)
      expect(document.getElementById(ids[0])?.textContent).toBe('你好')
      expect(ids[1]).toBe(opt.attributes('id'))
    }
    expect(options[2].attributes('aria-labelledby')).toBeUndefined()
    wrapper.unmount()
  })

  it('the single-row delete confirm warns about a Global copy and the edit hint says every copy is rewritten', async () => {
    const { state, i18n, wrapper } = await mountedWith(
      listing({
        groups: [
          {
            source_text: '你好',
            distinct_targets: 1,
            rows: [
              row({
                copies: [
                  { tier: 'work', unit_id: 1 },
                  { tier: 'global', unit_id: 5 },
                ],
              }),
            ],
          },
        ],
      }),
    )
    await state.deleteTmManagePair()
    await settle(wrapper)
    expect(wrapper.text()).toContain(i18n.t('tm.manage.delete_global_note'))
    expect(i18n.t('tm.manage.edit_hint')).toContain('Toàn cục')
    expect(i18n.t('tm.manage.edit_hint')).toContain('Tự dịch')
    wrapper.unmount()
  })

  it('Enter and Backspace on a focused action button neither edit nor arm a delete', async () => {
    const { state, wrapper } = await mountedWith(THREE_ROWS)
    const button = wrapper.get('.tm-actions button')
    ;(button.element as HTMLElement).focus()
    await button.trigger('keydown', { key: 'Enter' })
    await button.trigger('keydown', { key: 'Backspace' })
    expect(state.tmManageEditing.value).toBe(false)
    expect(state.tmManageDeletePending.value).toBe(false)
    wrapper.unmount()
  })

  it('with no Work open: push is disabled on a Work row, the work tier option is disabled and the note shows', async () => {
    const { i18n, wrapper } = await mountedWith(listing({ work_open: false }))
    expect(wrapper.get('.tm-actions button:nth-of-type(3)').attributes('disabled')).toBeDefined()
    const workOption = wrapper.findAll('.tm-toolbar select')[0].find('option[value="work"]')
    expect(workOption.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain(i18n.t('tm.manage.work_note'))
    wrapper.unmount()
  })

  it('Tab from the last stop wraps to the first and Shift+Tab from the first wraps to the last', async () => {
    const { wrapper } = await mountedWith(THREE_ROWS)
    const panel = wrapper.get('.tm-panel').element as HTMLElement
    const stops = Array.from(
      panel.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
          'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    )
    expect(stops.length).toBeGreaterThan(2)

    stops[stops.length - 1].focus()
    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(stops[0])

    await wrapper.get('.tm-scrim').trigger('keydown', { key: 'Tab', shiftKey: true })
    expect(document.activeElement).toBe(stops[stops.length - 1])
    wrapper.unmount()
  })
})
