/**
 * Proofreader strip content: title with progress, key hint built from the real chords,
 * Esc hides for the caret sentence, scope buttons, the "Chỗ đã bỏ qua" link, and the scan
 * record carrying the per-entry block counts.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import { flushPromises } from './support/flushMicrotasks'

const runMock = vi.fn()
const recordMock = vi.fn()
const dispatchMock = vi.fn()
const chordRows = ref<{ id: string; display: string[] }[]>([])

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: () => Promise.resolve(),
  recordProofreadScan: (...args: unknown[]) => recordMock(...args),
  getProofreadDecisionCounts: () => Promise.resolve({ counts: { accepted: 0, not_error: 0, skipped: 0, by_kind: [] }, error: null }),
}))
vi.mock('../../src/config/shortcutsState', () => ({ shortcutRows: chordRows }))
vi.mock('../../src/openWorkIdentity', () => ({ openWorkId: ref('work-1') }))
vi.mock('../../src/panels/editorPanelState', async () => {
  const { ref: vueRef } = await import('vue')
  return {
    editorCaretSegmentId: vueRef<number | null>(null),
    editorLastCaretSegmentId: vueRef<number | null>(null),
    editorSegments: vueRef<{ id: number; target_text: string; is_omitted: boolean }[]>([]),
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
    moveEditorCaretTo: () => true,
    replaceEditorSegment: () => {},
  }
})
vi.mock('../../src/panels/segmentSelectionState', async () => {
  const { ref: vueRef } = await import('vue')
  return { segmentSelectionIds: vueRef<number[]>([]) }
})
vi.mock('../../src/commands', () => ({ dispatch: (...args: unknown[]) => dispatchMock(...args) }))

const editor = await import('../../src/panels/editorPanelState')
const state = await import('../../src/proofreadState')
const { eligibleInlineStrips } = await import('../../src/inlineStripEligibility')
const ProofreaderStrip = (await import('../../src/ProofreaderStrip.vue')).default
const caretRef = editor.editorCaretSegmentId as unknown as { value: number | null }
const segmentsRef = editor.editorSegments as unknown as { value: unknown[] }

const FINDING = { kind: 'spelling', start: 0, end: 2, explanation: 'e', suggestion: 's' }
const result = (id: number, findings = [FINDING]) => ({
  segment_id: id,
  scanned_text: 'ab cd',
  findings,
  unlocated: 0,
  ambiguous: 0,
})

function scan(outcome: unknown = { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }): void {
  runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
    onEvent({ kind: 'planned', segments: 1, batches: 1 })
    onEvent({ kind: 'batch', usage: null, segments: [result(1, [FINDING, { ...FINDING, start: 3, end: 5 }])] })
    return Promise.resolve({ value: outcome, error: null })
  })
}

beforeEach(() => {
  segmentsRef.value = [
    { id: 1, target_text: 'ab cd', is_omitted: false },
    { id: 2, target_text: 'ef', is_omitted: false },
  ]
  caretRef.value = 1
  chordRows.value = [
    { id: 'ai.proofread.next', display: ['F8'] },
    { id: 'ai.proofread.prev', display: ['⇧F8'] },
  ]
  recordMock.mockResolvedValue({ ok: true, error: null })
})

afterEach(() => {
  state.resetProofread()
  runMock.mockReset()
  recordMock.mockReset()
  dispatchMock.mockReset()
})

async function mountWithScan() {
  scan()
  const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
  await state.runProofread([1])
  await flushPromises()
  return wrapper
}

describe('title and hint', () => {
  it('reads "Proofreader — phát hiện i trên n" and hints the real chords', async () => {
    const wrapper = await mountWithScan()
    expect(wrapper.get('.pfs-title').text()).toBe('Proofreader — phát hiện 1 trên 2')
    expect(wrapper.get('.pfs-hint').text()).toBe('F8 kế tiếp · ⇧F8 trước · Esc ẩn')
    wrapper.unmount()
  })

  it('follows a rebound chord instead of a hard-coded F8', async () => {
    chordRows.value = [
      { id: 'ai.proofread.next', display: ['F9'] },
      { id: 'ai.proofread.prev', display: ['⇧F9'] },
    ]
    const wrapper = await mountWithScan()
    expect(wrapper.get('.pfs-hint').text()).toBe('F9 kế tiếp · ⇧F9 trước · Esc ẩn')
    expect(wrapper.get('.pfs-hint').text()).not.toContain('F8')
    wrapper.unmount()
  })

  it('drops the hint for an unbound command', async () => {
    chordRows.value = [{ id: 'ai.proofread.next', display: [] }]
    const wrapper = await mountWithScan()
    expect(wrapper.get('.pfs-hint').text()).toBe('Esc ẩn')
    wrapper.unmount()
  })

  it('keeps the Prev and Next buttons', async () => {
    const wrapper = await mountWithScan()
    const labels = wrapper.findAll('.pfs-actions .pfs-btn').map((b) => b.text())
    expect(labels).toEqual(expect.arrayContaining(['Nhận đề xuất', 'Bỏ qua', 'Không phải lỗi', 'Phát hiện trước', 'Phát hiện kế']))
    wrapper.unmount()
  })
})

describe('Esc hides the strip for the caret sentence', () => {
  it('Esc dispatches ai.proofread.hide', async () => {
    const wrapper = await mountWithScan()
    await wrapper.get('.proofreader-strip').trigger('keydown', { key: 'Escape' })
    expect(dispatchMock).toHaveBeenCalledWith('ai.proofread.hide')
    wrapper.unmount()
  })

  it('hides for sentence 1, stays hidden there, and shows again after visiting sentence 2', async () => {
    const wrapper = await mountWithScan()
    expect(eligibleInlineStrips.value).toContain('proofreader')

    state.hideProofreadStrip()
    expect(eligibleInlineStrips.value).not.toContain('proofreader')
    expect(eligibleInlineStrips.value).not.toContain('proofreader_summary')

    caretRef.value = 2
    state.syncProofreadStripHidden(2)
    caretRef.value = 1
    state.syncProofreadStripHidden(1)
    expect(eligibleInlineStrips.value).toContain('proofreader')
    wrapper.unmount()
  })

  it('does not hide the scan-level strip while a scan runs', async () => {
    runMock.mockImplementation(() => new Promise(() => {}))
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    void state.runProofread([1])
    await flushPromises()
    state.hideProofreadStrip()
    expect(eligibleInlineStrips.value).toContain('proofreader')
    wrapper.unmount()
  })

  it('an Esc pressed during the scan does not hide the findings once it finishes', async () => {
    let finish: (value: unknown) => void = () => {}
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 1, batches: 1 })
      onEvent({ kind: 'batch', usage: null, segments: [result(1)] })
      return new Promise((resolve) => {
        finish = resolve
      })
    })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    const running = state.runProofread([1])
    await flushPromises()
    state.hideProofreadStrip()
    finish({ value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
    await running
    await flushPromises()
    expect(eligibleInlineStrips.value).toContain('proofreader')
    wrapper.unmount()
  })

  it('hiding from inside the strip puts focus back on the caret sentence translation cell', async () => {
    const cell = document.createElement('div')
    cell.dataset.col = 'tgt'
    cell.dataset.segmentId = '1'
    cell.tabIndex = -1
    document.body.appendChild(cell)
    const wrapper = await mountWithScan()
    ;(wrapper.get('.pfs-actions .pfs-btn').element as HTMLElement).focus()

    state.hideProofreadStrip()

    expect(document.activeElement).toBe(cell)
    wrapper.unmount()
    cell.remove()
  })
})

describe('scope buttons and the ignored-spots link', () => {
  it('each scope button dispatches its own scan command with the sentence count', async () => {
    const wrapper = await mountWithScan()
    const cells = new Map(wrapper.findAll('[data-proofread-strip-scope]').map((b) => [b.attributes('data-proofread-strip-scope'), b]))
    expect(cells.get('segment')?.text()).toContain('1 câu')
    expect(cells.get('chapter')?.text()).toContain('2 câu')
    expect(cells.get('selection')?.attributes('disabled')).toBeDefined()

    await cells.get('segment')?.trigger('click')
    await cells.get('chapter')?.trigger('click')
    expect(dispatchMock.mock.calls.map((c) => c[0])).toEqual(['ai.proofread.run', 'ai.proofread.run_chapter'])
    wrapper.unmount()
  })

  it('the link opens the memory list and takes focus on mouse down', async () => {
    const wrapper = await mountWithScan()
    const link = wrapper.get('[data-proofread-strip-memory]')
    await link.trigger('mousedown')
    expect(document.activeElement).toBe(link.element)
    await link.trigger('click')
    expect(dispatchMock).toHaveBeenCalledWith('ai.proofread.memory.open')
    wrapper.unmount()
  })
})

describe('scan record', () => {
  it('passes the per-entry block counts of a finished scan through', async () => {
    scan({ state: 'done', not_scanned: 0, filtered: 2, blocked: [{ ignore_id: 4, count: 2 }] })
    await state.runProofread([1], 'segment')
    await flushPromises()
    expect(recordMock).toHaveBeenCalledWith('work-1', 'segment', 2, 2, [{ ignore_id: 4, count: 2 }])
  })
})
