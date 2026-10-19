/**
 * Decision 14 (spec 11.5) — vòng `needs_confirmation`/`force` của `promote_ai_translation`, đi
 * qua NGƯỜI GỌI THẬT: `editorPanelState.ts::promoteAiTranslationToEditor`/`confirmPendingPromote`/
 * `cancelPendingPromote`, và bề mặt thật `AiTranslationPanel.vue` (câu hỏi `.ai-promote-confirm`,
 * `data-ai-translate-promote-confirm`/`-cancel`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * KHÁC `aiTranslate.test.ts::freshPanel()` — Ở ĐÂY `editorPanelState.ts` KHÔNG BỊ MOCK
 * ─────────────────────────────────────────────────────────────────────────────
 * `aiTranslate.test.ts`/`aiTranslationPromoteErrorAlert.test.ts` thay cả `editorPanelState.ts`
 * bằng một factory tay — đủ cho phạm vi của chúng (dịch AI, hiện lỗi), nhưng nó đi VÒNG QUA
 * chính hàm cần canh ở đây (Decision 14 — "a vitest through the real caller in
 * `editorPanelState.ts`"). Tệp này để `editorPanelState.ts` SỐNG THẬT và giả duy nhất ở BIÊN
 * dây: `config/segment.ts::promoteAiTranslation` — cùng khuôn `gridPanelRowErrorPriority.test.ts`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import { failNextSave, readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { CommandDeps } from '../../src/commands'
import type { PromoteAiTranslationOutcome } from '../../src/config/segment'

const promoteAiTranslationMock = vi.fn()

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: vi.fn(),
  cancelAiTranslateCall: vi.fn(),
}))

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    promoteAiTranslation: (...args: unknown[]) => promoteAiTranslationMock(...args),
    saveSegmentTargets: recordSave,
    readOpenChapterSegments: readFixture,
  }
})

function outcomeOf(patch: Partial<PromoteAiTranslationOutcome>): { outcome: PromoteAiTranslationOutcome; error: null } {
  return {
    outcome: {
      segment_id: 5,
      target_text: '',
      translation_origin: 'other',
      status: 'draft',
      needs_confirmation: false,
      unsigned_draft: null,
      ...patch,
    },
    error: null,
  }
}

/** Cùng khuôn `aiTranslate.test.ts::freshPanel()`, thu gọn cho đúng phạm vi Decision 14. */
async function freshPanel() {
  vi.resetModules()
  promoteAiTranslationMock.mockReset()

  const commands = await import('../../src/commands')
  const editorPanelState = await import('../../src/panels/editorPanelState')
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

  commands.installCommands({
    confirmPendingPromote: editorPanelState.confirmPendingPromote,
    cancelPendingPromote: editorPanelState.cancelPendingPromote,
  } as CommandDeps)

  return { editorPanelState, AiTranslationPanel }
}

