/**
 * Panel Đề xuất AI: hàng lô có chữ + token + trạng thái, khối lỗi
 * ba dòng, Huỷ lô / Bỏ lô, đầu panel "nhà cung cấp · mô hình", trạng thái chưa cấu hình, và
 * đưa cả lô sang Bản dịch qua `editorPanelState.ts` THẬT.
 *
 * Mock ở BIÊN IPC: `config/aitranslate` (lô), `config/aiconfig::aiConfigGet` (Rust quyết
 * `ready`), `config/segment` (`readOpenChapterSegments`, `promoteAiTranslation`,
 * `saveSegmentTargets`). Delta Glossary nằm ở `aiPromptInspector.test.ts`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import { FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS, recordSave, resetRecorder, saveCalls } from './support/segmentFixture'
import type { Component } from 'vue'
import type { AiTranslateBatchEventWire, AiTranslateOutcomeWire } from '../../src/config/aitranslate'
import type { ChapterSegment, PromoteAiTranslationOutcome, ReadChapterSegmentsResult } from '../../src/config/segment'
import type { CommandDeps } from '../../src/commands'
import type { IpcError } from '../../src/i18n'

const runSegmentMock = vi.fn()
const runBatchMock = vi.fn()
const cancelMock = vi.fn()
const aiConfigGetMock = vi.fn()
const promoteMock = vi.fn()

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: (...args: unknown[]) => runSegmentMock(...args),
  cancelAiTranslateCall: (...args: unknown[]) => cancelMock(...args),
  runAiTranslateBatchCall: (...args: unknown[]) => runBatchMock(...args),
}))

vi.mock('../../src/config/aiconfig', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/aiconfig')>()
  return { ...actual, aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args) }
})

const BATCH_SIZE = 20
const BATCH_IDS = Array.from({ length: BATCH_SIZE }, (_, i) => 101 + i)

function batchSegments(): ChapterSegment[] {
  return BATCH_IDS.map((id, i) => ({
    ...FIXTURE_SEGMENTS[0],
    id,
    ord: i + 1,
    source_text: `Nguon cau ${i + 1}`,
    target_text: `Ban cu ${i + 1}`,
    status: 'draft',
    translation_origin: 'user',
  }))
}

async function readTwentySegments(): Promise<ReadChapterSegmentsResult> {
  return {
    loaded: {
      chapter_id: FIXTURE_CHAPTER_ID,
      segments: batchSegments(),
      caret_segment_id: null,
      assets: [],
      assets_dir: '',
      tm_filled_segment_ids: [],
      tm_prefill: { kind: 'ran' },
    },
    error: null,
  }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: () => readTwentySegments(),
    saveSegmentTargets: recordSave,
    promoteAiTranslation: (...args: unknown[]) => promoteMock(...args),
  }
})

type RunResult = { value: AiTranslateOutcomeWire | null; error: IpcError | null }
type OnEvent = (event: AiTranslateBatchEventWire) => void

const PROVIDER_UNREACHABLE_ON_106: IpcError = {
  code: 'ai_translate.provider_unreachable',
  message_key: 'err.ai_translate.provider_unreachable',
  params: { segment_id: '106' },
  retryable: true,
}

const PROVIDER_REFUSED_ON_106: IpcError = {
  code: 'ai_translate.provider_refused',
  message_key: 'err.ai_translate.provider_refused',
  params: { segment_id: '106', status: '500' },
  retryable: false,
}

const UNREACHABLE_TEXT =
  'Không kết nối được tới nhà cung cấp AI — chưa có kết quả mới nào. Những đoạn đã nhận trước đó vẫn còn trên màn hình.'

function pendingBatchRun(): { emit: OnEvent; settle: (result: RunResult) => void } {
  let capturedOnEvent: OnEvent = () => {
    throw new Error('onEvent chưa được gán')
  }
  let resolve: (result: RunResult) => void = () => {
    throw new Error('resolve chưa được gán')
  }
  runBatchMock.mockImplementation((_ids: number[], _set: string | null, onEvent: OnEvent) => {
    capturedOnEvent = onEvent
    return new Promise<RunResult>((res) => {
      resolve = res
    })
  })
  return { emit: (e) => capturedOnEvent(e), settle: (r) => resolve(r) }
}

function usage(total: number) {
  return { prompt_tokens: total - 1, completion_tokens: 1, total_tokens: total, cost_usd: null }
}

function readyConfig(ready: boolean | null, provider = 'Ollama', model = 'qwen2.5:14b') {
  return {
    fields: [
      { field: 'provider', value: provider, tier: 'global', shadowed: null },
      { field: 'endpoint', value: 'http://localhost:11434', tier: 'global', shadowed: null },
      { field: 'model', value: model, tier: 'global', shadowed: null },
    ],
    workTierAvailable: false,
    keyConfigured: ready,
    ready,
    error: null,
  }
}

const openSettingsMock = vi.fn()
const openInspectorMock = vi.fn()

async function freshPanel() {
  vi.resetModules()
  runSegmentMock.mockReset()
  runBatchMock.mockReset()
  cancelMock.mockReset()
  promoteMock.mockReset()
  aiConfigGetMock.mockReset()
  openSettingsMock.mockReset()
  openInspectorMock.mockReset()
  aiConfigGetMock.mockResolvedValue(readyConfig(true))
  resetRecorder()

  const commands = await import('../../src/commands')
  const editorState = await import('../../src/panels/editorPanelState')
  const batchState = await import('../../src/aiTranslateBatchState')
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default
  const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')

  editorState.resetEditorPanel()
  await editorState.ensureSegmentsLoaded()

  commands.installCommands({
    ...aiTranslateHandlers,
    confirmPendingPromote: editorState.confirmPendingPromote,
    cancelPendingPromote: editorState.cancelPendingPromote,
    openSettings: openSettingsMock,
    openAiPromptInspector: openInspectorMock,
  } as unknown as CommandDeps)

  return { commands, editorState, batchState, AiTranslationPanel }
}

async function mountPanel(AiTranslationPanel: Component) {
  const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
  await flushPromises()
  await wrapper.vm.$nextTick()
  return wrapper
}

type Fresh = Awaited<ReturnType<typeof freshPanel>>

/** Lô 20 câu: 5 câu đầu xong (token 10..50), câu 6 đang chảy dở rồi lỗi, 14 câu chưa gọi. */
async function startBatchFailingAtSix(fresh: Fresh, error: IpcError) {
  const fake = pendingBatchRun()
  const wrapper = await mountPanel(fresh.AiTranslationPanel)
  void fresh.batchState.runAiTranslateBatch(null, BATCH_IDS)
  await flushPromises()
  for (let i = 0; i < 5; i += 1) {
    const id = BATCH_IDS[i]
    fake.emit({ kind: 'token', segment_id: id, text: `Dich ${i + 1}` })
    fake.emit({ kind: 'done', segment_id: id, usage: usage((i + 1) * 10) })
  }
  fake.emit({ kind: 'token', segment_id: BATCH_IDS[5], text: 'Dich dở' })
  fake.settle({ value: null, error })
  await flushPromises()
  await wrapper.vm.$nextTick()
  return { wrapper, fake }
}

