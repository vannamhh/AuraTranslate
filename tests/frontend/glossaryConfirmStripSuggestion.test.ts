/**
 * Đề xuất bản dịch bằng âm Hán Việt trên dải "Chờ chốt lần đầu gặp" — Story 3.7, FR113.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * PHẠM VI — bốn hàng §I/O Matrix ở TẦNG FRONTEND, khuôn `glossaryConfirmStripTemplate.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * Vế Rust (`suggest_han_viet_batch`, năm nhánh đóng) đã đóng ở
 * `glossary_han_viet_suggestion_contract.rs`. Tệp này canh riêng nửa còn lại: dữ liệu Rust
 * đã cầm sẵn (`GlossaryMark.han_viet_suggestion`/`han_viet_status`, đi qua dây IPC) có tới
 * đúng chỗ trên màn hình hay không — ô nhập điền sẵn, vùng chọn thắng đề xuất, dòng thông
 * báo khi chưa cài từ điển, và ô rỗng lại khi đổi mục.
 *
 * Mock `@tauri-apps/api/core` ở ĐÚNG BIÊN IPC (hoisted) — cùng khuôn mọi tệp Glossary khác;
 * `panels/editorPanelState.ts`/`glossaryQuickAddState.ts`/`commands` mock qua `vi.doMock`,
 * chép nguyên khuôn `freshStrip()` của `glossaryConfirmStripTemplate.test.ts`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type { GlossarySegmentSource } from '../../src/panels/glossaryMarksMap'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

type Segment = GlossarySegmentSource

/** Payload dây hợp lệ cho một span chờ chốt — mọi trường Story 3.7 truyền qua `overrides`. */
function pendingMarkWire(overrides: Record<string, unknown> = {}) {
  return {
    start: 0,
    end: 1,
    tier: 'global',
    is_confirmed: false,
    translation: null,
    id: 7,
    source_term: '萧',
    han_viet_suggestion: null,
    han_viet_status: 'not_requested',
    ...overrides,
  }
}

async function freshStrip() {
  vi.resetModules()
  mockInvoke.mockReset()

  const caretSegmentId: Ref<number | null> = ref(null)
  const segments: Ref<Segment[]> = ref([])
  const quickAddIsOpen = ref(false)
  const dispatchMock = vi.fn()

  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caretSegmentId,
    editorSegments: segments,
  }))
  vi.doMock('../../src/glossaryQuickAddState', () => ({ quickAddIsOpen }))
  vi.doMock('../../src/commands', () => ({ dispatch: (id: string) => dispatchMock(id) }))

  const state = await import('../../src/glossaryConfirmStripState')
  const marksState = await import('../../src/panels/glossaryMarksState')
  const i18n = await import('../../src/i18n')
  const GlossaryConfirmStrip = (await import('../../src/GlossaryConfirmStrip.vue')).default

  return { state, marksState, i18n, GlossaryConfirmStrip, caretSegmentId, segments, quickAddIsOpen, dispatchMock }
}

const STRIP_SELECTOR = '.glossary-confirm-strip'

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ═════════════════════════════════════════════════════════════════════════════════
// Dải mọc ⇒ ô nhập điền sẵn đề xuất, kèm nhãn "âm Hán Việt"
// ═════════════════════════════════════════════════════════════════════════════════

