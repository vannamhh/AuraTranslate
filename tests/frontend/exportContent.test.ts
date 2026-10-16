/**
 * Export screen, content options and preview (FR152): three switches default off and travel
 * as `options`, the mark is two-column only, numbering or mark announces the file cannot be
 * re-imported, the preview is re-asked on every change and a late reply is dropped, and the
 * attribution fields come from the first Chapter of the scope.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const listMock = vi.fn()
const summaryMock = vi.fn()
const folderMock = vi.fn()
const twoColumnMock = vi.fn()
const textMock = vi.fn()
const previewMock = vi.fn()
const nameMock = vi.fn()

vi.mock('../../src/config/chapter', () => ({ listChapters: (...a: unknown[]) => listMock(...a) }))
vi.mock('../../src/config/attribution', () => ({ translatorNameGet: (...a: unknown[]) => nameMock(...a) }))
vi.mock('../../src/config/export', () => ({
  exportScopeSummary: (...a: unknown[]) => summaryMock(...a),
  exportChooseFolder: (...a: unknown[]) => folderMock(...a),
  exportDocxTwoColumn: (...a: unknown[]) => twoColumnMock(...a),
  exportDocxOneBlock: () => Promise.resolve({ file: null, error: null }),
  exportText: (...a: unknown[]) => textMock(...a),
  exportPreview: (...a: unknown[]) => previewMock(...a),
}))

const written = {
  path: '/tmp/out/x.docx',
  chapter_count: 1,
  segment_count: 1,
  image_count: 0,
  images_skipped_missing_link: 0,
  images_dir: null,
}

function chapter(over: Record<string, unknown> = {}) {
  return {
    chapter_id: 1,
    ord: 1,
    title: 'Mở đầu',
    status: 'in_progress',
    segment_count: 5,
    translated_count: 5,
    unconfirmed_count: 2,
    in_translation_count: 5,
    unlinked_count: 0,
    updated_at: '2026-10-01T00:00:00Z',
    origin_author: 'Tác giả A',
    origin_site_name: null,
    origin_url: 'https://example.test/1',
    origin_published_at: null,
    ...over,
  }
}

function preview(over: Record<string, unknown> = {}) {
  return {
    chapter_id: 1,
    chapter_ord: 1,
    chapter_title: 'Mở đầu',
    file_name: 'x-hai-cot.docx',
    attribution: [],
    appendix: [],
    appendix_term_count: 0,
    not_reimportable: false,
    rows: [{ source: 'gốc', target: 'dịch' }],
    ...over,
  }
}

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, twoColumnMock, textMock, previewMock, nameMock]) m.mockReset()
  listMock.mockResolvedValue({ chapters: [chapter()], error: null })
  summaryMock.mockResolvedValue({
    counts: {
      chapter_count: 1,
      segment_count: 5,
      unconfirmed_count: 2,
      unconfirmed_translated_count: 0,
      untranslated_count: 0,
      image_count: 0,
      missing_link_images: [],
    },
    error: null,
  })
  folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/out' })
  twoColumnMock.mockResolvedValue({ file: written, error: null })
  textMock.mockResolvedValue({ file: written, error: null })
  previewMock.mockResolvedValue({ preview: preview(), error: null })
  nameMock.mockResolvedValue({ name: 'Ice', error: null })
  const state = await import('../../src/exportState')
  const Overlay = (await import('../../src/ExportOverlay.vue')).default
  return { state, Overlay }
}

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  for (let i = 0; i < 6; i += 1) await wrapper.vm.$nextTick()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('content options', () => {
  it('default off and sent as one options object on the two-column export', async () => {
    const { state } = await fresh()
    previewMock.mockResolvedValue({ preview: preview({ appendix_term_count: 3 }), error: null })
    await state.openExport()
    await vi.waitFor(() => expect(state.exportPreviewData.value).not.toBeNull())
    await state.chooseExportFolder()
    await state.runExport()
    expect(twoColumnMock.mock.calls[0].at(-1)).toEqual({
      number_segments: false,
      glossary_appendix: false,
      mark_unconfirmed: false,
    })
    state.setExportNumberSegments(true)
    state.setExportGlossaryAppendix(true)
    state.setExportMarkUnconfirmed(true)
    await state.runExport()
    expect(twoColumnMock.mock.calls[1].at(-1)).toEqual({
      number_segments: true,
      glossary_appendix: true,
      mark_unconfirmed: true,
    })
  })

  it('never sends the mark outside the two-column docx, and the switch is disabled with its reason', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportMarkUnconfirmed(true)
    state.setExportFormat('markdown')
    await settle(wrapper)
    expect((wrapper.get('[data-export-mark-toggle]').element as HTMLInputElement).disabled).toBe(true)
    expect(wrapper.find('[data-export-mark-disabled]').exists()).toBe(true)
    await state.chooseExportFolder()
    await state.runExport()
    expect(textMock.mock.calls[0].at(-1)).toMatchObject({ mark_unconfirmed: false })
    expect(previewMock.mock.calls.at(-1)?.[2]).toMatchObject({ mark_unconfirmed: false })
    wrapper.unmount()
  })

  it('shows the not-reimportable notice for numbering or the mark, not for the appendix', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    expect(wrapper.find('[data-export-content] [data-export-not-reimportable]').exists()).toBe(false)
    state.setExportGlossaryAppendix(true)
    await settle(wrapper)
    expect(wrapper.find('[data-export-content] [data-export-not-reimportable]').exists()).toBe(false)
    state.setExportNumberSegments(true)
    await settle(wrapper)
    expect(wrapper.find('[data-export-content] [data-export-not-reimportable]').exists()).toBe(true)
    state.setExportNumberSegments(false)
    state.setExportMarkUnconfirmed(true)
    await settle(wrapper)
    expect(wrapper.find('[data-export-content] [data-export-not-reimportable]').exists()).toBe(true)
    state.setExportMarkUnconfirmed(false)
    await settle(wrapper)
    expect(wrapper.find('[data-export-content] [data-export-not-reimportable]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the unconfirmed count; an empty Glossary disables the appendix switch with its note and is never sent', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportGlossaryAppendix(true)
    await settle(wrapper)
    expect(wrapper.get('[data-export-mark-count]').text()).toContain('2 câu')
    expect(wrapper.find('[data-export-appendix-empty]').exists()).toBe(true)
    const box = wrapper.get('[data-export-appendix-toggle]').element as HTMLInputElement
    expect(box.disabled).toBe(true)
    expect(box.checked).toBe(false)
    expect(state.currentExportOptions().glossary_appendix).toBe(false)
    wrapper.unmount()
  })

  it('shows the confirmed term count next to the appendix switch and keeps it usable', async () => {
    const { state, Overlay } = await fresh()
    previewMock.mockResolvedValue({ preview: preview({ appendix_term_count: 84 }), error: null })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    expect(wrapper.get('[data-export-appendix-count]').text()).toContain('84 thuật ngữ')
    expect((wrapper.get('[data-export-appendix-toggle]').element as HTMLInputElement).disabled).toBe(false)
    expect(wrapper.find('[data-export-appendix-empty]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('clears the old preview on a format change but keeps it on an option toggle', async () => {
    const { state } = await fresh()
    await state.openExport()
    await vi.waitFor(() => expect(state.exportPreviewData.value).not.toBeNull())
    previewMock.mockImplementation(() => new Promise(() => undefined))
    state.setExportNumberSegments(true)
    expect(state.exportPreviewData.value).not.toBeNull()
    state.setExportFormat('markdown')
    expect(state.exportPreviewData.value).toBeNull()
  })

  it('shows 0 câu when every sentence is confirmed', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue({
      counts: {
        chapter_count: 1,
        segment_count: 5,
        unconfirmed_count: 0,
        unconfirmed_translated_count: 0,
        untranslated_count: 0,
        image_count: 0,
        missing_link_images: [],
      },
      error: null,
    })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    expect(wrapper.get('[data-export-mark-count]').text()).toContain('0 câu')
    wrapper.unmount()
  })

  it('resetExport turns all three off', async () => {
    const { state } = await fresh()
    state.setExportNumberSegments(true)
    state.setExportGlossaryAppendix(true)
    state.setExportMarkUnconfirmed(true)
    state.resetExport()
    expect(state.currentExportOptions()).toEqual({
      number_segments: false,
      glossary_appendix: false,
      mark_unconfirmed: false,
    })
  })
})

describe('preview', () => {
  it('is asked again with the format, the options and the attribution flag', async () => {
    const { state } = await fresh()
    await state.openExport()
    await vi.waitFor(() => expect(previewMock).toHaveBeenCalled())
    state.setExportFormat('plain_text')
    state.setExportNumberSegments(true)
    state.setExportAttribution(true)
    const [scope, format, options, withAttribution] = previewMock.mock.calls.at(-1) as unknown[]
    expect(scope).toEqual({ kind: 'work' })
    expect(format).toBe('plain')
    expect(options).toMatchObject({ number_segments: true })
    expect(withAttribution).toBe(true)
  })

  it('drops a reply that arrives after a newer request', async () => {
    const { state } = await fresh()
    let releaseOld: (value: unknown) => void = () => undefined
    previewMock.mockReset()
    previewMock.mockImplementationOnce(() => new Promise((resolve) => (releaseOld = resolve)))
    previewMock.mockResolvedValueOnce({ preview: preview({ file_name: 'new.docx' }), error: null })
    const old = state.refreshExportPreview()
    await state.refreshExportPreview()
    releaseOld({ preview: preview({ file_name: 'old.docx' }), error: null })
    await old
    expect(state.exportPreviewData.value?.file_name).toBe('new.docx')
  })

  it('renders per format, names the file, and labels the run button with the extension', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    expect(wrapper.find('[data-export-word-preview]').exists()).toBe(true)
    expect(wrapper.get('[data-export-file-name]').text()).toContain('x-hai-cot.docx')
    expect(wrapper.get('[data-export-run]').text()).toBe('Xuất 1 Chương ra .docx')
    previewMock.mockResolvedValue({ preview: preview({ file_name: 'x.md', rows: [{ source: null, target: 'một' }, { source: null, target: 'hai' }] }), error: null })
    state.setExportFormat('markdown')
    await settle(wrapper)
    expect(wrapper.find('[data-export-word-preview]').exists()).toBe(false)
    expect(wrapper.get('[data-export-text-preview]').text()).toBe('một\n\nhai')
    expect(wrapper.get('[data-export-run]').text()).toBe('Xuất 1 Chương ra .md')
    wrapper.unmount()
  })

  it('shows a preview error inside the frame without disabling the run button', async () => {
    const { state, Overlay } = await fresh()
    previewMock.mockResolvedValue({
      preview: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    await settle(wrapper)
    expect(wrapper.find('[data-export-preview-error]').exists()).toBe(true)
    expect((wrapper.get('[data-export-run]').element as HTMLButtonElement).disabled).toBe(false)
    wrapper.unmount()
  })
})

describe('attribution fields', () => {
  it('lists the four fields of the first Chapter, empty ones as "không có"', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportAttribution(true)
    await settle(wrapper)
    expect(wrapper.get('[data-export-attribution-field="author"]').text()).toBe('Tác giả A')
    expect(wrapper.get('[data-export-attribution-field="site"]').text()).toBe('không có')
    expect(wrapper.get('[data-export-attribution-field="url"]').text()).toBe('https://example.test/1')
    expect(wrapper.get('[data-export-attribution-field="published"]').text()).toBe('không có')
    expect(wrapper.find('[data-export-translator]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('hides the fields while the switch is off', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)
    expect(wrapper.find('[data-export-attribution-fields]').exists()).toBe(false)
    wrapper.unmount()
  })
})
