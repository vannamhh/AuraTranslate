import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import type { Component } from 'vue'
import { readFixture } from './support/segmentFixture'
import type { AiTranslateBatchEventWire, AiTranslateOutcomeWire } from '../../src/config/aitranslate'
import type { CommandDeps } from '../../src/commands'
import type { IpcError } from '../../src/i18n'

const runSegmentMock = vi.fn()
const runBatchMock = vi.fn()
const resendCallMock = vi.fn()
const cancelMock = vi.fn()
const promoteMock = vi.fn()

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: (...args: unknown[]) => runSegmentMock(...args),
  cancelAiTranslateCall: (...args: unknown[]) => cancelMock(...args),
  runAiTranslateBatchCall: (...args: unknown[]) => runBatchMock(...args),
  resendAiCall: (...args: unknown[]) => resendCallMock(...args),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture }
})

vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, promoteAiTranslationToEditor: (...args: unknown[]) => promoteMock(...args) }
})

type RunResult = { value: AiTranslateOutcomeWire | null; error: IpcError | null }
type OnEvent = (event: AiTranslateBatchEventWire) => void

const DONE: AiTranslateOutcomeWire = { state: 'done', usage: null, maybe_truncated: false }

const PROVIDER_UNREACHABLE: IpcError = {
  code: 'ai_translate.provider_unreachable',
  message_key: 'err.ai_translate.provider_unreachable',
  params: { call_index: '1', first_segment_id: '11', last_segment_id: '13' },
  retryable: true,
}

function usage(total: number, cost: number | null = null) {
  return { prompt_tokens: total - 1, completion_tokens: 1, total_tokens: total, cost_usd: cost }
}

function received(callIndex: number, id: number, text: string, hasHan = false): AiTranslateBatchEventWire {
  return { kind: 'received', call_index: callIndex, segment_id: id, text, has_han: hasHan, source_hash: `h${id}` }
}

function pendingBatchRun(): { emit: OnEvent; settle: (result: RunResult) => void } {
  let captured: OnEvent = () => {
    throw new Error('runBatchMock chưa được gọi')
  }
  let resolveRun: (result: RunResult) => void = () => {
    throw new Error('runBatchMock chưa được gọi')
  }
  const impl = (_a: unknown, _b: unknown, onEvent: OnEvent) => {
    captured = onEvent
    return new Promise<RunResult>((res) => {
      resolveRun = res
    })
  }
  runBatchMock.mockImplementation(impl)
  resendCallMock.mockImplementation((_op: number, _call: number, onEvent: OnEvent) => impl(0, 0, onEvent))
  return { emit: (e) => captured(e), settle: (r) => resolveRun(r) }
}

async function freshPanel() {
  vi.resetModules()
  runSegmentMock.mockReset()
  runBatchMock.mockReset()
  resendCallMock.mockReset()
  cancelMock.mockReset()
  promoteMock.mockReset()

  const commands = await import('../../src/commands')
  const editorState = await import('../../src/panels/editorPanelState')
  const selectionState = await import('../../src/panels/segmentSelectionState')
  const state = await import('../../src/aiTranslateState')
  const batchState = await import('../../src/aiTranslateBatchState')
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

  editorState.resetEditorPanel()
  selectionState.resetSegmentSelection()
  await editorState.ensureSegmentsLoaded()

  const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
  commands.installCommands({
    ...aiTranslateHandlers,
    clearSegmentSelection: selectionState.clearSegmentSelection,
  } as CommandDeps)

  return { commands, editorState, selectionState, state, batchState, AiTranslationPanel }
}

function mountPanel(AiTranslationPanel: Component) {
  return mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
}

