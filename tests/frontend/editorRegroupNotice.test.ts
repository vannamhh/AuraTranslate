/**
 * Story 2.9 · AC4 — **dòng báo hệ quả sau một lượt gộp**, và người đọc đầu tiên của
 * `editorRegroupError`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 HAI KHOẢNG IM LẶNG STORY NÀY ĐÓNG — cả hai đã được ghi ra bằng chữ trước đó
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **Lượt gộp THÀNH CÔNG không nói gì.** AD-5 nói câu mới *"chưa xác nhận, lịch sử rỗng"* —
 *    một hệ quả người dùng **không nhìn thấy** trên lưới *(vạch lề của câu mới trông y hệt một
 *    câu draft bình thường)*. AC4 đòi nói ra.
 * ② **Lượt gộp BỊ TỪ CHỐI không đổi một pixel nào.** `editorRegroupError`
 *    (`editorPanelState.ts`) tồn tại từ Story 2.8 mà **chưa component nào đọc** — doc-comment
 *    ở `main.ts` đã ghi thẳng: *"đừng đọc dòng này thành 'đã có đường ra màn hình'"*.
 *    ⇒ Người dùng bấm `Backspace` ở câu **đầu Chương** và **không có gì xảy ra**. Đúng lớp
 *    *"rỗng IM LẶNG"* mà `project-context.md` cấm, và nó là ca **thường nhất** của cử chỉ này:
 *    câu số 1 của mọi Chương.
 *
 * 🔴 **Bảng tra phải ĐÓNG.** `CONFIRM_NOTICE_KEYS` đã lập tiền lệ và doc-comment của nó cấm
 * bằng chữ một nhánh mặc định: *"một `?? 'khoá nào đó'` sẽ nuốt im lặng một giá trị thứ tư"*.
 * ⇒ Story này thêm một `Record` **thứ hai**, đóng trên kiểu mới — **không** nới cái cũ thành
 * `string`.
 *
 * 🔴 **Vì sao vitest chứ không e2e:** ô nhớ + người đọc nó là module thuần cộng một component
 * vỏ — không hình học, không engine. Mệnh đề *"`Backspace` thật trong WKWebView có gộp
 * không"* thì thuộc e2e, và nó có nhà riêng ở `e2e/specs/segment-merge-split.e2e.mjs`.
 *
 * ⚠️ **GIỚI HẠN THẬT:** nhóm này canh **ô nhớ và người đọc nó**. Nó **không** canh việc cử chỉ
 * `Backspace` tới được `mergeCurrentSegment` — đường đó đi qua `onEditKeydown` của
 * `GridPanel.vue` và chỉ e2e đo được.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'

/** Hàng mới mà một lượt gộp 11+12 trả về — hình dạng dây, đúng `RegroupOutcome`. */
const HANG_GOP: ChapterSegment = {
  id: 14,
  ord: 1,
  source_text: '一。二。',
  target_text: 'Hắn đẩy cánh cửa ấy ra. Gió thổi tới từ cuối hành lang.',
  is_paragraph_end: true,
  retired_at: null,
  // 🔴 AD-5: câu mới bắt đầu ở **chưa xác nhận**, lịch sử rỗng. Đây chính là hệ quả mà AC4
  // buộc dòng báo phải nói ra bằng chữ.
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: true,
}

/** Lượt gộp kế tiếp trả gì. Đặt lại ở mỗi ca. */
const ketQuaGop: {
  value: { outcome: { retired: ChapterSegment[]; new_segments: ChapterSegment[] } | null; error: unknown }
} = { value: { outcome: null, error: null } }

async function mergeGia() {
  return ketQuaGop.value
}

// ⚠️ `vi.mock` được HOIST lên đầu tệp, và đường dẫn phân giải tương đối với TỆP NÀY.
vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    mergeSegments: mergeGia,
  }
})

/** Nạp lại module mỗi ca — state của Panel Editor là module-level singleton. */
async function tuoi() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  return { state, StatusBar }
}

beforeEach(() => {
  resetRecorder()
  ketQuaGop.value = { outcome: null, error: null }
})

afterEach(() => {
  vi.useRealTimers()
})

