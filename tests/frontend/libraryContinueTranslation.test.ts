import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const flushResult: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }

vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => flushResult.value }
})

const WORK_ROW = {
  work_id: 'id-a',
  atproj_path: '/tmp/A.atproj',
  name: 'A',
  source_lang: 'zh',
  genre: '',
  created_at: '2026-08-01T00:00:00.000Z',
  updated_at: '2026-08-01T00:00:00.000Z',
  chapter_count: 1,
  status: 'in_progress',
  status_is_override: false,
  chapter_done_count: 0,
}

const NONE_OPEN_ERROR = {
  code: 'work.none_open',
  message_key: 'err.work.none_open',
  params: {},
  retryable: false,
}

async function prepare(openWorkResult: 'ok' | 'error') {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: 1, matched: 1, works: [WORK_ROW], genres: [], source_langs: [] })
    }
    if (cmd === 'open_work') {
      if (openWorkResult === 'error') return Promise.reject(NONE_OPEN_ERROR)
      return Promise.resolve({ meta: { name: 'A' }, folder: '/tmp/A', chapter_id: 1 })
    }
    if (cmd === 'list_chapters') return Promise.resolve([])
    return Promise.reject(NONE_OPEN_ERROR)
  })
  const works = await import('../../src/modes/libraryWorks')
  await works.loadWorks()
  const chapters = await import('../../src/modes/libraryChapters')
  const modes = await import('../../src/modes/modeState')
  return { chapters, modes }
}

describe('modes/libraryChapters.ts::continueCurrentLibraryTranslation', () => {
  beforeEach(async () => {
    mockInvoke.mockReset()
    flushResult.value = 'clean'
    const works = await import('../../src/modes/libraryWorks')
    works.resetLibraryWorks()
    const chapters = await import('../../src/modes/libraryChapters')
    chapters.resetLibraryChapters()
    const modes = await import('../../src/modes/modeState')
    modes.setMode('library')
  })

  it('a successful open moves to Workspace and returns true', async () => {
    const { chapters, modes } = await prepare('ok')
    expect(await chapters.continueCurrentLibraryTranslation()).toBe(true)
    expect(modes.currentMode.value).toBe('workspace')
  })

  it('an IPC error from open_work stays in Library and returns false', async () => {
    const { chapters, modes } = await prepare('error')
    expect(await chapters.continueCurrentLibraryTranslation()).toBe(false)
    expect(modes.currentMode.value).toBe('library')
  })

  it('a blocked flush never calls open_work, stays in Library and returns false', async () => {
    flushResult.value = 'still-dirty'
    const { chapters, modes } = await prepare('ok')
    expect(await chapters.continueCurrentLibraryTranslation()).toBe(false)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_work')).toBe(false)
    expect(modes.currentMode.value).toBe('library')
  })
})

describe('continueCurrentLibraryTranslation — guards and registry', () => {
  beforeEach(async () => {
    mockInvoke.mockReset()
    flushResult.value = 'clean'
    const works = await import('../../src/modes/libraryWorks')
    works.resetLibraryWorks()
    const chapters = await import('../../src/modes/libraryChapters')
    chapters.resetLibraryChapters()
    const modes = await import('../../src/modes/modeState')
    modes.setMode('library')
  })

  it('outside Library it returns false without calling open_work', async () => {
    const { chapters, modes } = await prepare('ok')
    modes.setMode('workspace')
    expect(await chapters.continueCurrentLibraryTranslation()).toBe(false)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_work')).toBe(false)
    expect(modes.currentMode.value).toBe('workspace')
  })

  it('with the Work page open it returns false without calling open_work', async () => {
    const { chapters, modes } = await prepare('ok')
    await chapters.openCurrentLibraryWork()
    expect(chapters.libraryWorkPageWork.value).not.toBeNull()
    mockInvoke.mockClear()
    expect(await chapters.continueCurrentLibraryTranslation()).toBe(false)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_work')).toBe(false)
    expect(modes.currentMode.value).toBe('library')
  })

  it('dispatch of library.continue_translation reaches the installed port', async () => {
    const { dispatch, installCommands } = await import('../../src/commands')
    const port = vi.fn()
    installCommands({ continueCurrentLibraryTranslation: port } as never)
    dispatch('library.continue_translation')
    expect(port).toHaveBeenCalledTimes(1)
  })
})
