import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const addMock = vi.fn()
const listMock = vi.fn()
const scanNowMock = vi.fn()
const chapterId = ref<number | null>(7)

vi.mock('../../src/config/convention', () => ({
  addConventionRule: (...args: unknown[]) => addMock(...args),
  listConventionRules: () => listMock(),
}))
vi.mock('../../src/conventionState', () => ({ scanConventionNow: () => scanNowMock() }))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorChapterId: chapterId,
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))

const state = await import('../../src/conventionManageState')

const RULE = { id: 1, rule_key: 'đảm bảo', avoid_phrase: 'đảm bảo', replacement_phrase: 'bảo đảm', enabled: true, tier: 'global' as const }

beforeEach(() => {
  state.closeConventionManage()
  addMock.mockReset().mockResolvedValue({ ok: true, error: null })
  listMock.mockReset().mockResolvedValue({ rules: [], error: null })
  scanNowMock.mockReset()
  chapterId.value = 7
})

describe('Quản lý quy ước', () => {
  it('opens on the loaded list and clears the drafts', async () => {
    listMock.mockResolvedValue({ rules: [RULE], error: null })
    await state.openConventionManage()
    expect(state.conventionManageIsOpen.value).toBe(true)
    expect(state.conventionManageHasLoaded.value).toBe(true)
    expect(state.conventionManageRules.value).toEqual([RULE])
  })

  it('adds the trimmed rule, sends a blank replacement as null, reloads and rescans the open Chapter', async () => {
    await state.openConventionManage()
    state.conventionAvoidDraft.value = '  đảm bảo '
    state.conventionReplacementDraft.value = '   '
    listMock.mockResolvedValue({ rules: [RULE], error: null })
    await state.submitConventionRule()
    expect(addMock).toHaveBeenCalledWith('đảm bảo', null)
    expect(state.conventionManageRules.value).toEqual([RULE])
    expect(state.conventionAvoidDraft.value).toBe('')
    expect(scanNowMock).toHaveBeenCalledTimes(1)
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
})
