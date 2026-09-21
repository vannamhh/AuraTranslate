/**
 * Panel `Đề xuất AI` — dịch một segment với kết quả **chảy dần**, Story 4.8 (FR72/FR74,
 * AD-22, AD-47①/③). Phase 4b's own file — the two Rust-side test files (`ai_translate_contract.rs`,
 * `ipc_contract.rs`) are owned by a different agent running concurrently; this file only canh
 * phía webview.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CÙNG KHUÔN `aiPromptInspector.test.ts::freshPanel()` — mount THẬT, dispatch qua registry
 * THẬT, mock ĐÚNG BIÊN adapter (`config/aitranslate`), không mock cả `aiTranslateState.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `AGENTS.md` (root): "412 lines of ribbon tests never mounted the component" là một khuyết
 * tật ĐÃ XẢY RA trong dự án này. Mọi ca dưới đây `mount(AiTranslationPanel)` thật rồi
 * `dispatch('ai.translate.…')` qua ĐÚNG `CommandRegistry` thật (`installCommands`), với
 * `deps.runAiTranslate`/`cancelAiTranslate`/`promoteAiTranslate` trỏ THẲNG vào hàm SẢN PHẨM
 * của `aiTranslateState.ts` — đúng khuôn `main.ts::boot()`'s wiring (xem doc-comment
 * `freshPanel()` dưới đây cho lý do không `import` thẳng `main.ts`: nó tự chạy `void boot()`
 * ở top-level module, không có điểm vào tách rời để test).
 *
 * `config/aitranslate.ts` là biên IPC bị `vi.mock` (KHÔNG gọi `@tauri-apps/api/core` thật) —
 * mock trả về một `Promise` mà test tự điều khiển thời điểm trả lời VÀ tự giữ tham chiếu
 * `onToken` mà `aiTranslateState.ts::runAiTranslate` truyền vào, để mô phỏng CHÍNH XÁC những gì
 * một `Channel` thật làm: gọi `onToken` nhiều lần TRƯỚC khi lời gọi `invoke` trả lời — đây là
 * "fake Channel" mà §Code Map spec 4.8 → Tests that move đòi.
 *
 * `panels/editorPanelState.ts` bị `vi.doMock` thành một `ref()` THẬT mà test tự điều khiển
 * (`editorCaretSegmentId`) cộng một spy thay `promoteAiTranslationToEditor` — component chỉ
 * cần đúng hai export đó (không kéo theo cả Panel Editor thật, cùng lý do
 * `glossaryConfirmStripTemplate.test.ts` đã ghi).
 *
 * `config/promptset`/`config/aiprompt` KHÔNG bị mock: cả hai đi qua `hasIpcBridge()` và không
 * bao giờ ném khi không có cầu Tauri (`window.__TAURI_INTERNALS__` vắng mặt trong `happy-dom`),
 * nên `AiTranslationPanel.vue`'s `onMounted` (`loadPromptSets()` + `refreshAiPromptRecord()`)
 * mount an toàn mà không cần giả lập tầng đó — cùng đo đã ghi ở `freshPanel()` của
 * `aiPromptInspector.test.ts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CẢ NĂM TRẠNG THÁI, KHÔNG CHỈ "HAPPY PATH"
 * ─────────────────────────────────────────────────────────────────────────────
 * `AGENTS.md` (root): "a case that passes in two states guards neither". Năm ca trạng thái
 * dưới đây (`not_configured` NGHỈ · `not_configured` sau một lượt dịch · `generating` ·
 * `done` · `cancelled` · `error`) đều mount panel, dispatch dịch thật, rồi đọc TEMPLATE thật
 * (`data-ai-translate-state`, `.ai-translate-text`, `.ai-translate-alert`) — không chỉ gọi hàm
 * thuần của `aiTranslateState.ts` rồi đọc giá trị trả về.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Component, Ref } from 'vue'
import type { AiTranslateOutcomeWire } from '../../src/config/aitranslate'
import type { CommandDeps } from '../../src/commands'
import type { IpcError } from '../../src/i18n'

const runMock = vi.fn()
const cancelMock = vi.fn()
const promoteMock = vi.fn()

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: (...args: unknown[]) => runMock(...args),
  cancelAiTranslateCall: (...args: unknown[]) => cancelMock(...args),
}))

type RunResult = { value: AiTranslateOutcomeWire | null; error: IpcError | null }

/** Kiểu callback `onToken` mà `runAiTranslateSegment` (thật lẫn mock) nhận ở tham số thứ ba. */
type OnToken = (text: string) => void