describe('dải mọc cho một mục có đề xuất', () => {
  it('ô nhập ĐIỀN SẴN đúng chuỗi đề xuất, và nhãn "âm Hán Việt" hiện', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') {
        return Promise.resolve([
          pendingMarkWire({ han_viet_suggestion: 'Tiêu Viêm', han_viet_status: 'ok' }),
        ])
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(true)
    expect((wrapper.find('.gcs-input').element as HTMLInputElement).value).toBe('Tiêu Viêm')
    expect(wrapper.find('.gcs-suggestion-label').exists()).toBe(true)
    // Đúng MỘT đoạn trạng thái luôn render tại một thời điểm — không đoạn nào khác hiện.
    expect(wrapper.find('#gcs-status-msg').exists()).toBe(false)

    wrapper.unmount()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Vào dải bằng hợp âm khi ĐANG có vùng chọn ⇒ vùng chọn THẮNG đề xuất
// ═════════════════════════════════════════════════════════════════════════════════

describe('vào dải bằng hợp âm khi đang có vùng chọn', () => {
  it('vùng chọn THẮNG đề xuất — thao tác người dùng vừa làm đứng trên gợi ý của máy', async () => {
    const { state, marksState, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') {
        return Promise.resolve([
          pendingMarkWire({ han_viet_suggestion: 'Tiêu Viêm', han_viet_status: 'ok' }),
        ])
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1
    // `syncGlossaryConfirmStripTarget` chính là điều `watch` của component gọi — dựng lại ở
    // đây để canh state ĐỘC LẬP với việc component có mount hay không (khuôn
    // `glossaryConfirmStrip.test.ts`).
    state.syncGlossaryConfirmStripTarget(1, segs, marksState.glossaryMarksHaveLoaded(), marksState.glossaryMarks.value)

    expect(state.confirmStripTranslationInput.value).toBe('Tiêu Viêm') // de xuat da dien san luc mo

    const entered = state.focusGlossaryConfirmStrip('Bản dịch đã chọn', true)

    expect(entered).toBe(true)
    expect(state.confirmStripTranslationInput.value).toBe('Bản dịch đã chọn')
  })

  it('KHÔNG có đề xuất (mục thường) — luật cũ "chỉ điền khi RỖNG" vẫn đúng nguyên vẹn', async () => {
    const { state, marksState, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1
    state.syncGlossaryConfirmStripTarget(1, segs, marksState.glossaryMarksHaveLoaded(), marksState.glossaryMarks.value)

    expect(state.confirmStripTranslationInput.value).toBe('')

    state.focusGlossaryConfirmStrip('Vùng chọn', true)

    expect(state.confirmStripTranslationInput.value).toBe('Vùng chọn')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Dải mọc khi chưa cài từ điển ⇒ ô rỗng + dòng thông báo
// ═════════════════════════════════════════════════════════════════════════════════

describe('dải mọc khi chưa cài dữ liệu từ điển', () => {
  it('ô nhập RỖNG, và dòng "chưa cài dữ liệu từ điển" hiện — không im lặng', async () => {
    const { state, marksState, i18n, GlossaryConfirmStrip, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') {
        return Promise.resolve([pendingMarkWire({ han_viet_status: 'dict_unavailable' })])
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    expect((wrapper.find('.gcs-input').element as HTMLInputElement).value).toBe('')
    expect(wrapper.find('.gcs-suggestion-label').exists()).toBe(false)
    const notice = wrapper.find('#gcs-status-msg')
    expect(notice.exists()).toBe(true)
    expect(notice.text()).toBe(i18n.t('glossary.confirm.suggestion_unavailable'))
    expect(wrapper.find('.gcs-input').attributes('aria-describedby')).toBe('gcs-status-msg')

    expect(state.confirmStripSuggestionStatus.value).toBe('dict_unavailable')
    wrapper.unmount()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Đổi mục sang một thuật ngữ KHÔNG đề xuất được ⇒ ô RỖNG LẠI
// ═════════════════════════════════════════════════════════════════════════════════

describe('đổi mục — ô KHÔNG giữ chữ của mục trước', () => {
  it('mục A có đề xuất ⇒ ô điền sẵn; đổi sang mục B không đề xuất được ⇒ ô rỗng lại', async () => {
    const { state, marksState, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [
      { id: 1, source_text: '萧炎' },
      { id: 2, source_text: 'dragon' },
    ]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') {
        return Promise.resolve([
          pendingMarkWire({
            start: 0,
            end: 2,
            id: 7,
            source_term: '萧炎',
            han_viet_suggestion: 'Tiêu Viêm',
            han_viet_status: 'ok',
          }),
          pendingMarkWire({
            start: 3,
            end: 9,
            id: 8,
            source_term: 'dragon',
            han_viet_suggestion: null,
            han_viet_status: 'not_chinese',
          }),
        ])
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')

    caretSegmentId.value = 1
    state.syncGlossaryConfirmStripTarget(1, segs, marksState.glossaryMarksHaveLoaded(), marksState.glossaryMarks.value)
    expect(state.confirmStripTranslationInput.value).toBe('Tiêu Viêm')

    caretSegmentId.value = 2
    state.syncGlossaryConfirmStripTarget(2, segs, marksState.glossaryMarksHaveLoaded(), marksState.glossaryMarks.value)

    expect(state.confirmStripSourceTerm.value).toBe('dragon')
    // doi sang muc khong de xuat duoc -- o KHONG duoc giu chu cua muc truoc
    expect(state.confirmStripTranslationInput.value).toBe('')
    expect(state.confirmStripSuggestionStatus.value).toBe('not_chinese')
  })
})
