/**
 * The Prompt Library, Prompt Import, Glossary Import, TM Manage and Settings dialogs and the quick-add form expose an accessible name: `aria-labelledby`
 * resolves to an element that carries the visible title. One case per surface, so removing
 * the attribute from one surface turns only that case red.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'

const dispatchMock = vi.fn()
const promptSetListMock = vi.fn()
const promptSetOpenImportPreviewMock = vi.fn()
const openGlossaryImportPreviewMock = vi.fn()
const lookupGlossaryTermMock = vi.fn()
const tmListPairsMock = vi.fn()

vi.mock('../../src/commands', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/commands')>()),
  dispatch: (...args: unknown[]) => dispatchMock(...args),
}))

vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...args: unknown[]) => promptSetListMock(...args),
  promptSetOpenImportPreview: (...args: unknown[]) => promptSetOpenImportPreviewMock(...args),
  promptSetConfirmImport: vi.fn(),
  promptSetCancelImport: vi.fn(),
}))

vi.mock('../../src/config/glossary', () => ({
  openGlossaryImportPreview: (...args: unknown[]) => openGlossaryImportPreviewMock(...args),
  confirmGlossaryImport: vi.fn(),
  cancelGlossaryImport: vi.fn(),
  lookupGlossaryTerm: (...args: unknown[]) => lookupGlossaryTermMock(...args),
  addGlossaryTerm: vi.fn(),
  updateGlossaryTerm: vi.fn(),
}))

vi.mock('../../src/config/tm', () => ({
  tmListPairs: (...args: unknown[]) => tmListPairsMock(...args),
  tmUpdatePairTarget: vi.fn(),
  tmDeletePair: vi.fn(),
  tmDeleteOthers: vi.fn(),
  tmPushPairToGlobal: vi.fn(),
}))

vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

beforeEach(() => {
  document.body.innerHTML = ''
  vi.resetModules()
  dispatchMock.mockReset()
  promptSetListMock.mockReset()
  promptSetListMock.mockResolvedValue({ sets: [], workTierAvailable: false, error: null })
  promptSetOpenImportPreviewMock.mockReset()
  promptSetOpenImportPreviewMock.mockResolvedValue({
    outcome: 'loaded',
    preview: {
      file_name: 'a.prompt.md',
      name: 'A',
      body: 'x',
      warnings: { unknown_markers: [], glossary_terms_missing: false },
      global: { kind: 'new', existing_body: null },
      work: null,
    },
  })
  tmListPairsMock.mockReset()
  tmListPairsMock.mockResolvedValue({
    listing: { work_open: false, tm_empty: true, health: [], total_pairs: 0, total_groups: 0, groups: [] },
    error: null,
  })
  openGlossaryImportPreviewMock.mockReset()
  openGlossaryImportPreviewMock.mockResolvedValue({
    outcome: 'loaded',
    preview: {
      file_name: 'g.csv',
      tier: 'global',
      row_count: 1,
      recognized_column_count: 1,
      ignored_columns: [],
      term_origin_column_present: false,
      new_count: 1,
      identical_count: 0,
      conflicts: [],
    },
  })
})

/** The element `aria-labelledby` points at, resolved the way assistive tech resolves it. */
function labelOf(host: Element): HTMLElement {
  const id = host.getAttribute('aria-labelledby')
  expect(id, 'aria-labelledby is present').toBeTruthy()
  const target = document.getElementById(id ?? '')
  expect(target, `#${id} exists`).not.toBeNull()
  return target as HTMLElement
}

function expectNamedBy(wrapper: VueWrapper, hostSelector: string, titleSelector: string): void {
  const host = wrapper.get(hostSelector).element
  const label = labelOf(host)
  expect(label.textContent.trim().length).toBeGreaterThan(0)
  expect(label.textContent).toBe(wrapper.get(titleSelector).element.textContent)
}

describe('aria-labelledby names the surface with its visible title', () => {
  it('PromptLibraryOverlay', async () => {
    const state = await import('../../src/promptLibraryState')
    const Overlay = (await import('../../src/PromptLibraryOverlay.vue')).default
    state.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expectNamedBy(wrapper, '.pl-panel[role="dialog"]', '.pl-title')
    wrapper.unmount()
  })

  it('PromptImportOverlay', async () => {
    const state = await import('../../src/promptSetImportState')
    const Overlay = (await import('../../src/PromptImportOverlay.vue')).default
    await state.openPromptImportPreviewOverlay()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expectNamedBy(wrapper, '.pi-panel[role="dialog"]', '.pi-title')
    wrapper.unmount()
  })

  it('GlossaryImportOverlay', async () => {
    const state = await import('../../src/glossaryImportState')
    const Overlay = (await import('../../src/GlossaryImportOverlay.vue')).default
    await state.openGlossaryImportPreviewOverlay('global')
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expectNamedBy(wrapper, '.gi-panel[role="dialog"]', '.gi-title')
    wrapper.unmount()
  })

  it('TmManageOverlay', async () => {
    const state = await import('../../src/tmManageState')
    const Overlay = (await import('../../src/TmManageOverlay.vue')).default
    await state.openTmManage()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expectNamedBy(wrapper, '.tm-panel[role="dialog"]', '.tm-title')
    wrapper.unmount()
  })

  it('SettingsOverlay', async () => {
    const state = await import('../../src/settingsState')
    const Overlay = (await import('../../src/SettingsOverlay.vue')).default
    state.openSettings()
    const wrapper = mount(Overlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expectNamedBy(wrapper, '.set-panel[role="dialog"]', '.set-title')
    wrapper.unmount()
  })

  it('GlossaryQuickAdd form, in add and in edit mode', async () => {
    const state = await import('../../src/glossaryQuickAddState')
    const Form = (await import('../../src/GlossaryQuickAdd.vue')).default
    const wrapper = mount(Form, { attachTo: document.body })

    lookupGlossaryTermMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    state.openGlossaryQuickAdd('慕容')
    await flushPromises()
    expectNamedBy(wrapper, 'form.glossary-quick-add', '.gqa-title')
    const addTitle = wrapper.get('.gqa-title').text()

    state.closeGlossaryQuickAdd()
    await wrapper.vm.$nextTick()
    lookupGlossaryTermMock.mockResolvedValue({
      found: 'entry',
      workTierAvailable: true,
      entry: {
        tier: 'global',
        id: 7,
        source_term: '慕容',
        translation: 'Mộ Dung',
        note: '',
        category: 'person',
        term_origin: 'manual',
        created_at: '2026-08-20T00:00:00.000Z',
      },
    })
    state.openGlossaryQuickAdd('慕容')
    await flushPromises()
    expectNamedBy(wrapper, 'form.glossary-quick-add', '.gqa-title')
    expect(wrapper.get('.gqa-title').text()).not.toBe(addTitle)

    wrapper.unmount()
  })
})