const SOME_ERROR: IpcError = {
  code: 'ai_translate.provider_call_failed',
  message_key: 'err.ai_translate.provider_call_failed',
  params: {},
  retryable: true,
}

/**
 * Dựng một lượt `runAiTranslateSegment` KHÔNG trả lời ngay — trả lại cả `Promise` con và một
 * hàm `token(text)` gọi thẳng `onToken` mà `aiTranslateState.ts::runAiTranslate` đã đăng ký,
 * cộng một hàm `settle(result)` giả lập Rust trả lời sau cùng. Đây là "fake Channel" của Phase
 * 4b — mô phỏng đúng thứ tự thật: nhiều `onToken` trước, ĐÚNG MỘT lần `settle` sau.
 */
function pendingRun(): { token: OnToken; settle: (result: RunResult) => void } {
  let capturedOnToken: OnToken = () => {
    throw new Error('onToken chưa được gán — runMock chưa gọi tới')
  }
  let resolve: (result: RunResult) => void = () => {
    throw new Error('resolve chưa được gán — runMock chưa gọi tới')
  }
  runMock.mockImplementation((_segmentId: number, _promptSetName: string | null, onToken: OnToken) => {
    capturedOnToken = onToken
    return new Promise<RunResult>((res) => {
      resolve = res
    })
  })
  return {
    token: (text: string) => {
      capturedOnToken(text)
    },
    settle: (result: RunResult) => {
      resolve(result)
    },
  }
}

/**
 * Cùng khuôn `aiPromptInspector.test.ts::freshPanel()`: `vi.resetModules()` rồi `import` lại
 * mọi thứ, để mỗi ca có một `aiTranslateState.ts` NGUYÊN VẸN (không rò trạng thái giữa các ca).
 *
 * `commands.installCommands({...})` chép NGUYÊN VĂN cách `main.ts::boot()` nối ba dep này —
 * xem `main.ts` dòng đăng ký `runAiTranslate`/`cancelAiTranslate`/`promoteAiTranslate`. Đây
 * KHÔNG phải một guard phát minh riêng cho test: nó là hình dạng thật, chép lại vì `main.ts`
 * tự chạy `void boot()` ở top-level và không có điểm vào tách rời để `import` thẳng.
 */
async function freshPanel() {
  vi.resetModules()
  runMock.mockReset()
  cancelMock.mockReset()
  promoteMock.mockReset()

  const caretSegmentId: Ref<number | null> = ref(null)
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caretSegmentId,
    promoteAiTranslationToEditor: (...args: unknown[]) => promoteMock(...args),
  }))

  const commands = await import('../../src/commands')
  const state = await import('../../src/aiTranslateState')
  const editorPanelState = await import('../../src/panels/editorPanelState')
  const i18n = await import('../../src/i18n')
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

  commands.installCommands({
    runAiTranslate: () => {
      void state.runAiTranslate(null, caretSegmentId.value)
    },
    cancelAiTranslate: () => {
      state.cancelAiTranslate()
    },
    // I/O Matrix spec 4.8 "Promote while generating" → kêu, không ném, không ghi. Đây là lớp
    // phòng thủ THỨ HAI (lớp thứ nhất là nút `disabled` ở panel) — chép nguyên logic
    // `main.ts` thật để một chord (⌘⇧↵) bấm được dù nút đang khoá vẫn bị chặn ở đây.
    promoteAiTranslate: () => {
      const s = state.aiTranslateStateValue.value
      const segmentId = state.aiTranslateRunSegmentId.value
      const text = state.aiTranslateAccumulatedText.value
      if ((s !== 'done' && s !== 'cancelled') || segmentId === null || text === '') {
        console.warn(
          `[test] khong dua sang Editor: chua co ket qua hop le (state=${s}, segmentId=${String(segmentId)})`,
        )
        return
      }
      void editorPanelState.promoteAiTranslationToEditor(segmentId, text)
    },
  } as CommandDeps)

  return { commands, state, i18n, AiTranslationPanel, caretSegmentId }
}

