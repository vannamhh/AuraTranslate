import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { t } from '../../src/i18n'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

function chapterRow(id: number, status: string) {
  return {
    chapter_id: id,
    ord: id,
    title: null,
    status,
    segment_count: 1,
    translated_count: 1,
    unconfirmed_count: 0,
    in_translation_count: 0,
    unlinked_count: 0,
    updated_at: '2026-11-02T00:00:00.000Z',
    origin_author: null,
    origin_site_name: null,
    origin_url: null,
    origin_published_at: null,
  }
}

function runOf(ord: number, title: string | null, withChapter = true) {
  return {
    chapters: !withChapter ? [] : [
      {
        chapter_id: ord,
        chapter_ord: ord,
        chapter_title: title,
        blocks: [{ kind: 'paragraph', segments: [{ id: 11, source_text: 'a', target_text: 'b', is_confirmed: true, is_marked: false }] }],
        segment_count: 1,
        translation_origin_counts: { self_count: 1, other_count: 0, bilingual_import_count: 0 },
        images: [],
      },
    ],
    frontier: { kind: 'end-of-work', chapter: null },
    assets_dir: '',
  }
}

async function mountReading(options: { ord: number; title: string | null; done: number | null; withChapter?: boolean }) {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'read_reading_run') return Promise.resolve(runOf(options.ord, options.title, options.withChapter ?? true))
    if (cmd === 'list_chapters') {
      if (options.done === null) return Promise.resolve(null)
      return Promise.resolve(Array.from({ length: options.done }, (_, i) => chapterRow(i + 1, 'done')))
    }
    if (cmd === 'list_reading_marks') return Promise.resolve([])
    return Promise.resolve(null)
  })
  const state = await import('../../src/modes/readingState')
  const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
  const wrapper = mount(ReadingMode, { attachTo: document.body })
  await state.ensureReadingLoaded()
  await vi.waitFor(() => {
    expect(mockInvoke).toHaveBeenCalledWith('list_chapters')
  })
  await vi.waitFor(() => {
    if (options.done !== null) expect(state.readingChaptersDone.value).toBe(options.done)
  })
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
  return wrapper
}

let wrapper: Awaited<ReturnType<typeof mountReading>> | null = null

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the position strip', () => {
  it('shows the bar at 34% and the bold counts for 128 done Chapters at Chapter 44', async () => {
    wrapper = await mountReading({ ord: 44, title: 'Cánh cửa', done: 128 })
    const strip = wrapper.find('[data-reading-progress]')
    expect(strip.exists()).toBe(true)
    expect(strip.text()).toContain('Chương 44 · Cánh cửa')
    expect(strip.text()).toContain('128 Chương đã dịch · đang ở 44')
    expect(strip.findAll('b').map((b) => b.text())).toEqual(['Cánh cửa', '44'])
    const fill = strip.find('[data-reading-progress-fill]')
    expect(fill.attributes('style')).toContain('34%')
    expect(strip.find('[data-reading-progress-bar]').attributes('aria-hidden')).toBe('true')
  })

  it('stops the bar at 100% when the Chapter order passes the done count', async () => {
    wrapper = await mountReading({ ord: 130, title: null, done: 128 })
    expect(wrapper.find('[data-reading-progress-fill]').attributes('style')).toContain('100%')
  })

  it('draws 0% without dividing by zero when nothing is done', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 0 })
    const fill = wrapper.find('[data-reading-progress-fill]')
    expect(fill.exists()).toBe(true)
    expect(fill.attributes('style')).toContain('0%')
    expect(fill.attributes('style')).not.toContain('NaN')
  })

  it('counts done, in-progress and paused Chapters, not not-started or unknown ones', async () => {
    const statuses = ['done', 'in_progress', 'paused', 'not_started', 'finished']
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'read_reading_run') return Promise.resolve(runOf(1, null, true))
      if (cmd === 'list_chapters') return Promise.resolve(statuses.map((status, i) => chapterRow(i + 1, status)))
      return Promise.resolve([])
    })
    const state = await import('../../src/modes/readingState')
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await vi.waitFor(() => expect(state.readingChaptersDone.value).toBe(3))
  })

  it('is absent when the run has no Chapter at all', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 5, withChapter: false })
    expect(wrapper.find('.page').exists()).toBe(false)
    expect(wrapper.find('[data-reading-progress]').exists()).toBe(false)
  })

  it('is absent while the done count is unknown', async () => {
    wrapper = await mountReading({ ord: 3, title: 'Ba', done: null })
    expect(mockInvoke).toHaveBeenCalledWith('list_chapters')
    const state = await import('../../src/modes/readingState')
    expect(state.readingChaptersDone.value).toBeNull()
    expect(wrapper.find('[data-reading-progress]').exists()).toBe(false)
    expect(wrapper.find('[data-reading-chapter-origin]').exists()).toBe(true)
  })

  it('names an untitled Chapter once in bold and does not repeat it above the title', async () => {
    wrapper = await mountReading({ ord: 5, title: null, done: 9 })
    const label = t('mode.reading.chapter_untitled', { ord: '5' })
    const strip = wrapper.find('[data-reading-progress]')
    expect(strip.find('b').text()).toBe(label)
    expect(strip.text().split(label).length - 1).toBe(1)
    expect(wrapper.find('.chapter-label').exists()).toBe(false)
    expect(wrapper.find('.chapter-title').text()).toBe(label)
  })
})

