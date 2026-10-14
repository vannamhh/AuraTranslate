import { afterEach, describe, expect, it } from 'vitest'
import type { ChapterRow } from '../../src/config/chapter'
import type { ChapterViewItem } from '../../src/modes/libraryChapterSelection'
import {
  applyChapterClick,
  buildChapterView,
  chapterActionRules,
  chapterMatchesQuery,
  countChaptersByGroup,
  pruneChapterSelection,
  libraryChapterSelectedIds,
  libraryChapterAnchorId,
  resetLibraryChapterSelection,
  setChapterSelection,
  visibleChapterIds,
} from '../../src/modes/libraryChapterSelection'

function row(id: number, status: string, over: Partial<ChapterRow> = {}): ChapterRow {
  return {
    chapter_id: id,
    ord: id,
    title: `Chuong ${id}`,
    status,
    segment_count: 4,
    translated_count: 0,
    unconfirmed_count: 0,
    unlinked_count: 0,
    in_translation_count: 4,
    updated_at: '2026-10-01T00:00:00Z',
    origin_author: null,
    origin_site_name: null,
    origin_url: null,
    origin_published_at: null,
    ...over,
  }
}

function groupKeys(items: ChapterViewItem[]): string[] {
  const keys: string[] = []
  for (const item of items) if (item.kind === 'group') keys.push(item.key)
  return keys
}

const MIXED: ChapterRow[] = [
  row(1, 'done'),
  row(2, 'in_progress'),
  row(3, 'not_started'),
  row(4, 'not_started'),
  row(5, 'not_started'),
  row(6, 'paused'),
]

afterEach(() => resetLibraryChapterSelection())

describe('applyChapterClick', () => {
  const visible = [1, 2, 3, 4, 5]

  it('plain click selects only that row and moves the anchor', () => {
    const out = applyChapterClick(new Set([1, 2]), 1, visible, 4, { toggle: false, range: false })
    expect([...out.selected]).toEqual([4])
    expect(out.anchor).toBe(4)
  })

  it('Mod+click toggles a row in and out without touching the others', () => {
    const added = applyChapterClick(new Set([1]), 1, visible, 3, { toggle: true, range: false })
    expect([...added.selected].sort()).toEqual([1, 3])
    const removed = applyChapterClick(added.selected, added.anchor, visible, 1, { toggle: true, range: false })
    expect([...removed.selected]).toEqual([3])
  })

  it('Shift+click selects the range from the anchor and keeps the anchor', () => {
    const out = applyChapterClick(new Set([2]), 2, visible, 5, { toggle: false, range: true })
    expect([...out.selected].sort()).toEqual([2, 3, 4, 5])
    expect(out.anchor).toBe(2)
  })

  it('Shift+click works backwards', () => {
    const out = applyChapterClick(new Set([4]), 4, visible, 2, { toggle: false, range: true })
    expect([...out.selected].sort()).toEqual([2, 3, 4])
  })

  it('Shift+Mod+click adds the range to the existing selection', () => {
    const out = applyChapterClick(new Set([1, 4]), 4, visible, 5, { toggle: true, range: true })
    expect([...out.selected].sort()).toEqual([1, 4, 5])
  })

  it('Shift+click with no anchor falls back to a plain click', () => {
    const out = applyChapterClick(new Set(), null, visible, 3, { toggle: false, range: true })
    expect([...out.selected]).toEqual([3])
    expect(out.anchor).toBe(3)
  })
})