function mountPanel(AiTranslationPanel: Awaited<ReturnType<typeof freshPanel>>['AiTranslationPanel']) {
  return mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
}

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('Decision 14 — promote giữ lượt ghi khi cần hỏi, rồi gọi lại với force', () => {
  it('needs_confirmation ⇒ hỏi thật trên panel; bấm ĐỒNG Ý ⇒ lượt gọi thứ hai mang force: true', async () => {
    const { editorPanelState, AiTranslationPanel } = await freshPanel()
    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()

    promoteAiTranslationMock.mockResolvedValueOnce(
      outcomeOf({ needs_confirmation: true, unsigned_draft: 'Bản đang soạn, sắp mất' }),
    )

    // Lượt gọi ĐẦU — qua chính hàm sản phẩm, không một hàm test tự chế.
    void editorPanelState.promoteAiTranslationToEditor(5, 'Kết quả AI')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)
    expect(promoteAiTranslationMock).toHaveBeenNthCalledWith(1, 5, 'Kết quả AI', false)
    expect(editorPanelState.editorPendingPromote.value).toEqual({
      segmentId: 5,
      text: 'Kết quả AI',
      draft: 'Bản đang soạn, sắp mất',
    })

    // Câu hỏi phải THẬT SỰ hiện trên bề mặt — không chỉ đúng ở ô nhớ.
    const draft = wrapper.find('.ai-promote-draft')
    expect(draft.exists()).toBe(true)
    expect(draft.text()).toBe('Bản đang soạn, sắp mất')

    promoteAiTranslationMock.mockResolvedValueOnce(outcomeOf({ target_text: 'Kết quả AI' }))
    const confirmButton = wrapper.get('[data-ai-translate-promote-confirm]')
    await confirmButton.trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(2)
    expect(promoteAiTranslationMock).toHaveBeenNthCalledWith(2, 5, 'Kết quả AI', true)
    expect(editorPanelState.editorPendingPromote.value).toBeNull()
    expect(wrapper.find('.ai-promote-confirm').exists()).toBe(false)

    wrapper.unmount()
  })

  it('needs_confirmation ⇒ bấm TỪ CHỐI ⇒ câu hỏi đóng, và KHÔNG một lượt gọi thứ hai nào', async () => {
    const { editorPanelState, AiTranslationPanel } = await freshPanel()
    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()

    promoteAiTranslationMock.mockResolvedValueOnce(
      outcomeOf({ segment_id: 7, needs_confirmation: true, unsigned_draft: 'Bản khác, sắp mất' }),
    )

    void editorPanelState.promoteAiTranslationToEditor(7, 'Kết quả AI khác')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)
    expect(editorPanelState.editorPendingPromote.value).not.toBeNull()

    const cancelButton = wrapper.get('[data-ai-translate-promote-cancel]')
    await cancelButton.trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(editorPanelState.editorPendingPromote.value).toBeNull()
    expect(wrapper.find('.ai-promote-confirm').exists()).toBe(false)
    // Giữ bản đang soạn nghĩa là KHÔNG một lượt IPC thứ hai nào — lượt trên là toàn bộ.
    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })
})

describe('promote lên câu đã ký — webview phản chiếu status của Rust', () => {
  it('câu đã ký + outcome status draft ⇒ lưới hiện draft cùng văn bản mới', async () => {
    const { editorPanelState } = await freshPanel()
    await editorPanelState.ensureSegmentsLoaded()
    expect(editorPanelState.editorSegments.value.find((s) => s.id === 11)?.status).toBe('confirmed')

    promoteAiTranslationMock.mockResolvedValueOnce(
      outcomeOf({ segment_id: 11, target_text: 'Kết quả AI', status: 'draft' }),
    )
    await editorPanelState.promoteAiTranslationToEditor(11, 'Kết quả AI')

    const segment = editorPanelState.editorSegments.value.find((s) => s.id === 11)
    expect(segment?.status).toBe('draft')
    expect(segment?.target_text).toBe('Kết quả AI')
  })
})

describe('flush TRƯỚC lượt PROMOTE — bản đang soạn chưa xuống đĩa thì Rust không thấy được nó', () => {
  it('bản đang gõ chưa flush + lượt flush TRƯỢT ⇒ promote KHÔNG được gọi, bản đang soạn được GIỮ', async () => {
    const { editorPanelState, AiTranslationPanel } = await freshPanel()
    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()

    await editorPanelState.ensureSegmentsLoaded()
    editorPanelState.noteEditorEdit(11, 'Bản đang gõ, chưa xuống đĩa')
    failNextSave.value = true

    const result = await editorPanelState.promoteAiTranslationToEditor(11, 'Kết quả AI')

    expect(result).toBe('refused')
    // Chữ ký cần canh: KHÔNG một lượt `promote_ai_translation` nào chạy trước khi flush xong.
    expect(promoteAiTranslationMock).not.toHaveBeenCalled()
    expect(editorPanelState.editorPendingPromote.value).toBeNull()
    // Bản đang soạn vẫn còn nguyên trong bộ đệm — "giữ" nghĩa là không mất, không ghi đè.
    expect(editorPanelState.editorEditedText.value.get(11)).toBe('Bản đang gõ, chưa xuống đĩa')

    wrapper.unmount()
  })
})