describe('the Chapter head', () => {
  it('puts the uppercase label and the origin pill above the title', async () => {
    wrapper = await mountReading({ ord: 47, title: 'Cánh cửa cuối', done: 50 })
    const head = wrapper.find('.chapter-head')
    expect(head.find('.chapter-label').text()).toBe('Chương 47')
    expect(head.find('[data-reading-chapter-origin]').text()).toBe(t('mode.reading.origin_self'))
    expect(wrapper.find('.chapter-title').text()).toBe('Cánh cửa cuối')
    const order = head.element.compareDocumentPosition(wrapper.find('.chapter-title').element)
    expect(order & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })
})

describe('the toolbar chords', () => {
  it('draws no key chip while no chord is assigned', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    expect(wrapper.find('.toolbar').findAll('.ui-kbd')).toHaveLength(0)
  })

  it('draws the assigned chords and never the Settings chord on the tuner', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    commands.installCommands({ isMac: true } as Parameters<typeof commands.installCommands>[0])
    const shortcuts = await import('../../src/config/shortcutsState')
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    const display = (id: string) => shortcuts.shortcutRows.value.find((r) => r.id === id)?.display.join(' / ')
    const bilingual = wrapper.find('[data-reading-toggle-bilingual] .ui-kbd')
    expect(display('reading.toggle_bilingual')).toBeTruthy()
    expect(bilingual.text()).toBe(display('reading.toggle_bilingual'))
    expect(wrapper.find('[data-reading-toc] .ui-kbd').text()).toBe(display('reading.toc'))
  })
})

describe('the toolbar', () => {
  it('draws Bilingual as a switch that follows the state', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    const bilingual = wrapper.find('[data-reading-toggle-bilingual]')
    expect(bilingual.attributes('role')).toBe('switch')
    expect(bilingual.attributes('aria-checked')).toBe('false')
    expect(bilingual.find('.ui-switch__track').exists()).toBe(true)
    const state = await import('../../src/modes/readingState')
    state.toggleBilingual()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-reading-toggle-bilingual]').attributes('aria-checked')).toBe('true')
    state.toggleBilingual()
  })

  it('keeps every command hook and puts the marks button last', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    for (const hook of [
      'data-reading-toggle-bilingual',
      'data-reading-level="lg"',
      'data-reading-level="md"',
      'data-reading-level="sm"',
      'data-reading-font-larger',
      'data-reading-line-looser',
      'data-reading-toggle-theme',
      'data-reading-toc',
      'data-reading-marks',
    ]) {
      expect(wrapper.find(`[${hook}]`).exists(), hook).toBe(true)
    }
    const buttons = wrapper.findAll('.toolbar button')
    expect(buttons.at(-1)?.attributes('data-reading-marks')).toBeDefined()
  })

  it('splits the marks count out of its unit once the total is known', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_reading_marks') {
        return Promise.resolve(
          [1, 2, 3].map((id) => ({
            segment_id: id,
            navigation_segment_id: id,
            chapter_id: 1,
            chapter_ord: 1,
            chapter_title: null,
            source_text: 'a',
            target_text: 'b',
            is_retired: false,
            marked_at: '2026-11-02T00:00:00.000Z',
          })),
        )
      }
      return Promise.resolve(null)
    })
    await state.openReadingMarks()
    await state.closeReadingMarks()
    await wrapper.vm.$nextTick()
    const button = wrapper.find('[data-reading-marks]')
    expect(button.find('b').text()).toBe('3')
    expect(button.text()).toContain(t('mode.reading.marks_button_unit'))
  })
})

