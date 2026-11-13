/**
 * Double-click lookup in Reading mode's bilingual source notes. Only the IPC boundary is faked;
 * `ReadingMode.vue`, the popup, the command registry and the real lookup, pin and Concordance
 * handlers run. Placement geometry is WebKit's business and lives in the e2e spec.
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { KeepAlive, defineComponent, h as hyper, ref } from 'vue'
import ReadingMode from '../../src/modes/ReadingMode.vue'
import { dispatch, installCommands } from '../../src/commands'
import type { CommandDeps } from '../../src/commands'
import { attachSelectionWatcher, currentSelectionText } from '../../src/panels/selectionContract'
import { lookupAimedChar, resetLookupPanel, runLookup } from '../../src/panels/lookupPanelState'
import {
  loadPinnedEntries,
  lookupHistory,
  lookupTab,
  pinnedEntries,
  resetLookupHistory,
  selectLookupTab,
  toggleLookupPin,
} from '../../src/panels/lookupHistoryState'
import { loadDictSources, resetDictSources } from '../../src/panels/dictSourcesState'
import { openReadingLookup, readingLookupAnchor, resetReadingLookup } from '../../src/modes/readingLookupState'
import { openReadingLookupConcordance } from '../../src/tmConcordanceCommandDeps'
import {
  ensureReadingLoaded,
  readingBilingual,
  resetReading,
  setReadingLevel,
  toggleBilingual,
} from '../../src/modes/readingState'
import { currentMode, setMode } from '../../src/modes/modeState'
import { flushPromises } from './support/flushMicrotasks'
import { installFakeDragJs, pressDragHeader } from './support/dragRegionPress'
import { NOT_FOUND, SOURCES, group, hit, response, sense } from './support/lookupFixtures'

const KONGZHI = response(
  [group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 3, '控制', 'kong4 zhi4')])],
  { 'layer-cv': [sense(3, 31, 'khống chế')] },
)

const SOURCE_TEXT = '他控制住了局面。'
const TARGET_TEXT = 'Anh ấy kiểm soát được tình hình.'

const RUN = {
  chapters: [
    {
      chapter_id: 1,
      chapter_ord: 1,
      chapter_title: 'Chuong Mot',
      blocks: [
        { kind: 'paragraph', segments: [{ id: 1, source_text: SOURCE_TEXT, target_text: TARGET_TEXT, is_confirmed: true, is_marked: false }] },
      ],
      segment_count: 1,
      translation_origin_counts: { self_count: 0, other_count: 0, bilingual_import_count: 0 },
      images: [],
    },
  ],
  frontier: { kind: 'end-of-work', chapter: null },
  assets_dir: '/tmp/Work.atproj/assets',
}

const IPC_FAILURE = { code: 'dict.read', message_key: 'err.unknown', params: {}, retryable: false }

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> | undefined }[],
  answers: new Map<string, unknown>(),
  lookupFails: false,
  pinned: [] as unknown[],
  hold: null as Promise<void> | null,
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args })
    if (cmd === 'read_reading_run') return RUN
    if (cmd === 'list_dict_sources') return { sources: SOURCES, skipped: [] }
    if (cmd === 'lookup_dictionary') {
      if (ipc.hold !== null) await ipc.hold
      if (ipc.lookupFails) throw IPC_FAILURE
      return ipc.answers.get(String(args?.query)) ?? NOT_FOUND
    }
    if (cmd === 'list_pinned_entries') return ipc.pinned
    if (cmd === 'pin_entry') {
      ipc.pinned = [
        {
          id: 1,
          source_code: args?.sourceCode,
          entry_id: args?.entryId,
          headword: args?.headword,
          gloss: args?.gloss,
          pinned_at: '2026-10-29T00:00:00Z',
          source_version: null,
        },
      ]
      return ipc.pinned
    }
    if (cmd === 'tm_concordance') throw IPC_FAILURE
    return undefined
  }),
  convertFileSrc: (path: string) => path,
}))

beforeAll(() => {
  installCommands({
    isMac: true,
    setMode,
    runLookup,
    currentSelection: currentSelectionText,
    openReadingLookup,
    closeReadingLookup: resetReadingLookup,
    openReadingLookupConcordance,
    toggleLookupPin,
    lookupAimedChar,
  } as CommandDeps)
})

let wrapper: VueWrapper | null = null

const lookups = (): string[] => ipc.calls.filter((c) => c.cmd === 'lookup_dictionary').map((c) => String(c.args?.query))

async function mountReading(component: unknown = ReadingMode): Promise<VueWrapper> {
  await loadDictSources('')
  wrapper = mount(component as typeof ReadingMode, { attachTo: document.body })
  await ensureReadingLoaded()
  await flushPromises()
  return wrapper
}

function root(): HTMLElement {
  return document.querySelector<HTMLElement>('section.mode')!
}

function popup(): HTMLElement | null {
  return document.querySelector<HTMLElement>('[data-reading-lookup]')
}

/** Selects `word` inside `el`'s text, the way WebKit does before it fires `dblclick`. */
function selectWord(el: HTMLElement, word: string): void {
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const text = node as Text
    const at = text.data.indexOf(word)
    if (at === -1) continue
    const range = document.createRange()
    range.setStart(text, at)
    range.setEnd(text, at + word.length)
    window.getSelection()!.removeAllRanges()
    window.getSelection()!.addRange(range)
    return
  }
  throw new Error(`"${word}" not found in ${el.className}`)
}

