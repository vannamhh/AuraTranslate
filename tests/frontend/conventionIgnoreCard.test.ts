/**
 * Thẻ "Không phải lỗi": rows drawn as …trước [cụm] sau…, forgetting one row asks nothing,
 * clearing asks once, and no Work open says so and offers no write.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import { flushPromises } from './support/flushMicrotasks'

const listMock = vi.fn()
const removeMock = vi.fn()
const clearMock = vi.fn()
const rulesMock = vi.fn()
const scanNowMock = vi.fn()
const UNAVAILABLE = { code: 'convention.work_tier_unavailable', message_key: 'err.convention.work_tier_unavailable', params: {}, retryable: false }

vi.mock('../../src/config/convention', () => ({
  CONVENTION_WORK_TIER_UNAVAILABLE: 'convention.work_tier_unavailable',
  listConventionRules: (tier: string) => rulesMock(tier),
  listConventionIgnores: () => listMock(),
  removeConventionIgnore: (id: number) => removeMock(id),
  clearConventionIgnores: () => clearMock(),
  scanConventionSegments: () => Promise.resolve({ scan: null, error: null }),
}))
vi.mock('../../src/conventionState', () => ({ scanConventionNow: () => scanNowMock() }))
vi.mock('../../src/panels/editorPanelState', () => ({
  editorChapterId: ref<number | null>(7),
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))

const state = await import('../../src/conventionManageState')
const Card = (await import('../../src/ConventionIgnoreCard.vue')).default
vi.mock('../../src/commands', () => ({
  dispatch: (id: string) => {
    if (id === 'convention.manage.ignored_forget') void state.forgetConventionManageIgnore()
    if (id === 'convention.manage.ignored_clear') void state.clearConventionManageIgnores()
  },
}))

const row = (id: number, before: string, phrase: string, after: string) => ({
  id,
  rule_key: phrase,
  avoid_phrase: phrase,
  context_before: before,
  context_after: after,
  created_at: '2026-11-10T00:00:00Z',
})
let rows: ReturnType<typeof row>[]
let workOpen: boolean

beforeEach(() => {
  state.resetConventionManage()
  rows = [row(1, 'nhằm ', 'đảm bảo', ' vệ'), row(2, '', 'kỷ', ''), row(3, 'a ', 'b', ' c')]
  workOpen = true
  rulesMock.mockReset().mockImplementation((tier: string) =>
    Promise.resolve(tier === 'work' && !workOpen ? { rules: null, error: UNAVAILABLE } : { rules: [], error: null }),
  )
  listMock.mockReset().mockImplementation(() =>
    Promise.resolve(workOpen ? { rows, error: null } : { rows: null, error: UNAVAILABLE }),
  )
  removeMock.mockReset().mockResolvedValue({ removed: true, error: null })
  clearMock.mockReset().mockResolvedValue({ cleared: 3, error: null })
  scanNowMock.mockReset()
})

afterEach(() => {
  document.body.innerHTML = ''
})

async function mounted() {
  await state.openConventionManage()
  state.setConventionManageTab('ignored')
  const wrapper = mount(Card, { attachTo: document.body })
  await nextTick()
  return wrapper
}

describe('ConventionIgnoreCard', () => {
  it('draws one row per remembered place with the matched phrase between its contexts', async () => {
    const wrapper = await mounted()
    const lines = wrapper.findAll('[data-convention-ignore-row]').map((r) => r.find('.cvi-text').text())
    expect(lines).toHaveLength(3)
    expect(lines[0]).toContain('nhằm')
    expect(lines[0]).toContain('đảm bảo')
    expect(lines[0]).toContain('vệ')
    expect(lines[1]).toBe('kỷ')
    wrapper.unmount()
  })

  it('forgets the selected row with no confirmation and shows the done line', async () => {
    const wrapper = await mounted()
    rows = [rows[0], rows[2]]
    await wrapper.findAll('[data-convention-ignore-row]')[1].trigger('pointerdown')
    await wrapper.findAll('[data-convention-ignore-forget]')[1].trigger('click')
    await flushPromises()
    await nextTick()
    expect(removeMock).toHaveBeenCalledTimes(1)
    expect(removeMock).toHaveBeenCalledWith(2)
    expect(wrapper.findAll('[data-convention-ignore-row]')).toHaveLength(2)
    expect(wrapper.find('[data-convention-ignore-done]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('clears in two presses: the first writes nothing', async () => {
    const wrapper = await mounted()
    const button = wrapper.find('[data-convention-ignore-clear]')
    await button.trigger('click')
    await nextTick()
    expect(clearMock).not.toHaveBeenCalled()
    expect(state.conventionManageIgnoreClearPending.value).toBe(true)
    rows = []
    await button.trigger('click')
    await flushPromises()
    await nextTick()
    expect(clearMock).toHaveBeenCalledTimes(1)
    expect(wrapper.find('[data-convention-ignore-empty]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('shows the empty state when the Work has no remembered place, and no clear action is live', async () => {
    rows = []
    const wrapper = await mounted()
    expect(wrapper.find('[data-convention-ignore-empty]').exists()).toBe(true)
    expect(wrapper.find('[data-convention-ignore-clear]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('with no Work open says a Work is needed and offers no write', async () => {
    workOpen = false
    const wrapper = await mounted()
    expect(wrapper.find('[data-convention-ignore-needs-work]').exists()).toBe(true)
    expect(wrapper.find('[data-convention-ignore-clear]').exists()).toBe(false)
    expect(wrapper.find('[data-convention-ignore-forget]').exists()).toBe(false)
    expect(removeMock).not.toHaveBeenCalled()
    expect(clearMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
