/**
 * Fuzzy TM strip (FR59): scan trigger, stale-response drop, Esc per segment, accept through
 * the editor, slot priority, keys, and the threshold form.
 *
 * Only the IPC boundary (`config/segment.ts`) is faked; state, commands, the real editor
 * module and the real `TmFuzzyStrip.vue` run.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises as flushMicrotasks } from './support/flushMicrotasks'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { CommandDeps } from '../../src/commands'
import type { PromoteAiTranslationOutcome, TmFuzzyMatch, TmFuzzyMatches } from '../../src/config/segment'

/** Microtasks plus the scan debounce, so a caret move has settled and its scan has answered. */
async function flushPromises(): Promise<void> {
  await vi.advanceTimersByTimeAsync(200)
  await flushMicrotasks()
}

const tmFuzzyMatchesMock = vi.fn()
const acceptTmFuzzyMock = vi.fn()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    tmFuzzyMatches: (...args: unknown[]) => tmFuzzyMatchesMock(...args),
    acceptTmFuzzy: (...args: unknown[]) => acceptTmFuzzyMock(...args),
    saveSegmentTargets: recordSave,
    readOpenChapterSegments: readFixture,
  }
})

function matchOf(patch: Partial<TmFuzzyMatch> = {}): TmFuzzyMatch {
  return {
    tier: 'work',
    unit_id: 1,
    percent: 82,
    source_text: '他推開門。',
    target_text: 'Hắn đẩy cửa ra.',
    diff: [
      { kind: 'equal', text: '他' },
      { kind: 'delete', text: '推開' },
      { kind: 'insert', text: '關上' },
      { kind: 'equal', text: '門。' },
    ],
    side: 'mine',
    ...patch,
  }
}

function scanOf(segmentId: number, matches: TmFuzzyMatch[]): { outcome: TmFuzzyMatches; error: null } {
  return { outcome: { segment_id: segmentId, matches }, error: null }
}

function acceptedOf(patch: Partial<PromoteAiTranslationOutcome> = {}) {
  return {
    outcome: {
      segment_id: 12,
      target_text: 'Hắn đẩy cửa ra.',
      translation_origin: 'other',
      status: 'draft' as const,
      needs_confirmation: false,
      unsigned_draft: null,
      ...patch,
    },
    error: null,
  }
}

async function fresh() {
  vi.resetModules()
  tmFuzzyMatchesMock.mockReset()
  acceptTmFuzzyMock.mockReset()

  const commands = await import('../../src/commands')
  const editor = await import('../../src/panels/editorPanelState')
  const strip = await import('../../src/tmFuzzyStripState')
  const quickAdd = await import('../../src/glossaryQuickAddState')
  const confirmStrip = await import('../../src/glossaryConfirmStripState')
  const TmFuzzyStrip = (await import('../../src/TmFuzzyStrip.vue')).default
  const { topmostStrip } = await import('../../src/panels/inlineStripPriority')
  const { eligibleInlineStrips } = await import('../../src/inlineStripEligibility')

  const { tmFuzzyCommandDeps } = await import('../../src/tmFuzzyCommandDeps')
  commands.installCommands({ isMac: true, setMode: () => {}, ...tmFuzzyCommandDeps() } as CommandDeps)

  return { commands, editor, strip, quickAdd, confirmStrip, TmFuzzyStrip, topmostStrip, eligibleInlineStrips }
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] })
  document.body.innerHTML = ''
  resetRecorder()
})

afterEach(() => {
  vi.useRealTimers()
  vi.restoreAllMocks()
})

async function mountOn(segmentId: number, matches: TmFuzzyMatch[]) {
  const ctx = await fresh()
  tmFuzzyMatchesMock.mockResolvedValue(scanOf(segmentId, matches))
  await ctx.editor.ensureSegmentsLoaded()
  const wrapper = mount(ctx.TmFuzzyStrip, { attachTo: document.body })
  ctx.editor.setEditorCaret(segmentId)
  await flushPromises()
  return { ...ctx, wrapper }
}