function mountPanel(AiTranslationPanel: Component) {
  return mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Giá trị NGHỈ trước lượt dịch đầu tiên — §Code Map spec 4.8 Phase 3, "second judgment call"
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — giá trị nghỉ trước lượt dịch đầu tiên', () => {
  it('state === "not_configured" nhưng CHƯA từng dịch ⇒ KHÔNG hiện lời mời cấu hình (đóng bằng omission ở template, không một giá trị thứ sáu)', async () => {
    const { state, AiTranslationPanel } = await freshPanel()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()

    expect(state.aiTranslateStateValue.value).toBe('not_configured')
    expect(state.aiTranslateRunSegmentId.value).toBeNull()
    expect(wrapper.find('[data-ai-translate-state="not_configured"]').exists()).toBe(false)
    expect(runMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Năm trạng thái, mỗi trạng thái qua ĐÚNG "fake Channel" + template thật
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — dispatch("ai.translate.run") qua "fake Channel", năm trạng thái', () => {
  it('not_configured (sau một lượt dịch thật): 0 lời mời-là-lỗi, panel mời cấu hình, KHÔNG một `.ai-translate-alert`', async () => {
    const { state, i18n, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 5
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(state.aiTranslateStateValue.value).toBe('not_configured')
    expect(wrapper.get('[data-ai-translate-state="not_configured"]').text()).toBe(
      i18n.t('panel.ai_translation.status'),
    )
    expect(wrapper.find('.ai-translate-alert').exists()).toBe(false)

    wrapper.unmount()
  })

  it('generating: token chảy tới cộng dồn ĐÚNG THỨ TỰ ngay trên panel, nút Dịch khoá, nút Huỷ mở', async () => {
    const { state, i18n, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 42
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-run]').attributes('disabled')).toBeUndefined()

    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    expect(runMock).toHaveBeenCalledTimes(1)
    expect(runMock).toHaveBeenCalledWith(42, null, expect.any(Function))
    expect(state.aiTranslateStateValue.value).toBe('generating')
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-state="generating"]').text()).toBe(
      i18n.t('panel.ai_translation.state_generating'),
    )
    expect(wrapper.get('[data-ai-translate-run]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-ai-translate-cancel]').attributes('disabled')).toBeUndefined()

    fake.token('Xin ')
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-text]').text()).toBe('Xin')
    fake.token('chào')
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-ai-translate-text]').text()).toBe('Xin chào')

    wrapper.unmount()
  })

  it('done: state chảy generating → done, văn bản cuối cùng đứng nguyên, nút Dịch mở lại và Đưa-sang-bản-dịch mở', async () => {
    const { state, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 42
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    fake.token('Ket qua ')
    fake.token('day du.')
    await wrapper.vm.$nextTick()

    fake.settle({ value: { state: 'done' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(state.aiTranslateStateValue.value).toBe('done')
    expect(wrapper.find('[data-ai-translate-state="generating"]').exists()).toBe(false)
    expect(wrapper.get('[data-ai-translate-text]').text()).toBe('Ket qua day du.')
    expect(wrapper.get('[data-ai-translate-run]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-ai-translate-promote]').attributes('disabled')).toBeUndefined()

    wrapper.unmount()
  })

  it('cancelled: dispatch("ai.translate.cancel") gọi cancelAiTranslateCall, và CHỈ khi Rust xác nhận (lượt await trả `Cancelled`) state mới đổi — token đã nhận đứng nguyên', async () => {
    const { state, i18n, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 9
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    fake.token('Da nhan mot phan')
    await wrapper.vm.$nextTick()

    await wrapper.get('[data-ai-translate-cancel]').trigger('click')
    expect(cancelMock).toHaveBeenCalledTimes(1)
    // Doc-comment `cancelAiTranslate` (`aiTranslateState.ts`): KHÔNG tự đặt 'cancelled' ở đây
    // — vẫn 'generating' cho tới khi Rust xác nhận.
    expect(state.aiTranslateStateValue.value).toBe('generating')

    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(state.aiTranslateStateValue.value).toBe('cancelled')
    expect(wrapper.get('[data-ai-translate-state="cancelled"]').text()).toBe(
      i18n.t('panel.ai_translation.state_cancelled'),
    )
    expect(wrapper.get('[data-ai-translate-text]').text()).toBe('Da nhan mot phan')

    wrapper.unmount()
  })

  it('error: stream lỗi giữa chừng ⇒ `.ai-translate-alert` hiện đúng bản dịch của lỗi, token đã nhận trước đó KHÔNG bị xoá', async () => {
    const { state, i18n, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 3
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    fake.token('Mot phan ket qua')
    await wrapper.vm.$nextTick()

    fake.settle({ value: null, error: SOME_ERROR })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(state.aiTranslateStateValue.value).toBe('error')
    const alert = wrapper.get('.ai-translate-alert')
    expect(alert.attributes('role')).toBe('alert')
    expect(alert.text()).toBe(i18n.tError(SOME_ERROR))
    expect(wrapper.get('[data-ai-translate-text]').text()).toBe('Mot phan ket qua')

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// I/O Matrix "Caret moves during generation" — kết quả CŨ đánh dấu, không tự lặng câm
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiTranslationPanel.vue — caret dời câu trong lúc đang chảy (I/O Matrix "Caret moves during generation")', () => {
  it('đã có văn bản nhận được, caret dời sang câu khác ⇒ panel đánh dấu kết quả CŨ, nêu tên cả hai câu', async () => {
    const { i18n, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 7
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    fake.token('mot phan')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-ai-translate-stale-notice]').exists()).toBe(false)

    caretSegmentId.value = 8
    await wrapper.vm.$nextTick()

    const notice = wrapper.get('[data-ai-translate-stale-notice]')
    expect(notice.text()).toBe(
      i18n.t('ai.translate.stale_notice', { record_segment_id: '7', focused_segment_id: '8' }),
    )
    // Vẫn còn chảy — lượt dịch KHÔNG bị huỷ chỉ vì caret dời, đúng "the call keeps running and
    // lands against the segment it started on".
    expect(runMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Promote — hai lớp lưới "kêu, không ném" (I/O Matrix "Promote while generating" / "Promote the result")
// ═══════════════════════════════════════════════════════════════════════════════════

describe('dispatch("ai.translate.promote") — hai lớp phòng thủ, và landing đúng câu lượt dịch bắt đầu', () => {
  it('generating ⇒ nút bị `disabled` (lớp một) VÀ dispatch trực tiếp cũng bị từ chối, không ghi (lớp hai, mô phỏng ⌘⇧↵ bỏ qua nút khoá)', async () => {
    const { commands, state, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 11
    // Không bao giờ trả lời — state đứng ở 'generating' suốt ca này.
    runMock.mockImplementation(() => new Promise<RunResult>(() => {}))

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()
    expect(state.aiTranslateStateValue.value).toBe('generating')

    expect(wrapper.get('[data-ai-translate-promote]').attributes('disabled')).toBeDefined()

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => undefined)
    commands.dispatch('ai.translate.promote')

    expect(promoteMock).not.toHaveBeenCalled()
    expect(warnSpy).toHaveBeenCalled()

    wrapper.unmount()
  })

  it('done VỚI văn bản rỗng (0 token nào tới) ⇒ nút vẫn `disabled`, không đủ theo I/O Matrix "panel text non-empty"', async () => {
    const { state, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 21
    runMock.mockResolvedValue({ value: { state: 'done' }, error: null })

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(state.aiTranslateStateValue.value).toBe('done')
    expect(wrapper.get('[data-ai-translate-promote]').attributes('disabled')).toBeDefined()
    expect(promoteMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })

  it('done với văn bản không rỗng, caret ĐÃ DỜI sang câu khác trước khi bấm ⇒ promote ghi vào ĐÚNG câu lượt dịch bắt đầu, không câu caret hiện tại', async () => {
    const { AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 21
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    fake.token('Ket qua AI')
    fake.settle({ value: { state: 'done' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    // Caret dời sang câu 99 SAU khi lượt dịch đã xong — kết quả vẫn phải landing đúng câu 21.
    caretSegmentId.value = 99
    await wrapper.vm.$nextTick()

    const promoteButton = wrapper.get('[data-ai-translate-promote]')
    expect(promoteButton.attributes('disabled')).toBeUndefined()
    await promoteButton.trigger('click')

    expect(promoteMock).toHaveBeenCalledTimes(1)
    expect(promoteMock).toHaveBeenCalledWith(21, 'Ket qua AI')

    wrapper.unmount()
  })

  it('cancelled với văn bản không rỗng ⇒ cũng đủ điều kiện promote (I/O Matrix "state done hoặc cancelled")', async () => {
    const { AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 30
    const fake = pendingRun()

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()

    fake.token('Truoc khi huy')
    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const promoteButton = wrapper.get('[data-ai-translate-promote]')
    expect(promoteButton.attributes('disabled')).toBeUndefined()
    await promoteButton.trigger('click')

    expect(promoteMock).toHaveBeenCalledTimes(1)
    expect(promoteMock).toHaveBeenCalledWith(30, 'Truoc khi huy')

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// AC "mỗi lệnh dịch/huỷ/đưa-sang là một command đăng ký theo id, VÀ mỗi lệnh hoạt động khi
// được gán lại qua màn hình phím tắt" — vế REBIND, đo THẬT, không suy từ cấu trúc mã
// ═══════════════════════════════════════════════════════════════════════════════════
//
// 🔴 `ai.translate.run`/`.cancel` đăng ký `keys: undefined` (Decision 7, §Implementation
// Notes Phase 3) — KHÔNG có hợp âm mặc định, nên đường bàn phím duy nhất có thể canh ở đây
// là một lượt REBIND của người dùng (`applyBindings`, cùng cửa
// `ShortcutsOverlay.vue`/`shortcutsState.ts` dùng — Story 1.21 AC2/AC12), không phải một
// `KeyboardEvent` khớp phím mặc định. Ba `describe`/`it` phía trên (dispatch qua `@click`)
// canh CỔNG "đăng ký theo id" nhưng bỏ qua hẳn quãng đường bàn phím — cùng khuôn
// `aiPromptInspector.test.ts` §"AC3 — rebind qua tầng phím tắt" (dòng ~650-720, Story 4.7).
//
// ⚠️ `ai.translate.promote` ĐO ĐƯỢC khác hai lệnh kia: nó mang `keys: ['Mod+Shift+Enter']`
// sẵn (UX-DR35, `keys.ts:92-95`), không `undefined`. Nhưng câu AC nói "hoạt động khi được
// gán lại", không chỉ "khi bấm đúng phím mặc định" — nên ca của nó dưới đây vẫn gán nó sang
// một hợp âm KHÁC (`Shift+M`, không phải `Mod+Shift+Enter`) rồi bấm hợp âm MỚI đó, để phép
// đo thật sự đi qua `applyBindings`/rebuild-keymap thay vì trùng khớp tình cờ với mặc định.
//
// Cả ba hợp âm dưới đây (`Shift+K`/`Shift+L`/`Shift+M`) không `Mod` — tránh nhánh
// macOS/Windows của `detectIsMac()` rẽ khác nhau, cùng lý do `aiPromptInspector.test.ts`
// không cần biết đang chạy trên nền nào — và đo trước khi dùng: `grep "keys: \['"`
// (`src/commands/index.ts`) không có hợp âm `Shift+K`/`Shift+L`/`Shift+M` nào đã đăng ký ở
// bộ command sản phẩm thật, nên `applyBindings` dưới đây không đụng thao tác nào khác.
describe('AC "mỗi lệnh ... hoạt động khi được rebind qua màn hình phím tắt" — đo bằng một lượt rebind thật', () => {
  it('🔴 rebind ai.translate.run sang Shift+K rồi bấm ⇒ gọi runAiTranslateSegment đúng như click nút Dịch', async () => {
    const { commands, state, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 42
    const fake = pendingRun()

    const outcome = commands.applyBindings({ 'ai.translate.run': ['Shift+K'] })
    expect(outcome.ok).toBe(true) // hợp âm không xung đột với thao tác nào khác của registry thật

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    expect(runMock).not.toHaveBeenCalled()

    const host = document.createElement('div')
    document.body.appendChild(host)
    const detach = commands.attachKeyboard(host)
    try {
      host.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'K', code: 'KeyK', shiftKey: true, bubbles: true, cancelable: true }),
      )
      await flushPromises()
    } finally {
      detach()
      host.remove()
    }

    // Cùng hai khẳng định mà ca "generating" (dispatch-theo-id, `@click`) đã canh ở trên —
    // đường bàn phím phải tới đúng CÙNG đích, không một đường tắt khác.
    expect(runMock).toHaveBeenCalledTimes(1)
    expect(runMock).toHaveBeenCalledWith(42, null, expect.any(Function))
    expect(state.aiTranslateStateValue.value).toBe('generating')

    fake.settle({ value: { state: 'done' }, error: null })
    await flushPromises()
    wrapper.unmount()
  })

  it('🔴 rebind ai.translate.cancel sang Shift+L rồi bấm giữa lúc generating ⇒ gọi cancelAiTranslateCall đúng như click nút Huỷ', async () => {
    const { commands, state, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 9
    const fake = pendingRun()

    const outcome = commands.applyBindings({ 'ai.translate.cancel': ['Shift+L'] })
    expect(outcome.ok).toBe(true)

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()
    fake.token('Da nhan mot phan')
    await wrapper.vm.$nextTick()
    expect(state.aiTranslateStateValue.value).toBe('generating')

    const host = document.createElement('div')
    document.body.appendChild(host)
    const detach = commands.attachKeyboard(host)
    try {
      host.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'L', code: 'KeyL', shiftKey: true, bubbles: true, cancelable: true }),
      )
    } finally {
      detach()
      host.remove()
    }

    expect(cancelMock).toHaveBeenCalledTimes(1)
    // Doc-comment `cancelAiTranslate` (`aiTranslateState.ts`), cùng đối chứng ca dispatch-theo-id
    // ở trên: KHÔNG tự đặt 'cancelled' ở đây — vẫn 'generating' cho tới khi Rust xác nhận.
    expect(state.aiTranslateStateValue.value).toBe('generating')

    fake.settle({ value: { state: 'cancelled' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()
    expect(state.aiTranslateStateValue.value).toBe('cancelled')
    expect(wrapper.get('[data-ai-translate-text]').text()).toBe('Da nhan mot phan')

    wrapper.unmount()
  })

  it('🔴 rebind ai.translate.promote sang Shift+M (KHÁC hợp âm mặc định Mod+Shift+Enter) rồi bấm ⇒ gọi promoteAiTranslationToEditor đúng như click nút Đưa-sang-bản-dịch, landing đúng câu lượt dịch bắt đầu', async () => {
    const { commands, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 21
    const fake = pendingRun()

    const outcome = commands.applyBindings({ 'ai.translate.promote': ['Shift+M'] })
    expect(outcome.ok).toBe(true)

    const wrapper = mountPanel(AiTranslationPanel)
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-ai-translate-run]').trigger('click')
    await flushPromises()
    fake.token('Ket qua AI')
    fake.settle({ value: { state: 'done' }, error: null })
    await flushPromises()
    await wrapper.vm.$nextTick()

    // Caret dời SAU khi lượt dịch xong, giống ca dispatch-theo-id tương ứng ở trên — vẫn phải
    // landing đúng câu 21, không câu caret hiện tại, kể cả khi kích hoạt bằng bàn phím.
    caretSegmentId.value = 99
    await wrapper.vm.$nextTick()

    const host = document.createElement('div')
    document.body.appendChild(host)
    const detach = commands.attachKeyboard(host)
    try {
      host.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'M', code: 'KeyM', shiftKey: true, bubbles: true, cancelable: true }),
      )
    } finally {
      detach()
      host.remove()
    }

    expect(promoteMock).toHaveBeenCalledTimes(1)
    expect(promoteMock).toHaveBeenCalledWith(21, 'Ket qua AI')

    wrapper.unmount()
  })

  it('🔴 finding-shape: effectiveUnbound() phản ánh ĐÚNG lượt rebind — ai.translate.run rời danh sách "chưa gán" sau applyBindings, hai lệnh còn lại đứng nguyên', async () => {
    const { commands } = await freshPanel()

    const beforeIds = commands.effectiveUnbound().map((s: { id: string }) => s.id)
    expect(beforeIds).toContain('ai.translate.run')
    expect(beforeIds).toContain('ai.translate.cancel')
    // ⚠️ Đo được: `ai.translate.promote` mang `keys: ['Mod+Shift+Enter']` mặc định (khác
    // `.run`/`.cancel`), nên nó KHÔNG ở trong danh sách "chưa gán phím" ngay cả trước lượt
    // rebind này — `effectiveUnbound()` chỉ liệt các thao tác 0 hợp âm hiệu lực.
    expect(beforeIds).not.toContain('ai.translate.promote')

    commands.applyBindings({ 'ai.translate.run': ['Shift+K'] })

    const afterIds = commands.effectiveUnbound().map((s: { id: string }) => s.id)
    expect(afterIds).not.toContain('ai.translate.run')
    // Lệnh CÒN LẠI chưa rebind vẫn phải đứng nguyên — đối chứng không phải mọi thứ đột nhiên
    // "có phím" sau MỘT lượt `applyBindings`.
    expect(afterIds).toContain('ai.translate.cancel')
  })
})

/**
 * `resetAiTranslate()` phải HUỶ một lượt dịch đang bay, không chỉ dọn state phía webview —
 * bản sửa theo vòng rà 2026-09-21 (phát hiện #1, AD-22).
 *
 * 🔴 Trước bản sửa, `resetAiTranslate()` chỉ bơm `sequence` — thứ làm token/kết quả TRỄ rơi vào
 * im lặng Ở PHÍA WEBVIEW — và dọn bốn ô nhớ; nó KHÔNG gọi `cancelAiTranslateCall`. Đổi Chương
 * (`modes/libraryChapters.ts`) hay đóng một lượt nhập (`modes/libraryImport.ts`) giữa lúc đang
 * `'generating'` vì thế BỎ RƠI lượt gọi: Rust vẫn kéo token từ nhà cung cấp, và với BYOK đó là
 * tiền người dùng trả cho một kết quả không ai còn thấy hay đưa sang bản dịch được nữa.
 */
describe('resetAiTranslate huỷ lượt dịch đang bay (vòng rà 2026-09-21)', () => {
  it('reset giữa lúc generating ⇒ cancelAiTranslateCall đúng MỘT lần, state về not_configured', async () => {
    const { state } = await freshPanel()
    pendingRun()

    void state.runAiTranslate(null, 42)
    await flushPromises()
    expect(state.aiTranslateStateValue.value).toBe('generating')

    cancelMock.mockClear()
    state.resetAiTranslate()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.aiTranslateStateValue.value).toBe('not_configured')
  })

  it('reset khi KHÔNG có lượt nào đang bay ⇒ không gửi gì — đối chứng âm, kẻo ca trên xanh vì reset luôn huỷ', async () => {
    const { state } = await freshPanel()
    cancelMock.mockClear()

    state.resetAiTranslate()

    expect(cancelMock).not.toHaveBeenCalled()
  })
})
