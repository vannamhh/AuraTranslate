import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import { FIXTURE_SEGMENTS, readFixture } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'
import type { CommandDeps } from '../../src/commands'

const runSegmentMock = vi.fn()
const runBatchMock = vi.fn()
let rows: ChapterSegment[] = []
let openWork = true

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: (...args: unknown[]) => runSegmentMock(...args),
  cancelAiTranslateCall: vi.fn(),
  runAiTranslateBatchCall: (...args: unknown[]) => runBatchMock(...args),
  resendAiCall: vi.fn(),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: async () => {
      const base = await readFixture()
      if (base.loaded === null) return base
      return { ...base, loaded: { ...base.loaded, segments: openWork ? rows.map((r) => ({ ...r })) : [] } }
    },
  }
})

function row(index: number, patch: Partial<ChapterSegment>): ChapterSegment {
  return { ...FIXTURE_SEGMENTS[index % 3], id: 100 + index, ord: index, ...patch }
}

async function mountedPanel() {
  vi.resetModules()
  runSegmentMock.mockReset()
  runBatchMock.mockReset()
  const commands = await import('../../src/commands')
  const editorState = await import('../../src/panels/editorPanelState')
  const selectionState = await import('../../src/panels/segmentSelectionState')
  const single = await import('../../src/aiTranslateState')
  const batch = await import('../../src/aiTranslateBatchState')
  const i18n = await import('../../src/i18n')
  const Panel = (await import('../../src/panels/AiTranslationPanel.vue')).default
  editorState.resetEditorPanel()
  selectionState.resetSegmentSelection()
  await editorState.ensureSegmentsLoaded()
  const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
  commands.installCommands({ ...aiTranslateHandlers } as CommandDeps)
  const wrapper = mount(Panel, { props: { params: { params: {} } }, attachTo: document.body })
  await flushPromises()
  const button = () => wrapper.get('[data-ai-translate-chapter-run]')
  return { wrapper, button, single, batch, i18n }
}

beforeEach(() => {
  document.body.innerHTML = ''
  openWork = true
  rows = []
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('AiTranslationPanel chapter run button', () => {
  it('half-translated Chapter: label carries N and the button is enabled', async () => {
    rows = [
      ...Array.from({ length: 6 }, (_, i) => row(i, { target_text: 'da dich' })),
      row(6, { is_omitted: true, target_text: '' }),
      row(7, { target_text: '' }),
      row(8, { target_text: '' }),
      row(9, { target_text: '' }),
    ]
    const { button } = await mountedPanel()
    expect(button().text()).toBe('Dịch cả Chương (3 chưa dịch)')
    expect(button().attributes('disabled')).toBeUndefined()
  })

  it('everything translated or omitted: plain label, disabled', async () => {
    rows = [row(0, { target_text: 'da dich' }), row(1, { is_omitted: true, target_text: '' })]
    const { button } = await mountedPanel()
    expect(button().text()).toBe('Dịch cả Chương')
    expect(button().attributes('disabled')).toBeDefined()
  })

  it('no sentence loaded: disabled', async () => {
    rows = []
    const { button } = await mountedPanel()
    expect(button().attributes('disabled')).toBeDefined()
  })

  it('no Work open: disabled', async () => {
    rows = [row(0, { target_text: '' })]
    openWork = false
    const { button } = await mountedPanel()
    expect(button().text()).toBe('Dịch cả Chương')
    expect(button().attributes('disabled')).toBeDefined()
  })

  it('a single run generating disables it', async () => {
    rows = [row(0, { target_text: '' })]
    const { button, single } = await mountedPanel()
    runSegmentMock.mockImplementation(() => new Promise(() => {}))
    expect(button().attributes('disabled')).toBeUndefined()
    void single.runAiTranslate(null, 100)
    await flushPromises()
    expect(single.aiTranslateStateValue.value).toBe('generating')
    expect(button().attributes('disabled')).toBeDefined()
  })

  it('a batch generating disables it', async () => {
    rows = [row(0, { target_text: '' }), row(1, { target_text: '' })]
    const { button, batch } = await mountedPanel()
    runBatchMock.mockImplementation(() => new Promise(() => {}))
    void batch.runAiTranslateBatch(null, [100])
    await flushPromises()
    expect(batch.aiTranslateBatchStateValue.value).toBe('generating')
    expect(button().attributes('disabled')).toBeDefined()
  })
})
