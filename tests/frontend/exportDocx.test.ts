/**
 * Export screen, `.docx` two-column format (FR87): the format choice that says it can be
 * re-imported, the run button, and what the run reports. `config/export.ts` and
 * `config/chapter.ts` are the IPC boundary and are mocked; the command registry, the state and
 * the overlay are real.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { ChapterRow } from '../../src/config/chapter'

const listMock = vi.fn()
const summaryMock = vi.fn()
const folderMock = vi.fn()
const docxMock = vi.fn()

vi.mock('../../src/config/chapter', () => ({
  listChapters: (...args: unknown[]) => listMock(...args),
}))
vi.mock('../../src/config/export', () => ({
  exportScopeSummary: (...args: unknown[]) => summaryMock(...args),
  exportChooseFolder: (...args: unknown[]) => folderMock(...args),
  exportDocxTwoColumn: (...args: unknown[]) => docxMock(...args),
}))

function row(id: number, ord: number): ChapterRow {
  return {
    chapter_id: id,
    ord,
    title: null,
    status: 'not_started',
    segment_count: 3,
    origin_author: null,
    origin_site_name: null,
    origin_url: null,
    origin_published_at: null,
  }
}

function ipcError(code: string) {
  return { code, message_key: 'err.export.write_failed', params: {}, retryable: true }
}

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, docxMock]) m.mockReset()
  listMock.mockResolvedValue({ chapters: [row(10, 1), row(11, 2)], error: null })
  summaryMock.mockResolvedValue({
    counts: { chapter_count: 2, segment_count: 6, unconfirmed_count: 0, unconfirmed_translated_count: 0, untranslated_count: 0, image_count: 0, missing_link_images: [] },
    error: null,
  })
  folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/out' })
  const state = await import('../../src/exportState')
  const commands = await import('../../src/commands')
  const { exportCommandDeps } = await import('../../src/exportCommandDeps')
  const deps: Partial<CommandDeps> = exportCommandDeps()
  commands.installCommands(deps as CommandDeps)
  const Overlay = (await import('../../src/ExportOverlay.vue')).default
  return { state, commands, Overlay }
}

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  for (let i = 0; i < 4; i += 1) await wrapper.vm.$nextTick()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('format choice', () => {
  it('names the two-column .docx and says out loud that it can be imported back', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(wrapper.get('.ex-format').text()).toContain('Bảng hai cột (.docx)')
    expect(wrapper.get('[data-export-reimportable]').text()).toContain('nhập lại được')
    wrapper.unmount()
  })
})

describe('running the export', () => {
  it('sends the current scope and the chosen folder to Rust and shows the written path', async () => {
    const { state, commands, Overlay } = await fresh()
    docxMock.mockResolvedValue({ file: { path: '/tmp/out/Tac Pham-hai-cot.docx', chapter_count: 2, segment_count: 6, image_count: 0, images_skipped_missing_link: 0, images_dir: null }, error: null })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    await settle(wrapper)

    commands.dispatch('export.run')
    await vi.waitFor(() => expect(state.exportRunStatus.value).toBe('done'))
    await settle(wrapper)

    expect(docxMock).toHaveBeenCalledWith({ kind: 'work' }, 'file', '/tmp/out')
    expect(wrapper.get('[data-export-result]').text()).toBe('/tmp/out/Tac Pham-hai-cot.docx')
    wrapper.unmount()
  })

  it('cannot run without a folder, and the button says why', async () => {
    const { state, commands, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(wrapper.get('[data-export-run]').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Chọn thư mục đích để xuất.')
    commands.dispatch('export.run')
    await settle(wrapper)
    expect(docxMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('cannot run while nothing is selected', async () => {
    const { state } = await fresh()
    await state.openExport()
    await state.chooseExportFolder()
    await state.setExportScopeKind('chapters')

    await state.runExport()
    expect(docxMock).not.toHaveBeenCalled()
    expect(state.exportRunStatus.value).toBe('idle')
  })

  it('shows a Rust failure as an error and leaves no result', async () => {
    const { state, Overlay } = await fresh()
    docxMock.mockResolvedValue({ file: null, error: ipcError('export.write_failed') })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    await state.runExport()
    await settle(wrapper)

    expect(state.exportRunStatus.value).toBe('error')
    expect(state.exportRunResult.value).toBeNull()
    expect(wrapper.find('[data-export-result]').exists()).toBe(false)
    expect(wrapper.text()).toContain('Không ghi được tệp')
    wrapper.unmount()
  })

  it('tells an absent bridge apart from an error', async () => {
    const { state } = await fresh()
    docxMock.mockResolvedValue({ file: null, error: null })
    await state.openExport()
    await state.chooseExportFolder()
    await state.runExport()
    expect(state.exportRunStatus.value).toBe('ipc_unavailable')
  })

  it('ignores a second press while the file is being written', async () => {
    const { state } = await fresh()
    let release: (v: unknown) => void = () => undefined
    docxMock.mockImplementationOnce(() => new Promise((r) => (release = r)))
    await state.openExport()
    await state.chooseExportFolder()
    const first = state.runExport()
    void state.runExport()
    expect(docxMock).toHaveBeenCalledTimes(1)
    release({ file: { path: '/tmp/out/a.docx', chapter_count: 2, segment_count: 6, image_count: 0, images_skipped_missing_link: 0, images_dir: null }, error: null })
    await first
    expect(state.exportRunStatus.value).toBe('done')
  })

  it('forgets an old result when the scope or the folder changes', async () => {
    const { state } = await fresh()
    docxMock.mockResolvedValue({ file: { path: '/tmp/out/a.docx', chapter_count: 2, segment_count: 6, image_count: 0, images_skipped_missing_link: 0, images_dir: null }, error: null })
    await state.openExport()
    await state.chooseExportFolder()
    await state.runExport()
    expect(state.exportRunStatus.value).toBe('done')

    await state.setExportScopeKind('chapter')
    expect(state.exportRunStatus.value).toBe('idle')
    expect(state.exportRunResult.value).toBeNull()
  })
})
