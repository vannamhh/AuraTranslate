import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const invokeMock = vi.fn()

const ROWS = [
  { id: 2, kind: 'grammar', phrase: 'nhin', context_before: 'toi ', context_after: ' thay', created_at: 't', blocked_count: null },
  { id: 1, kind: 'spelling', phrase: 'sai', context_before: '', context_after: '', created_at: 't', blocked_count: null },
]

function seg(id: number, text: string): ChapterSegment {
  return { ...FIXTURE_SEGMENTS[0], id, ord: id, target_text: text }
}

type Handler = (cmd: string, args?: Record<string, unknown>) => unknown

async function fresh(handler: Handler = () => null) {
  vi.resetModules()
  invokeMock.mockReset()
  invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === 'ai_proofread_ignore_list') return Promise.resolve(ROWS)
    if (cmd === 'ai_proofread_segments') return Promise.resolve({ state: 'done', not_scanned: 0, filtered: 2, blocked: [] })
    return Promise.resolve(handler(cmd, args))
  })
  const caret = ref<number | null>(5)
  const segments = ref<ChapterSegment[]>([seg(5, 'toi nhin thay sai')])
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => invokeMock(...args),
    Channel: class {
      onmessage: (v: unknown) => void = () => {}
    },
  }))
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caret,
    editorLastCaretSegmentId: caret,
    editorSegments: segments,
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
  }))
  vi.doMock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: ref<number[]>([]) }))
  const commands = await import('../../src/commands')
  const { proofreadHandlers } = await import('../../src/proofreadHandlers')
  const state = await import('../../src/proofreadState')
  const memory = await import('../../src/proofreadMemoryState')
  const identity = await import('../../src/openWorkIdentity')
  const Overlay = (await import('../../src/ProofreadMemoryOverlay.vue')).default
  const Strip = (await import('../../src/ProofreaderStrip.vue')).default
  commands.installCommands({ ...proofreadHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, caret, state, memory, identity, Overlay, Strip }
}

function calls(name: string): Record<string, unknown>[] {
  return invokeMock.mock.calls.filter((c) => c[0] === name).map((c) => (c[1] ?? {}) as Record<string, unknown>)
}

const FINDING = { kind: 'grammar', start: 4, end: 8, explanation: 'e', suggestion: 'nhìn' }

async function scanOne(
  state: Awaited<ReturnType<typeof fresh>>['state'],
  scope: 'segment' | 'selection' | 'chapter' | null,
  outcome: unknown,
  findings: unknown[] = [FINDING],
): Promise<void> {
  const cfg = await import('../../src/config/proofread')
  vi.spyOn(cfg, 'runProofreadSegments').mockImplementation(((_ids: number[], onEvent: (e: unknown) => void) => {
    onEvent({ kind: 'planned', segments: 1, batches: 1 })
    onEvent({
      kind: 'batch',
      usage: null,
      segments: [{ segment_id: 5, scanned_text: 'toi nhin thay sai', findings, unlocated: 0, ambiguous: 0 }],
    })
    return Promise.resolve(outcome)
  }) as never)
  await state.runProofread([5], scope)
}

