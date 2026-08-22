/**
 * Dải "Chờ chốt lần đầu gặp" — **vế TEMPLATE**, Story 3.6 · FR114 (rà ba lớp 2026-08-22).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI TÁCH KHỎI `glossaryConfirmStrip.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `glossaryConfirmStrip.test.ts` kiểm rất kỹ **module state** nhưng không mount
 * `GlossaryConfirmStrip.vue` một lần nào — đúng lớp hồi quy mà `glossaryQuickAddStrip.test.ts`
 * (Story 3.3, lượt rà soát 2026-08-20) đã đo được: gỡ `:disabled`, đảo nhánh `v-if`, hay xoá
 * hẳn một đoạn lý do — cả `cargo test` lẫn `vitest` (chỉ-state) đều xanh. `watch([editorCaretSegmentId,
 * editorSegments, glossaryMarks], …)` — cơ chế THẬT làm dải mọc, tức chính FR114 — cùng
 * `isVisible`/`topmostStrip` sống TRONG component, nên chỉ một lượt `mount()` mới chạm được.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * BIÊN MOCK — bốn tệp, bốn lý do khác nhau
 * ─────────────────────────────────────────────────────────────────────────────
 * - `@tauri-apps/api/core` (hoisted, `vi.mock`) — biên IPC thật, khuôn `bootstrap.test.ts`.
 *   `config/glossary.ts`/`glossaryConfirmStripState.ts`/`glossaryMarksState.ts` chạy THẬT
 *   qua dây này.
 * - `panels/editorPanelState.ts` (`vi.doMock` trong `freshStrip()`) — module đó nặng (2000+
 *   dòng, kéo theo cả Panel Editor); component chỉ cần ĐÚNG HAI export
 *   (`editorCaretSegmentId`, `editorSegments`), nên mock chúng thành hai `ref()` THẬT (giữ
 *   phản ứng của Vue) mà test tự do điều khiển.
 * - `glossaryQuickAddState.ts` (`vi.doMock`) — tránh phải dựng cả vòng IPC `lookupGlossaryTerm`
 *   chỉ để bật `quickAddIsOpen`; mock thành một `ref()` THẬT, gán thẳng.
 * - `commands` (`vi.doMock`) — `dispatch()` NÉM với một id chưa đăng ký (AC1 của
 *   `CommandRegistry`), và tệp này không dựng `installCommands()`; thay bằng một spy để đối
 *   chứng `@submit`/`@keydown.esc` phát ĐÚNG id, không cần một `CommandRegistry` thật.
 *
 * `vi.doMock` (KHÔNG `vi.mock`) vì nó KHÔNG hoist — mỗi lượt `freshStrip()` tự dựng `ref()`
 * MỚI (biên `vue` đã `import` thật ở đầu tệp, nên gọi `ref()` trong hàm không vướng thứ tự
 * hoisting mà `vi.mock` top-level đòi).
 *
 * ⚠️ PHẠM VI dừng ở thứ `happy-dom` trả lời được — vế THỊ GIÁC (dải ĐẨY `.modeport`, không
 * PHỦ) và vùng chọn trên engine thật thuộc bàn đo tay, xem §Verification của story.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type { GlossarySegmentSource } from '../../src/panels/glossaryMarksMap'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

type Segment = GlossarySegmentSource

/** Payload dây hợp lệ cho một span chờ chốt phủ trọn segment 1 ký tự '萧'. */
function pendingMarkWire(overrides: Record<string, unknown> = {}) {
  return {
    start: 0,
    end: 1,
    tier: 'global',
    is_confirmed: false,
    translation: null,
    id: 7,
    source_term: '萧',
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

describe('dải chốt HIỆN khi caret đang ở một segment mang span chờ chốt', () => {
  it('caret ĐÃ ở segment đó lúc mount ⇒ dải hiện ngay (watch chạy `immediate`)', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    caretSegmentId.value = 1
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(true)
    wrapper.unmount()
  })

  it('caret CHƯA vào segment nào ⇒ dải KHÔNG hiện', async () => {
    const { marksState, GlossaryConfirmStrip, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(false)
    wrapper.unmount()
  })

  it('🔴 marks VỀ MUỘN (sau khi caret đã ở đúng segment TRƯỚC đó) ⇒ dải hiện khi marks về — `watch` phải phụ thuộc `glossaryMarks`, không chỉ caret/segments', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    caretSegmentId.value = 1 // caret ĐÃ ở segment này TỪ ĐẦU — sẽ KHÔNG đổi nữa trong ca này.

    let resolveMarks: (v: unknown) => void = () => {}
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') {
        return new Promise((resolve) => {
          resolveMarks = resolve
        })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    const loading = marksState.ensureGlossaryMarksLoaded(1, segs, 'zh') // chưa về.

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()
    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(false) // marks chưa nạp xong.

    resolveMarks([pendingMarkWire()])
    await loading
    await wrapper.vm.$nextTick()

    // 🔴 Mệnh đề trung tâm: dải hiện ra mà KHÔNG một lượt đổi caret nào xảy ra sau lượt mount —
    // chỉ có `glossaryMarks` đổi. Nếu `watch(...)` của component đánh rơi phần tử này khỏi
    // mảng phụ thuộc, ca này ĐỎ dù mọi test chỉ-state vẫn xanh nguyên.
    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('sổ ưu tiên — GlossaryQuickAdd đang mở thắng `v-if`', () => {
  it('quickAddIsOpen bật SAU khi dải chốt đã hiện ⇒ `v-if` ẩn dải ngay, bật lại khi quick-add đóng', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments, quickAddIsOpen } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()
    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(true)

    quickAddIsOpen.value = true
    await wrapper.vm.$nextTick()
    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(false)

    quickAddIsOpen.value = false
    await wrapper.vm.$nextTick()
    expect(wrapper.find(STRIP_SELECTOR).exists()).toBe(true)

    wrapper.unmount()
  })
})

describe('`<fieldset>` bị vô hiệu lúc đang lưu, nhánh lỗi render đúng câu', () => {
  it('đang lưu ⇒ `<fieldset>` mang `:disabled`', async () => {
    const { state, marksState, GlossaryConfirmStrip, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      if (cmd === 'glossary_confirm_pending_translation') return new Promise(() => {}) // dang bay vinh vien.
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    state.confirmStripTranslationInput.value = 'Tieu'
    void state.confirmGlossaryConfirmStrip(1, segs, 'zh') // KHONG await -- dang bay.
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.gcs-fieldset').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('lượt ghi TRƯỢT ⇒ nhánh `.gcs-error` render đúng câu `tError()`', async () => {
    const { state, marksState, i18n, GlossaryConfirmStrip, caretSegmentId, segments } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      if (cmd === 'glossary_confirm_pending_translation') return Promise.reject(err)
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    state.confirmStripTranslationInput.value = 'Tieu'
    await state.confirmGlossaryConfirmStrip(1, segs, 'zh')
    await wrapper.vm.$nextTick()

    const alert = wrapper.find('.gcs-status.gcs-error')
    expect(alert.exists()).toBe(true)
    expect(alert.text()).toBe(i18n.tError(err))
    expect(alert.attributes('role')).toBe('alert')
    // Ô nhập phải nối `aria-describedby` tới đúng đoạn đang hiện.
    expect(wrapper.find('.gcs-input').attributes('aria-describedby')).toBe('gcs-status-msg')

    wrapper.unmount()
  })
})

describe('`@submit`/`@keydown.esc` phát ĐÚNG `dispatch`', () => {
  it('`@submit` trên `<form>` phát `dispatch("glossary.confirm.save")`', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments, dispatchMock } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    await wrapper.find('form').trigger('submit')

    expect(dispatchMock).toHaveBeenCalledWith('glossary.confirm.save')
    wrapper.unmount()
  })

  it('`@keydown.esc` trên `<form>` phát `dispatch("glossary.confirm.defer")`', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments, dispatchMock } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    await wrapper.find('form').trigger('keydown', { key: 'Escape' })

    expect(dispatchMock).toHaveBeenCalledWith('glossary.confirm.defer')
    wrapper.unmount()
  })

  it('nút "Để sau" (`@click`) phát `dispatch("glossary.confirm.defer")`', async () => {
    const { marksState, GlossaryConfirmStrip, caretSegmentId, segments, dispatchMock } = await freshStrip()
    const segs: Segment[] = [{ id: 1, source_text: '萧炎登场' }]
    segments.value = segs
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([pendingMarkWire()])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    await marksState.ensureGlossaryMarksLoaded(1, segs, 'zh')
    caretSegmentId.value = 1

    const wrapper = mount(GlossaryConfirmStrip)
    await wrapper.vm.$nextTick()

    await wrapper.find('.gcs-act:not(.gcs-act-primary)').trigger('click')

    expect(dispatchMock).toHaveBeenCalledWith('glossary.confirm.defer')
    wrapper.unmount()
  })
})
