/**
 * Story 2.10 · AC6 · AC7 — **thanh trạng thái nói ra khi lệnh điều hướng không đi được đâu**,
 * và bất biến *"một cửa ghi duy nhất"* của ô nhớ thứ ba.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 KHOẢNG IM LẶNG STORY NÀY ĐÓNG — nó đã được ghi bằng chữ ở chính chỗ hỏng
 * ═════════════════════════════════════════════════════════════════════════════════
 * Trước lượt này, nhánh *"hết câu chưa dịch"* của `editor.next_untranslated` chỉ có một dòng
 * `console.info` ở `commands/index.ts`. Theo định nghĩa của dự án, `console` **là** im lặng:
 * người dùng bấm phím và **không một pixel nào đổi**. AC6 đòi *"báo rõ điều đó thay vì im lặng
 * không làm gì"* ⇒ mệnh đề ấy **không đạt** cho tới story này, dù AC12 của 2.5b đã "xanh".
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO NHÓM *"MỘT CỬA"* Ở CUỐI TỆP LÀ NHÓM QUAN TRỌNG NHẤT
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ô nhớ thứ ba làm bất biến *"thao tác vừa xảy ra sở hữu thanh trạng thái"* từ **hai chiều**
 * thành **N chiều**. Chi phí ấy được nêu và nhận trước khi Ice ký #4(b), và nó được trả bằng
 * một **cửa ghi duy nhất** (`editorPanelState.ts::datThongBao`) gán **cả ba** ref ở mọi lời
 * gọi — không ba lời gọi dọn rải rác.
 *
 * ⚠️ Lịch sử của chính bất biến này là lý do nhóm đó tồn tại: vế *"ai ghi một ô thì dọn ô còn
 * lại"* **đã hở một lần** với chỉ **hai** ô, và nó hở đủ lâu để đi qua trọn mười một cổng
 * *(Story 2.9, bắt được ở code review — xem `editorPanelState.ts::ghiRegroupNotice`)*. Với ba
 * ô, một lượt hở nữa là chuyện gần như chắc chắn nếu chỉ dựa vào kỷ luật người viết.
 *
 * 🔴 **Sáu ca "một cửa" phủ CẢ SÁU chiều**, không ba: mỗi cặp ô có hai chiều, và một cài đặt
 * dọn đúng một chiều là đúng hình dạng đã hở ở 2.9.
 *
 * 🔴 **Vì sao vitest chứ không e2e:** đây là logic state cộng một component vỏ — không hình
 * học, không engine. Mệnh đề *"phím thật có tới được lệnh không"* thuộc e2e; mệnh đề *"lượt
 * cuộn trông thế nào"* thuộc bàn đo và mắt Ice (`happy-dom` **không bố cục**).
 *
 * ⚠️ **GIỚI HẠN THẬT:** nhóm này canh **ô nhớ + người đọc nó + phép chọn**. Nó **không** canh
 * việc `dispatch('editor.next_segment')` tới được `goToNextSegment` — đường đó đi qua
 * `installCommands(deps)` ở `main.ts`, và cổng `check:commands` cộng e2e giữ nó.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'

/**
 * Lượt xác nhận kế tiếp trả gì. Đặt lại ở mỗi ca.
 *
 * ⚠️ Giả ở **biên IPC** (`src/config/segment.ts`), không bằng một hàm chỉ-test — cùng lý lẽ mà
 * `support/segmentFixture.ts` đã ghi: một setter chỉ-test đi vòng qua chính đường cần nghiệm thu.
 */
const ketQuaKy: { value: { outcome: { id: number; status: string } | null; error: unknown } } = {
  value: { outcome: null, error: null },
}

async function confirmGia() {
  return ketQuaKy.value
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    confirmSegment: confirmGia,
  }
})

/** Nạp lại module mỗi ca — state của Panel Editor là module-level singleton. */
async function tuoi() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  return { state, StatusBar }
}

