/**
 * Export screen, image mode (FR130): the two radios, the disabled link option, the list of
 * images without a source URL, and the mode sent with the run. `config/export.ts` and
 * `config/chapter.ts` are mocked; state and overlay are real.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { ChapterRow } from '../../src/config/chapter'
import type { ExportScopeCounts } from '../../src/config/export'

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

function counts(over: Partial<ExportScopeCounts> = {}): { counts: ExportScopeCounts; error: null } {
  return {
    counts: { chapter_count: 2, segment_count: 6, unconfirmed_count: 0, unconfirmed_translated_count: 0, untranslated_count: 0, image_count: 3, missing_link_images: [], ...over },
    error: null,
  }
}

const missing = (chapter_id: number, image_index: number, alt_text: string | null) => ({
  chapter_id,
  chapter_ord: chapter_id - 9,
  chapter_title: null,
  image_index,
  alt_text,
})

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, docxMock]) m.mockReset()
  listMock.mockResolvedValue({ chapters: [row(10, 1), row(11, 2)], error: null })
  summaryMock.mockResolvedValue(counts())
  folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/out' })
  docxMock.mockResolvedValue({
    file: { path: '/tmp/out/a.docx', chapter_count: 2, segment_count: 6, image_count: 2, images_skipped_missing_link: 1, images_dir: null },
    error: null,
  })
  const state = await import('../../src/exportState')
  const commands = await import('../../src/commands')
  const { exportCommandDeps } = await import('../../src/exportCommandDeps')
  commands.installCommands(exportCommandDeps() as unknown as CommandDeps)
  const Overlay = (await import('../../src/ExportOverlay.vue')).default
  return { state, commands, Overlay }
}

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  for (let i = 0; i < 4; i += 1) await wrapper.vm.$nextTick()
}

function radio(wrapper: ReturnType<typeof mount>, value: 'file' | 'link') {
  return wrapper.get(`input[name="export-image-mode"][value="${value}"]`)
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('image mode choice', () => {
  it('offers two native radios and defaults to the image file', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(radio(wrapper, 'file').attributes('type')).toBe('radio')
    expect(radio(wrapper, 'link').attributes('type')).toBe('radio')
    expect((radio(wrapper, 'file').element as HTMLInputElement).checked).toBe(true)
    expect((radio(wrapper, 'link').element as HTMLInputElement).checked).toBe(false)
    expect(wrapper.get('[data-export-image-count]').text()).toBe('3 ảnh trong phạm vi')
    wrapper.unmount()
  })

  it('is reachable by keyboard: radios of one group are focusable and not disabled', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    for (const v of ['file', 'link'] as const) {
      expect(radio(wrapper, v).attributes('disabled')).toBeUndefined()
      expect(radio(wrapper, v).attributes('tabindex')).toBeUndefined()
    }
    wrapper.unmount()
  })

  it('keeps the chosen mode across closing and reopening, and sends it with the run', async () => {
    const { state, commands, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    await radio(wrapper, 'link').setValue(true)
    expect(state.exportImageMode.value).toBe('link')

    state.closeExport()
    await state.openExport()
    await settle(wrapper)
    expect(state.exportImageMode.value).toBe('link')
    expect((radio(wrapper, 'link').element as HTMLInputElement).checked).toBe(true)

    await state.chooseExportFolder()
    commands.dispatch('export.run')
    await vi.waitFor(() => expect(state.exportRunStatus.value).toBe('done'))
    expect(docxMock).toHaveBeenCalledWith({ kind: 'work' }, 'link', '/tmp/out', false)
    wrapper.unmount()
  })

  it('sends file by default', async () => {
    const { state } = await fresh()
    await state.openExport()
    await state.chooseExportFolder()
    await state.runExport()
    expect(docxMock).toHaveBeenCalledWith({ kind: 'work' }, 'file', '/tmp/out', false)
  })
})

describe('images without a source URL', () => {
  it('lists them under the link option as soon as it is chosen, not before', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ missing_link_images: [missing(10, 2, 'Bản đồ'), missing(11, 1, null)] }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    expect(wrapper.find('[data-export-missing-links]').exists()).toBe(false)

    await radio(wrapper, 'link').setValue(true)
    const items = wrapper.get('[data-export-missing-links]').findAll('li').map((li) => li.text())
    expect(items).toEqual(['Chương 1, ảnh thứ 2 (Bản đồ)', 'Chương 2, ảnh thứ 1'])
    expect(wrapper.get('[data-export-missing-heading]').text()).toContain('2 ảnh thiếu link gốc')
    wrapper.unmount()
  })

  it('shows no list when every image has a link', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    await radio(wrapper, 'link').setValue(true)
    expect(wrapper.find('[data-export-missing-links]').exists()).toBe(false)
    expect(state.exportImageMode.value).toBe('link')
    wrapper.unmount()
  })

  it('disables the link option with a reason when no image carries a link, and falls back to file', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    await radio(wrapper, 'link').setValue(true)
    expect(state.exportImageMode.value).toBe('link')

    summaryMock.mockResolvedValue(counts({ image_count: 2, missing_link_images: [missing(10, 1, null), missing(10, 2, null)] }))
    await state.setExportScopeKind('chapter')
    await settle(wrapper)

    expect(radio(wrapper, 'link').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-export-link-disabled]').text()).toContain('không ảnh nào trong phạm vi có link gốc')
    expect(state.exportImageMode.value).toBe('file')
    expect((radio(wrapper, 'file').element as HTMLInputElement).checked).toBe(true)
    wrapper.unmount()
  })

  it('disables the link option when the scope has no image at all, and says 0', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ image_count: 0 }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(wrapper.get('[data-export-image-count]').text()).toBe('0 ảnh trong phạm vi')
    expect(radio(wrapper, 'link').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('refuses to select link while it is unusable', async () => {
    const { state } = await fresh()
    summaryMock.mockResolvedValue(counts({ image_count: 0 }))
    await state.openExport()
    state.setExportImageMode('link')
    expect(state.exportImageMode.value).toBe('file')
  })

  it('replaces the list with the new scope, ignoring a slower earlier answer', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    summaryMock.mockResolvedValueOnce(counts({ missing_link_images: [missing(10, 1, null)] }))
    await state.openExport()
    await radio(wrapper, 'link').setValue(true)

    let releaseSlow: (v: unknown) => void = () => undefined
    summaryMock.mockImplementationOnce(() => new Promise((r) => (releaseSlow = r)))
    const slow = state.setExportScopeKind('chapter')
    summaryMock.mockResolvedValueOnce(counts({ missing_link_images: [missing(11, 4, 'mới')] }))
    await state.setExportScopeKind('work')
    releaseSlow(counts({ missing_link_images: [missing(10, 9, 'cũ')] }))
    await slow
    await settle(wrapper)

    const items = wrapper.get('[data-export-missing-links]').findAll('li').map((li) => li.text())
    expect(items).toEqual(['Chương 2, ảnh thứ 4 (mới)'])
    wrapper.unmount()
  })
})

describe('result of a run', () => {
  it('reports images written and images skipped for a missing link', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    await state.runExport()
    await settle(wrapper)

    const text = wrapper.get('[data-export-image-result]').text()
    expect(text).toContain('Đã xuất 2 ảnh.')
    expect(text).toContain('Bỏ 1 ảnh vì thiếu link gốc.')
    wrapper.unmount()
  })

  it('shows the image folder in file mode', async () => {
    const { state, Overlay } = await fresh()
    docxMock.mockResolvedValue({
      file: { path: '/tmp/out/a.docx', chapter_count: 2, segment_count: 6, image_count: 3, images_skipped_missing_link: 0, images_dir: '/tmp/out/a-anh' },
      error: null,
    })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    await state.runExport()
    await settle(wrapper)

    expect(wrapper.get('[data-export-images-dir]').text()).toBe('/tmp/out/a-anh')
    expect(wrapper.get('[data-export-image-result]').text()).not.toContain('Bỏ')
    wrapper.unmount()
  })
})
