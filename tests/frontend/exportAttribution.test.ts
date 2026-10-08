/**
 * Export screen, attribution block: the toggle defaults off, sends `attribution` on every
 * format, shows the global translator name read-only and resets to off.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const listMock = vi.fn()
const summaryMock = vi.fn()
const folderMock = vi.fn()
const twoColumnMock = vi.fn()
const oneBlockMock = vi.fn()
const textMock = vi.fn()
const nameMock = vi.fn()

vi.mock('../../src/config/chapter', () => ({ listChapters: (...a: unknown[]) => listMock(...a) }))
vi.mock('../../src/config/attribution', () => ({ translatorNameGet: (...a: unknown[]) => nameMock(...a) }))
vi.mock('../../src/config/export', () => ({
  exportScopeSummary: (...a: unknown[]) => summaryMock(...a),
  exportChooseFolder: (...a: unknown[]) => folderMock(...a),
  exportDocxTwoColumn: (...a: unknown[]) => twoColumnMock(...a),
  exportDocxOneBlock: (...a: unknown[]) => oneBlockMock(...a),
  exportText: (...a: unknown[]) => textMock(...a),
}))

const written = {
  path: '/tmp/out/x.docx',
  chapter_count: 1,
  segment_count: 1,
  image_count: 0,
  images_skipped_missing_link: 0,
  images_dir: null,
}

async function fresh(name: string | null, error: unknown = null) {
  vi.resetModules()
  for (const m of [listMock, summaryMock, folderMock, twoColumnMock, oneBlockMock, textMock, nameMock]) m.mockReset()
  listMock.mockResolvedValue({ chapters: [], error: null })
  summaryMock.mockResolvedValue({
    counts: {
      chapter_count: 1,
      segment_count: 1,
      unconfirmed_count: 0,
      unconfirmed_translated_count: 0,
      untranslated_count: 0,
      image_count: 0,
      missing_link_images: [],
    },
    error: null,
  })
  folderMock.mockResolvedValue({ outcome: 'picked', path: '/tmp/out' })
  for (const m of [twoColumnMock, oneBlockMock, textMock]) m.mockResolvedValue({ file: written, error: null })
  nameMock.mockResolvedValue({ name, error })
  const state = await import('../../src/exportState')
  const Overlay = (await import('../../src/ExportOverlay.vue')).default
  return { state, Overlay }
}

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  for (let i = 0; i < 4; i += 1) await wrapper.vm.$nextTick()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('attribution toggle', () => {
  it('is off with the default-off note on first open', async () => {
    const { state, Overlay } = await fresh('Ice')
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    await settle(wrapper)

    const box = wrapper.get('[data-export-attribution-toggle]').element as HTMLInputElement
    expect(box.checked).toBe(false)
    expect(wrapper.get('[data-export-attribution-default]').text()).toContain('Mặc định tắt')
    expect(wrapper.find('[data-export-translator]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the global translator name when on, or a reminder when unset', async () => {
    const named = await fresh('Ice')
    const w1 = mount(named.Overlay, { attachTo: document.body })
    await named.state.openExport()
    named.state.setExportAttribution(true)
    await settle(w1)
    expect(w1.get('[data-export-translator]').text()).toContain('Ice')
    expect(w1.get('[data-export-translator]').text()).toContain('Toàn cục')
    w1.unmount()

    const unset = await fresh(null)
    const w2 = mount(unset.Overlay, { attachTo: document.body })
    await unset.state.openExport()
    unset.state.setExportAttribution(true)
    await settle(w2)
    expect(w2.find('[data-export-translator]').exists()).toBe(false)
    expect(w2.get('[data-export-translator-missing]').text()).toContain('Cài đặt')
    w2.unmount()
  })

  it('shows the read error instead of the missing-name reminder', async () => {
    const ipcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    const { state, Overlay } = await fresh(null, ipcError)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await state.openExport()
    state.setExportAttribution(true)
    await settle(wrapper)
    expect(wrapper.find('[data-export-translator-error]').exists()).toBe(true)
    expect(wrapper.find('[data-export-translator-missing]').exists()).toBe(false)
    wrapper.unmount()
  })

  it.each([
    ['docx_two_column', 'two'],
    ['docx_one_block', 'one'],
    ['markdown', 'text'],
    ['plain_text', 'text'],
  ] as const)('sends attribution=true on %s', async (format, which) => {
    const { state } = await fresh('Ice')
    await state.openExport()
    await state.chooseExportFolder()
    state.setExportFormat(format)
    state.setExportAttribution(true)
    await state.runExport()
    const mock = which === 'two' ? twoColumnMock : which === 'one' ? oneBlockMock : textMock
    expect(mock).toHaveBeenCalledTimes(1)
    expect(mock.mock.calls[0][mock.mock.calls[0].length - 1]).toBe(true)
  })

  it('resetExport turns it back off', async () => {
    const { state } = await fresh('Ice')
    state.setExportAttribution(true)
    state.resetExport()
    expect(state.exportAttribution.value).toBe(false)
    expect(state.exportTranslatorName.value).toBeNull()
  })
})