/** Câu đang hiện trên thanh, hoặc `null`. Đọc qua **chính** component, không qua ref. */
function cauTrenThanh(wrapper: ReturnType<typeof mount>): string | null {
  const el = wrapper.find('.notice')
  return el.exists() ? el.text() : null
}

beforeEach(() => {
  resetRecorder()
  ketQuaKy.value = { outcome: null, error: null }
})

afterEach(() => {
  vi.useRealTimers()
})

// Fixture: 11 = `confirmed` có chữ · 12 = `draft` có chữ · 13 = `draft` RỖNG (chưa dịch).
const CAU_DAU = FIXTURE_SEGMENTS[0].id
const CAU_GIUA = FIXTURE_SEGMENTS[1].id
const CAU_CUOI = FIXTURE_SEGMENTS[2].id

describe('AC6 — hết câu chưa dịch thì BÁO, không im lặng', () => {
  it('🔴 đứng ở câu chưa dịch CUỐI CÙNG ⇒ không dời, và thanh trạng thái NÓI RA', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    // 13 là câu chưa dịch duy nhất, và nó là câu cuối ⇒ không còn gì phía dưới.
    state.setEditorCaret(CAU_CUOI)

    expect(state.goToNextUntranslatedCoBao()).toBe(false)
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBe(
      'Không còn câu nào chưa dịch ở phía dưới — con trỏ giữ nguyên.',
    )
    // 🔴 Con trỏ **ở nguyên**. Một lượt quay vòng về đầu là quyết định đã bị cấm có chữ ký.
    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
    wrapper.unmount()
  })

  it('🔴 dời được ⇒ KHÔNG nói gì — một câu thừa đẩy mất mốc "Đã lưu"', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(CAU_DAU)

    expect(state.goToNextUntranslatedCoBao()).toBe(true)
    await wrapper.vm.$nextTick()

    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })

  /**
   * 🔵 **CA THÊM 2026-08-17 (code review ba tầng) — và ca NGAY TRÊN là lý do nó phải tồn tại.**
   *
   * Ca *"dời được ⇒ KHÔNG nói gì"* ở trên **đi qua trên một sản phẩm đang hỏng**, và khuyết tật
   * nó bỏ lọt là thật: `dieuHuongVaBao` nhánh thành công không chạm một ô nhớ nào, nên một câu
   * của thao tác **trước** kẹt lại trên thanh.
   *
   * 🔴 **Vì sao nó không bắt được:** `tuoi()` gọi `vi.resetModules()`, nên `navNotice` là `null`
   * **từ đầu ca**. Một phép so `toBeNull()` trên một ô **chưa ai ghi** không phân biệt được
   * *"đã dọn"* với *"chưa từng bẩn"* — nó khẳng định `X` đã xảy ra mà không chứng minh **mã nào**
   * làm `X`. Đúng bài học mà `deferred-work.md` vừa ghi từ ba lượt đột biến của story này.
   *
   * ⇒ Ca này bắt buộc phải **làm bẩn ô nhớ trước** bằng một lượt thất bại thật, rồi mới đo lượt
   * thành công. Đột biến để xác nhận nó đỏ được: đổi `if (daDoi) datThongBao({})` thành
   * `if (daDoi) {}` ⇒ ca này ĐỎ, ca trên vẫn XANH.
   */
  it('🔴 thất bại RỒI thành công ⇒ câu cũ phải bị DỌN, không kẹt lại', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    // ① Làm bẩn ô nhớ bằng một lượt thất bại THẬT — không bằng một setter chỉ-test.
    state.setEditorCaret(CAU_CUOI)
    expect(state.goToNextSegmentCoBao()).toBe(false)
    await wrapper.vm.$nextTick()
    expect(state.editorNavNotice.value).toBe('at-last')
    expect(cauTrenThanh(wrapper)).not.toBeNull()

    // ② Lượt điều hướng kế tiếp ĐI ĐƯỢC ⇒ nó sở hữu thanh, và nó không có gì để nói.
    expect(state.goToPrevSegmentCoBao()).toBe(true)
    await wrapper.vm.$nextTick()

    expect(state.editorCaretSegmentId.value).toBe(CAU_GIUA)
    expect(state.editorNavNotice.value).toBeNull()
    // 🔴 Vế người dùng thật sự thấy: thanh trở lại chỗ mốc *"Đã lưu N giây trước"* dùng được.
    // `navNoticeKey` đứng TRƯỚC `secondsSinceSave` trong chuỗi `v-else-if`, nên một câu kẹt lại
    // che mốc ấy vô thời hạn — đó là cái giá thật của khuyết tật, không chỉ một câu sai.
    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })
})

