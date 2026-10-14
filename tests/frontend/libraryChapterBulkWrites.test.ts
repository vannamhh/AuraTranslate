import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const flushResult: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => flushResult.value }
})

function row(id: number, status = 'in_progress', over: Record<string, unknown> = {}) {
  return {
    chapter_id: id,
    ord: id,
    title: `Chuong ${id}`,
    status,
    segment_count: 3,
    translated_count: 0,
    unconfirmed_count: 0,
    unlinked_count: 0,
    in_translation_count: 3,
    updated_at: '2026-10-01T00:00:00Z',
    origin_author: null,
    origin_site_name: null,
    origin_url: null,
    origin_published_at: null,
    ...over,
  }
}

const ERR_NOT_CONTIGUOUS = {
  code: 'chapter.merge_not_contiguous',
  message_key: 'err.chapter.merge_not_contiguous',
  params: {},
  retryable: false,
}

function segment(id: number, ord: number, text: string) {
  return {
    id,
    ord,
    source_text: text,
    target_text: '',
    is_paragraph_end: false,
    retired_at: null,
    status: 'untranslated',
    is_omitted: false,
    unlinked: false,
    is_target_paragraph_end: false,
    role: null,
    translation_origin: 'none',
  }
}

function chapterSegments(chapterId: number) {
  return {
    chapter_id: chapterId,
    segments: [segment(11, 1, 'Cau mot。'), segment(12, 2, 'Cau hai。'), segment(13, 3, 'Cau ba。')],
    caret_segment_id: null,
    assets: [],
    assets_dir: '',
    tm_filled_segment_ids: [],
    tm_prefill: { kind: 'ran' },
  }
}

const ROWS = [row(1), row(2), row(3, 'done'), row(4, 'not_started')]

async function loadList(rows = ROWS) {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'list_chapters') return Promise.resolve(rows)
    return Promise.resolve(null)
  })
  const state = await import('../../src/modes/libraryChapters')
  await state.loadChapters()
  return state
}

async function select(ids: number[]) {
  const selection = await import('../../src/modes/libraryChapterSelection')
  selection.setChapterSelection(new Set(ids), ids[0] ?? null)
}

beforeAll(async () => {
  const commands = await import('../../src/commands')
  const deps = (await import('../../src/libraryWorkPageCommandDeps')).libraryWorkPageCommandDeps()
  commands.installCommands({ isMac: false, ...deps } as CommandDeps)
})

beforeEach(async () => {
  mockInvoke.mockReset()
  flushResult.value = 'clean'
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
  vi.spyOn(console, 'warn').mockImplementation(() => undefined)
  const state = await import('../../src/modes/libraryChapters')
  state.resetLibraryChapters()
  const editor = await import('../../src/panels/editorPanelState')
  editor.resetEditorPanel()
  const merge = await import('../../src/chapterMergeState')
  merge.resetChapterMerge()
  const split = await import('../../src/chapterSplitState')
  split.resetChapterSplit()
})

afterEach(() => {
  vi.restoreAllMocks()
  document.body.innerHTML = ''
})

