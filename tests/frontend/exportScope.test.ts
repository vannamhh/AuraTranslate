/**
 * Export screen (FR89): scope choice, counts, the unconfirmed-segment warning, the folder dialog on Export
 * and the Word-like preview. `config/export.ts` and `config/chapter.ts` are the IPC boundary and
 * are mocked; the command registry, the state and the overlay are real.
 */
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { ChapterRow } from '../../src/config/chapter'
import type { ExportScope, ExportScopeCounts } from '../../src/config/export'

const listMock = vi.fn()
const summaryMock = vi.fn()
const folderMock = vi.fn()
const previewMock = vi.fn()

vi.mock('../../src/config/chapter', () => ({
  listChapters: (...args: unknown[]) => listMock(...args),
}))
vi.mock('../../src/config/export', () => ({
  exportScopeSummary: (...args: unknown[]) => summaryMock(...args),
  exportPreview: (...args: unknown[]) => previewMock(...args),
  exportChooseFolder: (...args: unknown[]) => folderMock(...args),
  exportDocxTwoColumn: () => Promise.resolve({ file: null, error: null }),
}))

function row(id: number, ord: number, title: string | null): ChapterRow {
  return {
    chapter_id: id,
    ord,
    title,
    status: 'not_started',
    segment_count: 3,
    translated_count: 0,
    unconfirmed_count: 0,
    unlinked_count: 0,
    in_translation_count: 3,
    updated_at: '2026-10-01T00:00:00Z',
    origin_author: null,
    origin_site_name: null,
    origin_url: null,
    origin_published_at: null,
  }
}

function counts(over: Partial<ExportScopeCounts> = {}): ExportScopeCounts {
  return { chapter_count: 3, segment_count: 12, unconfirmed_count: 0, unconfirmed_translated_count: 0, untranslated_count: 0, image_count: 0, missing_link_images: [], ...over }
}

function ipcError(code: string) {
  return { code, message_key: 'err.export.scope_empty', params: {}, retryable: false }
}

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, previewMock]) m.mockReset()
  previewMock.mockResolvedValue({
    preview: {
      chapter_id: 10,
      chapter_ord: 1,
      chapter_title: 'Mở đầu',
      file_name: 'x-hai-cot.docx',
      attribution: [],
      appendix: [],
      appendix_term_count: 0,
      not_reimportable: false,
      rows: [{ source: 'gốc', target: 'dịch' }],
    },
    error: null,
  })
  listMock.mockResolvedValue({
    chapters: [row(10, 1, 'Mở đầu'), row(11, 2, null), row(12, 3, 'Kết')],
    error: null,
  })
  summaryMock.mockResolvedValue({ counts: counts(), error: null })
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