describe('AC7 — biên Chương: đứng yên VÀ báo (Quyết định #5(a))', () => {
  it('câu cuối ⇒ `next_segment` không dời, thanh nói "đã ở câu cuối"', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(CAU_CUOI)

    expect(state.goToNextSegmentCoBao()).toBe(false)
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBe('Đã ở câu cuối Chương — không có câu nào phía dưới.')
    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
    wrapper.unmount()
  })

  it('câu đầu ⇒ `prev_segment` không dời, thanh nói "đã ở câu đầu"', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(CAU_DAU)

    expect(state.goToPrevSegmentCoBao()).toBe(false)
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBe('Đã ở câu đầu Chương — không có câu nào phía trên.')
    expect(state.editorCaretSegmentId.value).toBe(CAU_DAU)
    wrapper.unmount()
  })

  /**
   * 🔴 Hai câu báo ở biên phải **phân biệt được**. Một cài đặt dùng chung một khoá cho cả hai
   * chiều đi qua mọi cổng và mọi ca khác của tệp này — nó chỉ lộ ra ở đúng phép so này.
   */
  it('🔴 hai biên nói HAI câu khác nhau, không một câu chung', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    state.setEditorCaret(CAU_DAU)
    state.goToPrevSegmentCoBao()
    await wrapper.vm.$nextTick()
    const oDau = cauTrenThanh(wrapper)

    state.setEditorCaret(CAU_CUOI)
    state.goToNextSegmentCoBao()
    await wrapper.vm.$nextTick()
    const oCuoi = cauTrenThanh(wrapper)

    expect(oDau).not.toBe(oCuoi)
    wrapper.unmount()
  })

  it('AC1 · AC2 — giữa Chương thì dời được cả hai chiều, và không nói gì', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(CAU_GIUA)

    expect(state.goToNextSegmentCoBao()).toBe(true)
    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
    expect(state.goToPrevSegmentCoBao()).toBe(true)
    expect(state.editorCaretSegmentId.value).toBe(CAU_GIUA)

    await wrapper.vm.$nextTick()
    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })
})