function statuses(batchState: Awaited<ReturnType<typeof freshPanel>>['batchState']): string[] {
  return batchState.aiTranslateBatchRows.value.map((r) => r.status)
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('hàng lô theo sự kiện của Rust (AD-60 mục 5-6)', () => {
  it('rows are created from planned as queued; tokens stream; text comes only from received', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    expect(batchState.aiTranslateBatchRows.value).toEqual([])

    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12, 13] }] })
    expect(statuses(batchState)).toEqual(['queued', 'queued', 'queued'])
    expect(batchState.aiTranslateBatchRows.value.map((r) => r.callIndex)).toEqual([1, 1, 1])

    fake.emit({ kind: 'call_started', call_index: 1 })
    expect(batchState.aiTranslateBatchRunningCall.value).toBe(1)
    fake.emit({ kind: 'token', call_index: 1, segment_id: 11, text: 'Xin ' })
    fake.emit({ kind: 'token', call_index: 1, segment_id: 11, text: 'chào' })
    expect(batchState.aiTranslateBatchRows.value[0]).toMatchObject({ status: 'streaming', text: 'Xin chào' })
    expect(batchState.aiTranslateBatchTextForSegment(batchState.aiTranslateBatchRows.value, 11)).toBeNull()

    fake.emit(received(1, 11, 'Xin chào!', true))
    expect(batchState.aiTranslateBatchRows.value[0]).toMatchObject({
      status: 'received',
      text: 'Xin chào!',
      hasHan: true,
      sourceHash: 'h11',
    })
    expect(batchState.aiTranslateBatchResultForSegment(batchState.aiTranslateBatchRows.value, 11)).toEqual({
      text: 'Xin chào!',
      sourceHash: 'h11',
    })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
  })

  it('discarded clears the streamed text and makes the row missing with its reason (cannot be promoted)', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12] }] })
    fake.emit({ kind: 'token', call_index: 1, segment_id: 12, text: 'nửa câu' })
    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'no_finish' })

    expect(batchState.aiTranslateBatchRows.value[1]).toMatchObject({
      status: 'missing',
      reason: 'no_finish',
      text: '',
      sourceHash: null,
    })
    expect(batchState.aiTranslateBatchResultForSegment(batchState.aiTranslateBatchRows.value, 12)).toBeNull()
    expect(batchState.aiTranslateBatchPromotableItems(batchState.aiTranslateBatchRows.value)).toEqual([])
    fake.settle({ value: DONE, error: null })
    await flushPromises()
  })

  it('a late token never revives a row that was already resolved', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11])
    await flushPromises()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11] }] })
    fake.emit(received(1, 11, 'Xong'))
    fake.emit({ kind: 'token', call_index: 1, segment_id: 11, text: ' muộn' })
    expect(batchState.aiTranslateBatchRows.value[0]).toMatchObject({ status: 'received', text: 'Xong' })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
  })

  it('rows of calls never reached become not_sent once the command returns; a cancel keeps received rows', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    fake.emit({
      kind: 'planned',
      calls: [
        { call_index: 1, segment_ids: [11, 12] },
        { call_index: 2, segment_ids: [13] },
      ],
    })
    fake.emit({ kind: 'call_started', call_index: 1 })
    fake.emit(received(1, 11, 'Một'))
    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'interrupted' })
    fake.emit({ kind: 'call_done', call_index: 1, finish_reason: null, usage: null })
    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()

    expect(batchState.aiTranslateBatchStateValue.value).toBe('cancelled')
    expect(statuses(batchState)).toEqual(['received', 'missing', 'not_sent'])
    expect(batchState.aiTranslateBatchRetryIds(batchState.aiTranslateBatchRows.value)).toEqual([12, 13])
  })

  it('an error keeps received rows, turns unresolved ones into gaps and exposes the stopped call span', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12, 13] }] })
    fake.emit({ kind: 'call_started', call_index: 1 })
    fake.emit({ kind: 'token', call_index: 1, segment_id: 11, text: 'dở' })
    fake.settle({ value: null, error: PROVIDER_UNREACHABLE })
    await flushPromises()

    expect(batchState.aiTranslateBatchStateValue.value).toBe('error')
    expect(statuses(batchState)).toEqual(['missing', 'not_sent', 'not_sent'])
    expect(batchState.aiTranslateBatchRows.value[0]).toMatchObject({ reason: 'interrupted', text: '' })
    expect(batchState.aiTranslateBatchErrorSpan(batchState.aiTranslateBatchRows.value, PROVIDER_UNREACHABLE)).toEqual({
      callIndex: 1,
      first: 1,
      last: 3,
    })
    expect(batchState.aiTranslateBatchRunningCall.value).toBeNull()
  })

  it('a stale operation (reset while in flight) writes nothing', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11])
    await flushPromises()
    batchState.resetAiTranslateBatch()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11] }] })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
    expect(batchState.aiTranslateBatchRows.value).toEqual([])
    expect(batchState.aiTranslateBatchStateValue.value).toBe('not_configured')
  })
})