describe('the type adjusters', () => {
  async function openTuner() {
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    const state = await import('../../src/modes/readingState')
    state.setReadingLevel('md')
    await wrapper.vm.$nextTick()
    return { mounted: wrapper, state }
  }

  it('sit in the toolbar with no tuner button and no panel below it', async () => {
    wrapper = await mountReading({ ord: 1, title: null, done: 1 })
    expect(wrapper.find('[data-reading-toggle-tuner]').exists()).toBe(false)
    expect(wrapper.find('.tuner').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('Tinh chỉnh')
    const hooks = wrapper.findAll('.toolbar button').map((b) => Object.keys(b.attributes()).find((k) => k.startsWith('data-reading-')))
    const level = hooks.lastIndexOf('data-reading-level')
    expect(hooks.indexOf('data-reading-font-smaller')).toBe(level + 1)
    expect(hooks.indexOf('data-reading-line-looser')).toBe(hooks.indexOf('data-reading-toggle-theme') - 1)
  })

  it('offers icon buttons with names instead of range sliders', async () => {
    const { mounted } = await openTuner()
    expect(mounted.find('input[type="range"]').exists()).toBe(false)
    for (const hook of [
      'data-reading-font-smaller',
      'data-reading-font-default',
      'data-reading-font-larger',
      'data-reading-line-tighter',
      'data-reading-line-default',
      'data-reading-line-looser',
    ]) {
      const button = mounted.find(`.toolbar button[${hook}]`)
      expect(button.exists(), hook).toBe(true)
      expect(button.find('svg').exists(), hook).toBe(true)
      expect(button.attributes('aria-label'), hook).toBeTruthy()
    }
  })

  it('moves the font size one pixel at a time and resets to the level value', async () => {
    const { state } = await openTuner()
    state.stepFontSize(1)
    expect(state.effectiveFontSize.value).toBe(18.5)
    state.stepFontSize(-1)
    state.stepFontSize(-1)
    expect(state.effectiveFontSize.value).toBe(16.5)
    state.resetFontSize()
    expect(state.effectiveFontSize.value).toBe(17.5)
  })

  it('disables the font buttons at both ends of the range', async () => {
    const { mounted, state } = await openTuner()
    state.setFontSize(state.READING_FONT_SIZE_MAX)
    await mounted.vm.$nextTick()
    expect(mounted.find('[data-reading-font-larger]').attributes('disabled')).toBeDefined()
    expect(mounted.find('[data-reading-font-smaller]').attributes('disabled')).toBeUndefined()
    state.setFontSize(state.READING_FONT_SIZE_MIN)
    await mounted.vm.$nextTick()
    expect(mounted.find('[data-reading-font-smaller]').attributes('disabled')).toBeDefined()
    expect(mounted.find('[data-reading-font-larger]').attributes('disabled')).toBeUndefined()
  })

  it('steps the line box by one 7px unit between the floor and the ceiling', async () => {
    const { mounted, state } = await openTuner()
    expect(state.readingStyle.value.lineHeight).toBe('35px')
    expect(mounted.find('[data-reading-line-tighter]').attributes('disabled')).toBeDefined()

    state.stepLineHeight(1)
    await mounted.vm.$nextTick()
    expect(state.readingStyle.value.lineHeight).toBe('42px')
    expect(mounted.find('[data-reading-line-looser]').attributes('disabled')).toBeDefined()
    expect(mounted.find('[data-reading-line-tighter]').attributes('disabled')).toBeUndefined()

    state.stepLineHeight(-1)
    expect(state.readingStyle.value.lineHeight).toBe('35px')
    state.stepLineHeight(1)
    state.resetLineHeight()
    expect(state.readingStyle.value.lineHeight).toBe('35px')
  })

  it('never takes the ratio below the floor when stepping down at a large size', async () => {
    const { state } = await openTuner()
    state.setFontSize(24)
    state.setLineHeight(state.READING_LINE_HEIGHT_MAX)
    while (state.readingLineCanTighten.value) state.stepLineHeight(-1)
    expect(state.effectiveLineHeight.value).toBeGreaterThanOrEqual(state.READING_LINE_HEIGHT_FLOOR)
    expect(state.readingStyle.value.lineHeight).toBe('42px')
  })
})
