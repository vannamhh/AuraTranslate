/**
 * Memory cell of the "only my pairs" switch (`tmPolicyState.ts`) and the re-scan the toggle command
 * triggers. `config/tm.ts` is the IPC boundary and is mocked.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const getMock = vi.fn()
const setMock = vi.fn()
const syncStripMock = vi.fn()
const refreshSuggestionsMock = vi.fn()

vi.mock('../../src/config/tm', () => ({
  tmOnlyMineGet: (...args: unknown[]) => getMock(...args),
  tmOnlyMineSet: (...args: unknown[]) => setMock(...args),
}))
vi.mock('../../src/tmFuzzyStripState', () => ({ syncTmFuzzyStrip: (...args: unknown[]) => syncStripMock(...args) }))
vi.mock('../../src/glossaryConfirmStripState', () => ({
  refreshGlossaryConfirmTmSuggestions: () => refreshSuggestionsMock(),
}))

const FAILURE = { code: 'work.none_open', message_key: 'err.unknown', params: {}, retryable: false }

async function fresh() {
  vi.resetModules()
  getMock.mockReset()
  setMock.mockReset()
  syncStripMock.mockReset()
  refreshSuggestionsMock.mockReset()
  return import('../../src/tmPolicyState')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('tmPolicyState', () => {
  it('is unknown (null, not false) until Rust answers', async () => {
    const s = await fresh()
    expect(s.tmPolicyOnlyMine.value).toBeNull()
    expect(s.tmPolicyWorkOpen.value).toBeNull()
    getMock.mockResolvedValue({ state: { work_open: true, only_mine: true }, error: null })
    await s.loadTmPolicy()
    expect(s.tmPolicyOnlyMine.value).toBe(true)
    expect(s.tmPolicyWorkOpen.value).toBe(true)
  })

  it('a failed read keeps the value unknown and keeps the error', async () => {
    const s = await fresh()
    getMock.mockResolvedValue({ state: null, error: FAILURE })
    await s.loadTmPolicy()
    expect(s.tmPolicyOnlyMine.value).toBeNull()
    expect(s.tmPolicyError.value).toEqual(FAILURE)
  })

  it('toggle sends the opposite value and answers true when Rust accepted it', async () => {
    const s = await fresh()
    getMock.mockResolvedValue({ state: { work_open: true, only_mine: false }, error: null })
    await s.loadTmPolicy()
    setMock.mockResolvedValue({ state: { work_open: true, only_mine: true }, error: null })
    expect(await s.toggleTmOnlyMine()).toBe(true)
    expect(setMock).toHaveBeenCalledExactlyOnceWith(true)
    expect(s.tmPolicyOnlyMine.value).toBe(true)
  })

  it('toggle refuses while unknown, and while no Work is open', async () => {
    const s = await fresh()
    expect(await s.toggleTmOnlyMine()).toBe(false)
    getMock.mockResolvedValue({ state: { work_open: false, only_mine: false }, error: null })
    await s.loadTmPolicy()
    expect(await s.toggleTmOnlyMine()).toBe(false)
    expect(setMock).not.toHaveBeenCalled()
  })

  it('a refused toggle keeps the old value and answers false', async () => {
    const s = await fresh()
    getMock.mockResolvedValue({ state: { work_open: true, only_mine: false }, error: null })
    await s.loadTmPolicy()
    setMock.mockResolvedValue({ state: null, error: FAILURE })
    expect(await s.toggleTmOnlyMine()).toBe(false)
    expect(s.tmPolicyOnlyMine.value).toBe(false)
    expect(s.tmPolicyError.value).toEqual(FAILURE)
  })

  it('reset returns to unknown', async () => {
    const s = await fresh()
    getMock.mockResolvedValue({ state: { work_open: true, only_mine: true }, error: null })
    await s.loadTmPolicy()
    s.resetTmPolicy()
    expect(s.tmPolicyOnlyMine.value).toBeNull()
  })
})

describe('the toggle command', () => {
  async function deps() {
    const policy = await fresh()
    getMock.mockResolvedValue({ state: { work_open: true, only_mine: false }, error: null })
    await policy.loadTmPolicy()
    const { tmManageCommandDeps } = await import('../../src/tmManageCommandDeps')
    return tmManageCommandDeps()
  }

  it('re-scans the TM strip and the Glossary strip after Rust accepted the flip', async () => {
    const d = await deps()
    setMock.mockResolvedValue({ state: { work_open: true, only_mine: true }, error: null })
    d.toggleTmOnlyMine?.()
    await vi.waitFor(() => expect(syncStripMock).toHaveBeenCalledTimes(1))
    expect(refreshSuggestionsMock).toHaveBeenCalledTimes(1)
  })

  it('does not re-scan when Rust refused', async () => {
    const d = await deps()
    setMock.mockResolvedValue({ state: null, error: FAILURE })
    d.toggleTmOnlyMine?.()
    await vi.waitFor(() => expect(setMock).toHaveBeenCalled())
    await new Promise((r) => setTimeout(r, 0))
    expect(syncStripMock).not.toHaveBeenCalled()
    expect(refreshSuggestionsMock).not.toHaveBeenCalled()
  })
})