describe('scan trigger and rendering', () => {
  it('a hit renders percentage, diff, side and tier', async () => {
    const { wrapper } = await mountOn(12, [matchOf()])
    expect(tmFuzzyMatchesMock).toHaveBeenCalledWith(12)
    const row = wrapper.get('.tmf-row')
    expect(row.get('.tmf-pct').text()).toBe('82%')
    expect(row.get('.tmf-del').text()).toBe('推開')
    expect(row.get('.tmf-ins').text()).toBe('關上')
    expect(row.get('.tmf-source').text()).toBe('他推開關上門。')
    expect(row.get('.tmf-target').text()).toBe('Hắn đẩy cửa ra.')
    expect(row.get('.tmf-side').text()).toBe('Của tôi')
    expect(row.get('.tmf-tier').text()).toBe('Tác phẩm này')
    wrapper.unmount()
  })

  it('no matches ⇒ no strip', async () => {
    const { wrapper } = await mountOn(12, [])
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)
    wrapper.unmount()
  })

  it('a scan error is shown, not swallowed into an empty strip', async () => {
    const ctx = await fresh()
    tmFuzzyMatchesMock.mockResolvedValue({
      outcome: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await ctx.editor.ensureSegmentsLoaded()
    const wrapper = mount(ctx.TmFuzzyStrip, { attachTo: document.body })
    ctx.editor.setEditorCaret(12)
    await flushPromises()
    expect(wrapper.find('.tmf-error').exists()).toBe(true)
    wrapper.unmount()
  })

  it('a response for a segment the caret has left is dropped', async () => {
    const ctx = await fresh()
    let releaseFirst: (v: unknown) => void = () => {}
    tmFuzzyMatchesMock.mockImplementationOnce(
      () => new Promise((resolve) => { releaseFirst = resolve }),
    )
    tmFuzzyMatchesMock.mockResolvedValueOnce(scanOf(13, []))
    await ctx.editor.ensureSegmentsLoaded()
    const wrapper = mount(ctx.TmFuzzyStrip, { attachTo: document.body })
    ctx.editor.setEditorCaret(12)
    await flushPromises()
    ctx.editor.setEditorCaret(13)
    await flushPromises()
    releaseFirst(scanOf(12, [matchOf()]))
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('Esc in the strip', () => {
  it('hides it for that segment, returns focus, and a later visit to the same segment stays hidden', async () => {
    const { wrapper, editor, strip } = await mountOn(12, [matchOf()])
    const cell = document.createElement('textarea')
    document.body.appendChild(cell)
    cell.focus()
    expect(strip.focusTmFuzzyStrip(true)).toBe(true)
    await flushPromises()
    expect(document.activeElement).toBe(wrapper.get('.tm-fuzzy-strip').element)

    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)
    expect(document.activeElement).toBe(cell)

    tmFuzzyMatchesMock.mockClear()
    tmFuzzyMatchesMock.mockResolvedValue(scanOf(13, [matchOf({ unit_id: 9 })]))
    editor.setEditorCaret(13)
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)

    editor.setEditorCaret(12)
    await flushPromises()
    expect(tmFuzzyMatchesMock).toHaveBeenCalledTimes(1)
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('reset wiring', () => {
  it('resetting the editor (Work or Chapter change) forgets the per-segment Esc', async () => {
    const { wrapper, editor, strip } = await mountOn(12, [matchOf()])
    strip.hideTmFuzzyStrip()
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)

    editor.resetEditorPanel()
    await editor.ensureSegmentsLoaded()
    editor.setEditorCaret(12)
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('slot priority', () => {
  it('a Glossary strip eligible at the same time wins and the TM strip waits', async () => {
    const { wrapper, quickAdd, topmostStrip, eligibleInlineStrips } = await mountOn(12, [matchOf()])
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)

    quickAdd.openGlossaryQuickAdd('他')
    await flushPromises()
    expect(eligibleInlineStrips.value).toEqual(['glossary_quick_add', 'tm_fuzzy'])
    expect(topmostStrip(eligibleInlineStrips.value)).toBe('glossary_quick_add')
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)

    quickAdd.closeGlossaryQuickAdd()
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)
    wrapper.unmount()
  })

  it('the chord does nothing while another strip owns the slot', async () => {
    const { wrapper, quickAdd, commands } = await mountOn(12, [matchOf()])
    quickAdd.openGlossaryQuickAdd('他')
    await flushPromises()
    const errors = vi.spyOn(console, 'error').mockImplementation(() => {})
    commands.dispatch('tm.fuzzy.focus')
    expect(errors).toHaveBeenCalled()
    wrapper.unmount()
  })
})

describe('keys inside the strip', () => {
  async function withThree() {
    const matches = [matchOf({ unit_id: 1 }), matchOf({ unit_id: 2, tier: 'global', side: 'others' }), matchOf({ unit_id: 3 })]
    const ctx = await mountOn(12, matches)
    acceptTmFuzzyMock.mockResolvedValue(acceptedOf())
    return ctx
  }

  it('arrows move the aim, clamped at both ends', async () => {
    const { wrapper } = await withThree()
    const root = wrapper.get('.tm-fuzzy-strip')
    await root.trigger('keydown', { key: 'ArrowDown' })
    await root.trigger('keydown', { key: 'ArrowDown' })
    await root.trigger('keydown', { key: 'ArrowDown' })
    expect(wrapper.findAll('.tmf-row')[2]?.attributes('aria-current')).toBe('true')
    await root.trigger('keydown', { key: 'ArrowUp' })
    expect(wrapper.findAll('.tmf-row')[1]?.attributes('aria-current')).toBe('true')
    wrapper.unmount()
  })

  it('Enter accepts the aimed row, identified by tier and unit id', async () => {
    const { wrapper } = await withThree()
    const root = wrapper.get('.tm-fuzzy-strip')
    await root.trigger('keydown', { key: 'ArrowDown' })
    await root.trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(acceptTmFuzzyMock).toHaveBeenCalledWith(12, 'global', 2, false)
    wrapper.unmount()
  })

  it('digit 3 accepts the third row', async () => {
    const { wrapper } = await withThree()
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: '3' })
    await flushPromises()
    expect(acceptTmFuzzyMock).toHaveBeenCalledWith(12, 'work', 3, false)
    wrapper.unmount()
  })

  it('a modified key is not taken by the strip', async () => {
    const { wrapper } = await withThree()
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Enter', metaKey: true })
    await flushPromises()
    expect(acceptTmFuzzyMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})

describe('accept', () => {
  it('writes the pair as an unconfirmed draft with origin other and closes the strip', async () => {
    const { wrapper, editor } = await mountOn(13, [matchOf({ unit_id: 5 })])
    acceptTmFuzzyMock.mockResolvedValue(acceptedOf({ segment_id: 13 }))
    await wrapper.get('.tmf-row .tmf-btn').trigger('click')
    await flushPromises()
    expect(acceptTmFuzzyMock).toHaveBeenCalledWith(13, 'work', 5, false)
    const seg = editor.editorSegments.value.find((s) => s.id === 13)
    expect(seg?.target_text).toBe('Hắn đẩy cửa ra.')
    expect(seg?.status).toBe('draft')
    expect(seg?.translation_origin).toBe('other')
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)
    wrapper.unmount()
  })

  it('over a draft: asks first, writes nothing, then force overwrites', async () => {
    const { wrapper, editor } = await mountOn(12, [matchOf({ unit_id: 5 })])
    acceptTmFuzzyMock.mockResolvedValueOnce(
      acceptedOf({ needs_confirmation: true, unsigned_draft: 'Bản nháp đang có', target_text: 'Bản nháp đang có' }),
    )
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(wrapper.get('.tmf-draft').text()).toBe('Bản nháp đang có')
    expect(editor.editorSegments.value.find((s) => s.id === 12)?.target_text).toBe('Gió thổi tới từ cuối hành lang.')

    acceptTmFuzzyMock.mockResolvedValueOnce(acceptedOf())
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(acceptTmFuzzyMock).toHaveBeenNthCalledWith(2, 12, 'work', 5, true)
    expect(editor.editorSegments.value.find((s) => s.id === 12)?.target_text).toBe('Hắn đẩy cửa ra.')
    wrapper.unmount()
  })

  it('Esc on the overwrite question keeps the draft and the strip', async () => {
    const { wrapper, editor } = await mountOn(12, [matchOf({ unit_id: 5 })])
    acceptTmFuzzyMock.mockResolvedValueOnce(acceptedOf({ needs_confirmation: true, unsigned_draft: 'x' }))
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(wrapper.find('.tmf-overwrite').exists()).toBe(false)
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)
    expect(acceptTmFuzzyMock).toHaveBeenCalledTimes(1)
    expect(editor.editorSegments.value.find((s) => s.id === 12)?.target_text).toBe('Gió thổi tới từ cuối hành lang.')
    wrapper.unmount()
  })

  it('a pair that is gone shows its own message and writes nothing', async () => {
    const { wrapper, editor } = await mountOn(13, [matchOf({ unit_id: 5 })])
    acceptTmFuzzyMock.mockResolvedValue({
      outcome: null,
      error: { code: 'tm.pair_not_found', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(wrapper.get('.tmf-error').text()).toContain('không còn')
    expect(editor.editorSegments.value.find((s) => s.id === 13)?.target_text).toBe('')
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('threshold form', () => {
  it('accepts 50..99 only', async () => {
    const { parsedTmFuzzyThreshold } = await import('../../src/tmSettingsState')
    expect(parsedTmFuzzyThreshold('65')).toBe(65)
    expect(parsedTmFuzzyThreshold('50')).toBe(50)
    expect(parsedTmFuzzyThreshold('99')).toBe(99)
    expect(parsedTmFuzzyThreshold('49')).toBeNull()
    expect(parsedTmFuzzyThreshold('100')).toBeNull()
    expect(parsedTmFuzzyThreshold('6.5')).toBeNull()
    expect(parsedTmFuzzyThreshold('')).toBeNull()
  })

  it('an out-of-range value writes nothing; a valid one goes through put_config', async () => {
    vi.resetModules()
    const putConfig = vi.fn().mockResolvedValue(null)
    vi.doMock('../../src/config/bootstrap', async (importOriginal) => ({
      ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
      putConfig,
    }))
    const state = await import('../../src/tmSettingsState')
    state.loadTmSettingsForm()
    expect(state.tmSettingsThresholdInput.value).toBe('65')
    state.tmSettingsThresholdInput.value = '120'
    await state.saveTmSettings()
    expect(putConfig).not.toHaveBeenCalled()
    state.tmSettingsThresholdInput.value = '80'
    await state.saveTmSettings()
    expect(putConfig).toHaveBeenCalledWith('app_config', 'tm_fuzzy_threshold', '80')
    expect(state.tmSettingsSaved.value).toBe(true)
    vi.doUnmock('../../src/config/bootstrap')
  })
})

describe('review fixes', () => {
  it('a caret that keeps moving scans only the segment it settles on', async () => {
    const ctx = await fresh()
    tmFuzzyMatchesMock.mockResolvedValue(scanOf(13, []))
    await ctx.editor.ensureSegmentsLoaded()
    const wrapper = mount(ctx.TmFuzzyStrip, { attachTo: document.body })
    ctx.editor.setEditorCaret(11)
    await vi.advanceTimersByTimeAsync(50)
    ctx.editor.setEditorCaret(12)
    await vi.advanceTimersByTimeAsync(50)
    ctx.editor.setEditorCaret(13)
    await flushPromises()
    expect(tmFuzzyMatchesMock).toHaveBeenCalledTimes(1)
    expect(tmFuzzyMatchesMock).toHaveBeenCalledWith(13)
    wrapper.unmount()
  })

  it('saving a new threshold rescans the caret segment', async () => {
    const ctx = await fresh()
    vi.doMock('../../src/config/bootstrap', async (importOriginal) => ({
      ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
      putConfig: vi.fn().mockResolvedValue(null),
    }))
    tmFuzzyMatchesMock.mockResolvedValue(scanOf(12, []))
    await ctx.editor.ensureSegmentsLoaded()
    const wrapper = mount(ctx.TmFuzzyStrip, { attachTo: document.body })
    ctx.editor.setEditorCaret(12)
    await flushPromises()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)

    tmFuzzyMatchesMock.mockResolvedValue(scanOf(12, [matchOf()]))
    const settings = await import('../../src/tmSettingsState')
    settings.tmSettingsThresholdInput.value = '55'
    await settings.saveTmSettings()
    await flushPromises()
    expect(tmFuzzyMatchesMock).toHaveBeenCalledTimes(2)
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)
    vi.doUnmock('../../src/config/bootstrap')
    wrapper.unmount()
  })

  it('mousedown on a row button aims that row before the click accepts', async () => {
    const { wrapper } = await mountOn(12, [matchOf({ unit_id: 1 }), matchOf({ unit_id: 2 })])
    acceptTmFuzzyMock.mockResolvedValue(acceptedOf())
    const second = wrapper.findAll('.tmf-row .tmf-btn')[1]
    await second.trigger('mousedown')
    await second.trigger('click')
    await flushPromises()
    expect(acceptTmFuzzyMock).toHaveBeenCalledWith(12, 'work', 2, false)
    wrapper.unmount()
  })

  it('a digit beyond the shown rows does nothing', async () => {
    const { wrapper } = await mountOn(12, [matchOf()])
    await wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: '2' })
    await flushPromises()
    expect(acceptTmFuzzyMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('an accept that resolves after the caret moved leaves the new segment strip alone', async () => {
    const ctx = await mountOn(12, [matchOf({ unit_id: 5 })])
    let release: (v: unknown) => void = () => {}
    acceptTmFuzzyMock.mockImplementationOnce(() => new Promise((resolve) => { release = resolve }))
    await ctx.wrapper.get('.tm-fuzzy-strip').trigger('keydown', { key: 'Enter' })
    await flushPromises()
    tmFuzzyMatchesMock.mockResolvedValue(scanOf(13, [matchOf({ unit_id: 9 })]))
    ctx.editor.setEditorCaret(13)
    await flushPromises()
    release(acceptedOf())
    await flushPromises()
    expect(ctx.wrapper.find('.tm-fuzzy-strip').exists()).toBe(true)
    expect(ctx.strip.tmFuzzySegmentId.value).toBe(13)
    ctx.wrapper.unmount()
  })

  it('a second Enter while the first accept is in flight sends no second write', async () => {
    const { wrapper } = await mountOn(12, [matchOf({ unit_id: 5 })])
    let release: (v: unknown) => void = () => {}
    acceptTmFuzzyMock.mockImplementationOnce(() => new Promise((resolve) => { release = resolve }))
    const root = wrapper.get('.tm-fuzzy-strip')
    await root.trigger('keydown', { key: 'Enter' })
    await flushPromises()
    await root.trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(acceptTmFuzzyMock).toHaveBeenCalledTimes(1)
    release(acceptedOf())
    await flushPromises()
    wrapper.unmount()
  })

  it('errors do not follow the caret to another segment', async () => {
    const ctx = await fresh()
    tmFuzzyMatchesMock.mockResolvedValueOnce({
      outcome: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await ctx.editor.ensureSegmentsLoaded()
    const wrapper = mount(ctx.TmFuzzyStrip, { attachTo: document.body })
    ctx.editor.setEditorCaret(12)
    await flushPromises()
    expect(wrapper.find('.tmf-error').exists()).toBe(true)

    tmFuzzyMatchesMock.mockResolvedValue(scanOf(13, []))
    ctx.strip.hideTmFuzzyStrip()
    ctx.editor.setEditorCaret(12)
    await flushMicrotasks()
    expect(wrapper.find('.tm-fuzzy-strip').exists()).toBe(false)
    ctx.editor.setEditorCaret(13)
    await flushMicrotasks()
    expect(wrapper.find('.tmf-error').exists()).toBe(false)
    expect(ctx.strip.tmFuzzyScanError.value).toBeNull()
    wrapper.unmount()
  })

  it('moving the caret after a chord entry drops the saved focus, so Esc elsewhere does not jump back', async () => {
    const { wrapper, editor, strip } = await mountOn(12, [matchOf()])
    const oldCell = document.createElement('textarea')
    const newCell = document.createElement('textarea')
    document.body.append(oldCell, newCell)
    oldCell.focus()
    strip.focusTmFuzzyStrip(true)
    await flushPromises()

    tmFuzzyMatchesMock.mockResolvedValue(scanOf(13, [matchOf({ unit_id: 9 })]))
    editor.setEditorCaret(13)
    await flushPromises()
    newCell.focus()
    strip.hideTmFuzzyStrip()
    expect(document.activeElement).toBe(newCell)
    wrapper.unmount()
  })
})