describe('mergeSelectedChapters', () => {
  it('Merge three neighbours: one merge_chapters call with ids in list order, list replaced, survivor selected', async () => {
    const state = await loadList()
    await select([3, 1, 2])
    const rebuilt = [row(1, 'in_progress', { segment_count: 9, in_translation_count: 9 }), row(4, 'not_started', { ord: 2 })]
    mockInvoke.mockImplementation((cmd: string) => (cmd === 'merge_chapters' ? Promise.resolve(rebuilt) : Promise.resolve(null)))

    const outcome = await state.mergeSelectedChapters()

    expect(outcome).toBe('merged')
    expect(mockInvoke).toHaveBeenCalledWith('merge_chapters', { chapterIds: [1, 2, 3] })
    expect(state.libraryChapters.value.map((r) => r.chapter_id)).toEqual([1, 4])
    const selection = await import('../../src/modes/libraryChapterSelection')
    expect([...selection.libraryChapterSelectedIds.value]).toEqual([1])
    expect(state.libraryChapterReorgBusy.value).toBe(false)
  })

  it('Rust refuses (not contiguous): the list and the selection stay, the error is shown, busy is released', async () => {
    const state = await loadList()
    await select([1, 3])
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'merge_chapters' ? Promise.reject(ERR_NOT_CONTIGUOUS) : Promise.resolve(null),
    )

    const outcome = await state.mergeSelectedChapters()

    expect(outcome).toBe('refused')
    expect(state.libraryChapters.value.map((r) => r.chapter_id)).toEqual([1, 2, 3, 4])
    expect(state.libraryChapterReorgError.value?.code).toBe('chapter.merge_not_contiguous')
    const selection = await import('../../src/modes/libraryChapterSelection')
    expect([...selection.libraryChapterSelectedIds.value].sort()).toEqual([1, 3])
    expect(state.libraryChapterReorgBusy.value).toBe(false)
  })

  it('a malformed rebuilt list is refused and leaves the old list in place', async () => {
    const state = await loadList()
    await select([1, 2])
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'merge_chapters' ? Promise.resolve([{ chapter_id: 1 }]) : Promise.resolve(null),
    )
    expect(await state.mergeSelectedChapters()).toBe('refused')
    expect(state.libraryChapters.value).toHaveLength(4)
  })

  it('flush failed or still dirty blocks the write: no merge_chapters call, distinguishable notice', async () => {
    const state = await loadList()
    await select([1, 2])
    mockInvoke.mockClear()

    flushResult.value = 'still-dirty'
    expect(await state.mergeSelectedChapters()).toBe('blocked')
    expect(state.libraryChapterReorgNotice.value).toBe('still-dirty')

    flushResult.value = 'failed'
    expect(await state.mergeSelectedChapters()).toBe('blocked')
    expect(state.libraryChapterReorgNotice.value).toBe('flush-failed')
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'merge_chapters')).toBe(false)
  })

  it('fewer than two selected rows never reaches Rust', async () => {
    const state = await loadList()
    await select([2])
    mockInvoke.mockClear()
    expect(await state.mergeSelectedChapters()).toBe('refused')
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('the Editor reloads when its open Chapter was merged away', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.resolve(ROWS)
      if (cmd === 'read_open_chapter_segments') return Promise.resolve(chapterSegments(2))
      if (cmd === 'read_open_chapter') return Promise.resolve({ chapter_id: 1, source_text: 's', source_lang: 'zh' })
      if (cmd === 'merge_chapters') return Promise.resolve([row(1), row(4, 'not_started', { ord: 2 })])
      return Promise.resolve(null)
    })
    const state = await import('../../src/modes/libraryChapters')
    const editor = await import('../../src/panels/editorPanelState')
    await state.loadChapters()
    await editor.ensureSegmentsLoaded()
    expect(editor.editorChapterId.value).toBe(2)
    await select([1, 2])
    const readsBefore = mockInvoke.mock.calls.filter((c) => c[0] === 'read_open_chapter_segments').length

    expect(await state.mergeSelectedChapters()).toBe('merged')

    const readsAfter = mockInvoke.mock.calls.filter((c) => c[0] === 'read_open_chapter_segments').length
    expect(readsAfter).toBe(readsBefore + 1)
  })

  it('the Editor is left alone when its open Chapter was not part of the merge', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.resolve(ROWS)
      if (cmd === 'read_open_chapter_segments') return Promise.resolve(chapterSegments(4))
      if (cmd === 'merge_chapters') return Promise.resolve([row(1), row(3, 'done', { ord: 2 }), row(4, 'not_started', { ord: 3 })])
      return Promise.resolve(null)
    })
    const state = await import('../../src/modes/libraryChapters')
    const editor = await import('../../src/panels/editorPanelState')
    await state.loadChapters()
    await editor.ensureSegmentsLoaded()
    await select([1, 2])
    const readsBefore = mockInvoke.mock.calls.filter((c) => c[0] === 'read_open_chapter_segments').length

    expect(await state.mergeSelectedChapters()).toBe('merged')

    expect(mockInvoke.mock.calls.filter((c) => c[0] === 'read_open_chapter_segments').length).toBe(readsBefore)
  })
})

describe('selection follows the visible rows', () => {
  it('a status filter, a search and the fold each drop the selected rows they hide', async () => {
    const many = [row(1), row(2), row(3, 'done'), row(4, 'not_started'), row(5, 'not_started'), row(6, 'not_started')]
    await loadList(many)
    const selection = await import('../../src/modes/libraryChapterSelection')
    const ids = () => [...selection.libraryChapterSelectedIds.value].sort()

    await select([1, 3])
    selection.setChapterFilter('in_progress')
    expect(ids()).toEqual([1])
    selection.setChapterFilter('all')

    await select([1, 2])
    selection.libraryChapterSearch.value = 'Chuong 2'
    expect(ids()).toEqual([2])
    selection.libraryChapterSearch.value = ''

    selection.toggleNotStartedGroup()
    await select([4, 6])
    selection.toggleNotStartedGroup()
    expect(ids()).toEqual([4])
    selection.toggleNotStartedGroup()
  })
})

