/**
 * `library.add_chapters`: target resolution, the preset destination and its reset on a plain
 * open. `config/library` is the IPC edge; the Library, Work page and open-Work state are real.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const listWorksMock = vi.fn()

vi.mock('../../src/config/library', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/library')>()
  return { ...actual, listLibraryWorks: (...args: unknown[]) => listWorksMock(...args) }
})
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: async () => ({}) }))

function work(id: string, name: string) {
  return {
    work_id: id,
    atproj_path: `/tmp/${name}.atproj`,
    name,
    source_lang: 'zh',
    genre: 'xianxia',
    created_at: '2026-09-01T00:00:00.000Z',
    updated_at: '2026-09-01T00:00:00.000Z',
    chapter_count: 1,
    status: 'in_progress',
    status_is_override: false,
    chapter_done_count: 0,
  }
}

const KIEM = work('w-kiem', 'Kiếm Lai')
const TUYET = work('w-tuyet', 'Tuyết Trung')

function reply(works: unknown[]) {
  return { report: { total: works.length, matched: works.length, works, genres: [], source_langs: [] }, error: null }
}

async function load(listed: unknown[]) {
  vi.resetModules()
  listWorksMock.mockReset()
  listWorksMock.mockResolvedValue(reply(listed))
  const focus = await import('../../src/modes/libraryImportFocus')
  const imp = await import('../../src/modes/libraryImport')
  const overlay = await import('../../src/modes/libraryImportOverlayState')
  const mode = await import('../../src/modes/modeState')
  const works = await import('../../src/modes/libraryWorks')
  const identity = await import('../../src/openWorkIdentity')
  return { focus, imp, overlay, mode, works, identity }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('library.add_chapters', () => {
  it('Library grid cursor: the Work under the cursor is picked with its values and the overlay opens', async () => {
    const { focus, imp, overlay, mode, works } = await load([KIEM, TUYET])
    mode.setMode('library')
    await works.loadWorks()
    works.aimLibraryWork(1)
    await focus.openLibraryImportToAddChapters()
    expect(imp.destinationMode.value).toBe('existing')
    expect(imp.destinationWorkId.value).toBe('w-tuyet')
    expect(imp.pickedDestinationWork.value).not.toBeNull()
    expect(imp.effectiveSourceLang.value).toBe('zh')
    expect(imp.effectiveGenre.value).toBe('xianxia')
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })

  it('Work page open: its Work wins over the grid cursor', async () => {
    vi.resetModules()
    listWorksMock.mockReset()
    listWorksMock.mockResolvedValue(reply([KIEM, TUYET]))
    const { ref } = await import('vue')
    vi.doMock('../../src/modes/libraryChapters', () => ({ libraryWorkPageWork: ref(KIEM) }))
    try {
      const focus = await import('../../src/modes/libraryImportFocus')
      const imp = await import('../../src/modes/libraryImport')
      const mode = await import('../../src/modes/modeState')
      const works = await import('../../src/modes/libraryWorks')
      mode.setMode('library')
      await works.loadWorks()
      works.aimLibraryWork(1)
      await focus.openLibraryImportToAddChapters()
      expect(imp.destinationWorkId.value).toBe('w-kiem')
      expect(imp.effectiveName.value).toBe('Kiếm Lai')
    } finally {
      vi.doUnmock('../../src/modes/libraryChapters')
    }
  })

  it('outside Library: the open Work is the target and the mode moves to Library', async () => {
    const { focus, imp, overlay, mode, identity } = await load([KIEM, TUYET])
    mode.setMode('workspace')
    identity.setOpenWork('w-kiem', 'Kiếm Lai')
    await focus.openLibraryImportToAddChapters()
    expect(imp.destinationWorkId.value).toBe('w-kiem')
    expect(imp.effectiveName.value).toBe('Kiếm Lai')
    expect(mode.currentMode.value).toBe('library')
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })

  it('nothing to target: Tác phẩm đã có with an empty picker and the destination lock', async () => {
    const { focus, imp, overlay, mode } = await load([])
    mode.setMode('workspace')
    await focus.openLibraryImportToAddChapters()
    expect(focus.resolveAddChaptersTarget()).toBeNull()
    expect(imp.destinationMode.value).toBe('existing')
    expect(imp.destinationWorkId.value).toBeNull()
    expect(imp.destinationPending.value).toBe(true)
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })

  it('Library mode with an empty grid and no Work page: no target, picker empty, overlay opens', async () => {
    const { focus, imp, overlay, mode, works } = await load([])
    mode.setMode('library')
    await works.loadWorks()
    await focus.openLibraryImportToAddChapters()
    expect(focus.resolveAddChaptersTarget()).toBeNull()
    expect(imp.destinationMode.value).toBe('existing')
    expect(imp.destinationPending.value).toBe(true)
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })

  it('a target missing from a loaded list reloads it before the overlay opens', async () => {
    const { focus, imp, overlay, mode, identity } = await load([TUYET])
    await imp.loadDestinationWorks()
    expect(listWorksMock).toHaveBeenCalledTimes(1)
    listWorksMock.mockResolvedValue(reply([TUYET, KIEM]))
    mode.setMode('workspace')
    identity.setOpenWork('w-kiem', 'Kiếm Lai')
    const opening = focus.openLibraryImportToAddChapters()
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(false)
    await opening
    expect(listWorksMock).toHaveBeenCalledTimes(2)
    expect(imp.pickedDestinationWork.value?.name).toBe('Kiếm Lai')
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })

  it('a load error leaves the existing error alert state and still opens the overlay', async () => {
    const { focus, imp, overlay, mode, identity } = await load([KIEM])
    listWorksMock.mockResolvedValue({ report: null, error: { code: 'x', message_key: 'err.x', params: {}, retryable: true } })
    mode.setMode('workspace')
    identity.setOpenWork('w-kiem', 'Kiếm Lai')
    await focus.openLibraryImportToAddChapters()
    expect(imp.destinationWorksError.value).not.toBeNull()
    expect(overlay.libraryImportOverlayIsOpen.value).toBe(true)
  })

  it('a plain open after the preset resets to Tác phẩm mới', async () => {
    const { focus, imp, mode, identity } = await load([KIEM])
    mode.setMode('workspace')
    identity.setOpenWork('w-kiem', 'Kiếm Lai')
    await focus.openLibraryImportToAddChapters()
    focus.openLibraryImportForm()
    expect(imp.destinationMode.value).toBe('new')
    expect(imp.destinationWorkId.value).toBeNull()
  })

  it('a plain open through a tab invite also resets', async () => {
    const { focus, imp, mode, identity } = await load([KIEM])
    mode.setMode('workspace')
    identity.setOpenWork('w-kiem', 'Kiếm Lai')
    await focus.openLibraryImportToAddChapters()
    focus.openLibraryImportOnTab('paste')
    expect(imp.destinationMode.value).toBe('new')
  })

  it('a hand pick after the preset stays across the next plain open', async () => {
    const { focus, imp, mode, identity } = await load([KIEM, TUYET])
    mode.setMode('workspace')
    identity.setOpenWork('w-kiem', 'Kiếm Lai')
    await focus.openLibraryImportToAddChapters()
    imp.setDestinationWorkId('w-tuyet')
    focus.openLibraryImportForm()
    expect(imp.destinationMode.value).toBe('existing')
    expect(imp.destinationWorkId.value).toBe('w-tuyet')
  })

  it('a hand-picked destination is untouched by a plain open when no preset was set', async () => {
    const { focus, imp } = await load([KIEM])
    imp.setDestinationMode('existing')
    imp.setDestinationWorkId('w-kiem')
    focus.openLibraryImportForm()
    expect(imp.destinationWorkId.value).toBe('w-kiem')
  })
})
