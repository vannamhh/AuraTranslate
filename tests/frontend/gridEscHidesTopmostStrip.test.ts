import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, recordSave } from './support/segmentFixture'
import type { TmFuzzyMatchesResult } from '../../src/config/segment'

const tmFuzzyMatchesMock = vi.fn<(segmentId: number) => Promise<TmFuzzyMatchesResult>>()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    tmFuzzyMatches: (segmentId: number) => tmFuzzyMatchesMock(segmentId),
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }
const mounted: { unmount: () => void }[] = []

afterEach(() => {
  for (const w of mounted.splice(0)) w.unmount()
  document.body.innerHTML = ''
  tmFuzzyMatchesMock.mockReset()
})

const ONE_MATCH: TmFuzzyMatchesResult = {
  outcome: {
    segment_id: 11,
    matches: [
      {
        tier: 'work',
        unit_id: 1,
        percent: 90,
        source_text: 'x',
        target_text: 'y',
        diff: [],
        side: 'mine',
        created_at: '2026-01-01T00:00:00.000Z',
      },
    ],
    exact: [],
    hidden_count: 0,
  },
  error: null,
}

async function mountGrid() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const commands = await import('../../src/commands')
  const tmStrip = await import('../../src/tmFuzzyStripState')
  const quickAdd = await import('../../src/glossaryQuickAddState')
  const ports = {
    hideTmFuzzyStrip: vi.fn(),
    hideConvention: vi.fn(),
    hideAiProofread: vi.fn(),
    closeGlossaryQuickAdd: vi.fn(),
    deferGlossaryConfirmStrip: vi.fn(),
    clearSourceCuts: vi.fn(),
    clearSegmentSelection: vi.fn(),
  }
  commands.installCommands({ isMac: true, setMode: () => {}, ...ports })
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  mounted.push(wrapper)
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, tmStrip, quickAdd, ports, wrapper }
}

async function showTmStripAtCaret(ctx: Awaited<ReturnType<typeof mountGrid>>) {
  tmFuzzyMatchesMock.mockResolvedValue(ONE_MATCH)
  ctx.state.setEditorCaret(11)
  ctx.tmStrip.syncTmFuzzyStrip(ctx.state.editorCaretSegmentId.value)
  await vi.waitFor(() => expect(ctx.tmStrip.tmFuzzyIsEligible.value).toBe(true))
}

function pressEscapeInCaretCell(wrapper: { element: Element }) {
  const cell = document.querySelector<HTMLElement>('[data-col="tgt"][data-segment-id="11"]')
  expect(cell).not.toBeNull()
  expect(wrapper.element.contains(cell)).toBe(true)
  cell?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true, cancelable: true }))
}

function hideCalls(ports: Awaited<ReturnType<typeof mountGrid>>['ports']) {
  return {
    tm: ports.hideTmFuzzyStrip.mock.calls.length,
    convention: ports.hideConvention.mock.calls.length,
    proofread: ports.hideAiProofread.mock.calls.length,
    quickAdd: ports.closeGlossaryQuickAdd.mock.calls.length,
    confirm: ports.deferGlossaryConfirmStrip.mock.calls.length,
  }
}

const NONE = { tm: 0, convention: 0, proofread: 0, quickAdd: 0, confirm: 0 }

describe('Esc trong ô bản dịch ẩn dải nội tuyến đang hiện', () => {
  it('dải đề xuất TM đang hiện ở câu có con trỏ thì Esc trong ô ẩn đúng nó', async () => {
    const ctx = await mountGrid()
    await showTmStripAtCaret(ctx)

    pressEscapeInCaretCell(ctx.wrapper)

    expect(hideCalls(ctx.ports)).toEqual({ ...NONE, tm: 1 })
  })

  it('dải thêm thuật ngữ đứng trên dải TM thì Esc trong ô không ẩn dải nào, kể cả dải TM bên dưới', async () => {
    const ctx = await mountGrid()
    await showTmStripAtCaret(ctx)
    ctx.quickAdd.openGlossaryQuickAdd('萧炎')

    pressEscapeInCaretCell(ctx.wrapper)

    expect(hideCalls(ctx.ports)).toEqual(NONE)
  })

  it('không dải nào hiện thì Esc trong ô không ẩn gì mà vẫn xoá điểm cắt và bỏ vùng chọn câu', async () => {
    const ctx = await mountGrid()

    pressEscapeInCaretCell(ctx.wrapper)

    expect(hideCalls(ctx.ports)).toEqual(NONE)
    expect(ctx.ports.clearSourceCuts).toHaveBeenCalledOnce()
    expect(ctx.ports.clearSegmentSelection).toHaveBeenCalledOnce()
  })
})