describe('buildChapterView', () => {
  it('orders groups in progress, paused, done, not started and numbers each group', () => {
    const view = buildChapterView(MIXED, 'all', '', 'order', true)
    const groups = groupKeys(view.items)
    expect(groups).toEqual(['in_progress', 'paused', 'done', 'not_started'])
    expect(visibleChapterIds(view.items)).toEqual([2, 6, 1, 3, 4, 5])
    expect(view.matched).toBe(6)
  })

  it('folds the not-started group to two rows with a fold note while the filter is all and no query', () => {
    const view = buildChapterView(MIXED, 'all', '', 'order', false)
    expect(visibleChapterIds(view.items)).toEqual([2, 6, 1, 3, 4])
    const more = view.items.find((i) => i.kind === 'more')
    expect(more).toEqual({ kind: 'more', hidden: 1 })
    expect(view.items.find((i) => i.kind === 'group' && i.key === 'not_started')).toEqual({
      kind: 'group',
      key: 'not_started',
      total: 3,
    })
  })

  it('does not fold when a status filter or a query is active', () => {
    expect(visibleChapterIds(buildChapterView(MIXED, 'not_started', '', 'order', false).items)).toEqual([3, 4, 5])
    expect(visibleChapterIds(buildChapterView(MIXED, 'all', 'chuong', 'order', false).items)).toHaveLength(6)
  })

  it('filter keeps only the chosen status and drops empty groups', () => {
    const view = buildChapterView(MIXED, 'paused', '', 'order', false)
    expect(visibleChapterIds(view.items)).toEqual([6])
    expect(view.items.filter((i) => i.kind === 'group')).toHaveLength(1)
  })

  it('search matches the title case-insensitively or the number', () => {
    expect(chapterMatchesQuery(row(7, 'done', { title: 'Mo Dau' }), 'mo d')).toBe(true)
    expect(chapterMatchesQuery(row(88, 'done', { title: null }), '88')).toBe(true)
    expect(chapterMatchesQuery(row(7, 'done'), 'xyz')).toBe(false)
    expect(chapterMatchesQuery(row(7, 'done'), '   ')).toBe(true)
  })

  it('empty search result: no items and matched 0 (the page says no Chapter matches)', () => {
    const view = buildChapterView(MIXED, 'all', 'xyz', 'order', false)
    expect(view.items).toEqual([])
    expect(view.matched).toBe(0)
  })

  it('sort recent puts the latest updated_at first inside a group, ties broken by order', () => {
    const rows = [
      row(1, 'done', { updated_at: '2026-10-01T00:00:00Z' }),
      row(2, 'done', { updated_at: '2026-10-03T00:00:00Z' }),
      row(3, 'done', { updated_at: '2026-10-03T00:00:00Z' }),
    ]
    expect(visibleChapterIds(buildChapterView(rows, 'all', '', 'recent', false).items)).toEqual([2, 3, 1])
    expect(visibleChapterIds(buildChapterView(rows, 'all', '', 'order', false).items)).toEqual([1, 2, 3])
  })

  it('an unknown status lands in its own last group instead of vanishing', () => {
    const view = buildChapterView([row(1, 'weird'), row(2, 'done')], 'all', '', 'order', false)
    expect(groupKeys(view.items)).toEqual(['done', 'unknown'])
  })
})

describe('countChaptersByGroup', () => {
  it('counts every row once, including unknown statuses', () => {
    expect(countChaptersByGroup([...MIXED, row(9, 'weird')])).toEqual({
      in_progress: 1,
      paused: 1,
      done: 1,
      not_started: 3,
      unknown: 1,
    })
  })
})

describe('chapterActionRules', () => {
  it('nothing selected: every button off', () => {
    expect(Object.values(chapterActionRules(MIXED, new Set())).every((v) => v === false)).toBe(true)
  })

  it('merge needs two or more neighbours in list order', () => {
    expect(chapterActionRules(MIXED, new Set([2, 3, 4])).merge).toBe(true)
    expect(chapterActionRules(MIXED, new Set([3])).merge).toBe(false)
  })

  it('merge is off for rows that are not neighbours (3 and 5)', () => {
    expect(chapterActionRules(MIXED, new Set([3, 5])).merge).toBe(false)
  })

  it('single selection enables rename, move, open and split (4 sentences); status and export need one or more', () => {
    const rules = chapterActionRules(MIXED, new Set([3]))
    expect(rules).toMatchObject({ rename: true, moveUp: true, moveDown: true, open: true, split: true, status: true, export: true })
  })

  it('split is off for a one-sentence Chapter', () => {
    const rows = [row(1, 'done', { segment_count: 1 })]
    expect(chapterActionRules(rows, new Set([1])).split).toBe(false)
  })

  it('several rows: rename, move, open and split off; status and export on', () => {
    const rules = chapterActionRules(MIXED, new Set([1, 2]))
    expect(rules).toMatchObject({ rename: false, moveUp: false, moveDown: false, open: false, split: false, status: true, export: true })
  })
})

describe('pruneChapterSelection', () => {
  it('drops ids that are gone after a reload and clears a dead anchor', () => {
    setChapterSelection(new Set([1, 2, 99]), 99)
    pruneChapterSelection(MIXED)
    expect([...libraryChapterSelectedIds.value].sort()).toEqual([1, 2])
    expect(libraryChapterAnchorId.value).toBeNull()
  })

  it('keeps a still-valid selection untouched', () => {
    setChapterSelection(new Set([1, 2]), 2)
    pruneChapterSelection(MIXED)
    expect([...libraryChapterSelectedIds.value].sort()).toEqual([1, 2])
    expect(libraryChapterAnchorId.value).toBe(2)
  })
})