function lastScope(): ExportScope {
  return summaryMock.mock.calls[summaryMock.mock.calls.length - 1][0] as ExportScope
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('scope choice and counts', () => {
  it('opens on the whole Work, asks Rust for that scope and shows its Chapter and segment counts', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(lastScope()).toEqual({ kind: 'work' })
    const text = wrapper.get('[data-export-counts]').text()
    expect(text).toContain('3 Chương')
    expect(text).toContain('12 câu')
    wrapper.unmount()
  })

  it('offers exactly one Chapter, several Chapters or the whole Work, and sends each as the scope Rust counts', async () => {
    const { state } = await fresh()
    await state.openExport()

    await state.setExportScopeKind('chapter')
    expect(lastScope()).toEqual({ kind: 'chapters', chapter_ids: [10] })
    await state.selectExportSingleChapter(12)
    expect(lastScope()).toEqual({ kind: 'chapters', chapter_ids: [12] })

    await state.setExportScopeKind('chapters')
    await state.toggleExportChapter(12)
    await state.toggleExportChapter(10)
    expect(lastScope()).toEqual({ kind: 'chapters', chapter_ids: [12, 10] })
    await state.toggleExportChapter(12)
    expect(lastScope()).toEqual({ kind: 'chapters', chapter_ids: [10] })

    await state.setExportScopeKind('work')
    expect(lastScope()).toEqual({ kind: 'work' })
  })

  it('a multi-Chapter scope with nothing ticked asks Rust nothing and says nothing is selected', async () => {
    const { state } = await fresh()
    await state.openExport()
    const callsBefore = summaryMock.mock.calls.length

    await state.setExportScopeKind('chapters')
    expect(summaryMock.mock.calls.length).toBe(callsBefore)
    expect(state.exportCountsStatus.value).toBe('none_selected')
    expect(state.exportCounts.value).toBeNull()
  })

  it('shows a Rust error as an error, not as zero counts', async () => {
    const { state } = await fresh()
    summaryMock.mockResolvedValue({ counts: null, error: ipcError('export.scope_empty') })
    await state.openExport()
    expect(state.exportCountsStatus.value).toBe('error')
    expect(state.exportCounts.value).toBeNull()
  })

  it('an older, slower count never overwrites the newer scope', async () => {
    const { state } = await fresh()
    await state.openExport()

    let releaseFirst: (v: unknown) => void = () => undefined
    summaryMock.mockImplementationOnce(() => new Promise((r) => (releaseFirst = r)))
    const slow = state.setExportScopeKind('chapter')
    summaryMock.mockResolvedValueOnce({ counts: counts({ chapter_count: 1, segment_count: 4 }), error: null })
    await state.setExportScopeKind('work')
    releaseFirst({ counts: counts({ chapter_count: 9, segment_count: 99 }), error: null })
    await slow

    expect(state.exportCounts.value?.chapter_count).toBe(1)
  })
})