describe('🔴 BẤT BIẾN "một cửa" — ai ghi một ô thì dọn HAI ô kia, cả sáu chiều', () => {
  /**
   * Dựng sẵn một câu **xác nhận** trên thanh: `⌘Enter` lúc chưa có caret ⇒ `'no-caret'`.
   * ⚠️ Đi qua **đúng đường sản phẩm**, không gán thẳng ref — một lượt gán thẳng kiểm một
   * đường mà sản phẩm không có.
   */
  async function datCauXacNhan(state: Awaited<ReturnType<typeof tuoi>>['state']) {
    state.setEditorCaret(null)
    expect(await state.confirmCurrentSegment()).toBe('no-caret')
  }

  it('① nav ghi ⇒ câu XÁC NHẬN bị dọn', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    await datCauXacNhan(state)
    await wrapper.vm.$nextTick()
    expect(cauTrenThanh(wrapper)).toBe(
      'Chưa có câu nào đang được chọn — đặt con trỏ vào một câu rồi xác nhận.',
    )

    state.setEditorCaret(CAU_CUOI)
    state.goToNextSegmentCoBao()
    await wrapper.vm.$nextTick()

    // 🔴 Nếu ô xác nhận KHÔNG bị dọn thì `v-if` của nó thắng vô điều kiện và thanh vẫn hiện câu
    // cũ — đúng khuyết tật mà Story 2.9 phải vá, chỉ đổi ô nhớ.
    expect(cauTrenThanh(wrapper)).toBe('Đã ở câu cuối Chương — không có câu nào phía dưới.')
    expect(state.editorConfirmNotice.value).toBeNull()
    wrapper.unmount()
  })

  it('② câu XÁC NHẬN ghi ⇒ nav bị dọn', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    state.setEditorCaret(CAU_CUOI)
    state.goToNextSegmentCoBao()
    await wrapper.vm.$nextTick()
    expect(state.editorNavNotice.value).toBe('at-last')

    await datCauXacNhan(state)
    await wrapper.vm.$nextTick()

    expect(state.editorNavNotice.value).toBeNull()
    expect(cauTrenThanh(wrapper)).toBe(
      'Chưa có câu nào đang được chọn — đặt con trỏ vào một câu rồi xác nhận.',
    )
    wrapper.unmount()
  })

  /**
   * ⚠️ `noteEditorEdit` là cửa *"người dùng gõ tiếp"* — nó **không** trao thanh cho ai, nó dọn
   * **cả ba**. Cả ba giá trị đều mô tả một thời điểm đã trôi qua.
   */
  it('③ người dùng gõ tiếp ⇒ CẢ BA ô sạch', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    state.setEditorCaret(CAU_CUOI)
    state.goToNextSegmentCoBao()
    expect(state.editorNavNotice.value).toBe('at-last')

    state.noteEditorEdit(CAU_CUOI, 'người dùng gõ tiếp')
    await wrapper.vm.$nextTick()

    expect(state.editorNavNotice.value).toBeNull()
    expect(state.editorConfirmNotice.value).toBeNull()
    expect(state.editorRegroupNotice.value).toBeNull()
    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })

  /**
   * 🔴 **CẠM BẪY ⑤ — LẦN THỨ BA CỦA CÙNG MỘT LUẬT.** `sourceCut` (2.8) và
   * `regroupNotice`/`regroupError` (2.9) đều lọt qua trọn mười một cổng vì **không cổng nào
   * canh** luật *"mọi ô nhớ mới phải qua `resetEditorPanel()`"*.
   *
   * Triệu chứng nếu ca này đỏ: mở Tác phẩm B và thấy câu báo điều hướng của Tác phẩm A
   * **trước khi bấm bất cứ gì**.
   */
  it('🔴 ④ đổi Tác phẩm (`resetEditorPanel`) ⇒ ô nhớ điều hướng KHÔNG sống sót', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    state.setEditorCaret(CAU_CUOI)
    state.goToNextSegmentCoBao()
    expect(state.editorNavNotice.value).toBe('at-last')

    state.resetEditorPanel()
    await wrapper.vm.$nextTick()

    expect(state.editorNavNotice.value).toBeNull()
    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })
})

/**
 * 🔴 **Quyết định #6(c) — món nợ `deferred-work.md:2837-2847`, đóng MỘT NỬA.**
 *
 * `⌘Enter` ở câu **cuối** Chương ký thành công nhưng không dời được con trỏ, nên
 * `resolveSegmentRule` giữ `primary` thắng `confirmed` và **vạch lề không đổi màu**. Vế thị
 * giác **vẫn hụt** sau story này; cái đóng được là vế *"người dùng biết chuyện gì vừa xảy ra"*.
 *
 * ⚠️ **Ca này KHÔNG kiểm vạch lề**, có chủ ý — vạch vẫn `primary`, và một ca khẳng định điều
 * ngược lại sẽ là một lời hứa sai trong bộ test.
 */
