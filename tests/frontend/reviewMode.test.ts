/**
 * Review Mode: a read-only two-panel layout state inside Workspace. `config/alignment.ts` is the IPC
 * boundary and is mocked; `WorkspaceMode`, `ReviewDock` and `WorkspaceDock` mount for real.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { defineComponent, h, KeepAlive, nextTick } from 'vue'
import { readFileSync } from 'node:fs'
import type { ChapterAlignment, ReviewDiff, ReviewDiffSpan } from '../../src/config/alignment'
import GridPanel from '../../src/panels/GridPanel.vue'
import WorkspaceMode from '../../src/modes/WorkspaceMode.vue'
import { applyPreset, isDockSuspended, panelRing, togglePanel } from '../../src/layout/dockController'
import { declareFocus, dispatch, enterFocus, installCommands, MODE_IDS, releaseFocus } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import { currentMode, setMode } from '../../src/modes/modeState'
import * as state from '../../src/reviewModeState'
import { t } from '../../src/i18n'

const openMock = vi.fn()
const diffMock = vi.fn()
const putMock = vi.fn(() => Promise.resolve(null))
const acceptMock = vi.fn()
const skipMock = vi.fn()
const flushMock = vi.fn()
const replaceMock = vi.fn()

vi.mock('../../src/config/alignment', () => ({
  alignmentOpen: (...args: unknown[]) => openMock(...args),
  reviewDiff: (...args: unknown[]) => diffMock(...args),
  alignmentJoin: vi.fn(),
  alignmentSkip: vi.fn(),
  alignmentUnjoin: vi.fn(),
  reviewAcceptChange: (...args: unknown[]) => acceptMock(...args),
  reviewSkipChange: (...args: unknown[]) => skipMock(...args),
}))

vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  flushEditorBeforeDiscreteWrite: (...args: unknown[]) => flushMock(...args),
  replaceEditorSegment: (...args: unknown[]) => replaceMock(...args),
}))

vi.mock('../../src/config/bootstrap', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/bootstrap')>()),
  putConfig: (...args: unknown[]) => (putMock as (...a: unknown[]) => Promise<null>)(...args),
}))

function alignment(over: Partial<ChapterAlignment> = {}): ChapterAlignment {
  return {
    chapter_id: 7,
    file_name: 'review.docx',
    file_kind: 'docx',
    rows: [
      { id: 100, kind: 'text', source_text: 'nguồn một', target_text: 'sửa một' },
      { id: 101, kind: 'caption', source_text: null, target_text: 'chú thích' },
      { id: 102, kind: 'text', source_text: 'nguồn ba', target_text: '' },
    ],
    segments: [
      { id: 12, ord: 3, role: null, source_text: 'nguồn ba', target_text: '' },
      { id: 10, ord: 1, role: null, source_text: 'nguồn một', target_text: 'dịch một' },
      { id: 11, ord: 2, role: null, source_text: 'nguồn hai', target_text: 'dịch hai' },
    ],
    groups: [],
    unmatched_row_ids: [],
    unmatched_segment_ids: [],
    is_resolved: true,
    ...over,
  }
}

function pair(
  groupId: number,
  segmentIds: number[],
  rowIds: number[],
  spans: ReviewDiffSpan[],
  decision: 'accepted' | 'skipped' | null = null,
) {
  return { group_id: groupId, decided_by: 'machine' as const, segment_ids: segmentIds, row_ids: rowIds, spans, decision }
}

function diffOf(...pairs: ReturnType<typeof pair>[]): { diff: ReviewDiff; error: null } {
  return { diff: { chapter_id: 7, pairs }, error: null }
}

const eq = (text: string): ReviewDiffSpan => ({ kind: 'equal', text })
const del = (text: string): ReviewDiffSpan => ({ kind: 'delete', text })
const ins = (text: string): ReviewDiffSpan => ({ kind: 'insert', text })

function defaultDiff() {
  return diffOf(
    pair(1, [10], [100], [eq('dịch '), del('một'), ins('sửa một')]),
    pair(2, [11], [101], [eq('dịch hai')]),
    pair(3, [12], [102], []),
  )
}

const ok = (a: ChapterAlignment) => ({ alignment: a, error: null })
const failure = (code: string) => ({
  alignment: null,
  error: { code, message_key: 'err.unknown', params: {}, retryable: false },
})

async function settle(): Promise<void> {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await new Promise((resolve) => requestAnimationFrame(() => resolve(undefined)))
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

beforeEach(() => {
  openMock.mockReset()
  diffMock.mockReset()
  diffMock.mockResolvedValue(defaultDiff())
  acceptMock.mockReset()
  skipMock.mockReset()
  flushMock.mockReset()
  flushMock.mockResolvedValue('clean')
  replaceMock.mockReset()
  putMock.mockClear()
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: 1200 })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: 1000 })
  document.documentElement.style.setProperty('--space-titlebar-height', '40px')
  document.documentElement.style.setProperty('--space-status-height', '34px')
})

afterEach(() => {
  state.resetReviewMode()
})

describe('reviewModeState', () => {
  it('open: segments by ord, rows as returned, status open', async () => {
    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    expect(state.reviewModeStatus.value).toBe('open')
    expect(state.reviewModeSegments.value.map((s) => s.id)).toEqual([10, 11, 12])
    expect(state.reviewModeRows.value.map((r) => r.id)).toEqual([100, 101, 102])
    expect(openMock).toHaveBeenCalledWith(7)
  })

  it('NotImported, Stale and no Chapter are three separate notices and never an open layout', async () => {
    openMock.mockResolvedValue(failure('export.alignment_not_imported'))
    await state.openReviewMode(7)
    expect(state.reviewModeStatus.value).toBe('not_imported')
    expect(state.reviewModeIsOpen.value).toBe(false)
    expect(state.reviewModeSegments.value).toEqual([])

    state.resetReviewMode()
    openMock.mockResolvedValue(failure('export.alignment_stale'))
    await state.openReviewMode(7)
    expect(state.reviewModeStatus.value).toBe('stale')
    expect(state.reviewModeIsOpen.value).toBe(false)

    state.resetReviewMode()
    openMock.mockClear()
    await state.openReviewMode(null)
    expect(state.reviewModeStatus.value).toBe('no_chapter')
    expect(openMock).not.toHaveBeenCalled()
  })

  it('an unrelated failure is an error notice, not an empty panel', async () => {
    openMock.mockResolvedValue(failure('store.read_failed'))
    await state.openReviewMode(7)
    expect(state.reviewModeStatus.value).toBe('error')
    expect(state.reviewModeLoadError.value?.code).toBe('store.read_failed')
  })

  it('close during a pending open leaves nothing open', async () => {
    let release: (value: unknown) => void = () => undefined
    openMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    const opening = state.openReviewMode(7)
    await state.closeReviewMode()
    release(ok(alignment()))
    await opening
    expect(state.reviewModeStatus.value).toBe('idle')
    expect(state.reviewModeSegments.value).toEqual([])
  })

  it('reset clears the state and lets layout commands through again', async () => {
    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    expect(isDockSuspended()).toBe(true)
    state.resetReviewMode()
    expect(isDockSuspended()).toBe(false)
    expect(state.reviewModeStatus.value).toBe('idle')
    expect(state.reviewModeRows.value).toEqual([])
    const error = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    expect(applyPreset('layout.preset_grid')).toBe(false)
    expect(error.mock.calls.some((c) => String(c[0]).includes('Review Mode'))).toBe(false)
    error.mockRestore()
  })

  it('both resets that run on a Work change call resetReviewMode', () => {
    for (const file of ['src/modes/libraryChapters.ts', 'src/modes/libraryImport.ts']) {
      const code = readFileSync(file, 'utf8')
        .split('\n')
        .filter((line) => !line.trim().startsWith('//'))
        .join('\n')
      expect(code, file).toMatch(/\bresetReviewMode\(\)/)
    }
  })
})

describe('commands', () => {
  it('review.open defaults to Mod+Alt+3, review.close is bindable, MODE_IDS stays three', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const openReviewMode = vi.fn()
    const closeReviewMode = vi.fn()
    commands.installCommands({ openReviewMode, closeReviewMode } as never)
    const specs = commands.commandRegistry.list()
    const open = specs.find((s) => s.id === 'review.open')
    const close = specs.find((s) => s.id === 'review.close')
    expect(open?.keys).toEqual(['Mod+Alt+3'])
    expect(close).toBeDefined()
    expect(commands.commandRegistry.unbound().some((s) => s.id === 'review.close')).toBe(true)
    commands.dispatch('review.open')
    commands.dispatch('review.close')
    expect(openReviewMode).toHaveBeenCalledTimes(1)
    expect(closeReviewMode).toHaveBeenCalledTimes(1)
    expect(commands.MODE_IDS).toHaveLength(3)
    expect(commands.FOCUS_OWNERS).toContain('panel.review_mine')
    expect(commands.FOCUS_OWNERS).toContain('panel.review_copy')
  })
})

describe('diff commands', () => {
  it('review.diff_next and review.diff_prev are registered, rebindable and reach their handlers', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const reviewDiffNext = vi.fn()
    const reviewDiffPrev = vi.fn()
    commands.installCommands({ reviewDiffNext, reviewDiffPrev } as never)
    const specs = commands.commandRegistry.list()
    expect(specs.find((s) => s.id === 'review.diff_next')?.keys).toEqual(['Alt+ArrowDown'])
    expect(specs.find((s) => s.id === 'review.diff_prev')?.keys).toEqual(['Alt+ArrowUp'])
    const conflicts = specs.filter((s) => s.keys?.some((k) => k === 'Alt+ArrowDown' || k === 'Alt+ArrowUp'))
    expect(conflicts.map((s) => s.id).sort()).toEqual(['review.diff_next', 'review.diff_prev'])
    commands.dispatch('review.diff_next')
    commands.dispatch('review.diff_prev')
    expect(reviewDiffNext).toHaveBeenCalledTimes(1)
    expect(reviewDiffPrev).toHaveBeenCalledTimes(1)
  })

  it('main wiring spreads both handlers', async () => {
    const { reviewModeCommandDeps } = await import('../../src/reviewModeCommandDeps')
    const deps = reviewModeCommandDeps()
    expect(typeof deps.reviewDiffNext).toBe('function')
    expect(typeof deps.reviewDiffPrev).toBe('function')
  })
})

describe('mounted Review Mode', () => {
  let wrapper: VueWrapper | null = null

  async function mountWorkspace(): Promise<VueWrapper> {
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('workspace')
    wrapper = mount(WorkspaceMode, { attachTo: host })
    await settle()
    return wrapper
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  it('open: two read-only panels, left is mine, right is the reviewer rows, focus in the left panel', async () => {
    const w = await mountWorkspace()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    await settle()

    const mine = w.find('[data-review-mine]')
    const copy = w.find('[data-review-copy]')
    expect(mine.exists()).toBe(true)
    expect(copy.exists()).toBe(true)
    expect(mine.findAll('[data-review-mine-item]').map((e) => e.text())).toEqual([
      'dịch một',
      'dịch hai',
      t('review.untranslated'),
    ])
    expect(copy.findAll('[data-review-copy-item]').map((e) => e.text())).toEqual([
      'dịch sửa một',
      `${t('review.row_caption')}dịch hai`,
      t('review.row_empty'),
    ])
    expect(w.find('[data-review-dock] input, [data-review-dock] textarea, [data-review-dock] [contenteditable]').exists()).toBe(false)

    const active = document.activeElement
    expect(active).not.toBe(document.body)
    expect(mine.element.closest('section')?.contains(active)).toBe(true)
    expect(MODE_IDS).toHaveLength(3)
    expect(currentMode.value).toBe('workspace')
  })

  it('notices are separate and keep the layout; import is offered for NotImported and Stale', async () => {
    const w = await mountWorkspace()
    openMock.mockResolvedValue(failure('export.alignment_not_imported'))
    await state.openReviewMode(7)
    await settle()
    expect(w.find('[data-review-dock]').exists()).toBe(false)
    expect(w.find('[data-review-notice]').text()).toContain(t('review.notice.not_imported'))
    expect(w.find('[data-review-notice-import]').exists()).toBe(true)

    state.resetReviewMode()
    openMock.mockResolvedValue(failure('export.alignment_stale'))
    await state.openReviewMode(7)
    await settle()
    expect(w.find('[data-review-notice]').text()).toContain(t('review.notice.stale'))
    expect(w.find('[data-review-notice-import]').exists()).toBe(true)

    state.resetReviewMode()
    await state.openReviewMode(null)
    await settle()
    expect(w.find('[data-review-notice]').text()).toContain(t('review.notice.no_chapter'))
    expect(w.find('[data-review-notice-import]').exists()).toBe(false)
    expect(w.find('[data-review-dock]').exists()).toBe(false)
  })

  it('layout.* commands are refused while Review shows and leave the Workspace layout untouched', async () => {
    await mountWorkspace()
    expect(applyPreset('layout.preset_columns')).toBe(true)
    window.dispatchEvent(new Event('beforeunload'))
    const before = putMock.mock.calls.map((c) => String((c as unknown[])[2]))
    expect(before.length).toBeGreaterThan(0)
    const saved = before[before.length - 1]

    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    await settle()

    const error = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    expect(applyPreset('layout.preset_grid')).toBe(false)
    expect(togglePanel('panel.lookup')).toBe(false)
    expect(panelRing()).toEqual(['panel.review_mine', 'panel.review_copy'])
    error.mockRestore()

    window.dispatchEvent(new Event('beforeunload'))
    const after = putMock.mock.calls.map((c) => String((c as unknown[])[2]))
    for (const json of after) expect(json).not.toContain('review')
    expect(after[after.length - 1]).toBe(saved)

    await state.closeReviewMode()
    await settle()
    expect(applyPreset('layout.preset_grid')).toBe(true)
  })

  it('close: same GridPanel instance and scrollTop, focus back in the panel that was active, never body', async () => {
    const w = await mountWorkspace()
    const gridBefore = w.findComponent(GridPanel).vm.$.uid
    const slotBefore = w.find('[data-workspace-slot]').element
    expect(enterFocus('panel.grid')).toBe(true)
    await settle()
    const gridRoot = document.activeElement
    expect(gridRoot).not.toBe(document.body)
    const scroller = w.find('.panel-body').element as HTMLElement
    scroller.scrollTop = 42

    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    await settle()
    expect(w.find('[data-workspace-slot]').classes()).toContain('hidden')

    await state.closeReviewMode()
    await settle()

    expect(w.find('[data-review-dock]').exists()).toBe(false)
    expect(w.find('[data-workspace-slot]').element).toBe(slotBefore)
    expect(w.findComponent(GridPanel).vm.$.uid).toBe(gridBefore)
    expect(w.find('.panel-body').element).toBe(scroller)
    expect(scroller.scrollTop).toBe(42)
    expect(document.activeElement).not.toBe(document.body)
    expect(document.activeElement).toBe(gridRoot)
  })

  it('switching to Library and back keeps Review Mode open, as App.vue keeps modes alive', async () => {
    const Library = defineComponent({ render: () => h('section', { 'data-fake-library': '' }) })
    const Shell = defineComponent({
      render: () => h(KeepAlive, null, [currentMode.value === 'workspace' ? h(WorkspaceMode) : h(Library)]),
    })
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('workspace')
    wrapper = mount(Shell, { attachTo: host })
    await settle()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    await settle()
    const dockBefore = wrapper.find('[data-review-dock]').element

    setMode('library')
    await settle()
    expect(wrapper.find('[data-fake-library]').exists()).toBe(true)
    setMode('workspace')
    await settle()

    expect(state.reviewModeStatus.value).toBe('open')
    expect(wrapper.find('[data-review-dock]').element).toBe(dockBefore)
    expect(wrapper.find('[data-workspace-slot]').classes()).toContain('hidden')
    expect(document.activeElement).not.toBe(document.body)
  })

  it('focus.next_panel moves focus from the left Review panel to the right one', async () => {
    const w = await mountWorkspace()
    installCommands({ panelRing, closeReviewMode: () => void state.closeReviewMode() } as unknown as CommandDeps)
    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    await settle()
    dispatch('focus.next_panel')
    await settle()
    const copyFrame = w.find('[data-review-copy]').element.closest('section')
    expect(copyFrame?.contains(document.activeElement)).toBe(true)
  })

  it('dismissing a notice keeps focus off body', async () => {
    const w = await mountWorkspace()
    openMock.mockResolvedValue(failure('export.alignment_not_imported'))
    await state.openReviewMode(7)
    await settle()
    const dismiss = w.find('[data-review-notice-dismiss]').element as HTMLElement
    dismiss.focus()
    expect(document.activeElement).toBe(dismiss)
    await w.find('[data-review-notice-dismiss]').trigger('click')
    await settle()
    expect(w.find('[data-review-notice]').exists()).toBe(false)
    expect(document.activeElement).not.toBe(document.body)
    expect(document.activeElement).not.toBeNull()
  })

  it('opening from Library: close lands focus in Workspace, not body and not Library', async () => {
    const LibraryStub = defineComponent({
      setup() {
        declareFocus('mode.library', () => document.querySelector<HTMLElement>('[data-library-stub]'))
        return () => h('section', { tabindex: -1, 'data-library-stub': '' })
      },
      unmounted() {
        releaseFocus('mode.library')
      },
    })
    const Shell = defineComponent({
      setup: () => () => h(KeepAlive, null, [currentMode.value === 'library' ? h(LibraryStub) : h(WorkspaceMode)]),
    })
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('library')
    wrapper = mount(Shell, { attachTo: host })
    await settle()
    openMock.mockResolvedValue(ok(alignment()))
    setMode('workspace')
    await state.openReviewMode(7)
    await settle()
    expect(wrapper.find('[data-review-dock]').exists()).toBe(true)

    await state.closeReviewMode()
    await settle()
    const active = document.activeElement
    expect(active).not.toBe(document.body)
    expect(wrapper.find('[data-library-stub]').exists() && wrapper.find('[data-library-stub]').element.contains(active)).toBe(false)
    expect(wrapper.find('[data-workspace-slot]').element.closest('section')?.contains(active)).toBe(true)
  })

  it('opening and closing never reaches workspace_layout with a Review panel', async () => {
    await mountWorkspace()
    openMock.mockResolvedValue(ok(alignment()))
    await state.openReviewMode(7)
    await settle()
    window.dispatchEvent(new Event('beforeunload'))
    await state.closeReviewMode()
    await settle()
    window.dispatchEvent(new Event('beforeunload'))
    for (const call of putMock.mock.calls) expect(String((call as unknown[])[2])).not.toContain('review')
  })
})

describe('diff rendering', () => {
  let wrapper: VueWrapper | null = null

  async function openWith(diff: ReturnType<typeof diffOf>, over: Partial<ChapterAlignment> = {}): Promise<VueWrapper> {
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('workspace')
    wrapper = mount(WorkspaceMode, { attachTo: host })
    await settle()
    openMock.mockResolvedValue(ok(alignment(over)))
    diffMock.mockResolvedValue(diff)
    await state.openReviewMode(7)
    await settle()
    return wrapper
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  it('changed word: left marks the old word as del only, right marks the new word as ins only', async () => {
    const w = await openWith(defaultDiff())
    const left = w.find('[data-review-mine]')
    const right = w.find('[data-review-copy]')
    expect(left.findAll('del').map((e) => e.text())).toEqual(['một'])
    expect(left.findAll('ins')).toHaveLength(0)
    expect(right.findAll('ins').map((e) => e.text())).toEqual(['sửa một'])
    expect(right.findAll('del')).toHaveLength(0)
    expect(left.findAll('[data-review-mine-item]')[1]!.findAll('ins, del')).toHaveLength(0)
  })

  it('identical pair carries no ins or del on either side', async () => {
    const w = await openWith(diffOf(pair(2, [11], [101], [eq('giống hệt')])))
    expect(w.find('[data-review-dock]').findAll('ins, del')).toHaveLength(0)
    expect(w.find('[data-review-mine]').text()).toBe('giống hệt')
    expect(w.find('[data-review-copy]').text()).toContain('giống hệt')
  })

  it('ungrouped items are listed plain and labelled, never marked', async () => {
    const w = await openWith(diffOf(pair(1, [10], [100], [eq('a')])), {
      unmatched_segment_ids: [11],
      unmatched_row_ids: [101],
    })
    const left = w.findAll('[data-review-mine] [data-review-unmatched]')
    const right = w.findAll('[data-review-copy] [data-review-unmatched]')
    expect(left.map((e) => e.text())).toEqual([`${t('review.unmatched')}dịch hai`])
    expect(right.map((e) => e.text())).toEqual([`${t('review.unmatched')}${t('review.row_caption')}chú thích`])
    expect(w.find('[data-review-dock]').findAll('ins, del')).toHaveLength(0)
  })

  it('unmatched items sit between the neighbouring pairs, not at the end', async () => {
    const w = await openWith(
      diffOf(pair(1, [10], [100], [eq('a')]), pair(3, [12], [102], [eq('c')])),
      { unmatched_segment_ids: [11], unmatched_row_ids: [101] },
    )
    const order = (sel: string) =>
      w.findAll(sel).map((e) => (e.attributes('data-review-pair') ?? 'u'))
    expect(order('[data-review-mine-item]')).toEqual(['g1', 'u', 'g3'])
    expect(order('[data-review-copy-item]')).toEqual(['g1', 'u', 'g3'])
  })

  it('pairs follow the group ids from the diff, not the array position of segments or rows', async () => {
    const w = await openWith(
      diffOf(pair(9, [12], [102], [eq('chín')]), pair(4, [10], [100], [eq('bốn')])),
    )
    const keys = w.findAll('[data-review-mine-item]').map((e) => e.attributes('data-review-pair'))
    expect(keys).toEqual(['g9', 'g4'])
    const rightKeys = w.findAll('[data-review-copy-item]').map((e) => e.attributes('data-review-pair'))
    expect(rightKeys).toEqual(keys)
  })

  it('no source text reaches the DOM', async () => {
    const w = await openWith(defaultDiff())
    const html = w.find('[data-review-dock]').html()
    for (const seg of alignment().segments) expect(html).not.toContain(seg.source_text)
    for (const row of alignment().rows) if (row.source_text !== null) expect(html).not.toContain(row.source_text)
  })

  it('a failing diff is its own notice and shows neither panel nor colour', async () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('workspace')
    wrapper = mount(WorkspaceMode, { attachTo: host })
    await settle()
    openMock.mockResolvedValue(ok(alignment()))
    diffMock.mockResolvedValue({
      diff: null,
      error: { code: 'store.read_failed', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await state.openReviewMode(7)
    await settle()
    expect(state.reviewModeStatus.value).toBe('diff_failed')
    expect(wrapper.find('[data-review-notice]').text()).toContain(t('review.diff_failed'))
    expect(wrapper.find('[data-review-dock]').exists()).toBe(false)
    expect(isDockSuspended()).toBe(false)
  })

  it('the panels and the shared span view carry no colour literal', () => {
    for (const file of ['ReviewMinePanel', 'ReviewCopyPanel', 'ReviewSpans']) {
      const code = readFileSync(`src/panels/${file}.vue`, 'utf8')
      expect(code, file).not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgba?\(|hsla?\(/)
    }
    const spans = readFileSync('src/panels/ReviewSpans.vue', 'utf8')
    expect(spans).toContain('--color-diff-add-bg')
    expect(spans).toContain('--color-diff-del-bg')
    expect(spans).toContain('text-decoration: underline')
    expect(spans).toContain('text-decoration: line-through')
  })
})

describe('diff jumps and paired scrolling', () => {
  let wrapper: VueWrapper | null = null

  function threeChanges() {
    return diffOf(
      pair(1, [10], [100], [eq('a '), del('x'), ins('y')]),
      pair(2, [11], [101], [eq('same')]),
      pair(3, [12], [102], [del('p'), ins('q')]),
    )
  }

  async function open(diff = threeChanges()): Promise<VueWrapper> {
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('workspace')
    wrapper = mount(WorkspaceMode, { attachTo: host })
    await settle()
    openMock.mockResolvedValue(ok(alignment()))
    diffMock.mockResolvedValue(diff)
    await state.openReviewMode(7)
    await settle()
    return wrapper
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  it('next walks the changed pairs only, stops with a text notice at the last, never wraps', async () => {
    const w = await open()
    state.reviewDiffNext()
    await settle()
    expect(w.find('[data-review-diff-status]').text()).toBe(t('review.diff_position', { index: '1', count: '2' }))
    expect(w.findAll('[data-review-current]').map((e) => e.attributes('data-review-pair'))).toEqual(['g1', 'g1'])
    state.reviewDiffNext()
    await settle()
    expect(w.findAll('[data-review-current]').map((e) => e.attributes('data-review-pair'))).toEqual(['g3', 'g3'])
    state.reviewDiffNext()
    await settle()
    expect(w.find('[data-review-diff-status]').text()).toBe(t('review.diff_last', { count: '2' }))
    expect(w.findAll('[data-review-current]').map((e) => e.attributes('data-review-pair'))).toEqual(['g3', 'g3'])
  })

  it('prev stops at the first with a text notice', async () => {
    const w = await open()
    state.reviewDiffNext()
    state.reviewDiffPrev()
    await settle()
    expect(w.find('[data-review-diff-status]').text()).toBe(t('review.diff_first', { count: '2' }))
    expect(w.findAll('[data-review-current]').map((e) => e.attributes('data-review-pair'))).toEqual(['g1', 'g1'])
  })

  it('with nothing different the jump says so', async () => {
    const w = await open(diffOf(pair(1, [10], [100], [eq('a')])))
    state.reviewDiffNext()
    await settle()
    expect(w.find('[data-review-diff-status]').text()).toBe(t('review.diff_none'))
    expect(w.findAll('[data-review-current]')).toHaveLength(0)
  })

  it('a jump with focus on body lands focus in the left panel; it does nothing while Review is closed', async () => {
    const w = await open()
    ;(document.activeElement as HTMLElement | null)?.blur()
    expect(document.activeElement).toBe(document.body)
    state.reviewDiffNext()
    await settle()
    expect(document.activeElement).not.toBe(document.body)
    expect(w.find('[data-review-mine]').element.closest('section')?.contains(document.activeElement)).toBe(true)

    await state.closeReviewMode()
    await settle()
    state.reviewDiffNext()
    await settle()
    expect(w.find('[data-review-dock]').exists()).toBe(false)
    expect(state.reviewModeDiffMessage.value).toBeNull()
  })

  it('a jump scrolls both panels so the pair sits at the top of each', async () => {
    const w = await open()
    const left = w.find('[data-review-mine]').element as HTMLElement
    const right = w.find('[data-review-copy]').element as HTMLElement
    const stub = (container: HTMLElement, height: number) => {
      let top = 0
      Object.defineProperty(container, 'scrollTop', { configurable: true, get: () => top, set: (v: number) => (top = v) })
      container.getBoundingClientRect = () => ({ top: 0, bottom: 300, left: 0, right: 0, width: 0, height: 300, x: 0, y: 0, toJSON: () => '' })
      container.querySelectorAll<HTMLElement>('[data-review-pair]').forEach((el, i) => {
        el.getBoundingClientRect = () =>
          ({ top: i * height - top, bottom: (i + 1) * height - top, left: 0, right: 0, width: 0, height, x: 0, y: 0, toJSON: () => '' }) as DOMRect
      })
      return () => top
    }
    const leftTop = stub(left, 100)
    const rightTop = stub(right, 40)
    state.reviewDiffNext()
    state.reviewDiffNext()
    await settle()
    expect(leftTop()).toBe(200)
    expect(rightTop()).toBe(80)
  })

  it('scrolling one panel puts the other at the same pair, and the echo does not bounce back', async () => {
    const w = await open()
    const left = w.find('[data-review-mine]').element as HTMLElement
    const right = w.find('[data-review-copy]').element as HTMLElement
    const writes: string[] = []
    const stub = (name: string, container: HTMLElement, height: number) => {
      let top = 0
      Object.defineProperty(container, 'scrollTop', {
        configurable: true,
        get: () => top,
        set: (v: number) => {
          writes.push(`${name}:${v}`)
          top = v
        },
      })
      container.getBoundingClientRect = () => ({ top: 0, bottom: 300, left: 0, right: 0, width: 0, height: 300, x: 0, y: 0, toJSON: () => '' })
      container.querySelectorAll<HTMLElement>('[data-review-pair]').forEach((el, i) => {
        el.getBoundingClientRect = () =>
          ({ top: i * height - top, bottom: (i + 1) * height - top, left: 0, right: 0, width: 0, height, x: 0, y: 0, toJSON: () => '' }) as DOMRect
      })
    }
    stub('left', left, 100)
    stub('right', right, 40)
    left.scrollTop = 150
    left.dispatchEvent(new Event('scroll'))
    expect(right.scrollTop).toBe(90)
    right.dispatchEvent(new Event('scroll'))
    expect(left.scrollTop).toBe(150)
    expect(writes).toEqual(['left:150', 'right:90'])
  })
})

describe('accepting and skipping changes', () => {
  let wrapper: VueWrapper | null = null

  const acceptedOutcome = (over: Record<string, unknown> = {}) => ({
    outcome: {
      segment_id: 10,
      target_text: 'sửa một',
      translation_origin: 'other',
      status: 'draft',
      needs_confirmation: false,
      unsigned_draft: null,
      ...over,
    },
    error: null,
  })

  function threeChanges(decisions: Array<'accepted' | 'skipped' | null> = [null, null, null]) {
    return diffOf(
      pair(1, [10], [100], [eq('a '), del('x'), ins('y')], decisions[0]),
      pair(2, [11], [101], [eq('a '), del('m'), ins('n')], decisions[1]),
      pair(3, [12], [102], [del('p'), ins('q')], decisions[2]),
    )
  }

  async function open(diff = threeChanges()): Promise<VueWrapper> {
    const host = document.createElement('div')
    document.body.appendChild(host)
    setMode('workspace')
    wrapper = mount(WorkspaceMode, { attachTo: host })
    await settle()
    openMock.mockResolvedValue(ok(alignment()))
    diffMock.mockResolvedValue(diff)
    await state.openReviewMode(7)
    await settle()
    return wrapper
  }

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  it('the two commands are registered with Alt+Enter and Alt+Backspace, rebindable, and reach their handlers', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const reviewAcceptChange = vi.fn()
    const reviewSkipChange = vi.fn()
    commands.installCommands({ reviewAcceptChange, reviewSkipChange } as never)
    const specs = commands.commandRegistry.list()
    expect(specs.find((s) => s.id === 'review.accept_change')?.keys).toEqual(['Alt+Enter'])
    expect(specs.find((s) => s.id === 'review.skip_change')?.keys).toEqual(['Alt+Backspace'])
    const taken = new Map<string, string>()
    for (const spec of specs) {
      for (const key of spec.keys ?? []) {
        expect(taken.has(key), `${key} used by ${taken.get(key)} and ${spec.id}`).toBe(false)
        taken.set(key, spec.id)
      }
    }
    commands.dispatch('review.accept_change')
    commands.dispatch('review.skip_change')
    expect(reviewAcceptChange).toHaveBeenCalledTimes(1)
    expect(reviewSkipChange).toHaveBeenCalledTimes(1)
    const { reviewModeCommandDeps } = await import('../../src/reviewModeCommandDeps')
    const deps = reviewModeCommandDeps()
    expect(typeof deps.reviewAcceptChange).toBe('function')
    expect(typeof deps.reviewSkipChange).toBe('function')
    expect(typeof deps.reviewConfirmAccept).toBe('function')
    expect(typeof deps.reviewCancelAccept).toBe('function')
  })

  it('accepting flushes the typing buffer first, writes through Rust with the text shown, mirrors the Editor and jumps to the next pending change', async () => {
    const w = await open()
    state.reviewDiffNext()
    await settle()
    const order: string[] = []
    flushMock.mockImplementation(() => {
      order.push('flush')
      return Promise.resolve('clean')
    })
    acceptMock.mockImplementation(() => {
      order.push('accept')
      return Promise.resolve(acceptedOutcome())
    })
    diffMock.mockResolvedValue(threeChanges(['accepted', null, null]))

    await state.reviewAcceptChange()
    await settle()

    expect(order).toEqual(['flush', 'accept'])
    expect(acceptMock).toHaveBeenCalledWith(7, 1, 'a x', false)
    expect(replaceMock).toHaveBeenCalledWith(10, { target_text: 'sửa một', translation_origin: 'other', status: 'draft' })
    expect(state.reviewModeCurrentPairKey.value).toBe('g2')
    expect(w.find('[data-review-stats]').text()).toBe(t('review.stats', { count: '3', handled: '1' }))
    expect(w.find('[data-review-pair="g1"] [data-review-label="accepted"]').exists()).toBe(true)
  })

  it('a flush that fails writes nothing', async () => {
    await open()
    state.reviewDiffNext()
    flushMock.mockResolvedValue('failed')
    await state.reviewAcceptChange()
    expect(acceptMock).not.toHaveBeenCalled()
    expect(state.reviewModeDiffMessage.value?.key).toBe('review.accept_flush_failed')
  })

  it('nothing is written when the cursor is on no change', async () => {
    await open()
    await state.reviewAcceptChange()
    await state.reviewSkipChange()
    expect(acceptMock).not.toHaveBeenCalled()
    expect(skipMock).not.toHaveBeenCalled()
    expect(state.reviewModeDiffMessage.value?.key).toBe('review.change_none_current')
  })

  it('an accept that Rust holds back asks in place, writes nothing, and forces only after the user agrees', async () => {
    const w = await open()
    state.reviewDiffNext()
    acceptMock.mockResolvedValueOnce(acceptedOutcome({ target_text: 'bản nháp', needs_confirmation: true, unsigned_draft: 'bản nháp' }))
    await state.reviewAcceptChange()
    await settle()

    expect(replaceMock).not.toHaveBeenCalled()
    const block = w.find('[data-review-confirm]')
    expect(block.exists()).toBe(true)
    expect(block.text()).toContain('bản nháp')
    expect(w.findAll('[role="dialog"]')).toHaveLength(0)

    acceptMock.mockResolvedValueOnce(acceptedOutcome())
    expect(w.find('[data-review-overwrite]').exists()).toBe(true)
    await state.confirmPendingAccept()
    await settle()
    expect(acceptMock).toHaveBeenLastCalledWith(7, 1, 'a x', true)
    expect(replaceMock).toHaveBeenCalledTimes(1)
    expect(w.find('[data-review-confirm]').exists()).toBe(false)
  })

  it('keeping my text drops the question and leaves everything as it was', async () => {
    const w = await open()
    state.reviewDiffNext()
    acceptMock.mockResolvedValueOnce(acceptedOutcome({ needs_confirmation: true, unsigned_draft: 'bản nháp' }))
    await state.reviewAcceptChange()
    await settle()
    expect(w.find('[data-review-keep]').exists()).toBe(true)
    state.cancelPendingAccept()
    await settle()
    expect(w.find('[data-review-confirm]').exists()).toBe(false)
    expect(acceptMock).toHaveBeenCalledTimes(1)
    expect(replaceMock).not.toHaveBeenCalled()
  })

  it('a refusal because the text changed says so and reloads the diff', async () => {
    await open()
    state.reviewDiffNext()
    acceptMock.mockResolvedValue({
      outcome: null,
      error: { code: 'review.change_text_changed', message_key: 'err.review.change_text_changed', params: {}, retryable: false },
    })
    diffMock.mockClear()
    await state.reviewAcceptChange()
    await settle()
    expect(diffMock).toHaveBeenCalledTimes(1)
    expect(state.reviewModeDiffMessage.value?.key).toBe('err.review.change_text_changed')
    expect(replaceMock).not.toHaveBeenCalled()
  })

  it('skipping keeps the Editor untouched, counts as handled and survives closing and opening again', async () => {
    const w = await open()
    state.reviewDiffNext()
    skipMock.mockResolvedValue({ skipped: true, error: null })
    diffMock.mockResolvedValue(threeChanges(['skipped', null, null]))
    await state.reviewSkipChange()
    await settle()

    expect(skipMock).toHaveBeenCalledWith(7, 1)
    expect(replaceMock).not.toHaveBeenCalled()
    expect(flushMock).not.toHaveBeenCalled()
    expect(w.find('[data-review-pair="g1"] [data-review-label="skipped"]').exists()).toBe(true)

    await state.closeReviewMode()
    await state.openReviewMode(7)
    await settle()
    expect(state.reviewModeChangeStats.value).toEqual({ count: 3, handled: 1 })
    expect(state.reviewModePairs.value[0].pending).toBe(false)
  })

  it('when no change is left unprocessed it says so in words and stays put', async () => {
    await open(threeChanges([null, 'skipped', 'accepted']))
    state.reviewDiffNext()
    skipMock.mockResolvedValue({ skipped: true, error: null })
    diffMock.mockResolvedValue(threeChanges(['skipped', 'skipped', 'accepted']))
    await state.reviewSkipChange()
    await settle()
    expect(state.reviewModeDiffMessage.value?.key).toBe('review.all_processed')
  })

  it('a group that is not one to one has no accept button, only the hand-edit notice, and refuses the command', async () => {
    const w = await open(
      diffOf(
        pair(1, [10, 11], [100], [del('p'), ins('q')]),
        pair(2, [12], [], [del('mình tôi')]),
      ),
    )
    state.reviewDiffNext()
    await settle()
    expect(w.find('[data-review-accept]').exists()).toBe(false)
    expect(w.find('[data-review-skip]').exists()).toBe(true)
    expect(state.reviewModeChangeStats.value).toEqual({ count: 1, handled: 0 })
    expect(w.find('[data-review-pair="g1"] [data-review-label="manual"]').exists()).toBe(true)
    await state.reviewAcceptChange()
    expect(acceptMock).not.toHaveBeenCalled()
    expect(state.reviewModeDiffMessage.value?.key).toBe('review.change_manual')

    state.reviewDiffNext()
    await settle()
    expect(w.find('[data-review-pair="g2"] [data-review-skip]').exists()).toBe(false)
    expect(w.find('[data-review-pair="g2"] [data-review-label="manual"]').exists()).toBe(true)
    await state.reviewSkipChange()
    expect(skipMock).not.toHaveBeenCalled()
  })

  it('an accepted pair keeps its place among the changes that the jumps walk, with no buttons', async () => {
    const w = await open(
      diffOf(
        pair(1, [10], [100], [eq('a x')], 'accepted'),
        pair(2, [11], [101], [eq('same')]),
        pair(3, [12], [102], [del('p'), ins('q')]),
      ),
    )
    expect(state.reviewModeChangeStats.value).toEqual({ count: 2, handled: 1 })
    state.reviewDiffNext()
    await settle()
    expect(state.reviewModeCurrentPairKey.value).toBe('g1')
    expect(w.find('[data-review-actions]').exists()).toBe(false)
    state.reviewDiffNext()
    expect(state.reviewModeCurrentPairKey.value).toBe('g3')
  })
})