describe('unconfirmed-segment warning', () => {
  it('names how many segments are unconfirmed, before any export', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue({ counts: counts({ unconfirmed_count: 7 }), error: null })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    const warning = wrapper.get('[data-export-unconfirmed]')
    expect(warning.text()).toContain('7 câu chưa xác nhận')
    expect(wrapper.get('[data-export-counts]').attributes('role')).toBe('status')
    expect(wrapper.get('[data-export-counts]').element.contains(warning.element)).toBe(true)
    expect(wrapper.get('[data-export-counts]').text()).toContain('3 Chương · 12 câu · 7 câu chưa xác nhận')
    wrapper.unmount()
  })

  it('shows no warning when every segment in scope is confirmed', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(wrapper.find('[data-export-unconfirmed]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('destination folder', () => {
  it('tells a dialog error and an absent bridge apart when Export is pressed', async () => {
    const { state } = await fresh()
    await state.openExport()
    folderMock.mockResolvedValue({ outcome: 'error', error: ipcError('export.folder_invalid') })
    await state.runExport()
    expect(state.exportFolderError.value?.code).toBe('export.folder_invalid')

    folderMock.mockResolvedValue({ outcome: 'ipc_unavailable' })
    await state.runExport()
    expect(state.exportFolderError.value).toBeNull()
    expect(state.exportFolderUnavailable.value).toBe(true)
  })

  it('is opened from Rust through the export.run command, not from the webview', async () => {
    const { state, commands } = await fresh()
    await state.openExport()
    folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/cmd' })
    commands.dispatch('export.run')
    await vi.waitFor(() => expect(folderMock).toHaveBeenCalledTimes(1))

    const adapter = readFileSync(resolve(__dirname, '../../src/config/export.ts'), 'utf8')
    expect(adapter).not.toMatch(/plugin-dialog|plugin-fs/)
    const capabilities = readFileSync(resolve(__dirname, '../../src-tauri/capabilities/main.json'), 'utf8')
    expect(capabilities).not.toMatch(/dialog|fs:/)
  })
})

describe('keyboard', () => {
  it('the Công cụ ▾ list no longer carries export; the Work page action bar owns the opener hook; Escape closes the screen and returns focus to the opener', async () => {
    const { state, commands, Overlay } = await fresh()
    const menuSource = readFileSync(resolve(__dirname, '../../src/TitlebarToolsMenu.vue'), 'utf8')
    expect(menuSource).not.toMatch(/data-export-open|dispatch\('export\.open'\)/)
    const pageSource = readFileSync(resolve(__dirname, '../../src/modes/LibraryWorkPage.vue'), 'utf8')
    expect(/<button[^>]*data-export-open[^>]*>/.test(pageSource)).toBe(true)
    expect(pageSource).toMatch(/data-export-open[\s\S]*?@click="dispatch\('export\.open'\)"/)

    const opener = document.createElement('button')
    opener.setAttribute('data-export-open', '')
    opener.addEventListener('click', () => commands.dispatch('export.open'))
    document.body.appendChild(opener)
    opener.focus()

    const wrapper = mount(Overlay, { attachTo: document.body })
    opener.click()
    await vi.waitFor(() => expect(state.exportOverlayIsOpen.value).toBe(true))
    await settle(wrapper)
    expect(document.activeElement).toBe(wrapper.get('[role="dialog"]').element)

    await wrapper.get('.ex-scrim').trigger('keydown', { key: 'Escape' })
    await settle(wrapper)
    expect(state.exportOverlayIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(opener)
    wrapper.unmount()
  })

  it('every control is a native radio, checkbox or button, so Tab and Space or Enter reach all of them', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.setExportScopeKind('chapters')
    await settle(wrapper)

    const interactive = Array.from((wrapper.element as HTMLElement).querySelectorAll<HTMLElement>('input, button, select, textarea, [tabindex]'))
    const allowed = interactive.filter(
      (el) =>
        el.matches('input[type="radio"], input[type="checkbox"], button[type="button"]') ||
        el.getAttribute('role') === 'dialog',
    )
    expect(interactive.length).toBeGreaterThan(5)
    expect(allowed.length).toBe(interactive.length)
    expect(wrapper.findAll('input[type="checkbox"]').length).toBe(8)

    await wrapper.findAll('input[type="checkbox"]')[1].trigger('change')
    expect(lastScope()).toEqual({ kind: 'chapters', chapter_ids: [11] })
    wrapper.unmount()
  })
})

describe('Word-like preview', () => {
  const source = readFileSync(resolve(__dirname, '../../src/ExportOverlay.vue'), 'utf8')

  const tokens = JSON.parse(readFileSync(resolve(__dirname, '../../src/tokens/tokens.json'), 'utf8')) as {
    colors: { light: Record<string, string>; dark: Record<string, string> }
    contrast: { pairs: { fg: string; bg: string }[] }
  }
  const wordTokens = ['word-page', 'word-ink', 'word-rule']

  it('takes every colour from the word-* role tokens, with no literal and no inline style', () => {
    for (const name of wordTokens) expect(source).toContain(`var(--color-${name})`)
    expect(source).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(source).not.toContain('aura-allow-literal')
    expect(source).not.toMatch(/:style=/)
    expect(source).toContain('data-export-word-preview')
  })

  it('keeps the word-* tokens identical in both themes and declares their contrast pairs', () => {
    for (const name of wordTokens) expect(tokens.colors.dark[name]).toBe(tokens.colors.light[name])
    const declared = tokens.contrast.pairs.map((p) => `${p.fg}|${p.bg}`)
    for (const pair of ['word-ink|word-page', 'word-rule|word-page']) {
      expect(declared).toContain(pair)
    }
  })

  it('renders a two-column table with no header row, as in the file', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    const preview = wrapper.get('[data-export-word-preview]')
    expect(preview.findAll('th').length).toBe(0)
    expect(preview.findAll('thead').length).toBe(0)
    expect(preview.findAll('tbody td').length).toBe(2)
    wrapper.unmount()
  })
})
