import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { GlossaryCandidate } from '../../src/config/glossary'
import type { CommandDeps } from '../../src/commands'

const pendingMock = vi.fn()
const approveMock = vi.fn()
const rejectMock = vi.fn()
const lookupMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  approveGlossaryCandidate: (...args: unknown[]) => approveMock(...args),
  rejectGlossaryCandidate: (id: number) => rejectMock(id),
  countGlossaryEntries: async () => null,
}))

function harvest(over: Partial<GlossaryCandidate> = {}): GlossaryCandidate {
  return {
    id: 1,
    source_term: 'Bắc Lương vương',
    candidate_origin: 'review_harvest',
    resolution: null,
    created_at: '2026-10-15T00:00:00.000Z',
    occurrence_count: 24,
    context_example: null,
    han_viet_suggestion: null,
    han_viet_status: 'not_chinese',
    chapter_span_count: 0,
    replaced_translation: 'Bắc Lương vương',
    proposed_translation: 'vương Bắc Lương',
    changed_count: 23,
    seen_count: 24,
    ...over,
  }
}

async function fresh(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  pendingMock.mockReset()
  approveMock.mockReset()
  rejectMock.mockReset()
  lookupMock.mockReset()
  lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const state = await import('../../src/glossaryHarvestState')
  const i18n = await import('../../src/i18n')
  const Overlay = (await import('../../src/GlossaryHarvestOverlay.vue')).default
  return { commands, state, i18n, Overlay }
}

const three = [
  harvest({ id: 1 }),
  harvest({ id: 2, source_term: 'đả khai', replaced_translation: 'đả khai', proposed_translation: 'mở', changed_count: 19, seen_count: 19 }),
  harvest({ id: 3, source_term: 'hành lang', replaced_translation: 'hành lang', proposed_translation: 'hàng lang', changed_count: 4, seen_count: 11 }),
]

beforeEach(() => {
  document.body.innerHTML = ''
})
afterEach(() => {
  vi.restoreAllMocks()
})

describe('glossaryHarvestState', () => {
  it('lists only review_harvest candidates, pre-selecting the consistent ones', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({
      candidates: [...three, harvest({ id: 9, candidate_origin: 'import_scan' })],
      error: null,
    })
    await state.openGlossaryHarvest()
    expect(state.harvestRows.value.map((r) => [r.candidate.id, r.selected])).toEqual([
      [1, true],
      [2, true],
      [3, false],
    ])
  })

  it('accepting the ticked rows writes one id at a time with the proposed translation and leaves the rest', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await state.openGlossaryHarvest()
    await state.acceptGlossaryHarvestSelected()
    expect(approveMock.mock.calls.map((c) => [c[0], c[1]])).toEqual([
      [1, 'vương Bắc Lương'],
      [2, 'mở'],
    ])
    expect(rejectMock).not.toHaveBeenCalled()
    expect(state.harvestRows.value.map((r) => r.outcome)).toEqual(['accepted', 'accepted', null])
  })

  it('the next id is not written until the previous write has returned', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    let release: (value: { value: number; error: null }) => void = () => {}
    approveMock
      .mockImplementationOnce(() => new Promise((resolve) => { release = resolve }))
      .mockResolvedValue({ value: 2, error: null })
    await state.openGlossaryHarvest()
    const running = state.acceptGlossaryHarvestSelected()
    await Promise.resolve()
    expect(approveMock).toHaveBeenCalledTimes(1)
    release({ value: 1, error: null })
    await running
    expect(approveMock).toHaveBeenCalledTimes(2)
  })

  it('a failed write marks only its row, keeps it ticked, and the batch continues', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    const conflict = { code: 'glossary.conflict', message_key: 'err.glossary.queue_already_decided', params: {}, retryable: false }
    approveMock.mockResolvedValueOnce({ value: null, error: conflict }).mockResolvedValueOnce({ value: 2, error: null })
    await state.openGlossaryHarvest()
    await state.acceptGlossaryHarvestSelected()
    const rows = state.harvestRows.value
    expect(rows[0]?.outcome).toBeNull()
    expect(rows[0]?.selected).toBe(true)
    expect(rows[0]?.error).not.toBeNull()
    expect(rows[1]?.outcome).toBe('accepted')
  })

  it('closing writes nothing', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    await state.openGlossaryHarvest()
    state.closeGlossaryHarvest()
    expect(approveMock).not.toHaveBeenCalled()
    expect(rejectMock).not.toHaveBeenCalled()
    expect(state.harvestOverlayIsOpen.value).toBe(false)
  })

  it('an empty list with no Work open is no_work, not an empty list', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({ candidates: [], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: false })
    await state.openGlossaryHarvest()
    expect(state.harvestStatus.value).toBe('no_work')
  })

  it('deciding the row under the cursor moves the cursor to the next undecided row', async () => {
    const { state } = await fresh()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    rejectMock.mockResolvedValue({ ok: true, error: null })
    await state.openGlossaryHarvest()
    await state.acceptGlossaryHarvestRow()
    expect(state.harvestCursor.value).toBe(1)
    await state.rejectGlossaryHarvestRow()
    expect(state.harvestCursor.value).toBe(2)
  })

  it('a read error is an error state, not an empty list', async () => {
    const { state } = await fresh()
    const err = { code: 'x', message_key: 'err.glossary.scope_error', params: {}, retryable: false }
    pendingMock.mockResolvedValue({ candidates: null, error: err })
    await state.openGlossaryHarvest()
    expect(state.harvestStatus.value).toBe('error')
  })
})