describe('moveCurrentChapterDown', () => {
  it('the cursor follows the moved Chapter to its new position after the reload', async () => {
    const state = await loadList()
    await select([1])
    const reordered = [row(2, 'in_progress', { ord: 1 }), row(1, 'in_progress', { ord: 2 }), row(3, 'done'), row(4, 'not_started')]
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'move_chapter') return Promise.resolve(null)
      if (cmd === 'list_chapters') return Promise.resolve(reordered)
      return Promise.resolve(null)
    })

    await state.moveCurrentChapterDown()

    expect(state.libraryChapters.value[state.libraryChapterCursor.value].chapter_id).toBe(1)
  })
})

describe('setSelectedChaptersStatus', () => {
  it('sends every selected id in one call, applies the lifecycle it returns and reloads the list', async () => {
    const state = await loadList()
    await select([1, 2, 3])
    const paused = [row(1, 'paused'), row(2, 'paused'), row(3, 'paused'), row(4, 'not_started')]
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'set_chapters_status') {
        return Promise.resolve({ status: 'paused', status_is_override: false, derived_status: 'paused' })
      }
      if (cmd === 'list_chapters') return Promise.resolve(paused)
      return Promise.resolve(null)
    })

    await state.setSelectedChaptersStatus('paused')

    expect(mockInvoke.mock.calls.filter((c) => c[0] === 'set_chapters_status')).toEqual([
      ['set_chapters_status', { chapterIds: [1, 2, 3], status: 'paused' }],
    ])
    const groups = (await import('../../src/modes/libraryChapterSelection')).countChaptersByGroup(state.libraryChapters.value)
    expect(groups.paused).toBe(3)
    const works = await import('../../src/modes/libraryWorks')
    expect(works.openWorkLifecycleDerivedStatus.value).toBe('paused')
    expect(state.libraryChapterReorgBusy.value).toBe(false)
  })

  it('Rust refuses: the list is not reloaded, the error is shown, busy is released', async () => {
    const state = await loadList()
    await select([1, 2])
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'set_chapters_status'
        ? Promise.reject({ code: 'segment.chapter_not_found', message_key: 'err.segment.chapter_not_found', params: {}, retryable: false })
        : Promise.resolve(null),
    )
    mockInvoke.mockClear()
    await state.setSelectedChaptersStatus('done')
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'list_chapters')).toBe(false)
    expect(state.libraryChapterReorgError.value?.code).toBe('segment.chapter_not_found')
    expect(state.libraryChapterReorgBusy.value).toBe(false)
    expect(state.libraryChapters.value.map((r) => r.status)).toEqual(['in_progress', 'in_progress', 'done', 'not_started'])
  })

  it('nothing selected: no call', async () => {
    const state = await loadList()
    mockInvoke.mockClear()
    await state.setSelectedChaptersStatus('done')
    expect(mockInvoke).not.toHaveBeenCalled()
  })
})

describe('merge screen', () => {
  let wrapper: VueWrapper | null = null
  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('does not open for a selection the rules refuse (non-contiguous)', async () => {
    await loadList()
    await select([1, 3])
    const merge = await import('../../src/chapterMergeState')
    merge.openChapterMerge()
    expect(merge.chapterMergeOverlayIsOpen.value).toBe(false)
  })

  it('opens with a snapshot of the selected rows, lists them, and closing returns focus to the opener', async () => {
    await loadList()
    await select([1, 2, 3])
    const opener = document.createElement('button')
    opener.setAttribute('data-library-chapter-merge-open', '')
    document.body.appendChild(opener)
    opener.focus()

    const { default: Overlay } = await import('../../src/ChapterMergeOverlay.vue')
    wrapper = mount(Overlay, { attachTo: document.body })
    const merge = await import('../../src/chapterMergeState')
    merge.openChapterMerge()
    await wrapper.vm.$nextTick()

    expect(merge.chapterMergeRows.value.map((r) => r.chapter_id)).toEqual([1, 2, 3])
    expect(document.querySelectorAll('[data-library-chapter-merge-list] li')).toHaveLength(3)
    expect(document.querySelector('[data-library-chapter-merge-result]')?.textContent).toContain('9')

    const { dispatch } = await import('../../src/commands')
    dispatch('library.chapter_merge.close')
    await wrapper.vm.$nextTick()
    expect(document.querySelector('[data-library-chapter-merge-overlay]')).toBeNull()
    expect(document.activeElement).toBe(opener)
  })

  it('confirm on a successful merge closes the screen; a refusal keeps it open with the error', async () => {
    const state = await loadList()
    await select([1, 2])
    const merge = await import('../../src/chapterMergeState')
    merge.openChapterMerge()

    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'merge_chapters' ? Promise.reject(ERR_NOT_CONTIGUOUS) : Promise.resolve(null),
    )
    await merge.confirmChapterMerge()
    expect(merge.chapterMergeOverlayIsOpen.value).toBe(true)
    expect(state.libraryChapterReorgError.value).not.toBeNull()

    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'merge_chapters' ? Promise.resolve([row(1), row(3, 'done', { ord: 2 }), row(4, 'not_started', { ord: 3 })]) : Promise.resolve(null),
    )
    await merge.confirmChapterMerge()
    expect(merge.chapterMergeOverlayIsOpen.value).toBe(false)
  })
})

