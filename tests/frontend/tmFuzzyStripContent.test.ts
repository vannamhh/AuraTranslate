/**
 * Fuzzy TM strip content: title with count, key hint, "khớp", meta with date, and the
 * in-place "Xem cả câu" expansion.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises as flushMicrotasks } from './support/flushMicrotasks'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { CommandDeps } from '../../src/commands'
import type { TmExactTarget, TmFuzzyMatch, TmFuzzyMatches } from '../../src/config/segment'

/** Microtasks plus the scan debounce, so a caret move has settled and its scan has answered. */
async function flushPromises(): Promise<void> {
  await vi.advanceTimersByTimeAsync(200)
  await flushMicrotasks()
}

const tmFuzzyMatchesMock = vi.fn()
const acceptTmFuzzyMock = vi.fn()
const acceptTmExactMock = vi.fn()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    tmFuzzyMatches: (...args: unknown[]) => tmFuzzyMatchesMock(...args),
    acceptTmFuzzy: (...args: unknown[]) => acceptTmFuzzyMock(...args),
    acceptTmExact: (...args: unknown[]) => acceptTmExactMock(...args),
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
    created_at: '2026-07-14T12:00:00.000Z',
    ...patch,
  }
}

function scanOf(
  segmentId: number,
  matches: TmFuzzyMatch[],
  exact: TmExactTarget[] = [],
): { outcome: TmFuzzyMatches; error: null } {
  return { outcome: { segment_id: segmentId, matches, exact, hidden_count: 0 }, error: null }
}

async function fresh() {
  vi.resetModules()
  tmFuzzyMatchesMock.mockReset()
  acceptTmFuzzyMock.mockReset()
  acceptTmExactMock.mockReset()

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


describe('strip content', () => {
  it('titles the strip with the match count and the 1 2 3 hint', async () => {
    const { wrapper } = await mountOn(12, [matchOf(), matchOf({ unit_id: 2, percent: 71 })])
    expect(wrapper.get('.tmf-title').text()).toBe('Translation Memory — 2 câu gần giống')
    expect(wrapper.get('.tmf-hint').text()).toBe('1 2 3 nhận · Esc ẩn dải')
    expect(wrapper.get('.tmf-pct').text()).toBe('82% khớp')
    wrapper.unmount()
  })

  it('shows side, tier and the dd-MM date of the pair', async () => {
    const { wrapper } = await mountOn(12, [matchOf()])
    expect(wrapper.get('.tmf-side').text()).toBe('Của tôi')
    expect(wrapper.get('.tmf-tier').text()).toBe('Tác phẩm này')
    expect(wrapper.get('.tmf-date-meta').text()).toBe('14-07')
    wrapper.unmount()
  })

  it('omits the date when created_at is not a date', async () => {
    const { wrapper } = await mountOn(12, [matchOf({ created_at: 'not a date' })])
    expect(wrapper.find('.tmf-date-meta').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('Xem cả câu', () => {
  it('expands and folds the aimed row in place through its own command', async () => {
    const { wrapper } = await mountOn(12, [matchOf(), matchOf({ unit_id: 2, percent: 71 })])
    const rows = wrapper.findAll('.tmf-row')
    const toggle = rows[1].findAll('.tmf-btn')[1]
    expect(toggle.text()).toBe('Xem cả câu')
    expect(toggle.attributes('aria-expanded')).toBe('false')

    await toggle.trigger('mousedown')
    await toggle.trigger('click')
    expect(rows[1].classes()).toContain('tmf-row-expanded')
    expect(rows[0].classes()).not.toContain('tmf-row-expanded')
    expect(rows[1].findAll('.tmf-btn')[1].text()).toBe('Thu gọn')

    await rows[1].findAll('.tmf-btn')[1].trigger('click')
    expect(rows[1].classes()).not.toContain('tmf-row-expanded')
    wrapper.unmount()
  })

  it('forgets the expansion when the strip is hidden and shown again', async () => {
    const { wrapper, strip } = await mountOn(12, [matchOf()])
    await wrapper.get('.tmf-row').findAll('.tmf-btn')[1].trigger('click')
    expect(wrapper.get('.tmf-row').classes()).toContain('tmf-row-expanded')
    strip.hideTmFuzzyStrip()
    expect(strip.tmFuzzyExpandedRows.value).toEqual([])
    wrapper.unmount()
  })
})