async function doubleClickWord(el: HTMLElement, word: string): Promise<void> {
  selectWord(el, word)
  el.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }))
  await flushPromises()
}

const sourceNote = (): HTMLElement => document.querySelector<HTMLElement>('[data-reading-source-note]')!

async function openOn(word: string): Promise<void> {
  await mountReading()
  await doubleClickWord(sourceNote(), word)
}

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
  ipc.calls = []
  ipc.answers = new Map([['控制', KONGZHI]])
  ipc.lookupFails = false
  ipc.pinned = []
  ipc.hold = null
  if (!readingBilingual.value) toggleBilingual()
})

afterEach(async () => {
  wrapper?.unmount()
  wrapper = null
  resetReadingLookup()
  resetReading()
  resetLookupPanel()
  resetLookupHistory()
  resetDictSources()
  selectLookupTab('record')
  setMode('library')
  await flushPromises()
})

describe('opening the popup', () => {
  it('a double-click on a word of the source note looks it up and shows word, reading and gloss beside it', async () => {
    await openOn('控制')

    expect(lookups()).toEqual(['控制'])
    const box = popup()
    expect(box).not.toBeNull()
    expect(box!.parentElement).toBe(root())
    expect(box!.querySelector('[data-reading-lookup-headword]')?.textContent).toBe('控制')
    expect(box!.textContent).toContain('kòng zhì')
    expect(box!.textContent).toContain('khống chế')
    expect(lookupHistory.value.map((row) => row.query)).toEqual(['控制'])
  })

  it('a drag selection released over the source note looks nothing up and opens nothing', async () => {
    await mountReading()
    const detach = attachSelectionWatcher(document, () => dispatch('lookup.lookup_selection'))
    try {
      selectWord(sourceNote(), '控制住')
      sourceNote().dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
      await flushPromises()
    } finally {
      detach()
    }

    expect(lookups()).toEqual([])
    expect(popup()).toBeNull()
    expect(lookupHistory.value).toEqual([])
  })

  it('a double-click on the Vietnamese translation opens nothing', async () => {
    await mountReading()
    await doubleClickWord(document.querySelector<HTMLElement>('.segment-text')!, 'kiểm')

    expect(lookups()).toEqual([])
    expect(popup()).toBeNull()
    expect(readingLookupAnchor.value).toBeNull()
  })

  it('the open command refuses a selection outside the source notes', async () => {
    await mountReading()
    selectWord(document.querySelector<HTMLElement>('.segment-text')!, 'kiểm')
    dispatch('reading.lookup_open')
    await flushPromises()

    expect(lookups()).toEqual([])
    expect(readingLookupAnchor.value).toBeNull()
  })

  it('a selection anchored on the note element rather than its text opens nothing', async () => {
    await mountReading()
    const range = document.createRange()
    range.selectNodeContents(sourceNote())
    range.setStart(sourceNote(), 0)
    window.getSelection()!.removeAllRanges()
    window.getSelection()!.addRange(range)
    expect(window.getSelection()!.anchorNode).toBe(sourceNote())
    sourceNote().dispatchEvent(new MouseEvent('dblclick', { bubbles: true }))
    await flushPromises()

    expect(lookups()).toEqual([])
    expect(readingLookupAnchor.value).toBeNull()
  })

  it('a fresh double-click keeps the popup hidden until its own lookup answers', async () => {
    await mountReading()
    let release!: () => void
    ipc.hold = new Promise((resolve) => (release = resolve))
    await doubleClickWord(sourceNote(), '控制')

    expect(popup()).not.toBeNull()
    expect(popup()!.style.visibility).toBe('hidden')

    release()
    await flushPromises()
    expect(popup()!.style.visibility).toBe('')
    expect(popup()!.querySelector('[data-reading-lookup-headword]')?.textContent).toBe('控制')
  })

  it('a lookup started inside the popup keeps the same popup shown and focus where it was', async () => {
    await openOn('控制住')
    const box = popup()!
    const char = box.querySelector<HTMLButtonElement>('[data-lookup-char="控"]')!
    const concordance = box.querySelector<HTMLButtonElement>('[data-reading-lookup-concordance]')!
    concordance.focus()
    let release!: () => void
    ipc.hold = new Promise((resolve) => (release = resolve))

    char.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    char.click()
    await flushPromises()
    expect(popup()).toBe(box)
    expect(box.style.visibility).toBe('')
    expect(document.activeElement).toBe(concordance)

    release()
    await flushPromises()
    expect(popup()).toBe(box)
    expect(document.activeElement).toBe(concordance)
    expect(lookups()).toEqual(['控制住', '控'])
  })

  it('a phrase the dictionary lacks says so and offers each character as its own lookup', async () => {
    await openOn('控制住')

    expect(popup()!.querySelector('[data-reading-lookup-not-found]')?.textContent).toContain('控制住')
    const chars = [...popup()!.querySelectorAll<HTMLButtonElement>('[data-lookup-char]')]
    expect(chars.map((b) => b.dataset.lookupChar)).toEqual(['控', '制', '住'])

    chars[0]!.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    chars[0]!.click()
    await flushPromises()
    expect(lookups()).toEqual(['控制住', '控'])
    expect(popup()).not.toBeNull()
  })

  it('an IPC failure shows the Lookup panel failure sentence, not an empty popup', async () => {
    ipc.lookupFails = true
    await openOn('控制')

    expect(popup()!.querySelector('[data-reading-lookup-error]')?.textContent).toContain('Không tra cứu được')
  })
})

