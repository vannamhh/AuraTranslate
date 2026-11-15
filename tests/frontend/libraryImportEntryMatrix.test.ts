/**
 * The import entry's I/O matrix (Story 6.25): one common block, three tabs, one primary button.
 * Drives the real overlay and the real `libraryImport.ts`; `config/library` and `config/project`
 * are the IPC edge, so the Rust verdict is a hand-written double (`support/importClassify.ts`);
 * nothing compares it to `classify_import_paths`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import { fakeClassification } from './support/importClassify'
import { pickSelectOption } from './support/selectMenu'
import { seedGenres } from './support/genreSeed'

const previewFileMock = vi.fn()
const previewTextMock = vi.fn()
const previewBilingualMock = vi.fn()
const pickFilesMock = vi.fn()
const mockInvoke = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewBilingualImportFromFile: (...args: unknown[]) => previewBilingualMock(...args),
  }
})

vi.mock('../../src/config/library', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/library')>()
  return {
    ...actual,
    pickImportFiles: (...args: unknown[]) => pickFilesMock(...args),
    classifyImportPaths: async (paths: string[], docx: string[]) => ({
      classification: fakeClassification(paths, docx),
      error: null,
    }),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: async (): ReturnType<typeof actual.readOpenChapterSegments> => ({
      loaded: { chapter_id: 1, segments: [], caret_segment_id: null, assets: [], assets_dir: '', tm_filled_segment_ids: [], tm_prefill: { kind: 'ran', hidden_count: 0 } },
      error: null,
    }),
  }
})

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const WORK = {
  work_id: 'w-1',
  atproj_path: '/tmp/Kiem Lai.atproj',
  name: 'Kiem Lai',
  source_lang: 'zh',
  genre: 'action',
  created_at: '2026-09-01T00:00:00.000Z',
  updated_at: '2026-09-01T00:00:00.000Z',
  chapter_count: 3,
  status: 'in_progress',
  status_is_override: false,
  chapter_done_count: 1,
}

let wrapper: VueWrapper | null = null

function batchFor(paths: string[]) {
  return {
    batch: {
      items: paths.map((path) => ({ path, ok: true, byte_len: null, error: null })),
      encoding_preview: null,
    },
    error: null,
  }
}

async function open() {
  vi.resetModules()
  await seedGenres()
  mockInvoke.mockReset()
  previewFileMock.mockReset()
  previewTextMock.mockReset()
  previewBilingualMock.mockReset()
  pickFilesMock.mockReset()
  mockInvoke.mockImplementation((cmd: string) =>
    cmd === 'library_list_works'
      ? Promise.resolve({ total: 1, matched: 1, works: [WORK], genres: [], source_langs: [] })
      : Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }),
  )
  const nhap = await import('../../src/modes/libraryImport')
  const overlay = await import('../../src/modes/libraryImportOverlayState')
  const commands = await import('../../src/commands')
  commands.installCommands({
    setMode: () => {},
    isMac: false,
    closeLibraryImportForm: overlay.closeLibraryImportOverlay,
    selectImportTab: nhap.selectImportTab,
    submitActiveImportTab: () => void nhap.submitActiveImportTab(),
    pickImportFiles: () => void nhap.chooseImportFiles(),
    removeAimedImportFile: () => void nhap.removeAimedImportFile(),
  } as CommandDeps)
  await nhap.loadDestinationWorks()
  const Overlay = (await import('../../src/LibraryImportOverlay.vue')).default
  overlay.openLibraryImportOverlay()
  wrapper = mount(Overlay, { attachTo: document.body })
  await flushPromises()
  nhap.selectImportTab('files')
  nhap.name.value = 'Ten Tac Pham'
  await flushPromises()
  return { nhap, overlay, view: wrapper }
}

async function pick(nhap: Awaited<ReturnType<typeof open>>['nhap'], paths: string[]) {
  pickFilesMock.mockResolvedValue({ classification: fakeClassification(paths), error: null })
  await nhap.chooseImportFiles()
  await flushPromises()
}

const primary = (view: VueWrapper): HTMLButtonElement =>
  view.get('[data-import-preview-open]').element as HTMLButtonElement

const rows = (view: VueWrapper) => view.findAll('[data-import-file-path]')

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('Tệp tab — mixed kinds, bilingual limits and unsupported rows', () => {
  it('2 .txt + 1 .csv: the .csv row is marked, the primary is locked with a sentence, Bỏ on the .csv unlocks and opens the text preview', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.txt', '/d/b.txt', '/d/c.csv'])

    const csvRow = rows(view)[2]
    expect(csvRow.attributes('data-import-file-problem')).toBe('kind_mismatch')
    expect(csvRow.text()).toContain('song ngữ, khác loại')
    expect(view.get('[data-import-files-lock]').text()).toContain('Một lượt nhập chỉ một loại')
    expect(primary(view).disabled).toBe(true)

    await csvRow.get('[data-import-file-remove]').trigger('mousedown')
    await csvRow.get('[data-import-file-remove]').trigger('click')
    await flushPromises()

    expect(rows(view)).toHaveLength(2)
    expect(view.find('[data-import-files-lock]').exists()).toBe(false)
    expect(primary(view).disabled).toBe(false)

    previewFileMock.mockResolvedValue(batchFor(['/d/a.txt', '/d/b.txt']))
    await view.get('[data-import-preview-open]').trigger('click')
    await flushPromises()
    expect(previewFileMock).toHaveBeenCalledTimes(1)
    expect(previewFileMock.mock.calls[0][0]).toEqual(['/d/a.txt', '/d/b.txt'])
  })

  it('2 .csv: both rows are marked and the sentence names the one-file rule', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.csv', '/d/b.tsv'])

    expect(rows(view).map((r) => r.attributes('data-import-file-problem'))).toEqual([
      'one_bilingual_only',
      'one_bilingual_only',
    ])
    expect(view.get('[data-import-files-lock]').text()).toContain('từng tệp một')
    expect(primary(view).disabled).toBe(true)
  })

  it('a dropped .pdf is marked "không nhận loại này" and the primary stays locked until Bỏ', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.txt', '/d/x.pdf'])

    const pdfRow = rows(view)[1]
    expect(pdfRow.attributes('data-import-file-kind')).toBe('unsupported')
    expect(pdfRow.text()).toContain('không nhận loại này')
    expect(primary(view).disabled).toBe(true)

    await pdfRow.get('[data-import-file-remove]').trigger('mousedown')
    await pdfRow.get('[data-import-file-remove]').trigger('click')
    await flushPromises()
    expect(primary(view).disabled).toBe(false)
  })

  it('1 .csv opens the bilingual preview with the destination mode; "auto" language is the existing bilingual_needs_source_lang error', async () => {
    const { nhap, view } = await open()
    previewBilingualMock.mockResolvedValue({ preview: null, error: null })
    await pick(nhap, ['/d/song-ngu.csv'])
    expect(primary(view).disabled).toBe(false)

    nhap.sourceLang.value = 'auto'
    await view.get('[data-import-preview-open]').trigger('click')
    await flushPromises()
    expect(previewBilingualMock).not.toHaveBeenCalled()
    expect(nhap.lastError.value?.code).toBe('import.bilingual_needs_source_lang')

    nhap.sourceLang.value = 'zh'
    await view.get('[data-import-preview-open]').trigger('click')
    await flushPromises()
    expect(previewBilingualMock).toHaveBeenCalledTimes(1)
    expect(previewBilingualMock.mock.calls[0][0]).toBe('/d/song-ngu.csv')
  })

  it('a .docx row carries "Đọc như song ngữ", off by default; on, it counts as bilingual and the one-kind rule applies', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/bang.docx'])
    expect(nhap.importFilesVerdict.value).toBe('text')
    const toggle = view.get('[data-import-file-path="/d/bang.docx"] input[type="checkbox"]')
    expect((toggle.element as HTMLInputElement).checked).toBe(false)

    await toggle.setValue(true)
    await flushPromises()
    expect(nhap.importFilesVerdict.value).toBe('bilingual')
    expect(rows(view)[0].attributes('data-import-file-kind')).toBe('bilingual')

    await pick(nhap, ['/d/more.txt'])
    expect(nhap.importFilesVerdict.value).toBe('mixed_kinds')
    expect(rows(view)[0].attributes('data-import-file-problem')).toBe('kind_mismatch')
    expect(view.find('[data-import-file-path="/d/more.txt"] input[type="checkbox"]').exists()).toBe(false)
    expect(primary(view).disabled).toBe(true)
  })

  it('a .txt row has no toggle', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.txt'])
    expect(view.find('input[type="checkbox"]').exists()).toBe(false)
  })
})

describe('tabs keep their own content and only the selected tab is submitted', () => {
  it('switching Dán, Tệp and URL never clears another tab', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.txt'])
    nhap.pastedText.value = 'doan van dan'
    nhap.pastedUrls.value = 'https://a.example/1'

    for (const tab of ['paste', 'urls', 'files', 'paste'] as const) {
      await view.get(`[data-import-tab="${tab}"]`).trigger('click')
      await flushPromises()
    }

    expect(nhap.pastedText.value).toBe('doan van dan')
    expect(nhap.pastedUrls.value).toBe('https://a.example/1')
    expect(nhap.importFiles.value.map((f) => f.path)).toEqual(['/d/a.txt'])
    expect((view.get('[data-import-tabpanel="paste"] textarea').element as HTMLTextAreaElement).value).toBe(
      'doan van dan',
    )
  })

  it('the primary button submits the selected tab, never the others', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.txt'])
    nhap.pastedText.value = 'doan van dan'
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    await view.get('[data-import-tab="paste"]').trigger('click')
    await view.get('[data-import-preview-open]').trigger('click')
    await flushPromises()

    expect(previewTextMock).toHaveBeenCalledTimes(1)
    expect(previewFileMock).not.toHaveBeenCalled()
  })
})

describe('an existing Work locks Name, Ngôn ngữ nguồn and Thể loại on every tab', () => {
  it('shows the Work values, disabled, whichever tab is selected', async () => {
    const { nhap, view } = await open()
    await view.get('[data-import-destination] input[value="existing"]').setValue(true)
    await flushPromises()
    await pickSelectOption(view.get('[data-import-destination-picker] [data-dropdown-menu]'), 'w-1')
    await flushPromises()

    for (const tab of ['paste', 'files', 'urls'] as const) {
      await view.get(`[data-import-tab="${tab}"]`).trigger('click')
      await flushPromises()
      const locked = view.findAll('.import-form .field input[disabled]')
      expect(locked, tab).toHaveLength(3)
      expect(locked.map((i) => (i.element as HTMLInputElement).value)).toContain('Kiem Lai')
      expect(view.find('[data-import-source-lang]').exists(), tab).toBe(false)
    }
    expect(nhap.effectiveName.value).toBe('Kiem Lai')
    expect(view.get('.li-dest').text()).toBe('Thêm Chương vào cuối «Kiem Lai»')
  })

  it('a bilingual file with an existing Work sends the destination id, names the Work and can confirm', async () => {
    const { nhap, view } = await open()
    const loaded = {
      confidence: 'high',
      selected_encoding: 'UTF-8',
      candidates: [
        {
          label: 'UTF-8',
          encoding: 'UTF-8',
          preview: 'a,b',
          row_count: 1,
          chapter_count: 1,
          pair_count: 1,
          skipped_target_sentence_count: 0,
          mismatches: [],
          chapters: null,
        },
      ],
      sample_rows: [['a', 'b']],
      row_count: 1,
      column_count: 2,
      source_tables: [],
      table_index: null,
      table_choice_required: false,
    }
    previewBilingualMock.mockResolvedValue({ preview: loaded, error: null })
    await view.get('[data-import-destination] input[value="existing"]').setValue(true)
    await flushPromises()
    await pickSelectOption(view.get('[data-import-destination-picker] [data-dropdown-menu]'), 'w-1')
    await flushPromises()
    await pick(nhap, ['/d/song-ngu.csv'])
    const state = await import('../../src/bilingualImportPreviewState')
    const { default: BilingualOverlay } = await import('../../src/BilingualImportPreviewOverlay.vue')
    const bilingual = mount(BilingualOverlay, { attachTo: document.body })

    await view.get('[data-import-preview-open]').trigger('click')
    await flushPromises()

    expect(state.bilingualImportPreviewStatus.value).toBe('loaded')
    expect(state.bilingualImportPreviewDestinationMode.value).toBe('existing')
    expect(state.bilingualImportPreviewDestinationWorkId.value).toBe('w-1')
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(true)
    expect(previewBilingualMock.mock.calls[0][1]).toBe('zh')
    expect(previewBilingualMock.mock.calls[0][7]).toBe('w-1')
    expect(bilingual.find('[data-bilingual-use-new-work]').exists()).toBe(false)
    expect(bilingual.find('[data-bip-work-name]').exists()).toBe(false)
    expect(bilingual.get('[data-bip-destination]').text()).toBe("Đích: thêm vào \"Kiem Lai\"")
    bilingual.unmount()
  })

  it('removing a row with the focused Bỏ button leaves focus inside the overlay', async () => {
    const { nhap, view } = await open()
    await pick(nhap, ['/d/a.txt', '/d/b.txt'])
    const remove = view.get('[data-import-file-path="/d/a.txt"] [data-import-file-remove]')
    ;(remove.element as HTMLElement).focus()

    await remove.trigger('click')
    await flushPromises()

    expect(rows(view)).toHaveLength(1)
    expect(view.get('.li-panel').element.contains(document.activeElement)).toBe(true)
  })
})