describe('split box', () => {
  let wrapper: VueWrapper | null = null
  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  function mockSplitBackend(extra: Record<string, unknown> = {}) {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd in extra) return (extra[cmd] as { message_key?: string } | null)?.message_key !== undefined ? Promise.reject(extra[cmd]) : Promise.resolve(extra[cmd])
      if (cmd === 'list_chapters') return Promise.resolve(ROWS)
      if (cmd === 'open_chapter') return Promise.resolve({ chapter_id: 2, source_text: 's', source_lang: 'zh' })
      if (cmd === 'read_open_chapter') return Promise.resolve({ chapter_id: 2, source_text: 's', source_lang: 'zh' })
      if (cmd === 'read_open_chapter_segments') return Promise.resolve(chapterSegments(2))
      return Promise.resolve(null)
    })
  }

  it('does not open unless exactly one Chapter with two or more sentences is selected', async () => {
    await loadList()
    mockSplitBackend()
    const split = await import('../../src/chapterSplitState')
    await select([1, 2])
    await split.openChapterSplit()
    expect(split.chapterSplitOverlayIsOpen.value).toBe(false)
  })

  it('lists the sentences, keeps confirm off for the first one and on for a later one', async () => {
    await loadList()
    mockSplitBackend()
    await select([2])
    const opener = document.createElement('button')
    opener.setAttribute('data-library-chapter-split-open', '')
    document.body.appendChild(opener)
    opener.focus()
    const { default: Overlay } = await import('../../src/ChapterSplitOverlay.vue')
    wrapper = mount(Overlay, { attachTo: document.body })
    const split = await import('../../src/chapterSplitState')

    await split.openChapterSplit()
    await wrapper.vm.$nextTick()

    expect(split.chapterSplitLoadStatus.value).toBe('loaded')
    const confirm = () => document.querySelector<HTMLButtonElement>('[data-library-chapter-split-confirm]')
    expect(document.querySelectorAll('[data-library-chapter-split-list] input')).toHaveLength(3)
    expect(document.querySelector<HTMLInputElement>('[data-library-chapter-split-sentence="11"]')?.disabled).toBe(true)
    expect(confirm()?.disabled).toBe(true)

    split.pickChapterSplitSentence(11)
    await wrapper.vm.$nextTick()
    expect(confirm()?.disabled).toBe(true)

    split.pickChapterSplitSentence(12)
    await wrapper.vm.$nextTick()
    expect(confirm()?.disabled).toBe(false)

    split.closeChapterSplit()
    await wrapper.vm.$nextTick()
    expect(document.querySelector('[data-library-chapter-split-overlay]')).toBeNull()
    expect(document.activeElement).toBe(opener)
  })

  it('Rust refusing the split keeps the box open and shows the error', async () => {
    const refusal = {
      code: 'chapter.split_leaves_empty',
      message_key: 'err.chapter.split_leaves_empty',
      params: {},
      retryable: false,
    }
    await loadList()
    mockSplitBackend({ split_chapter_at_segment: refusal })
    await select([2])
    const split = await import('../../src/chapterSplitState')
    await split.openChapterSplit()
    split.pickChapterSplitSentence(12)

    await split.confirmChapterSplit()

    expect(split.chapterSplitOverlayIsOpen.value).toBe(true)
    expect(split.chapterSplitFailure.value?.error?.code ?? split.chapterSplitFailure.value?.notice).toBeTruthy()
  })

  it('a successful split closes the box and reloads the Chapter list', async () => {
    await loadList()
    mockSplitBackend({ split_chapter_at_segment: null })
    await select([2])
    const split = await import('../../src/chapterSplitState')
    await split.openChapterSplit()
    split.pickChapterSplitSentence(12)
    const listsBefore = mockInvoke.mock.calls.filter((c) => c[0] === 'list_chapters').length

    await split.confirmChapterSplit()

    expect(split.chapterSplitOverlayIsOpen.value).toBe(false)
    expect(mockInvoke.mock.calls.filter((c) => c[0] === 'list_chapters').length).toBe(listsBefore + 1)
  })
})