beforeEach(() => {
  vi.doUnmock('@tauri-apps/api/core')
})
afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('ignoreProofreadFinding', () => {
  it('sends the finding, its kind and the scanned text, then drops it from the strip', async () => {
    const { state } = await fresh()
    await scanOne(state, 'segment', { value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    expect(state.proofreadFindingCount.value).toBe(1)
    await state.ignoreProofreadFinding()
    expect(calls('ai_proofread_ignore_add')).toEqual([
      { segmentId: 5, expectedText: 'toi nhin thay sai', start: 4, end: 8, expectedPhrase: 'nhin', kind: 'grammar' },
    ])
    expect(state.proofreadFindingCount.value).toBe(0)
  })

  it('keeps the finding and shows the error when Rust refuses', async () => {
    const err = { code: 'ai_proofread.text_changed', message_key: 'err.ai_proofread.text_changed', params: { segment_id: '5' }, retryable: false }
    const { state } = await fresh((cmd) => {
      if (cmd === 'ai_proofread_ignore_add') throw err
      return null
    })
    invokeMock.mockImplementation((cmd: string) =>
      cmd === 'ai_proofread_ignore_add' ? Promise.reject(err) : Promise.resolve(null),
    )
    await scanOne(state, 'segment', { value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    await state.ignoreProofreadFinding()
    expect(state.proofreadFindingCount.value).toBe(1)
    expect(state.proofreadActionError.value).toEqual(err)
  })

  it('"Bỏ qua" of 9.4 stays memory only: no ai_proofread_ignore_add', async () => {
    const { state } = await fresh()
    await scanOne(state, 'segment', { value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    state.dismissProofreadFinding()
    expect(calls('ai_proofread_ignore_add')).toHaveLength(0)
    expect(state.proofreadFindingCount.value).toBe(0)
  })
})

describe('the scan record', () => {
  const DONE = (filtered: number) => ({ value: { state: 'done', not_scanned: 0, filtered, blocked: [] }, error: null })

  it('is written once after Done with the Work, the scope, the findings that arrived and the filtered total', async () => {
    const { state, identity } = await fresh()
    identity.setOpenWorkId('work-1')
    await scanOne(state, 'chapter', DONE(2), [FINDING, { ...FINDING, start: 0, end: 3 }])
    await vi.waitFor(() => {
      expect(calls('ai_proofread_scan_record')).toHaveLength(1)
    })
    expect(calls('ai_proofread_scan_record')[0]).toEqual({ workId: 'work-1', scope: 'chapter', reported: 2, filtered: 2, blocked: [] })
    expect(state.proofreadFiltered.value).toBe(2)
  })

  it('counts the findings that reached the strip even if the user dismissed them', async () => {
    const { state, identity } = await fresh()
    identity.setOpenWorkId('work-1')
    await scanOne(state, 'segment', DONE(0))
    expect(calls('ai_proofread_scan_record')[0]).toMatchObject({ reported: 1 })
  })

  it.each([
    ['a cancelled scan', { value: { state: 'cancelled', not_scanned: 1 }, error: null }],
    ['a failed scan', { value: null, error: { code: 'x', message_key: 'x', params: {}, retryable: false } }],
    ['no AI configured', { value: { state: 'not_configured' }, error: null }],
  ])('is not written after %s', async (_name, outcome) => {
    const { state, identity } = await fresh()
    identity.setOpenWorkId('work-1')
    await scanOne(state, 'segment', outcome)
    expect(calls('ai_proofread_scan_record')).toHaveLength(0)
  })

  it('is skipped, with a warning, while no open Work id is known', async () => {
    const { state } = await fresh()
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    await scanOne(state, 'segment', DONE(0))
    expect(calls('ai_proofread_scan_record')).toHaveLength(0)
    expect(warn).toHaveBeenCalled()
  })

  it('a rejected record does not disturb the scan result', async () => {
    const { state, identity } = await fresh()
    identity.setOpenWorkId('work-1')
    invokeMock.mockImplementation((cmd: string) =>
      cmd === 'ai_proofread_scan_record'
        ? Promise.reject({ code: 'ai_proofread.scan_record_rejected', message_key: 'k', params: {}, retryable: false })
        : Promise.resolve(null),
    )
    vi.spyOn(console, 'warn').mockImplementation(() => {})
    await scanOne(state, 'segment', DONE(0))
    await vi.waitFor(() => {
      expect(calls('ai_proofread_scan_record')).toHaveLength(1)
    })
    expect(state.proofreadStateValue.value).toBe('done')
  })
})

describe('ProofreaderStrip', () => {
  it('shows the filtered total after a scan, and the "Không phải lỗi" button next to "Bỏ qua"', async () => {
    const { state, Strip } = await fresh()
    const { t } = await import('../../src/i18n')
    const wrapper = mount(Strip, { attachTo: document.body })
    await scanOne(state, 'segment', { value: { state: 'done', not_scanned: 0, filtered: 3, blocked: [] }, error: null })
    await nextTick()
    expect(wrapper.text()).toContain(t('proofread.strip.filtered', { filtered_count: '3' }))
    expect(wrapper.text()).toContain(t('command.ai.proofread.ignore'))
    wrapper.unmount()
  })

  it('does not mention filtering when nothing was filtered', async () => {
    const { state, Strip } = await fresh()
    const { t } = await import('../../src/i18n')
    const wrapper = mount(Strip, { attachTo: document.body })
    await scanOne(state, 'segment', { value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    await nextTick()
    expect(wrapper.text()).not.toContain(t('proofread.strip.filtered', { filtered_count: '0' }))
    wrapper.unmount()
  })

  it('the button dispatches ai.proofread.ignore', async () => {
    const { state, Strip } = await fresh()
    const { t } = await import('../../src/i18n')
    const wrapper = mount(Strip, { attachTo: document.body })
    await scanOne(state, 'segment', { value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    await nextTick()
    const button = wrapper.findAll('button').find((b) => b.text() === t('command.ai.proofread.ignore'))
    await button?.trigger('click')
    await vi.waitFor(() => {
      expect(calls('ai_proofread_ignore_add')).toHaveLength(1)
    })
    wrapper.unmount()
  })
})

describe('ProofreadMemoryOverlay', () => {
  it('renders nothing until opened, then lists the rows newest first with their context', async () => {
    const { memory, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    expect(wrapper.find('[data-proofread-memory-row]').exists()).toBe(false)
    await memory.openProofreadMemory()
    await nextTick()
    const rows = wrapper.findAll('[data-proofread-memory-row]')
    expect(rows).toHaveLength(2)
    expect(rows[0]?.text()).toContain('nhin')
    expect(rows[0]?.text()).toContain('toi ')
    expect(rows[1]?.text()).toContain('sai')
    wrapper.unmount()
  })

  it('says it is empty only after the list has loaded', async () => {
    const { memory, Overlay } = await fresh()
    const { t } = await import('../../src/i18n')
    invokeMock.mockImplementation((cmd: string) => Promise.resolve(cmd === 'ai_proofread_ignore_list' ? [] : null))
    const wrapper = mount(Overlay, { attachTo: document.body })
    const opening = memory.openProofreadMemory()
    await nextTick()
    expect(wrapper.text()).not.toContain(t('proofread.memory.empty'))
    await opening
    await nextTick()
    expect(wrapper.text()).toContain(t('proofread.memory.empty'))
    wrapper.unmount()
  })

  it('shows the error and not "empty" when the list cannot be read', async () => {
    const { memory, Overlay } = await fresh()
    const { t } = await import('../../src/i18n')
    const err = { code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }
    invokeMock.mockImplementation(() => Promise.reject(err))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    expect(wrapper.text()).not.toContain(t('proofread.memory.empty'))
    expect(wrapper.find('[role="alert"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('forgets the selected row and reloads', async () => {
    const { memory, Overlay } = await fresh((cmd) => (cmd === 'ai_proofread_ignore_remove' ? true : null))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    expect(wrapper.find('[data-proofread-memory-forget]').attributes('disabled')).toBeDefined()
    await wrapper.findAll('[data-proofread-memory-row]')[1]?.trigger('pointerdown')
    await wrapper.find('[data-proofread-memory-forget]').trigger('click')
    await vi.waitFor(() => {
      expect(calls('ai_proofread_ignore_remove')).toEqual([{ id: 1 }])
    })
    await vi.waitFor(() => {
      expect(calls('ai_proofread_ignore_list')).toHaveLength(2)
    })
    wrapper.unmount()
  })

  it('"Xoá toàn bộ" clears nothing on the first click and clears on the second, with the irreversible hint between', async () => {
    const { memory, Overlay } = await fresh()
    const { t } = await import('../../src/i18n')
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    await nextTick()
    expect(calls('ai_proofread_ignore_clear')).toHaveLength(0)
    expect(wrapper.text()).toContain(t('proofread.memory.clear_hint', { memory_count: '2' }))
    expect(t('proofread.memory.clear_hint', { memory_count: '2' })).toContain('không hoàn tác được')
    invokeMock.mockImplementation((cmd: string) => Promise.resolve(cmd === 'ai_proofread_ignore_clear' ? 2 : cmd === 'ai_proofread_ignore_list' ? [] : null))
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    await vi.waitFor(() => {
      expect(calls('ai_proofread_ignore_clear')).toHaveLength(1)
    })
    await vi.waitFor(() => {
      expect(wrapper.text()).toContain(t('proofread.memory.cleared', { memory_count: '2' }))
    })
    wrapper.unmount()
  })

  it('selecting a row between the two clicks disarms the clear', async () => {
    const { memory, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    await wrapper.findAll('[data-proofread-memory-row]')[0]?.trigger('pointerdown')
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    await nextTick()
    expect(calls('ai_proofread_ignore_clear')).toHaveLength(0)
    expect(memory.proofreadMemoryClearPending.value).toBe(true)
    wrapper.unmount()
  })

  it('closing re-arms nothing: a reopened overlay starts with the first-click state', async () => {
    const { memory, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    memory.closeProofreadMemory()
    await memory.openProofreadMemory()
    expect(memory.proofreadMemoryClearPending.value).toBe(false)
    wrapper.unmount()
  })

  it('Escape closes it and returns the focus to the title-bar button', async () => {
    const { memory, Overlay } = await fresh()
    const opener = document.createElement('button')
    opener.setAttribute('data-proofread-memory-open', '')
    document.body.appendChild(opener)
    opener.focus()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    await nextTick()
    await wrapper.find('.pfmem-scrim').trigger('keydown', { key: 'Escape' })
    expect(memory.proofreadMemoryIsOpen.value).toBe(false)
    await nextTick()
    expect(document.activeElement).toBe(opener)
    wrapper.unmount()
  })

  it('after forget and after clear, focus is back inside the panel so Esc still closes it', async () => {
    const { memory, Overlay } = await fresh((cmd) => {
      if (cmd !== 'ai_proofread_ignore_remove' && cmd !== 'ai_proofread_ignore_clear') return null
      ;(document.activeElement as HTMLElement | null)?.blur()
      return cmd === 'ai_proofread_ignore_remove' ? true : 2
    })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await memory.openProofreadMemory()
    await nextTick()
    const panelEl = (): Element | null => document.querySelector('.pfmem-panel')
    await wrapper.findAll('[data-proofread-memory-row]')[0]?.trigger('pointerdown')
    const forget = wrapper.find('[data-proofread-memory-forget]').element as HTMLElement
    forget.focus()
    await wrapper.find('[data-proofread-memory-forget]').trigger('click')
    await vi.waitFor(() => {
      expect(calls('ai_proofread_ignore_remove')).toHaveLength(1)
    })
    await vi.waitFor(() => {
      expect(panelEl()?.contains(document.activeElement)).toBe(true)
    })
    await wrapper.find('.pfmem-scrim').trigger('keydown', { key: 'Escape' })
    expect(memory.proofreadMemoryIsOpen.value).toBe(false)
    await memory.openProofreadMemory()
    await nextTick()
    const clear = wrapper.find('[data-proofread-memory-clear]').element as HTMLElement
    clear.focus()
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    await wrapper.find('[data-proofread-memory-clear]').trigger('click')
    await vi.waitFor(() => {
      expect(calls('ai_proofread_ignore_clear')).toHaveLength(1)
    })
    await vi.waitFor(() => {
      expect(panelEl()?.contains(document.activeElement)).toBe(true)
    })
    wrapper.unmount()
  })

  it('the memory commands open, close, forget and clear through dispatch', async () => {
    const { commands, memory } = await fresh()
    commands.dispatch('ai.proofread.memory.open')
    await vi.waitFor(() => {
      expect(memory.proofreadMemoryHasLoaded.value).toBe(true)
    })
    commands.dispatch('ai.proofread.memory.forget')
    await Promise.resolve()
    expect(calls('ai_proofread_ignore_remove')).toHaveLength(0)
    commands.dispatch('ai.proofread.memory.clear')
    commands.dispatch('ai.proofread.memory.close')
    expect(memory.proofreadMemoryIsOpen.value).toBe(false)
  })
})

describe('openWorkIdentity', () => {
  it('is null until a Work is created or opened, and can be forgotten', async () => {
    const { identity } = await fresh()
    expect(identity.openWorkId.value).toBeNull()
    identity.setOpenWorkId('w')
    expect(identity.openWorkId.value).toBe('w')
    identity.resetOpenWorkId()
    expect(identity.openWorkId.value).toBeNull()
  })
})