describe('usage is summed per call_index, once per call (FR76)', () => {
  it('a call_done repeated for the same call_index is counted once', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    fake.emit({
      kind: 'planned',
      calls: [
        { call_index: 1, segment_ids: [11] },
        { call_index: 2, segment_ids: [12] },
      ],
    })
    fake.emit({ kind: 'call_done', call_index: 1, finish_reason: 'stop', usage: usage(100, 0.01) })
    fake.emit({ kind: 'call_done', call_index: 1, finish_reason: 'stop', usage: usage(100, 0.01) })
    fake.emit({ kind: 'call_done', call_index: 2, finish_reason: 'stop', usage: usage(50, 0.005) })
    expect(batchState.aiTranslateBatchUsageSummary.value).toEqual({
      tokenCount: 150,
      reportedCount: 2,
      sentenceCount: 2,
      costUsd: 0.015,
    })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
  })

  it('a call without usage is counted as a call but adds no tokens; cost falls back to null when one reporting call has none', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    fake.emit({
      kind: 'planned',
      calls: [
        { call_index: 1, segment_ids: [11] },
        { call_index: 2, segment_ids: [12] },
        { call_index: 3, segment_ids: [13] },
      ],
    })
    fake.emit({ kind: 'call_done', call_index: 1, finish_reason: 'stop', usage: usage(10, 0.01) })
    fake.emit({ kind: 'call_done', call_index: 2, finish_reason: 'stop', usage: null })
    fake.emit({ kind: 'call_done', call_index: 3, finish_reason: 'stop', usage: usage(20, null) })
    expect(batchState.aiTranslateBatchUsageSummary.value).toEqual({
      tokenCount: 30,
      reportedCount: 2,
      sentenceCount: 3,
      costUsd: null,
    })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
  })

  it('a retry operation restarts call_index at 1 but adds to the total instead of replacing the first call', async () => {
    const { batchState } = await freshPanel()
    const first = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    first.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12] }] })
    first.emit(received(1, 11, 'Một'))
    first.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'missing' })
    first.emit({ kind: 'call_done', call_index: 1, finish_reason: 'stop', usage: usage(100) })
    first.settle({ value: DONE, error: null })
    await flushPromises()

    const second = pendingBatchRun()
    void batchState.retryAiTranslateBatch(null, batchState.aiTranslateBatchRetryIds(batchState.aiTranslateBatchRows.value))
    await flushPromises()
    second.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [12] }] })
    second.emit(received(1, 12, 'Hai'))
    second.emit({ kind: 'call_done', call_index: 1, finish_reason: 'stop', usage: usage(40) })
    second.settle({ value: DONE, error: null })
    await flushPromises()

    expect(batchState.aiTranslateBatchUsageSummary.value.tokenCount).toBe(140)
    expect(statuses(batchState)).toEqual(['received', 'received'])
    expect(runBatchMock.mock.calls[1]?.[0]).toEqual({ kind: 'selection', segment_ids: [12] })
  })
})

