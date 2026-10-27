import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import type { CreatedWork } from '../../src/config/project'

const OPENED = { meta: { work_id: 'w-mo', genre: '', name: 'Phàm Nhân Tu Tiên' }, folder: '/tmp/w-mo.atproj', chapter_id: 1 }
const CREATED: CreatedWork = {
  meta: {
    meta_schema_version: 1,
    work_id: 'w-tao',
    name: 'Tác phẩm vừa nhập',
    source_lang: 'zh',
    genre: '',
    created_at: '2026-10-27T00:00:00Z',
    updated_at: '2026-10-27T00:00:00Z',
    chapter_count: 1,
  },
  folder: '/tmp/w-tao.atproj',
  images_saved: 0,
  images_failed: 0,
  source_lang_mismatch: false,
}

vi.mock('../../src/config/library', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/library')>()
  return { ...actual, openWork: async () => ({ opened: OPENED, error: null }) }
})
vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, listChapters: async () => ({ chapters: [], error: null }) }
})
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => 'clean' as const, flushChapterPositionNow: async () => {} }
})
vi.mock('../../src/promptSetState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/promptSetState')>()
  return { ...actual, restorePromptSetForWork: async () => {} }
})
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

beforeEach(() => {
  vi.resetModules()
})

describe('title bar Work name', () => {
  it('is absent until a Work is opened in this session', async () => {
    const { setMode } = await import('../../src/modes/modeState')
    const { titlebarWorkName } = await import('../../src/titlebarWorkName')
    setMode('workspace')
    expect(titlebarWorkName.value).toBeNull()
  })

  it('shows the name of the Work opened from Library in Workspace and Reading, not in Library', async () => {
    const { setMode } = await import('../../src/modes/modeState')
    const chapters = await import('../../src/modes/libraryChapters')
    const { titlebarWorkName } = await import('../../src/titlebarWorkName')

    await chapters.openWorkById('w-mo')
    expect(chapters.libraryOpenWorkError.value).toBeNull()

    setMode('workspace')
    expect(titlebarWorkName.value).toBe('Phàm Nhân Tu Tiên')
    setMode('reading')
    expect(titlebarWorkName.value).toBe('Phàm Nhân Tu Tiên')
    setMode('library')
    expect(titlebarWorkName.value).toBeNull()
  })

  it('shows the name of a Work just created by an import', async () => {
    const { setMode } = await import('../../src/modes/modeState')
    const importing = await import('../../src/modes/libraryImport')
    const { titlebarWorkName } = await import('../../src/titlebarWorkName')

    importing.finishImportSubmission(CREATED, null)
    setMode('workspace')
    expect(titlebarWorkName.value).toBe('Tác phẩm vừa nhập')
  })

  it('a blank Work name shows nothing rather than a lone divider', async () => {
    const { setMode } = await import('../../src/modes/modeState')
    const identity = await import('../../src/openWorkIdentity')
    const { titlebarWorkName } = await import('../../src/titlebarWorkName')

    identity.setOpenWork('w-trong', '  ')
    setMode('workspace')
    expect(titlebarWorkName.value).toBeNull()
  })

  it('a failed import keeps the name of the Work already open', async () => {
    const { setMode } = await import('../../src/modes/modeState')
    const chapters = await import('../../src/modes/libraryChapters')
    const importing = await import('../../src/modes/libraryImport')
    const { titlebarWorkName } = await import('../../src/titlebarWorkName')

    await chapters.openWorkById('w-mo')
    importing.finishImportSubmission(null, { code: 'x', message_key: 'x', params: {}, retryable: false })
    setMode('workspace')
    expect(titlebarWorkName.value).toBe('Phàm Nhân Tu Tiên')
  })

  it('App.vue renders it in the title bar, between the mode tabs and the right-hand cluster', () => {
    const app = readFileSync(resolve(__dirname, '../../src/App.vue'), 'utf8')
    const header = app.slice(app.indexOf('<header class="titlebar"'), app.indexOf('</header>'))
    const name = header.indexOf('{{ titlebarWorkName }}')
    expect(name).toBeGreaterThan(header.indexOf('</nav>'))
    expect(name).toBeLessThan(header.indexOf('<div class="titlebar-right">'))
  })

  it('the name carries the same transparent underline as a mode tab, so both texts share one centre line', () => {
    const app = readFileSync(resolve(__dirname, '../../src/App.vue'), 'utf8')
    const rule = (selector: string) => new RegExp(`\\n${selector} \\{([^}]*)\\}`).exec(app)?.[1] ?? ''
    const underline = /border-bottom: \d+px solid transparent;/.exec(rule('\\.mode-tab'))?.[0]
    expect(underline).toBeDefined()
    expect(rule('\\.titlebar-work')).toContain(underline)
  })
})
