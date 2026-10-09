/**
 * Review Mode: a read-only two-panel layout state inside Workspace. `config/alignment.ts` is the IPC
 * boundary and is mocked; `WorkspaceMode`, `ReviewDock` and `WorkspaceDock` mount for real.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { defineComponent, h, KeepAlive, nextTick } from 'vue'
import { readFileSync } from 'node:fs'
import type { ChapterAlignment } from '../../src/config/alignment'
import GridPanel from '../../src/panels/GridPanel.vue'
import WorkspaceMode from '../../src/modes/WorkspaceMode.vue'
import { applyPreset, isDockSuspended, panelRing, togglePanel } from '../../src/layout/dockController'
import { declareFocus, dispatch, enterFocus, installCommands, MODE_IDS, releaseFocus } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import { currentMode, setMode } from '../../src/modes/modeState'
import * as state from '../../src/reviewModeState'
import { t } from '../../src/i18n'

const openMock = vi.fn()
const putMock = vi.fn(() => Promise.resolve(null))

vi.mock('../../src/config/alignment', () => ({
  alignmentOpen: (...args: unknown[]) => openMock(...args),
  alignmentJoin: vi.fn(),
  alignmentSkip: vi.fn(),
  alignmentUnjoin: vi.fn(),
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
      'sửa một',
      `${t('review.row_caption')}chú thích`,
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
