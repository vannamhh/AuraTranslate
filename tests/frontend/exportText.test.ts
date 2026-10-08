/**
 * Export screen, Markdown and plain-text formats (FR88): the two radios, the "cannot be
 * re-imported" line for plain text only, the two warnings and the command each format sends.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { ChapterRow } from '../../src/config/chapter'
import type { ExportScopeCounts } from '../../src/config/export'

const listMock = vi.fn()
const summaryMock = vi.fn()
const folderMock = vi.fn()
const twoColumnMock = vi.fn()
const oneBlockMock = vi.fn()
const textMock = vi.fn()

vi.mock('../../src/config/chapter', () => ({
  listChapters: (...args: unknown[]) => listMock(...args),
}))
vi.mock('../../src/config/export', () => ({
  exportScopeSummary: (...args: unknown[]) => summaryMock(...args),
  exportChooseFolder: (...args: unknown[]) => folderMock(...args),
  exportDocxTwoColumn: (...args: unknown[]) => twoColumnMock(...args),
  exportDocxOneBlock: (...args: unknown[]) => oneBlockMock(...args),
  exportText: (...args: unknown[]) => textMock(...args),
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
    counts: {
      chapter_count: 2,
      segment_count: 6,
      unconfirmed_count: 0,
      unconfirmed_translated_count: 0,
      untranslated_count: 0,
      image_count: 0,
      missing_link_images: [],
      ...over,
    },
    error: null,
  }
}

const written = {
  path: '/tmp/out/Tac Pham.md',
  chapter_count: 2,
  segment_count: 6,
  image_count: 0,
  images_skipped_missing_link: 0,
  images_dir: null,
}

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, twoColumnMock, oneBlockMock, textMock]) m.mockReset()
  listMock.mockResolvedValue({ chapters: [row(10, 1), row(11, 2)], error: null })
  summaryMock.mockResolvedValue(counts())
  folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/out' })
  textMock.mockResolvedValue({ file: written, error: null })
  const state = await import('../../src/exportState')
  const commands = await import('../../src/commands')
  const { exportCommandDeps } = await import('../../src/exportCommandDeps')
  const deps: Partial<CommandDeps> = exportCommandDeps()
  commands.installCommands(deps as CommandDeps)
  const Overlay = (await import('../../src/ExportOverlay.vue')).default
  return { state, Overlay }
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
  it('offers Markdown and plain text as native radios', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    const radios = wrapper.findAll('input[name="export-format"]')
    expect(radios).toHaveLength(4)
    expect(radios[2].element.closest('label')?.textContent).toContain('Markdown (.md)')
    expect(radios[3].element.closest('label')?.textContent).toContain('Text thuần (.txt)')
    wrapper.unmount()
  })

  it('says plain text cannot be re-imported and Markdown does not', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    await wrapper.findAll('input[name="export-format"]')[2].setValue(true)
    await settle(wrapper)
    expect(wrapper.find('[data-export-not-reimportable]').exists()).toBe(false)
    expect(wrapper.find('[data-export-reimportable]').exists()).toBe(false)

    await wrapper.findAll('input[name="export-format"]')[3].setValue(true)
    await settle(wrapper)
    expect(wrapper.get('[data-export-not-reimportable]').text()).toContain('không nhập lại được')
    wrapper.unmount()
  })
})

describe('warnings', () => {
  it.each(['markdown', 'plain_text'] as const)('names both warning counts for %s', async (format) => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ unconfirmed_count: 9, unconfirmed_translated_count: 4, untranslated_count: 5 }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportFormat(format)
    await settle(wrapper)

    expect(wrapper.get('[data-export-block-unconfirmed]').text()).toContain('4 câu')
    expect(wrapper.get('[data-export-block-untranslated]').text()).toContain('5 câu chưa dịch')
    expect(wrapper.find('[data-export-unconfirmed]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('run', () => {
  it.each([
    ['markdown', 'markdown'],
    ['plain_text', 'plain'],
  ] as const)('sends %s to the text command as %s and not to the docx ones', async (format, wire) => {
    const { state } = await fresh()
    await state.openExport()
    await state.chooseExportFolder()
    state.setExportFormat(format)
    await state.runExport()

    expect(textMock).toHaveBeenCalledWith({ kind: 'work' }, 'file', wire, '/tmp/out', false)
    expect(twoColumnMock).not.toHaveBeenCalled()
    expect(oneBlockMock).not.toHaveBeenCalled()
    expect(state.exportRunStatus.value).toBe('done')
  })
})