describe('GlossaryHarvestOverlay.vue', () => {
  it('renders one row per harvest candidate, the inconsistent one with its words and the counter on the button', async () => {
    const { state, i18n, Overlay } = await fresh()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    await state.openGlossaryHarvest()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.gh-row')).toHaveLength(3)
    expect(wrapper.findAll('.gh-row')[2]?.text()).toContain(
      i18n.t('glossary.harvest.change_inconsistent', { changed: '4', seen: '11' }),
    )
    expect(wrapper.get('.gh-act-primary').text()).toBe(i18n.t('glossary.harvest.accept_selected', { count: '2' }))
    wrapper.unmount()
  })

  it('"Để lại trong bảng chờ" dispatches the close command and calls no write', async () => {
    const closeGlossaryHarvest = vi.fn()
    const { state, i18n, Overlay } = await fresh({ closeGlossaryHarvest })
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    await state.openGlossaryHarvest()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const leave = wrapper.findAll('.gh-act').find((b) => b.text() === i18n.t('glossary.harvest.leave'))
    await leave?.trigger('click')
    expect(closeGlossaryHarvest).toHaveBeenCalledTimes(1)
    expect(approveMock).not.toHaveBeenCalled()
    expect(rejectMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('focus returns to the panel when the focused row button disappears after a decision', async () => {
    const { state, Overlay } = await fresh()
    pendingMock.mockResolvedValue({ candidates: [harvest({ id: 1 })], error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await state.openGlossaryHarvest()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const button = wrapper.get('.gh-mini').element as HTMLElement
    button.focus()
    await state.acceptGlossaryHarvestRow()
    button.remove()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    expect(document.activeElement).toBe(wrapper.get('.gh-panel').element)
    wrapper.unmount()
  })

  it('shows the empty sentence when no harvest is waiting', async () => {
    const { state, i18n, Overlay } = await fresh()
    pendingMock.mockResolvedValue({ candidates: [harvest({ id: 5, candidate_origin: 'import_scan' })], error: null })
    await state.openGlossaryHarvest()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain(i18n.t('glossary.harvest.empty'))
    wrapper.unmount()
  })

  it('Esc closes without a write and returns focus to the button that opened it', async () => {
    let closeIt = () => {}
    const { state, Overlay } = await fresh({ closeGlossaryHarvest: () => closeIt() })
    closeIt = () => state.closeGlossaryHarvest()
    pendingMock.mockResolvedValue({ candidates: three, error: null })
    const opener = document.createElement('button')
    opener.setAttribute('data-glossary-harvest-open', '')
    document.body.appendChild(opener)
    opener.focus()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openGlossaryHarvest()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    expect(document.querySelector('.gh-panel')?.contains(document.activeElement)).toBe(true)

    await wrapper.get('.gh-scrim').trigger('keydown', { key: 'Escape' })
    await wrapper.vm.$nextTick()
    expect(state.harvestOverlayIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(opener)
    expect(approveMock).not.toHaveBeenCalled()
    expect(rejectMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
