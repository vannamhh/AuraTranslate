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

/**
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 NHÓM NÀY SINH RA Ở LƯỢT CODE REVIEW BA TẦNG, 2026-08-17
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ba khuyết tật dưới đây đi qua **trọn 177 ca** của lượt dev cộng mười một cổng, và cả ba cùng
 * một lớp: **một ô nhớ mới thiếu một cửa mà tệp đó đã có sẵn cửa cho ô nhớ cũ.** Ba lần, cùng
 * một tệp.
 *
 * 🔴 Vì sao lượt dev không bắt được: mọi ca của nó dựng một trạng thái **sạch** rồi gộp. Không ca
 * nào dựng **một câu báo đã có sẵn** trước lượt gộp, và không ca nào chạy `resetEditorPanel()`.
 * Bài học phương pháp, ghi ra để lượt sau đọc: một ô nhớ mới cần ba câu hỏi — *ai dọn nó?* ·
 * *nó thuộc Tác phẩm hay thuộc ứng dụng?* · *hai lượt gọi chồng nhau thì sao?*
 */
describe('🔵 Code review 2026-08-17 — ba cửa mà hai ô nhớ mới của Story 2.9 còn thiếu', () => {
  it('🔴 ① câu của lượt XÁC NHẬN không được che câu của lượt gộp THÀNH CÔNG', async () => {
    // Đường hỏng nguyên bản: `StatusBar.vue` khai `v-if` của `confirmNotice` **trước**
    // `v-else-if` của câu gộp, và `regroup()` không dọn `confirmNotice`. Cử chỉ `Backspace`
    // cũng không đi qua `noteEditorEdit` *(`preventDefault()` cắt chuỗi `input`)*.
    // ⇒ AC4 không đạt: gộp xong mà thanh vẫn nói câu của một thao tác đã trôi qua.
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    // ① Một lượt `⌘Enter` hụt — chưa đặt caret ⇒ `confirmNotice = 'no-caret'`.
    expect(await state.confirmCurrentSegment()).toBe('no-caret')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notice').text()).toBe(
      'Chưa có câu nào đang được chọn — đặt con trỏ vào một câu rồi xác nhận.',
    )

    // ② Người dùng **không gõ chữ nào** *(nên `noteEditorEdit` không chạy)*, chỉ đặt caret rồi
    //    bấm `Backspace`.
    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }
    expect(await state.mergeCurrentSegment()).toBe('done')
    await wrapper.vm.$nextTick()

    // 🔴 Mệnh đề: thanh nói về **thao tác vừa xảy ra**, không về một thao tác đã trôi qua.
    expect(wrapper.find('.notice').text()).toBe(
      'Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được.',
    )
    wrapper.unmount()
  })

  it('🔴 ①b chiều NGƯỢC LẠI — câu gộp cũ không được che mốc "Đã lưu" của lượt xác nhận', async () => {
    // Cùng một khuyết tật, đổi chiều: `confirmCurrentSegment` không dọn `regroupNotice`, nên một
    // câu *"Đã gộp hai câu…"* sống sót qua một lượt xác nhận **thành công** và che mốc mà UX-DR30
    // đòi. Vế đối xứng của bất biến, và nó cần ca riêng vì nó đi qua **cửa khác**.
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

    // Một lượt xác nhận chạy sau đó — kết quả nào cũng được, mệnh đề không phụ thuộc vào nó.
    await state.confirmCurrentSegment()
    await wrapper.vm.$nextTick()

    // 🔴 Đo trên **toàn** thanh, không trên `.notice`: một lượt xác nhận bị từ chối dọn **cả hai**
    // ô nhớ *(lý do từ chối đi ra qua `confirmError`, thứ `GridPanel.vue` đọc — không phải thanh
    // này)*, nên thanh có thể không còn phần tử nào. `.find('.notice').text()` sẽ **ném** ở đó, và
    // một ca ném vì hình dạng bề mặt là một ca nói về bề mặt, không về mệnh đề đang kiểm.
    expect(wrapper.text()).not.toContain('Đã gộp hai câu')
    wrapper.unmount()
  })

  it('🔴 ② `resetEditorPanel()` dọn câu báo — nó thuộc TÁC PHẨM, không thuộc ứng dụng', async () => {
    // `segment.id` là `AUTOINCREMENT` trong `project.db` của **từng** Tác phẩm, nên mỗi Tác phẩm
    // đếm lại từ 1 ⇒ số `11` trong câu từ chối của Tác phẩm A trỏ một câu **khác** ở Tác phẩm B.
    // Đúng lớp lỗi đã bị bắt và vá cho `confirmError`/`caretPlacement` ở code review 2026-08-15.
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
    expect(wrapper.find('.notice').exists()).toBe(true)

    // Đúng hàm `modes/libraryImport.ts::finishSubmit` gọi khi Tác phẩm đang mở BỊ THAY.
    state.resetEditorPanel()
    await wrapper.vm.$nextTick()

    // 🔴 Không một câu nào của Tác phẩm cũ được ở lại — người dùng chưa bấm gì ở Tác phẩm mới.
    expect(wrapper.find('.notice').exists()).toBe(false)
    wrapper.unmount()
  })

  it('🔴 ③ hai lượt gộp CHỒNG NHAU ⇒ lượt sau trả `busy` và NÓI RA, không ghi đè câu của lượt đầu', async () => {
    // `event.repeat` chỉ chặn auto-repeat của **hệ điều hành**; hai cú `Backspace` **rời rạc** bấm
    // nhanh thì không. Và vì nhánh đó `preventDefault()`, DOM cùng caret y nguyên ở offset 0 nên
    // cú thứ hai qua trọn `caretAtCellStart` và dispatch lại cho **cùng** một `id`.
    //
    // 🔴 Không có khoá, lượt IPC thứ hai trả `refused` *(segment đã về hưu)* và ghi đè câu
    // `'merged'` ⇒ thanh báo *"chưa gộp được"* cho một thao tác **đã gộp xong**. Nói dối đúng
    // chiều nguy hiểm, trên một lượt ghi mà AD-5 không cho hoàn tác.
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }

    // Hai lượt phát **cùng một tick**, không `await` giữa chúng — đúng hình dạng hai cú bấm nhanh.
    const [dau, sau] = await Promise.all([state.mergeCurrentSegment(), state.mergeCurrentSegment()])

    // 🔴 Lượt đầu xong việc; lượt sau bị **từ chối và nói ra**, không NHẬP vào lượt đầu. Nhập vào
    // sẽ trả `'done'` cho một thao tác chưa từng được phát — một lượt đánh rơi im lặng.
    expect(dau).toBe('done')
    expect(sau).toBe('busy')
    wrapper.unmount()
  })

  it('③b `busy` có câu chữ — bảng tra ĐÓNG không cho một giá trị nào lọt ra màn hình câm', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }

    const chay = state.mergeCurrentSegment()
    // Lượt thứ hai phát trong lúc lượt đầu còn bay ⇒ nó ghi câu `'busy'`.
    expect(await state.mergeCurrentSegment()).toBe('busy')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notice').text()).toBe(
      'Một lượt gộp hoặc tách đang chạy — lượt vừa bấm chưa được phát. Chờ nó xong rồi bấm lại.',
    )

    await chay
    wrapper.unmount()
  })
})
