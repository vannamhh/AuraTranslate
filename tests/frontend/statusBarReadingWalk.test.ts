import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const mockOpenChapterById = vi.fn()
const mark = (id: number) => ({
  segment_id: id,
  navigation_segment_id: id,
  chapter_id: id,
  chapter_ord: id,
  chapter_title: null,
  source_text: 'a',
  target_text: 'b',
  is_retired: false,
  marked_at: 't',
})

vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: () => {},
}))
vi.mock('../../src/config/reading', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/reading')>()),
  listReadingMarks: async () => ({ marks: [mark(1), mark(2), mark(3)], error: null }),
}))
vi.mock('../../src/config/tm', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/tm')>()),
  tmCountPairs: async () => null,
}))
vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  countGlossaryEntries: async () => null,
}))
vi.mock('@tauri-apps/api/core', () => ({ convertFileSrc: (p: string) => p, invoke: vi.fn(async () => null) }))

async function armedBar() {
  vi.resetModules()
  mockOpenChapterById.mockReset().mockResolvedValue(true)
  const reading = await import('../../src/modes/readingState')
  const modes = await import('../../src/modes/modeState')
  const commands = await import('../../src/commands')
  commands.installCommands({
    isMac: true,
    setMode: modes.setMode,
    walkNextReadingMark: () => void reading.walkNextReadingMark(),
  } as never)
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  await reading.openReadingMarks()
  await reading.walkReadingMarks()
  const wrapper = mount(StatusBar, { attachTo: document.body })
  await flushPromises()
  return { reading, modes, wrapper }
}

// The first transform of StatusBar.vue is paid here, in the hook budget, not by the first case.
beforeAll(async () => {
  await import('../../src/StatusBar.vue')
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('Workspace step-through control in the status bar', () => {
  it('shows "Chỗ kế tiếp (1/3)" in the Workspace and steps on click', async () => {
    const { modes, wrapper, reading } = await armedBar()
    modes.setMode('workspace')
    await flushPromises()
    const button = wrapper.get('[data-reading-walk-next]')
    expect(button.text()).toBe('Chỗ kế tiếp (1/3)')
    await button.trigger('click')
    await flushPromises()
    expect(mockOpenChapterById).toHaveBeenLastCalledWith(2, 2)
    expect(wrapper.get('[data-reading-walk-next]').text()).toBe('Chỗ kế tiếp (2/3)')
    expect(reading.readingMarksWalkProgress.value).toEqual({ next: 2, total: 3 })
    wrapper.unmount()
  })

  it('is hidden outside the Workspace even while a walk is armed', async () => {
    const { modes, wrapper } = await armedBar()
    modes.setMode('reading')
    await flushPromises()
    expect(wrapper.find('[data-reading-walk-next]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('disappears when the last mark has been opened', async () => {
    const { modes, wrapper, reading } = await armedBar()
    modes.setMode('workspace')
    await flushPromises()
    await reading.walkNextReadingMark()
    await reading.walkNextReadingMark()
    await flushPromises()
    expect(wrapper.find('[data-reading-walk-next]').exists()).toBe(false)
    wrapper.unmount()
  })
})