describe('đổi Tác phẩm giữa lô PROMOTE — lô dừng, không ghi id trùng số vào Tác phẩm mới', () => {
  const batch = [
    { segmentId: 11, text: 'Một' },
    { segmentId: 12, text: 'Hai' },
    { segmentId: 13, text: 'Ba' },
  ]

  function deferred<T>() {
    let resolve!: (value: T) => void
    const promise = new Promise<T>((r) => {
      resolve = r
    })
    return { promise, resolve }
  }

  it('reset giữa lượt thứ nhất ⇒ không còn lượt gọi nào, không câu hỏi, không báo cáo', async () => {
    const { editorPanelState } = await freshPanel()
    const first = deferred<ReturnType<typeof outcomeOf>>()
    promoteAiTranslationMock.mockReturnValueOnce(first.promise)
    promoteAiTranslationMock.mockResolvedValue(outcomeOf({ target_text: 'x' }))

    const running = editorPanelState.promoteAiTranslationBatchToEditor(batch)
    await flushPromises()
    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)

    editorPanelState.resetEditorPanel()
    first.resolve(outcomeOf({ target_text: 'Một' }))

    expect(await running).toBe('refused')
    await flushPromises()
    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)
    expect(editorPanelState.editorPendingPromoteBatch.value).toBeNull()
    expect(editorPanelState.editorPromoteBatchReport.value).toBeNull()
  })

  it('lô của Tác phẩm cũ kết thúc KHÔNG làm sập cờ bận của lô Tác phẩm mới', async () => {
    const { editorPanelState } = await freshPanel()
    const oldFirst = deferred<ReturnType<typeof outcomeOf>>()
    promoteAiTranslationMock.mockReturnValueOnce(oldFirst.promise)

    const oldRun = editorPanelState.promoteAiTranslationBatchToEditor(batch)
    await flushPromises()
    editorPanelState.resetEditorPanel()

    const newFirst = deferred<ReturnType<typeof outcomeOf>>()
    promoteAiTranslationMock.mockReturnValueOnce(newFirst.promise)
    const newRun = editorPanelState.promoteAiTranslationBatchToEditor([{ segmentId: 11, text: 'Mới' }])
    await flushPromises()

    oldFirst.resolve(outcomeOf({ target_text: 'Một' }))
    await oldRun
    expect(await editorPanelState.promoteAiTranslationBatchToEditor([{ segmentId: 12, text: 'Chen' }])).toBe('refused')

    newFirst.resolve(outcomeOf({ target_text: 'Mới' }))
    expect(await newRun).toBe('done')
  })

  it('reset giữa lượt force=true ⇒ các câu còn lại của lô đồng ý ghi đè không được gọi', async () => {
    const { editorPanelState } = await freshPanel()
    promoteAiTranslationMock.mockResolvedValue(
      outcomeOf({ needs_confirmation: true, unsigned_draft: 'Nháp' }),
    )
    expect(await editorPanelState.promoteAiTranslationBatchToEditor(batch)).toBe('needs-confirmation')
    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(3)

    const firstForced = deferred<ReturnType<typeof outcomeOf>>()
    promoteAiTranslationMock.mockReset()
    promoteAiTranslationMock.mockReturnValueOnce(firstForced.promise)
    promoteAiTranslationMock.mockResolvedValue(outcomeOf({ target_text: 'x' }))

    editorPanelState.confirmPendingPromote()
    await flushPromises()
    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)
    expect(promoteAiTranslationMock).toHaveBeenNthCalledWith(1, 11, 'Một', true)

    editorPanelState.resetEditorPanel()
    firstForced.resolve(outcomeOf({ target_text: 'Một' }))
    await flushPromises()

    expect(promoteAiTranslationMock).toHaveBeenCalledTimes(1)
    expect(editorPanelState.editorPromoteBatchReport.value).toBeNull()
  })
})