describe('thử lại và gửi lại lượt', () => {
  it('retry covers missing and not_sent rows, even after a clean done', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    fake.emit({
      kind: 'planned',
      calls: [
        { call_index: 1, segment_ids: [11, 12] },
        { call_index: 2, segment_ids: [13] },
      ],
    })
    fake.emit(received(1, 11, 'Một'))
    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'empty' })
    fake.settle({ value: DONE, error: null })
    await flushPromises()

    const ids = batchState.aiTranslateBatchRetryIds(batchState.aiTranslateBatchRows.value)
    expect(ids).toEqual([12, 13])
    expect(batchState.aiTranslateBatchMayRetry('done', null, ids)).toBe(true)
    expect(batchState.aiTranslateBatchMayRetry('cancelled', null, ids)).toBe(true)
    expect(batchState.aiTranslateBatchMayRetry('error', { ...PROVIDER_UNREACHABLE, retryable: false }, ids)).toBe(false)
    expect(batchState.aiTranslateBatchMayRetry('error', PROVIDER_UNREACHABLE, ids)).toBe(true)
    expect(batchState.aiTranslateBatchMayRetry('done', null, [])).toBe(false)
    expect(batchState.aiTranslateBatchMayRetry('generating' as never, null, ids)).toBe(false)
  })

  it('resending a call replays operation_id and call_index through the same row handling', async () => {
    const { batchState } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.resendAiTranslateCall(7, 3)
    await flushPromises()
    expect(resendCallMock.mock.calls[0]?.slice(0, 2)).toEqual([7, 3])

    fake.emit({ kind: 'planned', calls: [{ call_index: 3, segment_ids: [21, 22] }] })
    fake.emit(received(3, 21, 'A'))
    fake.emit(received(3, 22, 'B'))
    fake.emit({ kind: 'call_done', call_index: 3, finish_reason: 'stop', usage: usage(9) })
    fake.settle({ value: DONE, error: null })
    await flushPromises()

    expect(statuses(batchState)).toEqual(['received', 'received'])
    expect(batchState.aiTranslateBatchUsageSummary.value.tokenCount).toBe(9)
  })

  it('a second operation cannot start while one is generating', async () => {
    const { batchState } = await freshPanel()
    pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11])
    await flushPromises()
    await batchState.runAiTranslateBatch(null, [12])
    await batchState.resendAiTranslateCall(1, 1)
    expect(runBatchMock).toHaveBeenCalledTimes(1)
    expect(resendCallMock).not.toHaveBeenCalled()
  })

  it('empty selection sends nothing; chapter scope sends no ids', async () => {
    const { batchState } = await freshPanel()
    vi.spyOn(console, 'warn').mockImplementation(() => undefined)
    runBatchMock.mockResolvedValue({ value: DONE, error: null })
    await batchState.runAiTranslateBatch(null, [])
    expect(runBatchMock).not.toHaveBeenCalled()
    await batchState.runAiTranslateBatch('Set', null)
    expect(runBatchMock.mock.calls[0]?.slice(0, 2)).toEqual([{ kind: 'chapter' }, 'Set'])
  })
})