function targetsSnapshot(fresh: Fresh): string[] {
  return fresh.editorState.editorSegments.value.map((s) => `${s.id}:${s.target_text}:${s.translation_origin}`)
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('hàng lô 20 câu, lỗi ở câu 6 (matrix: "Lô chạy dở có một lỗi")', () => {
  it('mỗi hàng nêu số câu · chữ · token (chỉ hàng xong) · chữ trạng thái; hàng chưa gọi hiện câu nguồn', async () => {
    const fresh = await freshPanel()
    const { wrapper } = await startBatchFailingAtSix(fresh, PROVIDER_UNREACHABLE_ON_106)

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows).toHaveLength(BATCH_SIZE)

    const first = rows[0]
    expect(first.get('[data-ai-translate-batch-row-no]').text()).toBe('1')
    expect(first.get('[data-ai-translate-batch-row-text]').text()).toBe('Dich 1')
    expect(first.get('[data-ai-translate-batch-row-tokens]').text()).toBe('10 token')
    expect(first.text()).toContain('Xong')
    expect(rows[4].get('[data-ai-translate-batch-row-tokens]').text()).toBe('50 token')

    const failed = rows[5]
    expect(failed.attributes('data-ai-translate-batch-row-status')).toBe('error')
    expect(failed.attributes('data-ai-translate-batch-row-segment-id')).toBe('106')
    expect(failed.get('[data-ai-translate-batch-row-text]').text()).toBe('Dich dở')
    expect(failed.find('[data-ai-translate-batch-row-tokens]').exists()).toBe(false)
    expect(failed.text()).toContain('Lỗi')

    for (let i = 6; i < BATCH_SIZE; i += 1) {
      expect(rows[i].attributes('data-ai-translate-batch-row-status')).toBe('pending')
      expect(rows[i].get('[data-ai-translate-batch-row-no]').text()).toBe(String(i + 1))
      expect(rows[i].get('[data-ai-translate-batch-row-text]').text()).toBe(`Nguon cau ${i + 1}`)
      expect(rows[i].find('[data-ai-translate-batch-row-tokens]').exists()).toBe(false)
      expect(rows[i].text()).toContain('Chưa tới lượt')
    }
    wrapper.unmount()
  })

  it('khối lỗi ba dòng đúng chữ, ba nút Thử lại từ câu 6 · Đưa 5 câu · Bỏ lô, và Bản dịch không đổi', async () => {
    const fresh = await freshPanel()
    const before = targetsSnapshot(fresh)
    const { wrapper } = await startBatchFailingAtSix(fresh, PROVIDER_UNREACHABLE_ON_106)

    const alert = wrapper.get('[data-ai-translate-batch-alert]')
    expect(alert.attributes('role')).toBe('alert')
    expect(alert.get('[data-ai-batch-error-stopped]').text()).toBe('Lô dừng ở câu 6.')
    expect(alert.get('[data-ai-batch-error-intact]').text()).toBe('Câu 1–5 còn nguyên trong panel này.')
    expect(alert.get('[data-ai-batch-error-cause]').text()).toBe(`Câu 6: ${UNREACHABLE_TEXT}`)
    expect(alert.get('[data-ai-batch-error-not-called]').text()).toBe('Câu 7–20 chưa gọi, chưa tính phí.')

    expect(wrapper.get('[data-ai-translate-batch-retry]').text()).toBe('Thử lại từ câu 6')
    expect(wrapper.get('[data-ai-translate-promote-batch]').text()).toBe('Đưa 5 câu đã xong sang Bản dịch')
    expect(wrapper.get('[data-ai-translate-promote-batch]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-ai-translate-batch-discard]').text()).toBe('Bỏ lô')

    expect(promoteMock).not.toHaveBeenCalled()
    expect(targetsSnapshot(fresh)).toEqual(before)
    wrapper.unmount()
  })

  it('lỗi KHÔNG retryable ⇒ ẩn Thử lại, còn Đưa sang và Bỏ lô', async () => {
    const fresh = await freshPanel()
    const { wrapper } = await startBatchFailingAtSix(fresh, PROVIDER_REFUSED_ON_106)

    expect(wrapper.find('[data-ai-translate-batch-retry]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-promote-batch]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-translate-batch-discard]').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('Huỷ lô (matrix: "Huỷ giữa lô")', () => {
  it('nhãn nút Huỷ là "Huỷ lô" khi lô đang chạy; huỷ dừng ở ranh giới câu, giữ chữ và token hàng xong, bỏ chữ dở', async () => {
    const fresh = await freshPanel()
    const fake = pendingBatchRun()
    const wrapper = await mountPanel(fresh.AiTranslationPanel)
    expect(wrapper.get('[data-ai-translate-cancel]').text()).not.toBe('Huỷ lô')

    void fresh.batchState.runAiTranslateBatch(null, BATCH_IDS)
    await flushPromises()
    fake.emit({ kind: 'token', segment_id: 101, text: 'Xong mot' })
    fake.emit({ kind: 'done', segment_id: 101, usage: usage(30) })
    fake.emit({ kind: 'token', segment_id: 102, text: 'Dang sinh dở' })
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-cancel]').text()).toBe('Huỷ lô')
    await wrapper.get('[data-ai-translate-cancel]').trigger('click')
    expect(cancelMock).toHaveBeenCalledTimes(1)

    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('[data-ai-translate-batch-rows] li')
    expect(rows[0].attributes('data-ai-translate-batch-row-status')).toBe('done')
    expect(rows[0].get('[data-ai-translate-batch-row-text]').text()).toBe('Xong mot')
    expect(rows[0].get('[data-ai-translate-batch-row-tokens]').text()).toBe('30 token')
    expect(rows[1].attributes('data-ai-translate-batch-row-status')).toBe('cancelled')
    expect(rows[1].get('[data-ai-translate-batch-row-text]').text()).toBe('Nguon cau 2')
    wrapper.unmount()
  })
})

describe('Bỏ lô (matrix: "Bỏ lô")', () => {
  it('không hàng nào ⇒ nút ẩn', async () => {
    const fresh = await freshPanel()
    const wrapper = await mountPanel(fresh.AiTranslationPanel)
    expect(wrapper.find('[data-ai-translate-batch-discard]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('lô đang chạy ⇒ huỷ lượt đang bay, xoá hàng · tiến độ · tổng token · lỗi; Bản dịch không đổi; không câu nào bị gọi thêm', async () => {
    const fresh = await freshPanel()
    const before = targetsSnapshot(fresh)
    const fake = pendingBatchRun()
    const wrapper = await mountPanel(fresh.AiTranslationPanel)

    void fresh.batchState.runAiTranslateBatch(null, BATCH_IDS)
    await flushPromises()
    fake.emit({ kind: 'token', segment_id: 101, text: 'Mot' })
    fake.emit({ kind: 'done', segment_id: 101, usage: usage(30) })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-ai-translate-batch-progress]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-translate-batch-usage]').exists()).toBe(true)

    await wrapper.get('[data-ai-translate-batch-discard]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(wrapper.find('[data-ai-translate-batch-rows]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-batch-progress]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-batch-usage]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-batch-alert]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-batch-discard]').exists()).toBe(false)
    expect(fresh.batchState.aiTranslateBatchRows.value).toEqual([])

    // Khung và lời đáp đến muộn của lượt đã bỏ không được dựng lại hàng nào.
    fake.emit({ kind: 'token', segment_id: 102, text: 'Muon' })
    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()
    expect(fresh.batchState.aiTranslateBatchRows.value).toEqual([])
    expect(runBatchMock).toHaveBeenCalledTimes(1)
    expect(promoteMock).not.toHaveBeenCalled()
    expect(targetsSnapshot(fresh)).toEqual(before)
    wrapper.unmount()
  })

  it('lô đã lỗi ⇒ Bỏ lô xoá khối lỗi và mọi hàng, không gọi huỷ (không còn gì đang bay)', async () => {
    const fresh = await freshPanel()
    const { wrapper } = await startBatchFailingAtSix(fresh, PROVIDER_UNREACHABLE_ON_106)

    await wrapper.get('[data-ai-translate-batch-discard]').trigger('click')
    await wrapper.vm.$nextTick()

    expect(cancelMock).not.toHaveBeenCalled()
    expect(wrapper.find('[data-ai-translate-batch-alert]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-batch-rows]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('đầu panel và trạng thái chưa cấu hình (matrix: "Đã cấu hình", "Chưa cấu hình")', () => {
  it('ready = true ⇒ "Ollama · qwen2.5:14b" cạnh tiêu đề, MỘT link Xem prompt mở lớp phủ hiện có', async () => {
    const fresh = await freshPanel()
    aiConfigGetMock.mockResolvedValue(readyConfig(true, 'Ollama', 'qwen2.5:14b'))
    const wrapper = await mountPanel(fresh.AiTranslationPanel)

    expect(wrapper.get('[data-ai-panel-provider]').text()).toBe('Ollama · qwen2.5:14b')
    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(false)
    expect(wrapper.findAll('[data-ai-prompt-inspector-open]')).toHaveLength(1)
    await wrapper.get('[data-ai-prompt-inspector-open]').trigger('click')
    expect(openInspectorMock).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('đọc cấu hình lỗi ⇒ không dòng nhà cung cấp, không khối chưa cấu hình, panel vẫn dùng được', async () => {
    const fresh = await freshPanel()
    aiConfigGetMock.mockResolvedValue({
      fields: null,
      workTierAvailable: false,
      keyConfigured: null,
      ready: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    const wrapper = await mountPanel(fresh.AiTranslationPanel)

    expect(wrapper.find('[data-ai-panel-provider]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-key-unchecked]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-translate-run]').exists()).toBe(true)
    expect(wrapper.find('[data-ai-prompt-inspector-open]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('ready = false ⇒ lời mời (tiêu đề + hai đoạn + lối mở Cài đặt) KHÔNG role="alert", không dòng nhà cung cấp, nút Dịch vẫn tới ai.translate.run', async () => {
    const fresh = await freshPanel()
    aiConfigGetMock.mockResolvedValue(readyConfig(false, 'Ollama', 'qwen2.5:14b'))
    runSegmentMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    const wrapper = await mountPanel(fresh.AiTranslationPanel)

    const block = wrapper.get('[data-ai-unconfigured]')
    expect(block.text()).toContain('Chưa cấu hình AI')
    expect(block.text()).toContain('Mọi năng lực khác vẫn chạy đầy đủ')
    expect(block.text()).toContain('Nhà cung cấp cục bộ như Ollama / LM Studio vẫn cần một khoá bất kỳ.')
    expect(block.attributes('role')).toBeUndefined()
    expect(block.find('[role="alert"]').exists()).toBe(false)
    expect(block.classes().join(' ')).not.toContain('error')
    expect(wrapper.find('[data-ai-panel-provider]').exists()).toBe(false)
    expect(wrapper.find('[data-ai-key-unchecked]').exists()).toBe(false)

    await block.get('[data-ai-unconfigured-open-settings]').trigger('click')
    expect(openSettingsMock).toHaveBeenCalledTimes(1)

    fresh.editorState.setEditorCaret(101)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()
    expect(runSegmentMock).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('ready = null (khoá không hỏi được) ⇒ dòng riêng "không kiểm tra được khoá", KHÔNG phải lời mời chưa cấu hình', async () => {
    const fresh = await freshPanel()
    aiConfigGetMock.mockResolvedValue(readyConfig(null))
    const wrapper = await mountPanel(fresh.AiTranslationPanel)

    expect(wrapper.find('[data-ai-unconfigured]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-key-unchecked]').text()).toContain('khoá')
    expect(wrapper.find('[data-ai-panel-provider]').exists()).toBe(false)
    expect(wrapper.find('[role="alert"]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('đưa cả lô sang Bản dịch (matrix: "Đưa cả lô sang", "Đưa sang khi lô đang chạy hoặc không hàng xong")', () => {
  const NEEDS_DRAFT = new Set([103, 104])

  function outcomeFor(
    segmentId: number,
    text: string,
    force: boolean,
  ): { outcome: PromoteAiTranslationOutcome | null; error: IpcError | null } {
    const draft = NEEDS_DRAFT.has(segmentId) && !force
    return {
      outcome: {
        segment_id: segmentId,
        target_text: draft ? '' : text,
        translation_origin: 'other',
        status: 'draft',
        needs_confirmation: draft,
        unsigned_draft: draft ? `Nhap chua ky ${segmentId}` : null,
      },
      error: null,
    }
  }

  async function batchWithFiveDone(fresh: Fresh) {
    const fake = pendingBatchRun()
    const wrapper = await mountPanel(fresh.AiTranslationPanel)
    void fresh.batchState.runAiTranslateBatch(null, BATCH_IDS.slice(0, 5))
    await flushPromises()
    for (let i = 0; i < 5; i += 1) {
      fake.emit({ kind: 'token', segment_id: BATCH_IDS[i], text: `Moi ${i + 1}` })
      fake.emit({ kind: 'done', segment_id: BATCH_IDS[i], usage: usage(10) })
    }
    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()
    return wrapper
  }

  it('5 hàng xong, 2 câu có bản nháp chưa ký ⇒ 3 câu ghi ngay, MỘT câu hỏi; đồng ý ⇒ 2 câu còn lại ghi với force = true (cả 5 mang origin other)', async () => {
    const fresh = await freshPanel()
    promoteMock.mockImplementation((id: number, text: string, force: boolean) =>
      Promise.resolve(outcomeFor(id, text, force)),
    )
    const wrapper = await batchWithFiveDone(fresh)

    await wrapper.get('[data-ai-translate-promote-batch]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promoteMock.mock.calls).toEqual([
      [101, 'Moi 1', false],
      [102, 'Moi 2', false],
      [103, 'Moi 3', false],
      [104, 'Moi 4', false],
      [105, 'Moi 5', false],
    ])
    expect(wrapper.findAll('[data-ai-translate-promote-batch-confirm]')).toHaveLength(1)
    expect(wrapper.findAll('.ai-promote-confirm')).toHaveLength(1)
    const drafts = wrapper.findAll('.ai-promote-draft').map((d) => d.text())
    expect(drafts).toEqual(['Nhap chua ky 103', 'Nhap chua ky 104'])

    const segments = fresh.editorState.editorSegments.value
    for (const id of [101, 102, 105]) {
      const segment = segments.find((s) => s.id === id)
      expect(segment?.target_text).toBe(`Moi ${id - 100}`)
      expect(segment?.translation_origin).toBe('other')
    }
    for (const id of [103, 104]) {
      expect(segments.find((s) => s.id === id)?.target_text).toBe(`Ban cu ${id - 100}`)
    }

    await wrapper.get('[data-ai-translate-promote-confirm]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promoteMock.mock.calls.slice(5)).toEqual([
      [103, 'Moi 3', true],
      [104, 'Moi 4', true],
    ])
    for (const id of [101, 102, 103, 104, 105]) {
      const segment = fresh.editorState.editorSegments.value.find((s) => s.id === id)
      expect(segment?.target_text).toBe(`Moi ${id - 100}`)
      expect(segment?.translation_origin).toBe('other')
    }
    expect(wrapper.find('[data-ai-translate-promote-batch-confirm]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-translate-promote-batch-report]').text()).toBe('Đã đưa 5 câu sang Bản dịch.')
    wrapper.unmount()
  })

  it('từ chối câu hỏi ⇒ không có lượt force = true nào, hai câu nháp giữ nguyên, báo 3 câu đã ghi / 2 câu bỏ', async () => {
    const fresh = await freshPanel()
    promoteMock.mockImplementation((id: number, text: string, force: boolean) =>
      Promise.resolve(outcomeFor(id, text, force)),
    )
    const wrapper = await batchWithFiveDone(fresh)

    await wrapper.get('[data-ai-translate-promote-batch]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-ai-translate-promote-cancel]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promoteMock.mock.calls.filter((c) => c[2] === true)).toHaveLength(0)
    expect(fresh.editorState.editorSegments.value.find((s) => s.id === 103)?.target_text).toBe('Ban cu 3')
    expect(wrapper.get('[data-ai-translate-promote-batch-report]').text()).toBe(
      'Đã đưa 3 câu sang Bản dịch, bỏ 2 câu.',
    )
    wrapper.unmount()
  })

  it('Bỏ lô khi câu hỏi đưa cả lô đang chờ ⇒ câu hỏi biến mất, không lượt force = true nào', async () => {
    const fresh = await freshPanel()
    promoteMock.mockImplementation((id: number, text: string, force: boolean) =>
      Promise.resolve(outcomeFor(id, text, force)),
    )
    const wrapper = await batchWithFiveDone(fresh)

    await wrapper.get('[data-ai-translate-promote-batch]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-ai-translate-promote-batch-confirm]').exists()).toBe(true)

    await wrapper.get('[data-ai-translate-batch-discard]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-ai-translate-promote-batch-confirm]').exists()).toBe(false)
    expect(fresh.editorState.editorPendingPromoteBatch.value).toBeNull()
    expect(promoteMock.mock.calls.filter((c) => c[2] === true)).toHaveLength(0)
    expect(fresh.editorState.editorSegments.value.find((s) => s.id === 103)?.target_text).toBe('Ban cu 3')
    wrapper.unmount()
  })

  it('một câu không còn trong Chương ⇒ câu đó bị bỏ, các câu khác vẫn ghi; báo số câu đã ghi / bị bỏ', async () => {
    const fresh = await freshPanel()
    const missing: IpcError = {
      code: 'segment.not_found',
      message_key: 'err.segment.not_found',
      params: {},
      retryable: false,
    }
    promoteMock.mockImplementation((id: number, text: string, force: boolean) =>
      id === 102 ? Promise.resolve({ outcome: null, error: missing }) : Promise.resolve(outcomeFor(id, text, force)),
    )
    const wrapper = await batchWithFiveDone(fresh)

    await wrapper.get('[data-ai-translate-promote-batch]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-ai-translate-promote-confirm]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    const segments = fresh.editorState.editorSegments.value
    expect(segments.find((s) => s.id === 101)?.target_text).toBe('Moi 1')
    expect(segments.find((s) => s.id === 102)?.target_text).toBe('Ban cu 2')
    expect(segments.find((s) => s.id === 105)?.target_text).toBe('Moi 5')
    expect(wrapper.get('[data-ai-translate-promote-batch-report]').text()).toBe(
      'Đã đưa 4 câu sang Bản dịch, bỏ 1 câu.',
    )
    wrapper.unmount()
  })

  it('gõ vào câu đang chờ hỏi rồi đồng ý ⇒ flush chạy TRƯỚC lượt force = true', async () => {
    const fresh = await freshPanel()
    const savesWhenForced: number[] = []
    promoteMock.mockImplementation((id: number, text: string, force: boolean) => {
      if (force) savesWhenForced.push(saveCalls.length)
      return Promise.resolve(outcomeFor(id, text, force))
    })
    const wrapper = await batchWithFiveDone(fresh)

    await wrapper.get('[data-ai-translate-promote-batch]').trigger('click')
    await flushPromises()
    const savesBeforeTyping = saveCalls.length
    fresh.editorState.noteEditorEdit(103, 'Gõ thêm lúc đang hỏi')
    await wrapper.get('[data-ai-translate-promote-confirm]').trigger('click')
    await flushPromises()

    expect(savesWhenForced).toHaveLength(2)
    for (const seen of savesWhenForced) expect(seen).toBeGreaterThan(savesBeforeTyping)
    wrapper.unmount()
  })

  it('lô đang chạy ⇒ nút khoá và dispatch không gọi promote; không hàng xong ⇒ nút khoá', async () => {
    const fresh = await freshPanel()
    const fake = pendingBatchRun()
    const wrapper = await mountPanel(fresh.AiTranslationPanel)
    void fresh.batchState.runAiTranslateBatch(null, BATCH_IDS.slice(0, 3))
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-translate-promote-batch]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-ai-translate-promote-batch]').text()).toBe('Đưa 0 câu đã xong sang Bản dịch')

    fake.emit({ kind: 'token', segment_id: 101, text: 'Mot' })
    fake.emit({ kind: 'done', segment_id: 101, usage: usage(5) })
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-promote-batch]').attributes('disabled')).toBeDefined()

    const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
    aiTranslateHandlers.promoteAiTranslateBatch()
    await flushPromises()
    expect(promoteMock).not.toHaveBeenCalled()

    fake.settle({ value: { state: 'done', usage: null }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-promote-batch]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })
})
