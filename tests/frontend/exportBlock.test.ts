/**
 * Export screen, `.docx` one-block format (FR121): the keyboard-reachable format choice, the
 * "not re-importable" note, the two pre-export warnings and the command the run sends.
 * `config/export.ts` and `config/chapter.ts` are mocked; the command registry, state and overlay are real.
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

vi.mock('../../src/config/chapter', () => ({
  listChapters: (...args: unknown[]) => listMock(...args),
}))
vi.mock('../../src/config/export', () => ({
  exportScopeSummary: (...args: unknown[]) => summaryMock(...args),
  exportChooseFolder: (...args: unknown[]) => folderMock(...args),
  exportDocxTwoColumn: (...args: unknown[]) => twoColumnMock(...args),
  exportDocxOneBlock: (...args: unknown[]) => oneBlockMock(...args),
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
  path: '/tmp/out/Tac Pham-mot-khoi.docx',
  chapter_count: 2,
  segment_count: 6,
  image_count: 0,
  images_skipped_missing_link: 0,
  images_dir: null,
}

async function fresh() {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, twoColumnMock, oneBlockMock]) m.mockReset()
  listMock.mockResolvedValue({ chapters: [row(10, 1), row(11, 2)], error: null })
  summaryMock.mockResolvedValue(counts())
  folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/out' })
  twoColumnMock.mockResolvedValue({ file: { ...written, path: '/tmp/out/x-hai-cot.docx' }, error: null })
  oneBlockMock.mockResolvedValue({ file: written, error: null })
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
  it('offers the one-block format as a native radio the keyboard can reach', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    const radios = wrapper.findAll('input[name="export-format"]')
    expect(radios).toHaveLength(4)
    expect(radios[1].element.tagName).toBe('INPUT')
    expect(radios[1].attributes('type')).toBe('radio')
    expect(radios[1].attributes('disabled')).toBeUndefined()
    expect(radios[1].attributes('tabindex')).toBeUndefined()
    expect(radios[1].element.closest('label')?.textContent).toContain('Một khối (.docx)')
    wrapper.unmount()
  })

  it('swaps the re-importable line for a "cannot be re-imported" line when one-block is chosen', async () => {
    const { state, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(wrapper.find('[data-export-not-reimportable]').exists()).toBe(false)
    expect(wrapper.find('[data-export-reimportable]').exists()).toBe(true)

    await wrapper.findAll('input[name="export-format"]')[1].setValue(true)
    await settle(wrapper)

    expect(wrapper.find('[data-export-reimportable]').exists()).toBe(false)
    const note = wrapper.get('[data-export-not-reimportable]').text()
    expect(note).toContain('không nhập lại được')
    expect(note).toContain('đăng bài')

    await wrapper.findAll('input[name="export-format"]')[0].setValue(true)
    await settle(wrapper)
    expect(wrapper.find('[data-export-not-reimportable]').exists()).toBe(false)
    expect(wrapper.find('[data-export-reimportable]').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('warnings before exporting one block', () => {
  it('names unconfirmed-but-translated and untranslated segments separately', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ unconfirmed_count: 9, unconfirmed_translated_count: 4, untranslated_count: 5 }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportFormat('docx_one_block')
    await settle(wrapper)

    const unconfirmed = wrapper.get('[data-export-block-unconfirmed]')
    expect(unconfirmed.text()).toContain('4 câu')
    expect(unconfirmed.text()).toContain('không phân biệt')
    const untranslated = wrapper.get('[data-export-block-untranslated]')
    expect(untranslated.text()).toContain('5 câu chưa dịch')
    expect(untranslated.text()).toContain('vắng mặt ở cột phải')
    expect(wrapper.find('[data-export-unconfirmed]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('hides each warning whose number is zero', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ unconfirmed_count: 3, unconfirmed_translated_count: 3, untranslated_count: 0 }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportFormat('docx_one_block')
    await settle(wrapper)
    expect(wrapper.find('[data-export-block-unconfirmed]').exists()).toBe(true)
    expect(wrapper.find('[data-export-block-untranslated]').exists()).toBe(false)

    summaryMock.mockResolvedValue(counts({ unconfirmed_count: 2, unconfirmed_translated_count: 0, untranslated_count: 2 }))
    await state.setExportScopeKind('work')
    await settle(wrapper)
    expect(wrapper.find('[data-export-block-unconfirmed]').exists()).toBe(false)
    expect(wrapper.find('[data-export-block-untranslated]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('leaves the two-column warning as it was', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ unconfirmed_count: 7, unconfirmed_translated_count: 2, untranslated_count: 5 }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    expect(wrapper.get('[data-export-unconfirmed]').text()).toContain('7 câu chưa xác nhận')
    expect(wrapper.find('[data-export-block-unconfirmed]').exists()).toBe(false)
    expect(wrapper.find('[data-export-block-untranslated]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('does not block the export', async () => {
    const { state, Overlay } = await fresh()
    summaryMock.mockResolvedValue(counts({ unconfirmed_translated_count: 1, untranslated_count: 1 }))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    state.setExportFormat('docx_one_block')
    await settle(wrapper)
    expect(wrapper.get('[data-export-run]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })
})

describe('running the one-block export', () => {
  it('sends scope, image mode and folder to the one-block command and never the two-column one', async () => {
    const { state, commands, Overlay } = await fresh()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await state.chooseExportFolder()
    await wrapper.findAll('input[name="export-format"]')[1].setValue(true)
    await settle(wrapper)

    commands.dispatch('export.run')
    await vi.waitFor(() => expect(state.exportRunStatus.value).toBe('done'))
    await settle(wrapper)

    expect(oneBlockMock).toHaveBeenCalledWith({ kind: 'work' }, 'file', '/tmp/out')
    expect(twoColumnMock).not.toHaveBeenCalled()
    expect(wrapper.get('[data-export-result]').text()).toBe('/tmp/out/Tac Pham-mot-khoi.docx')
    wrapper.unmount()
  })

  it('still sends the two-column command when that format is selected', async () => {
    const { state, commands } = await fresh()
    await state.openExport()
    await state.chooseExportFolder()

    commands.dispatch('export.run')
    await vi.waitFor(() => expect(state.exportRunStatus.value).toBe('done'))

    expect(twoColumnMock).toHaveBeenCalledWith({ kind: 'work' }, 'file', '/tmp/out')
    expect(oneBlockMock).not.toHaveBeenCalled()
  })

  it('forwards the chosen image mode', async () => {
    const { state } = await fresh()
    summaryMock.mockResolvedValue(counts({ image_count: 2 }))
    await state.openExport()
    await state.chooseExportFolder()
    state.setExportFormat('docx_one_block')
    state.setExportImageMode('link')

    await state.runExport()
    expect(oneBlockMock).toHaveBeenCalledWith({ kind: 'work' }, 'link', '/tmp/out')
  })
})