describe('không hằng trần lượt ở webview (AD-60 mục 1, ca s2)', () => {
  const codeLines = (path: string): string[] =>
    readFileSync(resolve(process.cwd(), path), 'utf8')
      .split('\n')
      .filter((line) => !/^\s*(\/\/|\*|\/\*)/.test(line))

  it.each([
    'src/aiTranslateBatchState.ts',
    'src/aiTranslateHandlers.ts',
    'src/config/aitranslate.ts',
    'src/panels/AiTranslationPanel.vue',
  ])('%s has no call-size constant', (path) => {
    const hits = codeLines(path).filter((line) => /(?<![\w.%#'"-])100(?![\w%.])/.test(line) || /CALL_MAX|MAX_SEGMENTS/.test(line))
    expect(hits).toEqual([])
  })
})

describe('AiTranslationPanel.vue: lô theo lượt', () => {
  it('progress, running call and gaps line follow the rows', async () => {
    const { batchState, AiTranslationPanel } = await freshPanel()
    const fake = pendingBatchRun()
    const wrapper = mountPanel(AiTranslationPanel)
    void batchState.runAiTranslateBatch(null, [11, 12, 13])
    await flushPromises()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12, 13] }] })
    fake.emit({ kind: 'call_started', call_index: 1 })
    fake.emit(received(1, 11, 'Một'))
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-batch-progress]').text()).toBe('Đã nhận 1/3 · còn 2 câu chưa có kết quả')
    expect(wrapper.get('[data-ai-translate-batch-running]').text()).toBe('Đang dịch lượt 1')
    expect(wrapper.find('[data-ai-translate-batch-gaps]').exists()).toBe(false)

    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'missing' })
    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 13, reason: 'no_finish' })
    fake.emit({ kind: 'call_done', call_index: 1, finish_reason: 'length', usage: null })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-ai-translate-batch-running]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-translate-batch-gaps]').text()).toBe('2 câu thiếu kết quả AI · 0 câu chưa gửi')
    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[1].text()).toContain('Thiếu kết quả AI')
    expect(rows[1].text()).not.toContain('chưa dịch')
    expect(wrapper.get('[data-ai-translate-batch-retry]').text()).toBe('Thử lại 2 câu thiếu hoặc chưa gửi')
    wrapper.unmount()
  })

  it('the retry button dispatches a selection run over the gaps', async () => {
    const { batchState, AiTranslationPanel } = await freshPanel()
    const fake = pendingBatchRun()
    const wrapper = mountPanel(AiTranslationPanel)
    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12] }] })
    fake.emit(received(1, 11, 'Một'))
    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'missing' })
    fake.settle({ value: DONE, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    runBatchMock.mockResolvedValue({ value: DONE, error: null })
    await wrapper.get('[data-ai-translate-batch-retry]').trigger('click')
    await flushPromises()
    expect(runBatchMock.mock.calls[1]?.[0]).toEqual({ kind: 'selection', segment_ids: [12] })
    wrapper.unmount()
  })
})

describe('dispatch("ai.translate.promote") nhánh lô', () => {
  it('hands over only a received row, with its source_hash', async () => {
    const { batchState, editorState, commands } = await freshPanel()
    const fake = pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11, 12])
    await flushPromises()
    fake.emit({ kind: 'planned', calls: [{ call_index: 1, segment_ids: [11, 12] }] })
    fake.emit(received(1, 11, 'Bản dịch 11'))
    fake.emit({ kind: 'discarded', call_index: 1, segment_id: 12, reason: 'missing' })
    fake.settle({ value: DONE, error: null })
    await flushPromises()

    editorState.setEditorCaret(12)
    commands.dispatch('ai.translate.promote')
    await flushPromises()
    expect(promoteMock).not.toHaveBeenCalled()

    editorState.setEditorCaret(11)
    commands.dispatch('ai.translate.promote')
    await flushPromises()
    expect(promoteMock).toHaveBeenCalledWith(11, 'Bản dịch 11', false, 'h11')
  })
})

describe('resetAiTranslateBatch huỷ lô đang bay', () => {
  it('reset while generating cancels once and clears rows; reset when idle sends nothing', async () => {
    const { batchState } = await freshPanel()
    pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11])
    await flushPromises()
    batchState.resetAiTranslateBatch()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(batchState.aiTranslateBatchRows.value).toEqual([])

    batchState.resetAiTranslateBatch()
    batchState.cancelAiTranslateBatch()
    expect(cancelMock).toHaveBeenCalledTimes(1)
  })
})

describe('cổng loại trừ lẫn nhau giữa lượt đơn và lô', () => {
  it('a generating batch blocks the single run and a generating single run blocks the batch', async () => {
    const { commands, batchState } = await freshPanel()
    pendingBatchRun()
    void batchState.runAiTranslateBatch(null, [11])
    await flushPromises()
    commands.dispatch('ai.translate.run')
    expect(runSegmentMock).not.toHaveBeenCalled()
  })
})