describe('closing the popup', () => {
  it('Esc closes it and focus stays on the reading page', async () => {
    await openOn('控制')
    popup()!.querySelector<HTMLButtonElement>('[data-reading-lookup-concordance]')!.focus()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flushPromises()

    expect(popup()).toBeNull()
    expect(document.activeElement).toBe(root())
  })

  it('a press outside closes it; a press inside keeps it', async () => {
    await openOn('控制')

    popup()!.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await flushPromises()
    expect(popup()).not.toBeNull()

    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await flushPromises()
    expect(popup()).toBeNull()
  })

  it('a press on a drag-region header closes it although drag.js swallows the bubble phase', async () => {
    installFakeDragJs()
    await openOn('控制')

    pressDragHeader()
    await flushPromises()
    expect(popup()).toBeNull()
  })

  it('turning the bilingual column off closes it for good', async () => {
    await openOn('控制')

    toggleBilingual()
    await flushPromises()
    expect(popup()).toBeNull()
    expect(readingLookupAnchor.value).toBeNull()

    toggleBilingual()
    await flushPromises()
    expect(popup()).toBeNull()
  })

  it('leaving Reading mode closes it', async () => {
    const shown = ref(true)
    const Host = defineComponent({
      setup: () => () => hyper(KeepAlive, null, [shown.value ? hyper(ReadingMode) : hyper('div')]),
    })
    await mountReading(Host)
    await doubleClickWord(sourceNote(), '控制')
    expect(popup()).not.toBeNull()

    shown.value = false
    await flushPromises()
    expect(readingLookupAnchor.value).toBeNull()

    shown.value = true
    await flushPromises()
    expect(popup()).toBeNull()
  })

  it('a reflow of the text under it closes it: reading level, a replaced run, a window resize', async () => {
    await openOn('控制')
    setReadingLevel('lg')
    await flushPromises()
    expect(readingLookupAnchor.value).toBeNull()

    await doubleClickWord(sourceNote(), '控制')
    expect(popup()).not.toBeNull()
    resetReading()
    await ensureReadingLoaded()
    await flushPromises()
    expect(readingLookupAnchor.value).toBeNull()

    await doubleClickWord(sourceNote(), '控制')
    expect(popup()).not.toBeNull()
    window.dispatchEvent(new Event('resize'))
    await flushPromises()
    expect(popup()).toBeNull()
  })

  it('a new double-click replaces the word shown', async () => {
    ipc.answers.set('局面', response([group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 4, '局面', 'ju2 mian4')])], {}))
    await openOn('控制')

    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    await doubleClickWord(sourceNote(), '局面')

    expect(lookups()).toEqual(['控制', '局面'])
    expect(popup()!.querySelector('[data-reading-lookup-headword]')?.textContent).toBe('局面')
  })
})

describe('popup actions', () => {
  it('the pin button pins the entry into the Lookup panel pinned list', async () => {
    await loadPinnedEntries()
    await openOn('控制')

    popup()!.querySelector<HTMLButtonElement>('[data-reading-lookup-pin]')!.click()
    await flushPromises()

    expect(ipc.calls.filter((c) => c.cmd === 'pin_entry').map((c) => c.args?.entryId)).toEqual([3])
    expect(pinnedEntries.value.map((p) => p.headword)).toEqual(['控制'])
  })

  it('the Concordance line closes the popup and opens Workspace on the Concordance tab for the word', async () => {
    await openOn('控制')

    popup()!.querySelector<HTMLButtonElement>('[data-reading-lookup-concordance]')!.click()
    await flushPromises()

    expect(readingLookupAnchor.value).toBeNull()
    expect(currentMode.value).toBe('workspace')
    expect(lookupTab.value).toBe('concordance')
    expect(ipc.calls.filter((c) => c.cmd === 'tm_concordance').map((c) => c.args?.query)).toEqual(['控制'])
  })
})
