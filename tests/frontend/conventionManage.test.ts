import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const addMock = vi.fn()
const editMock = vi.fn()
const deleteMock = vi.fn()
const enabledMock = vi.fn()
const listMock = vi.fn()
const ignoreListMock = vi.fn()
const ignoreRemoveMock = vi.fn()
const ignoreClearMock = vi.fn()
const scanNowMock = vi.fn()
const chapterId = ref<number | null>(7)

vi.mock('../../src/config/convention', () => ({
  scanConventionSegments: () => Promise.resolve({ scan: null, error: null }),
  CONVENTION_WORK_TIER_UNAVAILABLE: 'convention.work_tier_unavailable',
  addConventionRule: (...args: unknown[]) => addMock(...args),
  editConventionRule: (...args: unknown[]) => editMock(...args),
  deleteConventionRule: (...args: unknown[]) => deleteMock(...args),
  setConventionRuleEnabled: (...args: unknown[]) => enabledMock(...args),
  listConventionRules: (tier: string) => listMock(tier),
  addConventionIgnore: vi.fn(),
  listConventionIgnores: () => ignoreListMock(),
  removeConventionIgnore: (id: number) => ignoreRemoveMock(id),
  clearConventionIgnores: () => ignoreClearMock(),
}))
vi.mock('../../src/conventionState', () => ({ scanConventionNow: () => scanNowMock() }))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorChapterId: chapterId,
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))

const state = await import('../../src/conventionManageState')

type Status = 'in_use' | 'shadowed' | 'disabled_in_work'

function rule(id: number, tier: 'global' | 'work', avoid: string, replacement: string | null, status: Status = 'in_use') {
  return { id, rule_key: avoid, avoid_phrase: avoid, replacement_phrase: replacement, enabled: true, tier, status }
}

const GLOBAL = rule(1, 'global', 'đảm bảo', 'bảo đảm')
const OTHER = rule(2, 'global', 'kỷ', 'kỉ')
const UNAVAILABLE = {
  code: 'convention.work_tier_unavailable',
  message_key: 'err.convention.work_tier_unavailable',
  params: {},
  retryable: false,
}

let ignores: ReturnType<typeof ignoreRow>[]

function ignoreRow(id: number, phrase = 'đảm bảo') {
  return { id, rule_key: phrase, avoid_phrase: phrase, context_before: 'nhằm ', context_after: ' vệ', created_at: '2026-11-10' }
}

let lists: { global: ReturnType<typeof rule>[]; work: ReturnType<typeof rule>[] | null }

beforeEach(() => {
  state.resetConventionManage()
  lists = { global: [], work: [] }
  addMock.mockReset().mockResolvedValue({ ok: true, error: null })
  editMock.mockReset().mockResolvedValue({ ok: true, error: null })
  deleteMock.mockReset().mockResolvedValue({ ok: true, error: null })
  enabledMock.mockReset().mockResolvedValue({ ok: true, error: null })
  scanNowMock.mockReset()
  ignores = []
  ignoreListMock.mockReset().mockImplementation(() =>
    Promise.resolve(lists.work === null ? { rows: null, error: UNAVAILABLE } : { rows: ignores, error: null }),
  )
  ignoreRemoveMock.mockReset().mockResolvedValue({ removed: true, error: null })
  ignoreClearMock.mockReset().mockResolvedValue({ cleared: 0, error: null })
  listMock.mockReset().mockImplementation((tier: string) =>
    Promise.resolve(
      tier === 'global'
        ? { rules: lists.global, error: null }
        : lists.work === null
          ? { rules: null, error: UNAVAILABLE }
          : { rules: lists.work, error: null },
    ),
  )
  chapterId.value = 7
})

