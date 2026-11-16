import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

function chapterRow(id: number) {
  return {
    chapter_id: id,
    ord: id,
    title: null,
    status: 'done',
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

function runWith(chapterId: number | null) {
  return {
    chapters: chapterId === null ? [] : [
      {
        chapter_id: chapterId,
        chapter_ord: chapterId,
        chapter_title: null,
        blocks: [{ kind: 'paragraph', segments: [{ id: 71, source_text: 'a', target_text: 'b', is_confirmed: true, is_marked: false }] }],
        segment_count: 1,
        translation_origin_counts: { self_count: 1, other_count: 0, bilingual_import_count: 0 },
        images: [],
      },
    ],
    frontier: { kind: 'end-of-work', chapter: null },
    assets_dir: '',
  }
}

const counts = {
  chapter_count: 1,
  segment_count: 1,
  unconfirmed_count: 0,
  unconfirmed_translated_count: 0,
  untranslated_count: 0,
  image_count: 0,
  missing_link_images: [],
}

async function mountReadingWithExport(runChapterId: number | null) {
  vi.resetModules()
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'read_reading_run') return Promise.resolve(runWith(runChapterId))
    if (cmd === 'list_chapters') return Promise.resolve([chapterRow(3), chapterRow(7)])
    if (cmd === 'list_reading_marks') return Promise.resolve([])
    if (cmd === 'export_scope_summary') return Promise.resolve(counts)
    return Promise.resolve(null)
  })
  const commands = await import('../../src/commands')
  const { exportCommandDeps } = await import('../../src/exportCommandDeps')
  const { readingExportCommandDeps } = await import('../../src/readingExportCommandDeps')
  const deps: Partial<CommandDeps> = { ...exportCommandDeps(), ...readingExportCommandDeps() }
  commands.installCommands(deps as CommandDeps)
  const reading = await import('../../src/modes/readingState')
  const exportState = await import('../../src/exportState')
  const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
  const wrapper = mount(ReadingMode, { attachTo: document.body })
  await reading.ensureReadingLoaded()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
  return { wrapper, exportState, commands }
}

let wrapper: Awaited<ReturnType<typeof mountReadingWithExport>>['wrapper'] | null = null

beforeEach(() => {
  mockInvoke.mockReset()
  document.body.innerHTML = ''
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the Reading toolbar export button', () => {
  it('opens the export screen scoped to the Chapter being read, not the whole Work or the first Chapter', async () => {
    const mounted = await mountReadingWithExport(7)
    wrapper = mounted.wrapper
    const button = wrapper.get('[data-reading-export]')
    expect(button.attributes('disabled')).toBeUndefined()
    expect(button.attributes('data-export-open')).toBeDefined()

    await button.trigger('mousedown')
    expect(document.activeElement).toBe(button.element)
    await button.trigger('click')

    await vi.waitFor(() => expect(mounted.exportState.exportOverlayIsOpen.value).toBe(true))
    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('export_scope_summary', { scope: { kind: 'chapters', chapter_ids: [7] } })
    })
    expect(mounted.exportState.exportScopeKind.value).toBe('chapter')
    expect(mounted.exportState.exportSingleChapterId.value).toBe(7)
    expect(mockInvoke).not.toHaveBeenCalledWith('export_scope_summary', { scope: { kind: 'work' } })
  })

  it('leaves the Library opener on its own scope once the Reading export screen closes', async () => {
    const mounted = await mountReadingWithExport(7)
    wrapper = mounted.wrapper
    await wrapper.get('[data-reading-export]').trigger('click')
    await vi.waitFor(() => expect(mounted.exportState.exportOverlayIsOpen.value).toBe(true))
    mounted.commands.dispatch('export.close')
    expect(mounted.exportState.exportOverlayIsOpen.value).toBe(false)

    mockInvoke.mockClear()
    mounted.commands.dispatch('export.open')
    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('export_scope_summary', { scope: { kind: 'work' } })
    })
    expect(mounted.exportState.exportScopeKind.value).toBe('work')
  })

  it('is disabled and opens nothing while the run has no Chapter to read', async () => {
    const mounted = await mountReadingWithExport(null)
    wrapper = mounted.wrapper
    const button = wrapper.get('[data-reading-export]')
    expect(button.attributes('disabled')).toBeDefined()
    mounted.commands.dispatch('reading.export_chapter')
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(mounted.exportState.exportOverlayIsOpen.value).toBe(false)
    expect(mockInvoke.mock.calls.some(([cmd]) => cmd === 'export_scope_summary')).toBe(false)
  })
})
