/**
 * Alignment overlay: state around the Rust-held machine, and the local keyboard. `config/alignment.ts`
 * is the IPC boundary and is mocked.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterAlignment } from '../../src/config/alignment'
import type { CommandDeps } from '../../src/commands'

const openMock = vi.fn()
const joinMock = vi.fn()
const skipMock = vi.fn()
const unjoinMock = vi.fn()

vi.mock('../../src/config/alignment', () => ({
  alignmentOpen: (...args: unknown[]) => openMock(...args),
  alignmentJoin: (...args: unknown[]) => joinMock(...args),
  alignmentSkip: (...args: unknown[]) => skipMock(...args),
  alignmentUnjoin: (...args: unknown[]) => unjoinMock(...args),
}))

function alignment(over: Partial<ChapterAlignment> = {}): ChapterAlignment {
  return {
    chapter_id: 7,
    file_name: 'review.docx',
    file_kind: 'docx',
    rows: [
      { id: 100, kind: 'text', source_text: 'nguồn một', target_text: 'sửa một' },
      { id: 101, kind: 'text', source_text: null, target_text: 'hàng lạ' },
      { id: 102, kind: 'text', source_text: 'nguồn ba', target_text: 'sửa ba' },
    ],
    segments: [
      { id: 10, ord: 1, role: null, source_text: 'nguồn một', target_text: 'dịch một' },
      { id: 11, ord: 2, role: null, source_text: 'nguồn hai', target_text: 'dịch hai' },
      { id: 12, ord: 3, role: null, source_text: 'nguồn ba', target_text: 'dịch ba' },
    ],
    groups: [
      { id: 1, decided_by: 'machine', row_ids: [100], segment_ids: [10] },
      { id: 2, decided_by: 'machine', row_ids: [102], segment_ids: [12] },
    ],
    unmatched_row_ids: [101],
    unmatched_segment_ids: [11],
    is_resolved: false,
    ...over,
  }
}

function ok(a: ChapterAlignment) {
  return { alignment: a, error: null }
}

function ipcError(code: string) {
  return { code, message_key: 'err.export.alignment_stale', params: {}, retryable: false }
}

async function freshState() {
  vi.resetModules()
  for (const m of [openMock, joinMock, skipMock, unjoinMock]) m.mockReset()
  return import('../../src/alignmentState')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('alignment state', () => {
  it('lists unprocessed segments, then unprocessed rows, then groups', async () => {
    const state = await freshState()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    expect(openMock).toHaveBeenCalledWith(7)
    expect(state.alignmentStatus.value).toBe('loaded')
    expect(state.alignmentEntries.value.map((e) => `${e.kind}:${e.id}`)).toEqual([
      'segment:11',
      'row:101',
      'group:1',
      'group:2',
    ])
  })

  it('no open Chapter is its own status and never calls Rust', async () => {
    const state = await freshState()
    await state.openAlignmentOverlay(null)
    expect(state.alignmentStatus.value).toBe('no_chapter')
    expect(openMock).not.toHaveBeenCalled()
  })

  it('a stale or not-imported copy is an error, not an empty list', async () => {
    const state = await freshState()
    openMock.mockResolvedValue({ alignment: null, error: ipcError('export.alignment_stale') })
    await state.openAlignmentOverlay(7)
    expect(state.alignmentStatus.value).toBe('error')
    expect(state.alignmentData.value).toBeNull()
    expect(state.alignmentLoadError.value?.code).toBe('export.alignment_stale')
  })

  it('a missing IPC bridge is reported as ipc_unavailable', async () => {
    const state = await freshState()
    openMock.mockResolvedValue({ alignment: null, error: null })
    await state.openAlignmentOverlay(7)
    expect(state.alignmentStatus.value).toBe('ipc_unavailable')
  })

  it('the cursor stops at both ends', async () => {
    const state = await freshState()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    state.prevAlignmentEntry()
    expect(state.alignmentCursor.value).toBe(0)
    for (let i = 0; i < 10; i += 1) state.nextAlignmentEntry()
    expect(state.alignmentCursor.value).toBe(3)
  })

  it('join sends the marked ids, adopts the returned state and clears the marks', async () => {
    const state = await freshState()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    state.toggleAlignmentMark()
    state.nextAlignmentEntry()
    state.toggleAlignmentMark()
    expect([...state.alignmentMarkedSegments.value]).toEqual([11])
    expect([...state.alignmentMarkedRows.value]).toEqual([101])

    joinMock.mockResolvedValue(
      ok(
        alignment({
          groups: [...alignment().groups, { id: 3, decided_by: 'user', row_ids: [101], segment_ids: [11] }],
          unmatched_row_ids: [],
          unmatched_segment_ids: [],
          is_resolved: true,
        }),
      ),
    )
    await state.joinAlignmentMarks()
    expect(joinMock).toHaveBeenCalledWith(7, [11], [101])
    expect(state.alignmentData.value?.is_resolved).toBe(true)
    expect([...state.alignmentMarkedSegments.value]).toEqual([])
    expect(state.alignmentEntries.value.map((e) => e.kind)).toEqual(['group', 'group', 'group'])
  })

  it('a rejected join keeps the list and the marks and shows the typed error', async () => {
    const state = await freshState()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    state.toggleAlignmentMark()
    joinMock.mockResolvedValue({ alignment: null, error: ipcError('export.alignment_invalid_selection') })
    await state.joinAlignmentMarks()
    expect(state.alignmentActionError.value?.code).toBe('export.alignment_invalid_selection')
    expect([...state.alignmentMarkedSegments.value]).toEqual([11])
    expect(state.alignmentEntries.value).toHaveLength(4)
  })

  it('skip sends exactly one of segment or row', async () => {
    const state = await freshState()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    skipMock.mockResolvedValue(ok(alignment()))
    await state.skipAlignmentEntry()
    expect(skipMock).toHaveBeenLastCalledWith(7, 11, null)
    state.nextAlignmentEntry()
    await state.skipAlignmentEntry()
    expect(skipMock).toHaveBeenLastCalledWith(7, null, 101)
  })

  it('skip on a group and unjoin on an item are no-ops without a round trip', async () => {
    const state = await freshState()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    await state.unjoinAlignmentEntry()
    state.nextAlignmentEntry()
    state.nextAlignmentEntry()
    await state.skipAlignmentEntry()
    expect(skipMock).not.toHaveBeenCalled()
    unjoinMock.mockResolvedValue(ok(alignment()))
    await state.unjoinAlignmentEntry()
    expect(unjoinMock).toHaveBeenCalledWith(7, 1)
  })

  it('a reply for a closed overlay is dropped', async () => {
    const state = await freshState()
    let release: (value: unknown) => void = () => undefined
    openMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    const opening = state.openAlignmentOverlay(7)
    state.closeAlignmentOverlay()
    release(ok(alignment()))
    await opening
    expect(state.alignmentData.value).toBeNull()
  })
})

async function freshOverlay(deps: Partial<CommandDeps> = {}) {
  const state = await freshState()
  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const AlignmentOverlay = (await import('../../src/AlignmentOverlay.vue')).default
  return { state, AlignmentOverlay }
}

describe('AlignmentOverlay.vue keyboard', () => {
  it('each key is exactly one dispatch of its own command', async () => {
    const spies = {
      nextAlignmentEntry: vi.fn(),
      prevAlignmentEntry: vi.fn(),
      toggleAlignmentMark: vi.fn(),
      joinAlignmentMarks: vi.fn(),
      skipAlignmentEntry: vi.fn(),
      unjoinAlignmentEntry: vi.fn(),
      closeAlignment: vi.fn(),
    }
    const { state, AlignmentOverlay } = await freshOverlay(spies)
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    const wrapper = mount(AlignmentOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.al-scrim')

    const cases: Array<[string, keyof typeof spies]> = [
      ['ArrowDown', 'nextAlignmentEntry'],
      ['ArrowUp', 'prevAlignmentEntry'],
      [' ', 'toggleAlignmentMark'],
      ['Enter', 'joinAlignmentMarks'],
      ['s', 'skipAlignmentEntry'],
      ['u', 'unjoinAlignmentEntry'],
      ['Escape', 'closeAlignment'],
    ]
    for (const [key, name] of cases) {
      await scrim.trigger('keydown', { key })
      for (const [other, spy] of Object.entries(spies)) {
        expect(spy, `${key} -> ${other}`).toHaveBeenCalledTimes(other === name ? 1 : 0)
      }
      for (const spy of Object.values(spies)) spy.mockClear()
    }
    wrapper.unmount()
  })

  it('a chord with a modifier dispatches nothing', async () => {
    const skipSpy = vi.fn()
    const { state, AlignmentOverlay } = await freshOverlay({ skipAlignmentEntry: skipSpy })
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    const wrapper = mount(AlignmentOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.get('.al-scrim').trigger('keydown', { key: 's', metaKey: true })
    expect(skipSpy).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('Enter on a focused button is left to the button, not a second dispatch', async () => {
    const joinSpy = vi.fn()
    const { state, AlignmentOverlay } = await freshOverlay({ joinAlignmentMarks: joinSpy })
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    const wrapper = mount(AlignmentOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.get('.al-act-primary').trigger('keydown', { key: 'Enter' })
    expect(joinSpy).not.toHaveBeenCalled()
    await wrapper.get('.al-act-primary').trigger('click')
    expect(joinSpy).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('renders the unprocessed items before the groups and says when nothing is resolved', async () => {
    const { state, AlignmentOverlay } = await freshOverlay()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openAlignmentOverlay(7)
    const wrapper = mount(AlignmentOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.al-row')).toHaveLength(4)
    expect(wrapper.text()).toContain('dịch hai')
    expect(wrapper.text()).toContain('hàng lạ')
    expect(wrapper.text()).toContain('Còn 2 mục chưa xử lý.')
    wrapper.unmount()
  })
})
