/**
 * Màn Xem prompt đầy đủ (tab Kết quả / Prompt đã gửi, khối theo tầng, token thật, Chép, Gửi lại,
 * phần "đã cân nhắc nhưng không chèn") — mỗi ca ứng với một hàng I/O & Edge-Case Matrix của
 * plan `story-xem-prompt-man-ay-u-tach-tang-va-gui-lai-plan.md`.
 *
 * Biên IPC giả bằng `vi.mock` ở tầng adapter `config/*`; mọi adapter của `config/segment` trừ
 * `tmConcordance` được bọc để đếm — "không ghi segment" là khẳng định trên chính bộ đếm đó.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type {
  AssembledPromptWire,
  GlossaryInjectionStatusWire,
  PendingGlossaryTermWire,
  TmInjectionStatusWire,
} from '../../src/config/aiprompt'
import type { AiTranslateOutcomeWire, AiTranslateUsageWire } from '../../src/config/aitranslate'
import type { CommandDeps } from '../../src/commands'
import type { IpcError } from '../../src/i18n'

const hoisted = vi.hoisted(() => ({
  segmentCalls: [] as string[],
  readRecord: vi.fn(),
  resend: vi.fn(),
  runSegment: vi.fn(),
  runBatch: vi.fn(),
  tmConcordance: vi.fn(),
}))

vi.mock('../../src/config/aiprompt', () => ({
  aiPromptAssemble: vi.fn(),
  aiPromptReadRecord: async () => ({ value: await hoisted.readRecord(), error: null }),
}))

vi.mock('../../src/config/promptset', () => ({
  promptSetList: vi.fn(async () => ({ value: null, error: null })),
  promptSetCreate: vi.fn(),
  promptSetDelete: vi.fn(),
  promptSetExport: vi.fn(),
  promptSetRename: vi.fn(),
  promptSetUpdateBody: vi.fn(),
}))

vi.mock('../../src/config/aitranslate', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/aitranslate')>()),
  runAiTranslateResendCall: (...args: unknown[]) => hoisted.resend(...args),
  runAiTranslateSegment: (...args: unknown[]) => hoisted.runSegment(...args),
  runAiTranslateBatchCall: (...args: unknown[]) => hoisted.runBatch(...args),
}))

vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  listGlossaryTmSuggestions: async () => ({ value: [], error: null }),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  const wrapped: Record<string, unknown> = {}
  for (const [name, value] of Object.entries(actual)) {
    wrapped[name] =
      typeof value === 'function' && name !== 'tmConcordance'
        ? (...args: unknown[]) => {
            hoisted.segmentCalls.push(name)
            return (value as (...a: unknown[]) => unknown)(...args)
          }
        : value
  }
  wrapped.tmConcordance = (...args: unknown[]) => hoisted.tmConcordance(...args)
  return wrapped
})

const USAGE: AiTranslateUsageWire = { prompt_tokens: 100, completion_tokens: 312, total_tokens: 412, cost_usd: 0.00412 }
const SENT_AT = '2026-10-15T08:09:10.000Z'

const NOT_ASKED: GlossaryInjectionStatusWire = {
  kind: 'not_asked',
  injected: null,
  suppressed_by_pending_overlap: null,
  pending_matched: null,
}

function askedWith(pending: PendingGlossaryTermWire[]): GlossaryInjectionStatusWire {
  return { kind: 'asked', injected: [], suppressed_by_pending_overlap: [], pending_matched: pending }
}

function searched(omitted: number): TmInjectionStatusWire {
  return { kind: 'searched', similar_segments: [], omitted_count: omitted }
}

const PIECES: AssembledPromptWire['ledger']['pieces'] = [
  { kind: 'authored', text: 'AAAA' },
  { kind: 'glossary', text: 'GG' },
  { kind: 'authored', text: 'AA' },
  { kind: 'tm', text: 'TTTT' },
  { kind: 'source_segment', text: 'SSSSSSSSSS' },
]

function record(overrides: Partial<AssembledPromptWire> = {}): AssembledPromptWire {
  return {
    prompt: 'AAAAGGAATTTTSSSSSSSSSS',
    segment_id: 1,
    chapter_id: 10,
    prompt_set_name: 'Happy',
    prompt_set_tier: 'global',
    ledger: {
      glossary: NOT_ASKED,
      tm: { kind: 'not_asked', similar_segments: null, omitted_count: null },
      unknown_markers: [],
      source_segment_missing: false,
      pieces: PIECES,
    },
    sent_at: null,
    sent_model: null,
    sent_usage: null,
    ...overrides,
  }
}

function withLedger(over: Partial<AssembledPromptWire['ledger']>): AssembledPromptWire {
  return record({ ledger: { ...record().ledger, ...over } })
}

const IPC_ERROR: IpcError = { code: 'x.fail', message_key: 'err.unknown', params: {}, retryable: false }

async function freshScreen() {
  vi.resetModules()
  hoisted.readRecord.mockReset()
  hoisted.resend.mockReset()
  hoisted.runSegment.mockReset()
  hoisted.runBatch.mockReset()
  hoisted.tmConcordance.mockReset()
  hoisted.tmConcordance.mockResolvedValue({ outcome: null, error: null })
  hoisted.segmentCalls.length = 0

  const caret: Ref<number | null> = ref(1)
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caret,
    editorPendingPromote: ref(null),
    editorPendingPromoteBatch: ref(null),
    editorPromoteBatchReport: ref(null),
    editorSegments: ref([]),
    editorPromoteAiTranslationError: ref(null),
    cancelPendingPromote: vi.fn(),
    promoteAiTranslationToEditor: vi.fn(),
    promoteAiTranslationBatchToEditor: vi.fn(),
  }))

  const commands = await import('../../src/commands')
  const state = await import('../../src/aiPromptInspectorState')
  const translate = await import('../../src/aiTranslateState')
  const batch = await import('../../src/aiTranslateBatchState')
  const strip = await import('../../src/glossaryConfirmStripState')
  const lookup = await import('../../src/panels/lookupHistoryState')
  const concordance = await import('../../src/panels/concordanceState')
  const i18n = await import('../../src/i18n')
  const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
  const { aiPromptInspectorHandlers } = await import('../../src/aiPromptInspectorHandlers')
  const Overlay = (await import('../../src/AiPromptInspectorOverlay.vue')).default

  const focusStrip = vi.fn()
  const openGlossaryManage = vi.fn()
  const openPromptLibrary = vi.fn()
  const deps: Partial<CommandDeps> = {
    openAiPromptInspector: state.openAiPromptInspector,
    closeAiPromptInspector: state.closeAiPromptInspector,
    ...aiTranslateHandlers,
    ...aiPromptInspectorHandlers,
    focusGlossaryConfirmStrip: focusStrip,
    openGlossaryManage,
    openPromptLibrary,
  }
  commands.installCommands(deps as CommandDeps)

  return {
    commands,
    state,
    translate,
    batch,
    strip,
    lookup,
    concordance,
    i18n,
    Overlay,
    caret,
    focusStrip,
    openGlossaryManage,
    openPromptLibrary,
  }
}

type Screen = Awaited<ReturnType<typeof freshScreen>>

async function openWith(screen: Screen, rec: AssembledPromptWire | null) {
  hoisted.readRecord.mockResolvedValue(rec)
  screen.state.openAiPromptInspector()
  await flushPromises()
  const wrapper = mount(screen.Overlay, { attachTo: document.body })
  await flushPromises()
  return wrapper
}

function pendingRun() {
  let finish: (result: { value: AiTranslateOutcomeWire | null; error: IpcError | null }) => void = () => {
    throw new Error('chua gui')
  }
  let emit: (text: string) => void = () => {
    throw new Error('chua gui')
  }
  const impl = (onToken: (text: string) => void) => {
    emit = onToken
    return new Promise<{ value: AiTranslateOutcomeWire | null; error: IpcError | null }>((resolve) => {
      finish = resolve
    })
  }
  return { impl, emit: (text: string) => emit(text), finish: (r: Parameters<typeof finish>[0]) => finish(r) }
}

beforeEach(() => {
  document.body.innerHTML = ''
  vi.spyOn(console, 'warn').mockImplementation(() => undefined)
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('khối theo tầng, tỉ lệ chèn động (hàm thuần)', () => {
  it('gộp mảnh liền kề cùng loại, đếm ký tự theo từng khối và theo tầng', async () => {
    const { state } = await freshScreen()
    const blocks = state.aiPromptTierBlocks(record())
    expect(blocks.map((b) => [b.kind, b.chars])).toEqual([
      ['authored', 4],
      ['glossary', 2],
      ['authored', 2],
      ['tm', 4],
      ['source_segment', 10],
    ])
    const merged = state.aiPromptTierBlocks(
      record({ ledger: { ...record().ledger, pieces: [{ kind: 'authored', text: 'ab' }, { kind: 'authored', text: 'cd' }, { kind: 'tm', text: '' }] } }),
    )
    expect(merged.map((b) => [b.kind, b.chars])).toEqual([['authored', 4]])
    expect(state.aiPromptCharsByKind(blocks)).toEqual({ authored: 6, glossary: 2, tm: 4, source_segment: 10 })
  })

  it('tỉ lệ chèn động = (Glossary + TM) / mọi ký tự; prompt rỗng ⇒ null, không 0 %', async () => {
    const { state } = await freshScreen()
    expect(state.aiPromptDynamicRatio(state.aiPromptTierBlocks(record()))).toBeCloseTo((6 / 22) * 100, 6)
    expect(state.aiPromptDynamicRatio([])).toBeNull()
    expect(state.formatPercentParam(27.2727)).toBe('27,3')
  })
})

describe('token thật và trạng thái chưa có số', () => {
  it('lượt đơn có usage 412 ⇒ "real", kèm chi phí', async () => {
    const { state } = await freshScreen()
    const line = state.aiPromptTokenLine(record({ sent_at: SENT_AT, sent_model: 'gpt-test', sent_usage: USAGE }))
    expect(line).toEqual({ kind: 'real', promptTokens: 100, costUsd: 0.00412 })
  })

  it('đã gửi nhưng usage null (lô lắp / provider không trả) ⇒ "unavailable"; chưa gửi ⇒ "unsent"; null ⇒ "unsent"', async () => {
    const { state } = await freshScreen()
    expect(state.aiPromptTokenLine(record({ sent_at: SENT_AT, sent_usage: null }))).toEqual({ kind: 'unavailable' })
    expect(state.aiPromptTokenLine(record())).toEqual({ kind: 'unsent' })
    expect(state.aiPromptTokenLine(null)).toEqual({ kind: 'unsent' })
  })
})

describe('màn: hàng "Sau một lượt dịch đơn" / "Bản ghi do lô lắp" / "Mới lắp, chưa gửi"', () => {
  it('lượt đơn: dòng gửi lúc · mô hình, khối có nhãn tầng + số ký tự, tổng token thật, tỉ lệ động, chi phí', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, record({ sent_at: SENT_AT, sent_model: 'gpt-test', sent_usage: USAGE }))
    const { i18n, state } = screen

    expect(wrapper.get('[data-aip-sent-meta]').text()).toBe(
      i18n.t('ai.prompt_inspector.meta_sent', { time: state.formatSentAtClock(SENT_AT), model: 'gpt-test' }),
    )
    const kinds = wrapper.findAll('[data-aip-block-kind]').map((b) => b.attributes('data-aip-block-kind'))
    expect(kinds).toEqual(['authored', 'glossary', 'authored', 'tm', 'source_segment'])
    const first = wrapper.get('[data-aip-block-kind="authored"]')
    expect(first.get('.aip-scope').text()).toBe(i18n.t('ai.prompt_inspector.tier_global'))
    expect(first.get('[data-aip-block-chars]').text()).toBe(i18n.t('ai.prompt_inspector.block_chars', { count: '4' }))
    expect(wrapper.get('[data-aip-token-state="real"]').text()).toBe(i18n.t('ai.prompt_inspector.tokens_real', { count: '100' }))
    expect(wrapper.get('[data-aip-dynamic-ratio]').text()).toBe(
      i18n.t('ai.prompt_inspector.dynamic_ratio', { percent: '27,3' }),
    )
    expect(wrapper.get('[data-aip-cost]').text()).toBe(
      i18n.t('ai.prompt_inspector.cost_estimate', { cost_usd: '0,0041' }),
    )
    wrapper.unmount()
  })

  it('provider không trả usage (hoặc bản ghi do lô lắp): "chưa có số token thật", không 0, không chi phí; dòng gửi lúc vẫn hiện', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, record({ sent_at: SENT_AT, sent_model: 'gpt-test', sent_usage: null }))
    const { i18n } = screen

    expect(wrapper.get('[data-aip-token-state="unavailable"]').text()).toBe(i18n.t('ai.prompt_inspector.tokens_unavailable'))
    expect(wrapper.find('[data-aip-token-state="real"]').exists()).toBe(false)
    expect(wrapper.find('[data-aip-cost]').exists()).toBe(false)
    expect(wrapper.get('[data-aip-token-line]').text()).not.toMatch(/(^|\s)0(\s|$)/)
    expect(wrapper.get('[data-aip-sent-meta]').text()).toContain('gpt-test')
    wrapper.unmount()
  })

  it('mới lắp, chưa gửi: "chưa gửi", không 0, không chi phí; khối tầng và tỉ lệ động vẫn hiện', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, record())
    const { i18n } = screen

    expect(wrapper.get('[data-aip-token-state="unsent"]').text()).toBe(i18n.t('ai.prompt_inspector.tokens_unsent'))
    expect(wrapper.find('[data-aip-cost]').exists()).toBe(false)
    expect(wrapper.get('[data-aip-sent-meta]').text()).toBe(i18n.t('ai.prompt_inspector.meta_unsent'))
    expect(wrapper.findAll('[data-aip-block-kind]')).toHaveLength(5)
    expect(wrapper.find('[data-aip-dynamic-ratio]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('usage có nhưng mô hình không có giá (cost_usd null) ⇒ token thật hiện, không dòng chi phí', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(
      screen,
      record({ sent_at: SENT_AT, sent_model: 'local', sent_usage: { ...USAGE, cost_usd: null } }),
    )
    expect(wrapper.find('[data-aip-token-state="real"]').exists()).toBe(true)
    expect(wrapper.find('[data-aip-cost]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('Gửi lại', () => {
  it('đã cấu hình: đổi sang tab Kết quả, chữ chảy dần vào aiTranslateState của bản ghi, không ghi segment, nạp lại bản ghi', async () => {
    const screen = await freshScreen()
    const run = pendingRun()
    hoisted.resend.mockImplementation(run.impl)
    const wrapper = await openWith(screen, record({ segment_id: 7 }))
    hoisted.readRecord.mockResolvedValue(record({ segment_id: 7, sent_at: SENT_AT, sent_model: 'gpt-test', sent_usage: USAGE }))
    const readsBefore = hoisted.readRecord.mock.calls.length

    await wrapper.get('[data-aip-resend]').trigger('click')
    await flushPromises()

    expect(screen.state.aiPromptInspectorTab.value).toBe('result')
    expect(hoisted.resend).toHaveBeenCalledTimes(1)
    expect(hoisted.runSegment).not.toHaveBeenCalled()
    expect(screen.translate.aiTranslateRunSegmentId.value).toBe(7)
    expect(screen.translate.aiTranslateStateValue.value).toBe('generating')

    run.emit('Xin ')
    run.emit('chào')
    await flushPromises()
    expect(wrapper.get('[data-aip-result-text]').text()).toBe('Xin chào')
    expect(wrapper.find('[data-aip-result-state="generating"]').exists()).toBe(true)

    run.finish({ value: { state: 'done', usage: USAGE }, error: null })
    await flushPromises()
    expect(screen.translate.aiTranslateStateValue.value).toBe('done')
    expect(screen.translate.aiTranslateAccumulatedText.value).toBe('Xin chào')
    expect(screen.translate.aiTranslateUsage.value).toEqual(USAGE)
    expect(wrapper.find('[data-aip-result-state="text"]').exists()).toBe(true)
    expect(hoisted.readRecord.mock.calls.length).toBeGreaterThan(readsBefore)
    expect(screen.state.aiPromptRecord.value?.sent_at).toBe(SENT_AT)
    expect(hoisted.segmentCalls).toEqual([])
    wrapper.unmount()
  })

  it('Gửi lại bị huỷ giữa chừng: tab Kết quả giữ phần đã nhận và nói rõ là đã huỷ', async () => {
    const screen = await freshScreen()
    const run = pendingRun()
    hoisted.resend.mockImplementation(run.impl)
    const wrapper = await openWith(screen, record({ segment_id: 7 }))

    await wrapper.get('[data-aip-resend]').trigger('click')
    await flushPromises()
    run.emit('Xin ')
    run.finish({ value: { state: 'cancelled' }, error: null })
    await flushPromises()

    expect(wrapper.get('[data-aip-result-text]').text()).toBe('Xin')
    expect(wrapper.get('[data-aip-result-state="cancelled"]').text()).toBe(
      screen.i18n.t('panel.ai_translation.state_cancelled'),
    )
    expect(wrapper.find('[data-aip-result-state="text"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('bản ghi của câu khác câu đang chọn (stale): vẫn gửi bản ghi, kết quả gắn segment_id của bản ghi, thông báo cũ vẫn hiện', async () => {
    const screen = await freshScreen()
    hoisted.resend.mockImplementation(async (onToken: (t: string) => void) => {
      onToken('ket qua')
      return { value: { state: 'done', usage: null }, error: null }
    })
    const wrapper = await openWith(screen, record({ segment_id: 7 }))
    screen.caret.value = 9
    await flushPromises()
    expect(wrapper.find('[data-aip-stale="true"]').exists()).toBe(true)

    screen.commands.dispatch('ai.prompt_inspector.resend')
    await flushPromises()

    expect(hoisted.resend).toHaveBeenCalledTimes(1)
    expect(screen.translate.aiTranslateRunSegmentId.value).toBe(7)
    expect(wrapper.get('[data-aip-result-text]').text()).toBe('ket qua')
    expect(wrapper.find('[data-aip-stale="true"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('chưa cấu hình: lời mời như panel, không thử lại', async () => {
    const screen = await freshScreen()
    hoisted.resend.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    const wrapper = await openWith(screen, record())

    await wrapper.get('[data-aip-resend]').trigger('click')
    await flushPromises()

    expect(screen.translate.aiTranslateStateValue.value).toBe('not_configured')
    expect(wrapper.get('[data-aip-result-state="not_configured"]').text()).toBe(
      screen.i18n.t('panel.ai_translation.status'),
    )
    expect(hoisted.resend).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('lỗi provider: hiện tError, không thử lại', async () => {
    const screen = await freshScreen()
    hoisted.resend.mockResolvedValue({ value: null, error: IPC_ERROR })
    const wrapper = await openWith(screen, record())

    await wrapper.get('[data-aip-resend]').trigger('click')
    await flushPromises()

    expect(screen.translate.aiTranslateStateValue.value).toBe('error')
    expect(wrapper.get('[data-aip-result-state="error"]').text()).toBe(screen.i18n.tError(IPC_ERROR))
    expect(hoisted.resend).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('khoá khi lượt đơn đang chạy: nút disabled và dispatch lặp chỉ cảnh báo', async () => {
    const screen = await freshScreen()
    const run = pendingRun()
    hoisted.resend.mockImplementation(run.impl)
    const wrapper = await openWith(screen, record())

    screen.commands.dispatch('ai.prompt_inspector.resend')
    await flushPromises()
    expect(wrapper.get('[data-aip-resend]').attributes('disabled')).toBeDefined()

    screen.commands.dispatch('ai.prompt_inspector.resend')
    await flushPromises()
    expect(hoisted.resend).toHaveBeenCalledTimes(1)
    expect(console.warn).toHaveBeenCalled()
    run.finish({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    wrapper.unmount()
  })

  it('khoá khi lô đang chạy: nút disabled, dispatch không gọi lệnh Rust', async () => {
    const screen = await freshScreen()
    hoisted.runBatch.mockImplementation(() => new Promise(() => undefined))
    const wrapper = await openWith(screen, record())

    void screen.batch.runAiTranslateBatch(null, [1, 2])
    await flushPromises()
    expect(screen.batch.aiTranslateBatchStateValue.value).toBe('generating')
    expect(wrapper.get('[data-aip-resend]').attributes('disabled')).toBeDefined()

    screen.commands.dispatch('ai.prompt_inspector.resend')
    await flushPromises()
    expect(hoisted.resend).not.toHaveBeenCalled()
    expect(console.warn).toHaveBeenCalled()
    wrapper.unmount()
  })

  it('chưa có bản ghi: không nút để bấm, dispatch không gọi lệnh Rust', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, null)

    expect(wrapper.find('[data-aip-resend]').exists()).toBe(false)
    screen.commands.dispatch('ai.prompt_inspector.resend')
    await flushPromises()
    expect(hoisted.resend).not.toHaveBeenCalled()
    expect(console.warn).toHaveBeenCalled()
    wrapper.unmount()
  })
})

describe('tab Kết quả theo segment_id của bản ghi', () => {
  it('chưa có kết quả cho câu của bản ghi ⇒ trạng thái rỗng có chữ', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, record({ segment_id: 3 }))
    screen.commands.dispatch('ai.prompt_inspector.tab_result')
    await flushPromises()

    expect(wrapper.find('[data-aip-tab-panel="result"]').exists()).toBe(true)
    expect(wrapper.get('[data-aip-result-state="empty"]').text()).toBe(
      screen.i18n.t('ai.prompt_inspector.result_empty', { segment_id: '3' }),
    )
    wrapper.unmount()
  })

  it('kết quả lượt đơn của câu KHÁC không hiện; kết quả của đúng câu thì hiện', async () => {
    const screen = await freshScreen()
    hoisted.runSegment.mockImplementation(async (_id: number, _set: string | null, onToken: (t: string) => void) => {
      onToken('cua cau chin')
      return { value: { state: 'done', usage: null }, error: null }
    })
    const wrapper = await openWith(screen, record({ segment_id: 3 }))
    screen.commands.dispatch('ai.prompt_inspector.tab_result')
    await screen.translate.runAiTranslate(null, 9)
    await flushPromises()
    expect(wrapper.find('[data-aip-result-state="empty"]').exists()).toBe(true)
    expect(wrapper.text()).not.toContain('cua cau chin')

    await screen.translate.runAiTranslate(null, 3)
    await flushPromises()
    expect(wrapper.get('[data-aip-result-text]').text()).toBe('cua cau chin')
    wrapper.unmount()
  })

  it('kết quả lô của câu bản ghi hiện; hàng lô của câu khác không hiện', async () => {
    const screen = await freshScreen()
    hoisted.runBatch.mockImplementation(
      async (_ids: number[], _set: string | null, onEvent: (e: unknown) => void) => {
        onEvent({ kind: 'token', segment_id: 3, text: 'lo ba' })
        onEvent({ kind: 'done', segment_id: 3, usage: null })
        onEvent({ kind: 'token', segment_id: 4, text: 'lo bon' })
        onEvent({ kind: 'done', segment_id: 4, usage: null })
        return { value: { state: 'done' }, error: null }
      },
    )
    const wrapper = await openWith(screen, record({ segment_id: 3 }))
    screen.commands.dispatch('ai.prompt_inspector.tab_result')
    await screen.batch.runAiTranslateBatch(null, [3, 4])
    await flushPromises()

    expect(wrapper.get('[data-aip-result-text]').text()).toBe('lo ba')
    expect(wrapper.text()).not.toContain('lo bon')
    wrapper.unmount()
  })

  it('hàm thuần: lượt đơn mới hơn lô thì thắng, lô mới hơn thì thắng, đang chạy luôn thắng', async () => {
    const { state } = await freshScreen()
    const base = {
      recordSegmentId: 1,
      singleSegmentId: 1,
      singleState: 'done',
      singleText: 'don',
      singleStamp: 5,
      batchText: 'lo',
      batchStamp: 4,
    }
    expect(state.aiPromptResultFor(base)).toEqual({ kind: 'text', text: 'don', from: 'single' })
    expect(state.aiPromptResultFor({ ...base, batchStamp: 6 })).toEqual({ kind: 'text', text: 'lo', from: 'batch' })
    expect(state.aiPromptResultFor({ ...base, singleState: 'generating', singleText: 'dang' })).toEqual({
      kind: 'generating',
      text: 'dang',
    })
    expect(state.aiPromptResultFor({ ...base, singleSegmentId: 2, batchText: null })).toEqual({ kind: 'empty' })
  })
})

describe('phần "đã cân nhắc nhưng không chèn" và lối sửa', () => {
  const PENDING: PendingGlossaryTermWire = { id: 41, source_term: 'dragon', start: 2, end: 8, tier: 'work' }

  function pendingSegments() {
    return [{ id: 1, source_text: 'A dragon roared.' }]
  }
  function pendingMark(over: Partial<{ id: number; tier: 'global' | 'work'; is_confirmed: boolean }> = {}) {
    return {
      start: 2,
      end: 8,
      tier: over.tier ?? 'work',
      is_confirmed: over.is_confirmed ?? false,
      translation: null,
      id: over.id ?? 41,
      source_term: 'dragon',
      han_viet_suggestion: null,
      han_viet_status: 'not_requested' as const,
      occurrence_count: null,
      term_origin: 'manual',
      category: 'other' as const,
    }
  }

  it('thuật ngữ chờ chốt khớp câu: hàng có lý do + "Chốt ngay"; bấm ⇒ đóng màn rồi dispatch glossary.confirm.focus', async () => {
    const screen = await freshScreen()
    screen.strip.syncGlossaryConfirmStripTarget(1, pendingSegments(), true, [pendingMark()])
    const wrapper = await openWith(screen, withLedger({ glossary: askedWith([PENDING]) }))

    const row = wrapper.get('[data-aip-excluded-kind="pending"]')
    expect(row.get('.aip-term-source').text()).toBe('dragon')
    expect(row.get('.aip-ewhy').text()).toBe(screen.i18n.t('ai.prompt_inspector.excluded_pending_why'))
    expect(row.find('[data-aip-fix="glossary_entry"]').exists()).toBe(false)
    await row.get('[data-aip-fix="confirm_now"]').trigger('click')
    await flushPromises()

    expect(screen.state.aiPromptInspectorIsOpen.value).toBe(false)
    expect(screen.focusStrip).toHaveBeenCalledTimes(1)
    expect(hoisted.segmentCalls).toEqual([])
    wrapper.unmount()
  })

  it('mục chờ chốt không còn là mục dải chốt đang hỏi (đã chốt, Để sau, rời câu, hoặc id khác): nút ẩn, hàng và lý do vẫn hiện', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, withLedger({ glossary: askedWith([PENDING]) }))

    // Dải chưa hỏi mục nào (mục đã hết chờ chốt / không còn trong câu).
    expect(wrapper.find('[data-aip-excluded-kind="pending"]').exists()).toBe(true)
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="confirm_now"]').exists()).toBe(false)
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="glossary_entry"]').exists()).toBe(true)

    // Dải đang hỏi mục KHÁC (id khác) ⇒ vẫn ẩn, vì "Chốt ngay" sẽ tới nhầm mục.
    screen.strip.syncGlossaryConfirmStripTarget(1, pendingSegments(), true, [pendingMark({ id: 99 })])
    await flushPromises()
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="confirm_now"]').exists()).toBe(false)
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="glossary_entry"]').exists()).toBe(true)

    // Cùng id nhưng đã chốt ⇒ dải không còn mục ⇒ ẩn.
    screen.strip.syncGlossaryConfirmStripTarget(1, pendingSegments(), true, [pendingMark({ is_confirmed: true })])
    await flushPromises()
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="confirm_now"]').exists()).toBe(false)
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="glossary_entry"]').exists()).toBe(true)

    // Đúng mục (id + tầng) đang chờ chốt ⇒ hiện.
    screen.strip.syncGlossaryConfirmStripTarget(1, pendingSegments(), true, [pendingMark()])
    await flushPromises()
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="confirm_now"]').exists()).toBe(true)

    // Cùng id khác tầng ⇒ ẩn.
    screen.strip.syncGlossaryConfirmStripTarget(1, pendingSegments(), true, [pendingMark({ tier: 'global' })])
    await flushPromises()
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="confirm_now"]').exists()).toBe(false)
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="glossary_entry"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('bản ghi của câu khác câu đang chọn: "Chốt ngay" ẩn dù dải đang hỏi đúng mục', async () => {
    const screen = await freshScreen()
    screen.strip.syncGlossaryConfirmStripTarget(1, pendingSegments(), true, [pendingMark()])
    const wrapper = await openWith(screen, withLedger({ glossary: askedWith([PENDING]) }))
    expect(wrapper.find('[data-aip-excluded-kind="pending"] button').exists()).toBe(true)

    screen.caret.value = 5
    await flushPromises()
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="confirm_now"]').exists()).toBe(false)
    expect(wrapper.find('[data-aip-excluded-kind="pending"] [data-aip-fix="glossary_entry"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('hơn 3 cặp TM khớp: hàng "N câu TM" + "Xem cả N"; bấm ⇒ đóng màn, chọn tab TM Concordance, tra câu nguồn của bản ghi, không ghi segment', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, withLedger({ tm: searched(4) }))

    const row = wrapper.get('[data-aip-excluded-kind="tm_omitted"]')
    expect(row.text()).toContain(screen.i18n.t('ai.prompt_inspector.excluded_tm_label', { count: '4' }))
    expect(row.get('button').text()).toBe(screen.i18n.t('ai.prompt_inspector.tm_see_all', { count: '4' }))
    await row.get('button').trigger('click')
    await flushPromises()

    expect(screen.state.aiPromptInspectorIsOpen.value).toBe(false)
    expect(screen.lookup.lookupTab.value).toBe('concordance')
    expect(hoisted.tmConcordance).toHaveBeenCalledTimes(1)
    expect(hoisted.tmConcordance).toHaveBeenCalledWith('SSSSSSSSSS')
    expect(hoisted.segmentCalls).toEqual([])
    wrapper.unmount()
  })

  it('bản ghi không mang câu nguồn: "Xem cả N" chỉ cảnh báo, không đóng màn, không tra', async () => {
    const screen = await freshScreen()
    const rec = withLedger({
      tm: searched(2),
      pieces: [{ kind: 'authored', text: 'chi co than' }],
    })
    const wrapper = await openWith(screen, rec)

    await wrapper.get('[data-aip-excluded-kind="tm_omitted"] button').trigger('click')
    await flushPromises()

    expect(screen.state.aiPromptInspectorIsOpen.value).toBe(true)
    expect(hoisted.tmConcordance).not.toHaveBeenCalled()
    expect(console.warn).toHaveBeenCalled()
    wrapper.unmount()
  })

  it('cùng một mục chờ chốt khớp hai lần trong câu: hai hàng, mỗi hàng có lối sửa', async () => {
    const screen = await freshScreen()
    const twice = [PENDING, { ...PENDING, start: 12, end: 18 }]
    const wrapper = await openWith(screen, withLedger({ glossary: askedWith(twice) }))
    const rows = wrapper.findAll('[data-aip-excluded-kind="pending"]')
    expect(rows).toHaveLength(2)
    for (const row of rows) expect(row.find('[data-aip-fix]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('TM không vượt giới hạn và không mục chờ chốt: ghi chú rỗng có chữ, không hàng nào', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(screen, withLedger({ glossary: askedWith([]), tm: searched(0) }))
    expect(wrapper.find('[data-aip-excluded-kind]').exists()).toBe(false)
    expect(wrapper.get('[data-aip-excluded-empty]').text()).toBe(screen.i18n.t('ai.prompt_inspector.excluded_empty'))
    wrapper.unmount()
  })

  it('"Xem mục" và "Mở prompt Tác phẩm": đóng màn rồi mở đúng đích', async () => {
    const screen = await freshScreen()
    const wrapper = await openWith(
      screen,
      withLedger({
        glossary: {
          kind: 'asked',
          injected: [],
          pending_matched: [],
          suppressed_by_pending_overlap: [{ source_term: 'dog', translation: 'cho', start: 4, end: 7, tier: 'global' }],
        },
      }),
    )
    await wrapper.get('[data-aip-excluded-kind="suppressed"] button').trigger('click')
    await flushPromises()
    expect(screen.openGlossaryManage).toHaveBeenCalledTimes(1)
    expect(screen.state.aiPromptInspectorIsOpen.value).toBe(false)
    wrapper.unmount()

    const second = await openWith(screen, record())
    const openPrompts = second
      .findAll('button')
      .find((b) => b.text() === screen.i18n.t('command.ai.prompt_inspector.open_work_prompts'))
    expect(openPrompts).toBeDefined()
    await openPrompts?.trigger('click')
    await flushPromises()
    expect(screen.openPromptLibrary).toHaveBeenCalledTimes(1)
    second.unmount()
  })
})

describe('Chép', () => {
  function stubClipboard(writeText: (text: string) => Promise<void>) {
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
  }

  it('chép đúng toàn bộ record.prompt, nút báo "Đã chép"', async () => {
    const screen = await freshScreen()
    const writeText = vi.fn(async () => undefined)
    stubClipboard(writeText)
    const rec = record({ prompt: '  dong 1\n\tdong 2 \u{1F409}  ' })
    const wrapper = await openWith(screen, rec)

    await wrapper.get('[data-aip-copy]').trigger('click')
    await flushPromises()

    expect(writeText).toHaveBeenCalledWith(rec.prompt)
    expect(wrapper.get('[data-aip-copy]').attributes('data-aip-copy-state')).toBe('copied')
    expect(wrapper.get('[data-aip-copy]').text()).toBe(screen.i18n.t('ai.prompt_inspector.copy_done'))
    wrapper.unmount()
  })

  it('clipboard từ chối: báo lỗi trên nút, không im lặng', async () => {
    const screen = await freshScreen()
    stubClipboard(async () => {
      throw new Error('denied')
    })
    const wrapper = await openWith(screen, record())

    await wrapper.get('[data-aip-copy]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-aip-copy]').attributes('data-aip-copy-state')).toBe('failed')
    expect(wrapper.get('[data-aip-copy]').text()).toBe(screen.i18n.t('ai.prompt_inspector.copy_failed'))
    expect(console.error).toHaveBeenCalled()
    wrapper.unmount()
  })

  it('mở lại màn sau khi chép: nút về trạng thái ban đầu', async () => {
    const screen = await freshScreen()
    stubClipboard(async () => undefined)
    const wrapper = await openWith(screen, record())
    await wrapper.get('[data-aip-copy]').trigger('click')
    await flushPromises()
    screen.state.closeAiPromptInspector()
    screen.state.openAiPromptInspector()
    await flushPromises()
    expect(wrapper.get('[data-aip-copy]').attributes('data-aip-copy-state')).toBe('idle')
    wrapper.unmount()
  })
})
