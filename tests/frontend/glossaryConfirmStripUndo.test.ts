/**
 * Undo of the strip's confirm (FR147, AD-54 items 5-6): each event that kills the button, the
 * "Để sau" set after a successful undo, the failure branch, and the chord hint in the notice.
 * Mocks only the IPC boundary and the editor/command/shortcut modules the strip imports.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type { GlossarySegmentSource } from '../../src/panels/glossaryMarksMap'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const MARK = '2026-10-18T01:41:01.014Z'
const SEGMENTS: GlossarySegmentSource[] = [
  { id: 1, source_text: '萧炎和薰儿登场' },
  { id: 2, source_text: '别处' },
]

function markWire(opts: { start: number; end: number; id: number; sourceTerm: string; confirmed?: boolean; translation?: string | null }) {
  return {
    start: opts.start,
    end: opts.end,
    tier: 'global' as const,
    is_confirmed: opts.confirmed ?? false,
    translation: opts.translation ?? null,
    id: opts.id,
    source_term: opts.sourceTerm,
    han_viet_suggestion: null,
    han_viet_status: 'not_requested' as const,
    occurrence_count: null,
    term_origin: 'manual' as const,
    category: 'other' as const,
  }
}

const LEFT = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
const RIGHT = markWire({ start: 3, end: 5, id: 8, sourceTerm: '薰儿' })

type Chord = { id: string; display: string[] }

async function fresh(chords: Chord[] = []) {
  vi.resetModules()
  mockInvoke.mockReset()
  const caretSegmentId: Ref<number | null> = ref(null)
  const segments: Ref<GlossarySegmentSource[]> = ref([])
  const dispatchMock = vi.fn()
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caretSegmentId,
    editorSegments: segments,
  }))
  vi.doMock('../../src/glossaryQuickAddState', () => ({ quickAddIsOpen: ref(false) }))
  vi.doMock('../../src/commands', () => ({ dispatch: (id: string) => dispatchMock(id) }))
  const rows = ref<Chord[]>(chords)
  vi.doMock('../../src/config/shortcutsState', () => ({ shortcutRows: rows }))

  const state = await import('../../src/glossaryConfirmStripState')
  const glossary = await import('../../src/config/glossary')
  const marksState = await import('../../src/panels/glossaryMarksState')
  const Strip = (await import('../../src/GlossaryConfirmStrip.vue')).default
  return { state, glossary, marksState, Strip, caretSegmentId, segments, dispatchMock, rows }
}

type Ctx = Awaited<ReturnType<typeof fresh>>

function wire(handlers: Partial<Record<string, () => Promise<unknown>>>) {
  mockInvoke.mockImplementation((cmd: string) => {
    const handler = handlers[cmd]
    if (handler !== undefined) return handler()
    return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
  })
}

async function flush() {
  for (let i = 0; i < 6; i += 1) await Promise.resolve()
}

async function confirmLeft(ctx: Ctx, marks = [LEFT, RIGHT], pendingConfirmedAt: string | null = MARK) {
  wire({
    glossary_confirm_pending_translation: () => Promise.resolve(pendingConfirmedAt),
    glossary_marks_for_chapter: () => Promise.resolve([{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT]),
    glossary_count_entries: () => Promise.resolve(0),
  })
  ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, marks)
  ctx.state.confirmStripTranslationInput.value = 'Tieu Viem'
  const ok = await ctx.state.confirmGlossaryConfirmStrip(1, SEGMENTS, 'zh')
  await flush()
  return ok
}

function undoCalls() {
  return mockInvoke.mock.calls.filter(([cmd]) => cmd === 'glossary_unconfirm_translation')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('the notice after a confirm', () => {
  it('a confirm that returned a mark offers the undo with term and translation', async () => {
    const ctx = await fresh()
    expect(await confirmLeft(ctx)).toBe(true)
    expect(ctx.state.confirmStripUndo.value).toEqual({ sourceTerm: '萧炎', translation: 'Tieu Viem' })
  })

  it('a confirm whose caret went null in flight (the disabled fieldset dropped focus) still offers the undo', async () => {
    const ctx = await fresh()
    let release: (v: unknown) => void = () => {}
    wire({
      glossary_confirm_pending_translation: () => new Promise((resolve) => { release = resolve }),
      glossary_marks_for_chapter: () => Promise.resolve([{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT]),
      glossary_count_entries: () => Promise.resolve(0),
    })
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [LEFT, RIGHT])
    ctx.state.confirmStripTranslationInput.value = 'Tieu Viem'
    const pending = ctx.state.confirmGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    ctx.state.syncGlossaryConfirmStripTarget(null, SEGMENTS, true, [LEFT, RIGHT])
    release(MARK)
    await pending
    await flush()
    expect(ctx.state.confirmStripUndo.value).toEqual({ sourceTerm: '萧炎', translation: 'Tieu Viem' })
  })

  it('a confirm whose caret moved to another sentence in flight offers nothing', async () => {
    const ctx = await fresh()
    let release: (v: unknown) => void = () => {}
    wire({
      glossary_confirm_pending_translation: () => new Promise((resolve) => { release = resolve }),
      glossary_marks_for_chapter: () => Promise.resolve([]),
      glossary_count_entries: () => Promise.resolve(0),
    })
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [LEFT, RIGHT])
    ctx.state.confirmStripTranslationInput.value = 'Tieu Viem'
    const pending = ctx.state.confirmGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    ctx.state.syncGlossaryConfirmStripTarget(2, SEGMENTS, true, [])
    release(MARK)
    await pending
    await flush()
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })

  it('a confirm whose result carries no mark (row already confirmed) offers nothing', async () => {
    const ctx = await fresh()
    expect(await confirmLeft(ctx, [LEFT, RIGHT], null)).toBe(true)
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })

  it('the notice survives the marks reload that follows the confirm', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT])
    expect(ctx.state.confirmStripSourceTerm.value).toBe('薰儿')
    expect(ctx.state.confirmStripUndo.value).not.toBeNull()
  })
})

describe('events that kill the button (AD-54 item 6)', () => {
  it('moving to another segment', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.syncGlossaryConfirmStripTarget(2, SEGMENTS, true, [])
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })

  it('not a caret that goes null (focus left the grid after the marks reload)', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.syncGlossaryConfirmStripTarget(null, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT])
    expect(ctx.state.confirmStripUndo.value).not.toBeNull()
  })

  it('the strip moving to a different pending row that is not the confirmed one', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT])
    expect(ctx.state.confirmStripUndo.value).not.toBeNull()
    const other = markWire({ start: 5, end: 7, id: 9, sourceTerm: '登场' })
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [other])
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })

  it('deferring the next row', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT])
    ctx.state.deferGlossaryConfirmStrip()
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })

  it('any other Glossary write', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    wire({ glossary_reject_candidate: () => Promise.resolve(null) })
    await ctx.glossary.rejectGlossaryCandidate(3)
    expect(ctx.state.confirmStripUndo.value).toBeNull()
    expect(await ctx.state.undoGlossaryConfirmStrip(1, SEGMENTS, 'zh')).toBe(false)
    expect(undoCalls()).toHaveLength(0)
  })

  it('a new confirm starting', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }, RIGHT])
    wire({ glossary_confirm_pending_translation: () => new Promise(() => {}) })
    ctx.state.confirmStripTranslationInput.value = 'Huan Nhi'
    void ctx.state.confirmGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })

  it('reset (new chapter or work)', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    ctx.state.resetGlossaryConfirmStrip()
    expect(ctx.state.confirmStripUndo.value).toBeNull()
  })
})

describe('the grid track under the sentence', () => {
  it('keeps a track for the notice after the last pending term is confirmed, and drops it with the notice', async () => {
    const ctx = await fresh()
    const eligibility = await import('../../src/inlineStripEligibility')
    ctx.caretSegmentId.value = 1
    await confirmLeft(ctx, [LEFT])
    expect(ctx.state.confirmStripIsOpen.value).toBe(true)

    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }])
    expect(ctx.state.confirmStripIsOpen.value).toBe(false)
    expect(ctx.state.confirmStripUndo.value).not.toBeNull()
    expect(eligibility.inlineStripAnchorSegmentId.value).toBe(1)

    ctx.state.resetGlossaryConfirmStrip()
    expect(eligibility.inlineStripAnchorSegmentId.value).toBeNull()
  })

  it('keeps the track on the confirmed sentence while the caret is null', async () => {
    const ctx = await fresh()
    const eligibility = await import('../../src/inlineStripEligibility')
    ctx.caretSegmentId.value = 1
    await confirmLeft(ctx, [LEFT])
    ctx.caretSegmentId.value = null
    ctx.state.syncGlossaryConfirmStripTarget(null, SEGMENTS, true, [{ ...LEFT, is_confirmed: true, translation: 'Tieu Viem' }])
    expect(ctx.state.confirmStripUndo.value).not.toBeNull()
    expect(eligibility.inlineStripAnchorSegmentId.value).toBe(1)
  })
})

describe('the Glossary write counter', () => {
  const writers: Array<[string, (g: Ctx['glossary']) => Promise<unknown>]> = [
    ['add', (g) => g.addGlossaryTerm('global', 'a', null, '', 'other')],
    ['update', (g) => g.updateGlossaryTerm('global', 1, null, '', 'other')],
    ['confirm', (g) => g.confirmPendingGlossaryTranslation('global', 1, 'x')],
    ['approve', (g) => g.approveGlossaryCandidate(1, null, 'other')],
    ['reject', (g) => g.rejectGlossaryCandidate(1)],
    ['delete', (g) => g.deleteGlossaryTerm('global', 1)],
    ['confirm import', (g) => g.confirmGlossaryImport({})],
    ['promote', (g) => g.promoteGlossaryTermToGlobal(1)],
  ]

  it.each(writers)('%s bumps it', async (_name, call) => {
    const ctx = await fresh()
    wire({})
    const before = ctx.glossary.glossaryWriteCount.value
    await call(ctx.glossary)
    expect(ctx.glossary.glossaryWriteCount.value).toBeGreaterThan(before)
  })
})

describe('running the undo', () => {
  it('sends the written text and mark, then the term goes to "Để sau" and marks reload', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    wire({
      glossary_unconfirm_translation: () => Promise.resolve(null),
      glossary_marks_for_chapter: () => Promise.resolve([LEFT, RIGHT]),
      glossary_count_entries: () => Promise.resolve(0),
    })
    expect(await ctx.state.undoGlossaryConfirmStrip(1, SEGMENTS, 'zh')).toBe(true)
    await flush()

    expect(undoCalls()).toEqual([
      ['glossary_unconfirm_translation', { tier: 'global', id: 7, expectedTranslation: 'Tieu Viem', expectedPendingConfirmedAt: MARK }],
    ])
    expect(mockInvoke.mock.calls.some(([cmd]) => cmd === 'glossary_marks_for_chapter')).toBe(true)
    expect(ctx.state.confirmStripUndo.value).toBeNull()

    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [LEFT])
    expect(ctx.state.confirmStripIsOpen.value).toBe(false)
  })

  it('keeps the notice, shows "Đang hoàn tác…" and disables the button while the unconfirm is pending', async () => {
    const ctx = await fresh()
    ctx.segments.value = SEGMENTS
    ctx.caretSegmentId.value = 1
    wire({
      glossary_marks_for_chapter: () => Promise.resolve([LEFT, RIGHT]),
      glossary_confirm_pending_translation: () => Promise.resolve(MARK),
      glossary_count_entries: () => Promise.resolve(0),
    })
    await ctx.marksState.ensureGlossaryMarksLoaded(1, SEGMENTS, 'zh')
    const wrapper = mount(ctx.Strip)
    await wrapper.vm.$nextTick()
    ctx.state.confirmStripTranslationInput.value = 'Tieu Viem'
    await ctx.state.confirmGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    await flush()

    let release: (v: unknown) => void = () => {}
    wire({
      glossary_unconfirm_translation: () => new Promise((resolve) => { release = resolve }),
      glossary_marks_for_chapter: () => Promise.resolve([LEFT, RIGHT]),
      glossary_count_entries: () => Promise.resolve(0),
    })
    const undoing = ctx.state.undoGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    await wrapper.vm.$nextTick()

    expect(ctx.state.confirmStripUndo.value).not.toBeNull()
    const notice = wrapper.find('[data-glossary-confirm-done]')
    expect(notice.exists()).toBe(true)
    expect(notice.text()).toContain('Đang hoàn tác')
    expect(notice.find('button').attributes('disabled')).toBeDefined()

    release(null)
    await undoing
    await flush()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-glossary-confirm-done]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('a second press does nothing', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    wire({
      glossary_unconfirm_translation: () => Promise.resolve(null),
      glossary_marks_for_chapter: () => Promise.resolve([LEFT, RIGHT]),
      glossary_count_entries: () => Promise.resolve(0),
    })
    await ctx.state.undoGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    expect(await ctx.state.undoGlossaryConfirmStrip(1, SEGMENTS, 'zh')).toBe(false)
    expect(undoCalls()).toHaveLength(1)
  })

  it('a refusal shows the typed error, drops the button and does not defer the term', async () => {
    const ctx = await fresh()
    await confirmLeft(ctx)
    const err = { code: 'glossary.translation_changed', message_key: 'err.glossary.translation_changed', params: {}, retryable: false }
    wire({ glossary_unconfirm_translation: () => Promise.reject(err) })

    expect(await ctx.state.undoGlossaryConfirmStrip(1, SEGMENTS, 'zh')).toBe(false)
    expect(ctx.state.confirmStripUndoError.value).toEqual(err)
    expect(ctx.state.confirmStripUndo.value).toBeNull()

    ctx.state.syncGlossaryConfirmStripTarget(2, SEGMENTS, true, [])
    expect(ctx.state.confirmStripUndoError.value).toBeNull()
    ctx.state.syncGlossaryConfirmStripTarget(1, SEGMENTS, true, [LEFT])
    expect(ctx.state.confirmStripSourceTerm.value).toBe('萧炎')
  })
})

describe('the notice in the strip component', () => {
  async function mounted(chords: Chord[]) {
    const ctx = await fresh(chords)
    ctx.segments.value = SEGMENTS
    ctx.caretSegmentId.value = 1
    wire({
      glossary_marks_for_chapter: () => Promise.resolve([LEFT, RIGHT]),
      glossary_confirm_pending_translation: () => Promise.resolve(MARK),
      glossary_count_entries: () => Promise.resolve(0),
    })
    await ctx.marksState.ensureGlossaryMarksLoaded(1, SEGMENTS, 'zh')
    const wrapper = mount(ctx.Strip)
    await wrapper.vm.$nextTick()
    ctx.state.confirmStripTranslationInput.value = 'Tieu Viem'
    await ctx.state.confirmGlossaryConfirmStrip(1, SEGMENTS, 'zh')
    await flush()
    await wrapper.vm.$nextTick()
    return { ctx, wrapper }
  }

  it('prints the chord the user bound and dispatches the command on click', async () => {
    const { ctx, wrapper } = await mounted([{ id: 'glossary.confirm.undo', display: ['⌥U'] }])
    const notice = wrapper.find('[data-glossary-confirm-done]')
    expect(notice.exists()).toBe(true)
    expect(notice.text()).toContain('萧炎')
    expect(notice.text()).toContain('Tieu Viem')
    expect(notice.find('kbd').text()).toBe('⌥U')
    expect(notice.text()).not.toContain('⌘Z')

    await notice.find('button').trigger('click')
    expect(ctx.dispatchMock).toHaveBeenCalledWith('glossary.confirm.undo')
    wrapper.unmount()
  })

  it('prints no chord while the command is unbound', async () => {
    const { wrapper } = await mounted([{ id: 'glossary.confirm.undo', display: [] }])
    const notice = wrapper.find('[data-glossary-confirm-done]')
    expect(notice.exists()).toBe(true)
    expect(notice.find('kbd').exists()).toBe(false)
    wrapper.unmount()
  })

  it('follows a rebinding made while the notice is showing', async () => {
    const { ctx, wrapper } = await mounted([])
    expect(wrapper.find('[data-glossary-confirm-done] kbd').exists()).toBe(false)
    ctx.rows.value = [{ id: 'glossary.confirm.undo', display: ['⌃⇧U'] }]
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-glossary-confirm-done] kbd').text()).toBe('⌃⇧U')
    wrapper.unmount()
  })

  it('the notice disappears when another Glossary write happens', async () => {
    const { ctx, wrapper } = await mounted([])
    expect(wrapper.find('[data-glossary-confirm-done]').exists()).toBe(true)
    wire({ glossary_reject_candidate: () => Promise.resolve(null) })
    await ctx.glossary.rejectGlossaryCandidate(3)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-glossary-confirm-done]').exists()).toBe(false)
    wrapper.unmount()
  })
})