describe('Story 2.9 · AC4 — dòng báo hệ quả của lượt gộp', () => {
  it('🔴 ① gộp XONG ⇒ thanh trạng thái nói ra HỆ QUẢ, không chỉ "đã xong"', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }

    expect(await state.mergeCurrentSegment()).toBe('done')
    await wrapper.vm.$nextTick()

    // 🔴 Câu phải nêu **cả hai** hệ quả của AD-5 — *"chưa xác nhận"* và *"lịch sử vẫn tra lại
    // được"*. Một câu chỉ nói "Đã gộp hai câu" bỏ mất đúng phần người dùng cần biết để không
    // hoảng, và AC4 viết nguyên văn cả hai vế.
    expect(wrapper.find('.notice').text()).toBe(
      'Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được.',
    )
    wrapper.unmount()
  })

  it('🔴 ② câu ĐẦU CHƯƠNG bị từ chối ⇒ thanh trạng thái NÓI RA, không im lặng', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(11)
    ketQuaGop.value = {
      outcome: null,
      error: {
        code: 'segment_no_previous',
        message_key: 'err.segment.no_previous',
        params: { segment_id: '11' },
        retryable: false,
      },
    }

    expect(await state.mergeCurrentSegment()).toBe('refused')
    await wrapper.vm.$nextTick()

    // Câu của Rust, qua `tError()` — **không** một câu thứ hai viết lại ở frontend. Rust là
    // nguồn sự thật cho lý do từ chối; chép nó sang đây là hai nguồn cho một mệnh đề.
    expect(wrapper.find('.notice').text()).toBe(
      'Câu số 11 là câu đầu Chương — không có câu nào phía trên để gộp vào.',
    )
    wrapper.unmount()
  })

  it('③ chưa chọn câu nào ⇒ nói ra, và KHÔNG phát một lượt IPC nào', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    expect(await state.mergeCurrentSegment()).toBe('no-caret')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').text()).toBe(
      'Chưa có câu nào đang được chọn — đặt con trỏ vào một câu rồi gộp.',
    )
    wrapper.unmount()
  })

  it('④ dọn bằng SỰ KIỆN — người dùng gõ tiếp thì câu báo tắt', async () => {
    // ⚠️ Dọn ở `noteEditorEdit`, **không** bằng một `setTimeout`: một câu tự biến mất sau N
    // giây là một hẹn giờ phải chọn N, phải test, và nó vẫn sai với người đọc chậm. Khuôn đã
    // có sẵn cho `confirmNotice` từ 2026-08-15.
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }
    await state.mergeCurrentSegment()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notice').exists()).toBe(true)

    state.noteEditorEdit(14, 'người dùng gõ tiếp')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').exists()).toBe(false)
  })

  it('🔴 ⑤ một lượt gộp MỚI xoá câu báo cũ — không hai mệnh đề chồng nhau', async () => {
    // Ca này bảo vệ một chỗ dễ hỏng: `regroupError` được dọn ở đường thành công (Story 2.8),
    // nhưng ô nhớ **dòng báo** là một ô thứ hai. Nếu nó không được dọn cùng lúc thì một lượt
    // gộp thành công **sau** một lượt bị từ chối sẽ hiện câu từ chối cũ — thanh trạng thái nói
    // dối về thao tác vừa xảy ra.
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    state.setEditorCaret(11)
    ketQuaGop.value = {
      outcome: null,
      error: {
        code: 'segment_no_previous',
        message_key: 'err.segment.no_previous',
        params: { segment_id: '11' },
        retryable: false,
      },
    }
    await state.mergeCurrentSegment()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notice').text()).toContain('câu đầu Chương')

    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }
    await state.mergeCurrentSegment()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').text()).toBe(
      'Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được.',
    )
    wrapper.unmount()
  })

  it('⑥ câu báo gộp THAY chỗ mốc "Đã lưu", không đứng cạnh nó', async () => {
    // Thanh cao 34px chỉ chứa **một** mệnh đề — khuôn `v-if`/`v-else-if` đã có từ 2026-08-15.
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-08-17T12:00:00.000Z'))
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.noteEditorEdit(11, 'một câu vừa gõ')
    await state.flushEditorNow()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.saved').exists()).toBe(true)

    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }
    await state.mergeCurrentSegment()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').exists()).toBe(true)
    expect(wrapper.find('.saved').exists()).toBe(false)
    wrapper.unmount()
  })
})