describe('Quản lý quy ước', () => {
  it('opens on the loaded list and clears the drafts', async () => {
    lists.global = [GLOBAL]
    await state.openConventionManage()
    expect(state.conventionManageIsOpen.value).toBe(true)
    expect(state.conventionManageHasLoaded.value).toBe(true)
    expect(state.conventionManageRules.value).toEqual([GLOBAL])
  })

  it('adds the trimmed rule to the chosen tier, sends a blank replacement as null, reloads and rescans once', async () => {
    await state.openConventionManage()
    state.conventionAvoidDraft.value = '  đảm bảo '
    state.conventionReplacementDraft.value = '   '
    state.conventionAddTier.value = 'work'
    lists.global = [GLOBAL]
    await state.submitConventionRule()
    expect(addMock).toHaveBeenCalledWith('work', 'đảm bảo', null)
    expect(state.conventionManageRules.value).toEqual([GLOBAL])
    expect(state.conventionAvoidDraft.value).toBe('')
    expect(scanNowMock).toHaveBeenCalledTimes(1)
  })

  it('defaults the add tier to Work when a Work is open and to Global when none is', async () => {
    await state.openConventionManage()
    expect(state.conventionAddTier.value).toBe('work')
    state.resetConventionManage()
    lists.work = null
    await state.openConventionManage()
    expect(state.conventionAddTier.value).toBe('global')
  })

  it('does not rescan when no Chapter is open', async () => {
    chapterId.value = null
    await state.openConventionManage()
    state.conventionAvoidDraft.value = 'x'
    await state.submitConventionRule()
    expect(scanNowMock).not.toHaveBeenCalled()
  })

  it('keeps the drafts and shows the error when Rust refuses the rule', async () => {
    await state.openConventionManage()
    state.conventionAvoidDraft.value = 'đảm bảo'
    addMock.mockResolvedValue({
      ok: false,
      error: { code: 'convention.duplicate_avoid_phrase', message_key: 'convention.duplicate_avoid_phrase', params: {}, retryable: false },
    })
    await state.submitConventionRule()
    expect(state.conventionManageError.value?.code).toBe('convention.duplicate_avoid_phrase')
    expect(state.conventionAvoidDraft.value).toBe('đảm bảo')
    expect(scanNowMock).not.toHaveBeenCalled()
  })

  it('locks the Work tier and falls back to Both when no Work is open', async () => {
    lists = { global: [GLOBAL], work: null }
    await state.openConventionManage()
    expect(state.conventionManageWorkTierAvailable.value).toBe(false)
    state.setConventionManageTier('work')
    expect(state.conventionManageTierFilter.value).toBe('both')
    expect(listMock).toHaveBeenCalledWith('work')
  })

  it('counts both tiers whole in the summary, whatever the filter and the search show', async () => {
    const shadowedGlobal = rule(3, 'global', 'bằng hữu', 'bạn hữu', 'shadowed')
    const offGlobal = rule(4, 'global', 'thập kỷ', 'thập niên', 'disabled_in_work')
    lists.global = [GLOBAL, shadowedGlobal, offGlobal]
    lists.work = [rule(5, 'work', 'bằng hữu', 'bạn bè')]
    await state.openConventionManage()
    const whole = { rule_count: '4', disabled_count: '1', shadowed_count: '1' }
    expect(state.conventionManageSummary.value).toEqual(whole)

    state.setConventionManageTier('work')
    expect(state.conventionManageRows.value).toHaveLength(1)
    expect(state.conventionManageSummary.value).toEqual(whole)

    state.setConventionManageSearch('đảm bảo')
    expect(state.conventionManageRows.value.length).toBeLessThan(4)
    expect(state.conventionManageSummary.value).toEqual(whole)
  })

  it('keeps the search filter when the tier changes and counts per tier', async () => {
    lists = { global: [GLOBAL, OTHER], work: [rule(1, 'work', 'bằng hữu', 'bạn bè')] }
    await state.openConventionManage()
    expect(state.conventionManageCounts.value).toEqual({ global: 2, work: 1, both: 3 })
    state.setConventionManageSearch('bảo')
    state.setConventionManageTier('global')
    expect(state.conventionManageSearch.value).toBe('bảo')
    expect(state.conventionManageRows.value).toEqual([GLOBAL])
    state.setConventionManageTier('work')
    expect(state.conventionManageRows.value).toEqual([])
  })

  it('deletes in two steps: the first press arms the row and writes nothing', async () => {
    lists.global = [GLOBAL]
    await state.openConventionManage()
    await state.deleteConventionManageRule()
    expect(state.conventionManageDeletePending.value).toBe(true)
    expect(deleteMock).not.toHaveBeenCalled()
    lists.global = []
    await state.deleteConventionManageRule()
    expect(deleteMock).toHaveBeenCalledWith('global', 1)
    expect(state.conventionManageRules.value).toEqual([])
    expect(scanNowMock).toHaveBeenCalledTimes(1)
  })

  it('cancels a pending delete with Esc before closing, and when the cursor leaves the row', async () => {
    lists.global = [GLOBAL, OTHER]
    await state.openConventionManage()
    await state.deleteConventionManageRule()
    state.cancelConventionManageDeleteConfirm()
    expect(state.conventionManageDeletePending.value).toBe(false)
    expect(state.conventionManageIsOpen.value).toBe(true)
    await state.deleteConventionManageRule()
    state.stepConventionManageCursor(1)
    expect(state.conventionManageDeletePending.value).toBe(false)
    expect(deleteMock).not.toHaveBeenCalled()
    state.closeConventionManage()
    expect(state.conventionManageIsOpen.value).toBe(false)
  })

  it('edits the cursor row in place and rescans once', async () => {
    lists.global = [GLOBAL]
    await state.openConventionManage()
    state.beginConventionManageEdit()
    expect(state.conventionManageEditing.value).toBe(true)
    state.conventionEditReplacement.value = '  '
    await state.saveConventionManageEdit()
    expect(editMock).toHaveBeenCalledWith('global', 1, 'đảm bảo', null)
    expect(state.conventionManageEditing.value).toBe(false)
    expect(scanNowMock).toHaveBeenCalledTimes(1)
  })

  it('switches a Global rule off for the Work and back on, deleting the Work row', async () => {
    lists.global = [GLOBAL]
    await state.openConventionManage()
    lists.global = [rule(1, 'global', 'đảm bảo', 'bảo đảm', 'disabled_in_work')]
    await state.disableConventionManageRule()
    expect(enabledMock).toHaveBeenLastCalledWith('đảm bảo', false)
    expect(state.conventionManageRows.value.map((r) => r.tier)).toEqual(['global'])
    await state.enableConventionManageRule()
    expect(enabledMock).toHaveBeenLastCalledWith('đảm bảo', true)
    expect(scanNowMock).toHaveBeenCalledTimes(2)
  })

  it('never offers to switch off a Work row or a shadowed Global row', async () => {
    const shadowed = rule(1, 'global', 'a', 'b', 'shadowed')
    const mine = rule(2, 'work', 'a', 'c')
    lists = { global: [shadowed], work: [mine] }
    await state.openConventionManage()
    expect(state.conventionRuleCanDisable(mine)).toBe(false)
    expect(state.conventionRuleCanDisable(shadowed)).toBe(false)
    expect(state.conventionManageRows.value.map((r) => r.tier)).toEqual(['work', 'global'])
  })

  it('reloads the list when Rust says the rule is gone, and does not rescan', async () => {
    lists.global = [GLOBAL]
    await state.openConventionManage()
    state.beginConventionManageEdit()
    editMock.mockResolvedValue({
      ok: false,
      error: { code: 'convention.rule_missing', message_key: 'err.convention.rule_missing', params: {}, retryable: false },
    })
    lists.global = []
    await state.saveConventionManageEdit()
    expect(state.conventionManageRules.value).toEqual([])
    expect(state.conventionManageError.value?.code).toBe('convention.rule_missing')
    expect(scanNowMock).not.toHaveBeenCalled()
  })

  describe('thẻ Không phải lỗi', () => {
    it('loads the remembered rows with the lists and opens on the Rules tab', async () => {
      ignores = [ignoreRow(1), ignoreRow(2)]
      await state.openConventionManage()
      expect(ignoreListMock).toHaveBeenCalledTimes(1)
      expect(state.conventionManageIgnoreRows.value.map((r) => r.id)).toEqual([1, 2])
      expect(state.conventionManageTab.value).toBe('rules')
      state.setConventionManageTab('ignored')
      expect(state.conventionManageTab.value).toBe('ignored')
    })

    it('forgets one row without asking, reloads and rescans once', async () => {
      ignores = [ignoreRow(1), ignoreRow(2), ignoreRow(3)]
      await state.openConventionManage()
      state.setConventionManageTab('ignored')
      state.selectConventionManageIgnore(2)
      ignores = [ignoreRow(1), ignoreRow(3)]
      await state.forgetConventionManageIgnore()
      expect(ignoreRemoveMock).toHaveBeenCalledTimes(1)
      expect(ignoreRemoveMock).toHaveBeenCalledWith(2)
      expect(state.conventionManageIgnoreRows.value.map((r) => r.id)).toEqual([1, 3])
      expect(state.conventionManageIgnoreDone.value).toEqual({ kind: 'forgot', count: 1 })
      expect(scanNowMock).toHaveBeenCalledTimes(1)
    })

    it('treats a row that is already gone as harmless: no error, still reloads', async () => {
      ignores = [ignoreRow(1)]
      await state.openConventionManage()
      state.setConventionManageTab('ignored')
      state.selectConventionManageIgnore(1)
      ignoreRemoveMock.mockResolvedValue({ removed: false, error: null })
      ignores = []
      await state.forgetConventionManageIgnore()
      expect(state.conventionManageError.value).toBeNull()
      expect(state.conventionManageIgnoreRows.value).toEqual([])
      expect(state.conventionManageIgnoreDone.value).toBeNull()
    })

    it('clears everything in two steps: the first press arms and writes nothing', async () => {
      ignores = [ignoreRow(1), ignoreRow(2)]
      await state.openConventionManage()
      state.setConventionManageTab('ignored')
      await state.clearConventionManageIgnores()
      expect(state.conventionManageIgnoreClearPending.value).toBe(true)
      expect(ignoreClearMock).not.toHaveBeenCalled()
      ignoreClearMock.mockResolvedValue({ cleared: 2, error: null })
      ignores = []
      await state.clearConventionManageIgnores()
      expect(ignoreClearMock).toHaveBeenCalledTimes(1)
      expect(state.conventionManageIgnoreClearPending.value).toBe(false)
      expect(state.conventionManageIgnoreRows.value).toEqual([])
      expect(state.conventionManageIgnoreDone.value).toEqual({ kind: 'cleared', count: 2 })
      expect(scanNowMock).toHaveBeenCalledTimes(1)
    })

    it('disarms a pending clear when another row is selected or the tab changes', async () => {
      ignores = [ignoreRow(1), ignoreRow(2)]
      await state.openConventionManage()
      state.setConventionManageTab('ignored')
      await state.clearConventionManageIgnores()
      state.selectConventionManageIgnore(2)
      expect(state.conventionManageIgnoreClearPending.value).toBe(false)
      await state.clearConventionManageIgnores()
      state.setConventionManageTab('rules')
      expect(state.conventionManageIgnoreClearPending.value).toBe(false)
      expect(ignoreClearMock).not.toHaveBeenCalled()
    })

    it('with no Work open: no rows, the Work tier flag is off, and no write command is issued', async () => {
      lists = { global: [GLOBAL], work: null }
      await state.openConventionManage()
      expect(state.conventionManageWorkTierAvailable.value).toBe(false)
      expect(state.conventionManageIgnoreRows.value).toEqual([])
      state.setConventionManageTab('ignored')
      await state.forgetConventionManageIgnore()
      await state.clearConventionManageIgnores()
      await state.clearConventionManageIgnores()
      expect(ignoreRemoveMock).not.toHaveBeenCalled()
      expect(ignoreClearMock).not.toHaveBeenCalled()
    })

    it('drops the remembered rows, selection and tab on reset', async () => {
      ignores = [ignoreRow(1)]
      await state.openConventionManage()
      state.setConventionManageTab('ignored')
      state.selectConventionManageIgnore(1)
      state.resetConventionManage()
      expect(state.conventionManageIgnoreRows.value).toEqual([])
      expect(state.conventionManageIgnoreSelectedId.value).toBeNull()
      expect(state.conventionManageTab.value).toBe('rules')
    })
  })
})