describe('Quyết định #6(c) — xác nhận ở câu CUỐI Chương thì nói ra', () => {
  /** Hàng mà Rust trả về cho một lượt ký thành công. */
  const daKy = (id: number): { id: number; status: string } => ({ id, status: 'confirmed' })

  it('🔴 ký câu cuối ⇒ thanh nói "đã xác nhận câu cuối Chương"', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(CAU_CUOI)
    ketQuaKy.value = { outcome: daKy(CAU_CUOI), error: null }

    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBe(
      'Đã xác nhận câu cuối Chương. Con trỏ ở nguyên vì không còn câu nào phía dưới.',
    )
    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
    wrapper.unmount()
  })

  /**
   * 🔴 **Đối chứng ÂM, và nó là ca làm ca trên đọc được.** Ký một câu **không phải** câu cuối
   * thì con trỏ dời được ⇒ vạch lề tự nói ⇒ **không** câu báo nào. Thiếu ca này, một cài đặt
   * ghi `'confirmed-last'` ở **mọi** lượt ký thành công vẫn xanh.
   */
  it('🔴 ký câu GIỮA Chương ⇒ dời con trỏ, và KHÔNG nói gì', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(CAU_GIUA)
    ketQuaKy.value = { outcome: daKy(CAU_GIUA), error: null }

    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    await wrapper.vm.$nextTick()

    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
    expect(state.editorNavNotice.value).toBeNull()
    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })

  /**
   * ⚠️ Cờ [`kyTrungCauCuoi`] là state **module-level**, nên nó phải sạch giữa hai lượt ký. Ca
   * này lái đúng đường rò: ký câu cuối *(cờ lên)* rồi ký một câu giữa *(cờ phải xuống)*.
   * Một cài đặt đặt lại cờ ở **cuối** hàm thay vì ở **đầu** sẽ đỏ ở đây.
   */
  it('🔴 cờ "câu cuối" KHÔNG rò sang lượt ký sau', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    state.setEditorCaret(CAU_CUOI)
    ketQuaKy.value = { outcome: daKy(CAU_CUOI), error: null }
    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    expect(state.editorNavNotice.value).toBe('confirmed-last')

    state.setEditorCaret(CAU_DAU)
    ketQuaKy.value = { outcome: daKy(CAU_DAU), error: null }
    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    await wrapper.vm.$nextTick()

    expect(state.editorNavNotice.value).toBeNull()
    wrapper.unmount()
  })
})

/**
 * 🔴 **AC5 vẫn nguyên sau Quyết định #3** — và ca này là chỗ hai họ vị từ gặp nhau ở **tầng
 * state**, không ở tầng vị từ thuần.
 *
 * `segmentNavigation.test.ts` đã kiểm mệnh đề này trên hàm thuần. Ca dưới đây kiểm rằng đường
 * dây **thật** — `segments` + `editedText` → `navigationSegmentOf` → vị từ → `setEditorCaret` —
 * giữ đúng phân biệt đó. Một lượt nối dây sai họ vị từ *(dễ: hai tên chỉ khác nhau vài chữ)*
 * đi qua cả `segmentNavigation.test.ts` lẫn mọi cổng.
 */
describe('AC5 vs Quyết định #3 — cùng một Chương, hai lệnh đi khác nhau', () => {
  it('🔴 câu đã CẮT BỎ: next_segment DỪNG ở nó, next_untranslated NHẢY QUA', async () => {
    const { state } = await tuoi()
    await state.ensureSegmentsLoaded()
    // Câu giữa bị cắt bỏ **và** rỗng ⇒ nó là ứng viên của cả hai lệnh nếu luật lọc bị trộn.
    state.replaceEditorSegment(CAU_GIUA, { is_omitted: true, target_text: '' } as Partial<ChapterSegment>)

    state.setEditorCaret(CAU_DAU)
    expect(state.goToNextSegment()).toBe(true)
    expect(state.editorCaretSegmentId.value).toBe(CAU_GIUA)

    state.setEditorCaret(CAU_DAU)
    expect(state.goToNextUntranslated()).toBe(true)
    expect(state.editorCaretSegmentId.value).toBe(CAU_CUOI)
  })
})
